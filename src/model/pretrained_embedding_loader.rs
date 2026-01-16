//! Loader for pre-computed all-MiniLM-L6-v2 embeddings.
//!
//! This module loads pre-computed semantic embeddings from a JSON file,
//! enabling training with real all-MiniLM-L6-v2 embeddings instead of
//! training embeddings from scratch.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Pre-computed embedding sample with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingSample {
    /// The text that was embedded
    pub text: String,
    /// Whether this text contains a prompt injection
    pub is_injection: bool,
    /// Type of attack (optional)
    pub attack_type: Option<String>,
    /// The 384-dimensional embedding vector from all-MiniLM-L6-v2
    pub embedding: Vec<f32>,
    /// Embedding dimension (should be 384)
    pub embedding_dim: usize,
    /// Sample index
    pub index: usize,
}

impl EmbeddingSample {
    /// Validate that embedding has correct dimension.
    pub fn validate_dim(&self, expected_dim: usize) -> Result<()> {
        if self.embedding.len() != expected_dim {
            return Err(crate::error::Error::Model(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                expected_dim,
                self.embedding.len()
            )));
        }
        Ok(())
    }
}

/// Loader for pre-computed embeddings from JSON file.
pub struct EmbeddingLoader {
    samples: Vec<EmbeddingSample>,
}

impl EmbeddingLoader {
    /// Load embeddings from JSON file.
    ///
    /// Expected format:
    /// ```json
    /// [
    ///   {
    ///     "text": "...",
    ///     "is_injection": true/false,
    ///     "attack_type": "...",
    ///     "embedding": [...],
    ///     "embedding_dim": 384,
    ///     "index": 0
    ///   },
    ///   ...
    /// ]
    /// ```
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;

        let reader = BufReader::new(file);
        let samples: Vec<EmbeddingSample> = serde_json::from_reader(reader)?;

        // Validate at least one sample
        if samples.is_empty() {
            return Err(crate::error::Error::Dataset(
                "No embedding samples found in file".to_string(),
            ));
        }

        // Validate first sample has expected dimension
        let expected_dim = 384; // all-MiniLM-L6-v2 dimension
        samples[0].validate_dim(expected_dim)?;

        Ok(Self { samples })
    }

    /// Get all loaded samples.
    pub fn samples(&self) -> &[EmbeddingSample] {
        &self.samples
    }

    /// Get number of loaded samples.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if loader is empty.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get a sample by index.
    pub fn get(&self, index: usize) -> Option<&EmbeddingSample> {
        self.samples.get(index)
    }

    /// Get embedding dimension (should be 384 for all-MiniLM-L6-v2).
    pub fn embedding_dim(&self) -> usize {
        if let Some(first) = self.samples.first() {
            first.embedding.len()
        } else {
            384 // Default for all-MiniLM-L6-v2
        }
    }

    /// Split into train and test sets.
    ///
    /// # Arguments
    /// * `train_ratio` - Fraction of samples for training (0.0-1.0)
    ///
    /// # Returns
    /// Tuple of (`train_samples`, `test_samples`)
    pub fn train_test_split(
        &self,
        train_ratio: f32,
    ) -> (Vec<EmbeddingSample>, Vec<EmbeddingSample>) {
        let split_idx = ((self.samples.len() as f32 * train_ratio) as usize).max(1);
        let train = self.samples[..split_idx].to_vec();
        let test = self.samples[split_idx..].to_vec();
        (train, test)
    }

    /// Get samples by label.
    pub fn samples_by_label(&self, is_injection: bool) -> Vec<&EmbeddingSample> {
        self.samples
            .iter()
            .filter(|s| s.is_injection == is_injection)
            .collect()
    }

    /// Get class distribution statistics.
    pub fn class_distribution(&self) -> (usize, usize) {
        let injections = self.samples.iter().filter(|s| s.is_injection).count();
        let benign = self.samples.len() - injections;
        (injections, benign)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_sample_validation() {
        let sample = EmbeddingSample {
            text: "test".to_string(),
            is_injection: false,
            attack_type: None,
            embedding: vec![0.1; 384],
            embedding_dim: 384,
            index: 0,
        };

        assert!(sample.validate_dim(384).is_ok());
        assert!(sample.validate_dim(256).is_err());
    }

    #[test]
    fn test_class_distribution() {
        let samples = vec![
            EmbeddingSample {
                text: "inject".to_string(),
                is_injection: true,
                attack_type: None,
                embedding: vec![0.1; 384],
                embedding_dim: 384,
                index: 0,
            },
            EmbeddingSample {
                text: "benign".to_string(),
                is_injection: false,
                attack_type: None,
                embedding: vec![0.2; 384],
                embedding_dim: 384,
                index: 1,
            },
            EmbeddingSample {
                text: "inject2".to_string(),
                is_injection: true,
                attack_type: None,
                embedding: vec![0.3; 384],
                embedding_dim: 384,
                index: 2,
            },
        ];

        let loader = EmbeddingLoader {
            samples: samples.clone(),
        };

        let (inj, ben) = loader.class_distribution();
        assert_eq!(inj, 2);
        assert_eq!(ben, 1);
    }

    #[test]
    fn test_train_test_split() {
        let samples = (0..100)
            .map(|i| EmbeddingSample {
                text: format!("text{}", i),
                is_injection: i % 2 == 0,
                attack_type: None,
                embedding: vec![0.1; 384],
                embedding_dim: 384,
                index: i,
            })
            .collect::<Vec<_>>();

        let loader = EmbeddingLoader {
            samples: samples.clone(),
        };

        let (train, test) = loader.train_test_split(0.8);
        assert_eq!(train.len(), 80);
        assert_eq!(test.len(), 20);
        assert_eq!(train.len() + test.len(), 100);
    }
}
