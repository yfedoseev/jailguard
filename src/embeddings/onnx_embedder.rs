//! ONNX-based embedding generation for all-MiniLM-L6-v2
//!
//! Real neural embeddings using ONNX Runtime + HuggingFace tokenizer.
//! Based on the production implementation from secretguard.
//!
//! # Requirements
//! - ONNX model file (all-MiniLM-L6-v2.onnx or INT8 quantized variant)
//! - HuggingFace tokenizer.json
//!
//! # Usage
//! ```ignore
//! use jailguard::embeddings::OnnxEmbedder;
//!
//! let embedder = OnnxEmbedder::from_dir("models/")?;
//! let embedding = embedder.embed("Hello world")?;
//! assert_eq!(embedding.len(), 384);
//! ```

use ort::session::Session;
use ort::value::Value;
use std::path::Path;
use thiserror::Error;

/// Embedding dimension for all-MiniLM-L6-v2
pub const EMBEDDING_DIM: usize = 384;
/// Maximum sequence length for the model
pub const MAX_SEQ_LENGTH: usize = 256;

/// Errors from the ONNX embedder
#[derive(Error, Debug)]
pub enum OnnxEmbedderError {
    /// ONNX Runtime error
    #[error("ONNX Runtime error: {0}")]
    Ort(#[from] ort::Error),
    /// Tokenizer error
    #[error("Tokenizer error: {0}")]
    Tokenizer(String),
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Model file not found
    #[error("Model not found at: {0}")]
    ModelNotFound(String),
    /// Shape error
    #[error("Shape error: {0}")]
    Shape(String),
}

/// ONNX-based sentence embedder using all-MiniLM-L6-v2.
///
/// Produces 384-dimensional L2-normalized embeddings using proper
/// BERT tokenization and mean pooling with attention mask.
pub struct OnnxEmbedder {
    session: std::sync::Mutex<Session>,
    tokenizer: tokenizers::Tokenizer,
}

impl OnnxEmbedder {
    /// Load model and tokenizer from a directory.
    ///
    /// Expects:
    /// - `<dir>/model.onnx` or `<dir>/all-MiniLM-L6-v2.onnx`
    /// - `<dir>/tokenizer.json`
    pub fn from_dir<P: AsRef<Path>>(model_dir: P) -> Result<Self, OnnxEmbedderError> {
        let dir = model_dir.as_ref();

        let model_path = if dir.join("model.onnx").exists() {
            dir.join("model.onnx")
        } else if dir.join("all-MiniLM-L6-v2.onnx").exists() {
            dir.join("all-MiniLM-L6-v2.onnx")
        } else if dir.join("all-MiniLM-L6-v2-int8.onnx").exists() {
            dir.join("all-MiniLM-L6-v2-int8.onnx")
        } else {
            return Err(OnnxEmbedderError::ModelNotFound(format!(
                "No .onnx file found in {}",
                dir.display()
            )));
        };

        let tokenizer_path = dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            return Err(OnnxEmbedderError::ModelNotFound(format!(
                "tokenizer.json not found in {}",
                dir.display()
            )));
        }

        Self::from_files(&model_path, &tokenizer_path)
    }

    /// Load from explicit model and tokenizer paths.
    pub fn from_files<P: AsRef<Path>>(
        model_path: P,
        tokenizer_path: P,
    ) -> Result<Self, OnnxEmbedderError> {
        let session = Session::builder()?.commit_from_file(model_path.as_ref())?;

        let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path.as_ref())
            .map_err(|e| OnnxEmbedderError::Tokenizer(e.to_string()))?;

        Ok(Self {
            session: std::sync::Mutex::new(session),
            tokenizer,
        })
    }

    /// Generate embedding for a single text.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, OnnxEmbedderError> {
        let batch = self.embed_batch(&[text])?;
        Ok(batch.into_iter().next().unwrap_or_default())
    }

    /// Generate embeddings for a batch of texts.
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, OnnxEmbedderError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let batch_size = texts.len();

        // Tokenize
        let encodings = self
            .tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| OnnxEmbedderError::Tokenizer(e.to_string()))?;

        let max_len = encodings
            .iter()
            .map(|e| e.get_ids().len().min(MAX_SEQ_LENGTH))
            .max()
            .unwrap_or(0);

        // Build padded flat tensors
        let mut input_ids = Vec::with_capacity(batch_size * max_len);
        let mut attention_mask_flat = Vec::with_capacity(batch_size * max_len);
        let mut token_type_ids = Vec::with_capacity(batch_size * max_len);

        for encoding in &encodings {
            let ids = encoding.get_ids();
            let mask = encoding.get_attention_mask();
            let len = ids.len().min(max_len);

            for i in 0..len {
                input_ids.push(ids[i] as i64);
                attention_mask_flat.push(mask[i] as i64);
                token_type_ids.push(0i64);
            }
            for _ in len..max_len {
                input_ids.push(0i64);
                attention_mask_flat.push(0i64);
                token_type_ids.push(0i64);
            }
        }

        // Create ndarray Arrays (ndarray 0.16, matching ort 2.0.0-rc.9)
        let input_ids_array = ndarray::Array2::from_shape_vec((batch_size, max_len), input_ids)
            .map_err(|e| OnnxEmbedderError::Shape(e.to_string()))?;

        let attention_mask_array =
            ndarray::Array2::from_shape_vec((batch_size, max_len), attention_mask_flat.clone())
                .map_err(|e| OnnxEmbedderError::Shape(e.to_string()))?;

        let token_type_ids_array =
            ndarray::Array2::from_shape_vec((batch_size, max_len), token_type_ids)
                .map_err(|e| OnnxEmbedderError::Shape(e.to_string()))?;

        // Run ONNX inference
        let input_ids_value = Value::from_array(input_ids_array)?;
        let attention_mask_value = Value::from_array(attention_mask_array)?;
        let token_type_ids_value = Value::from_array(token_type_ids_array)?;

        let inputs = ort::inputs![
            "input_ids" => input_ids_value,
            "attention_mask" => attention_mask_value,
            "token_type_ids" => token_type_ids_value
        ];
        let mut session = self.session.lock().expect("session mutex poisoned");
        let outputs = session.run(inputs)?;

        // Extract output: shape [batch_size, seq_len, 384] — flat &[f32]
        let (_, output_data) = outputs[0].try_extract_tensor::<f32>()?;

        // Mean pooling with attention mask + L2 normalization
        let mut embeddings = Vec::with_capacity(batch_size);

        for b in 0..batch_size {
            let mut pooled = vec![0.0f32; EMBEDDING_DIM];
            let mut token_count = 0.0f32;

            for t in 0..max_len {
                let mask_val = attention_mask_flat[b * max_len + t] as f32;
                if mask_val > 0.0 {
                    for d in 0..EMBEDDING_DIM {
                        pooled[d] += output_data
                            [b * max_len * EMBEDDING_DIM + t * EMBEDDING_DIM + d]
                            * mask_val;
                    }
                    token_count += mask_val;
                }
            }

            if token_count > 0.0 {
                for d in 0..EMBEDDING_DIM {
                    pooled[d] /= token_count;
                }
            }

            // L2 normalize
            let norm: f32 = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for x in &mut pooled {
                    *x /= norm;
                }
            }

            embeddings.push(pooled);
        }

        Ok(embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_dim_constant() {
        assert_eq!(EMBEDDING_DIM, 384);
    }
}
