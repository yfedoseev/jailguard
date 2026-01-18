//! Pre-trained embedding layer using all-MiniLM-L6-v2 vectors.
//!
//! This module provides a pre-trained embedding layer that uses real semantic embeddings
//! from the all-MiniLM-L6-v2 model instead of training random embeddings from scratch.
//!
//! The all-MiniLM-L6-v2 model produces 384-dimensional embeddings trained on 1 billion
//! diverse sentence pairs, providing strong semantic understanding from the start.

use crate::error::Result;
use burn::nn::{LayerNorm, LayerNormConfig};
use burn::tensor::{backend::Backend, Tensor, TensorData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pre-trained embedding lookup table.
///
/// Maps normalized text to pre-computed all-MiniLM-L6-v2 embeddings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingLookup {
    /// Maps text -> 384-dim embedding vector
    lookup: HashMap<String, Vec<f32>>,
    /// Embedding dimension (384 for all-MiniLM-L6-v2)
    embed_dim: usize,
}

impl EmbeddingLookup {
    /// Create a new embedding lookup table.
    pub fn new(embed_dim: usize) -> Self {
        Self {
            lookup: HashMap::new(),
            embed_dim,
        }
    }

    /// Add an embedding to the lookup table.
    pub fn insert(&mut self, text: String, embedding: Vec<f32>) {
        self.lookup.insert(text, embedding);
    }

    /// Get an embedding from the lookup table.
    pub fn get(&self, text: &str) -> Option<&Vec<f32>> {
        self.lookup.get(text)
    }

    /// Get embedding dimension.
    pub fn embed_dim(&self) -> usize {
        self.embed_dim
    }

    /// Get number of cached embeddings.
    pub fn len(&self) -> usize {
        self.lookup.len()
    }

    /// Check if lookup table is empty.
    pub fn is_empty(&self) -> bool {
        self.lookup.is_empty()
    }
}

/// Pre-trained embedding layer using all-MiniLM-L6-v2.
///
/// This layer provides:
/// - Pre-computed semantic embeddings (384-dim from all-MiniLM-L6-v2)
/// - Layer normalization
///
/// The semantic embeddings are pre-computed and stored. Layer normalization
/// is applied to normalize the embeddings for stable training.
#[derive(Debug)]
pub struct PretrainedEmbedding<B: Backend> {
    /// Pre-computed embedding vectors (384-dim)
    lookup: EmbeddingLookup,
    /// Layer normalization
    layer_norm: LayerNorm<B>,
    /// Embedding dimension (384 for all-MiniLM-L6-v2)
    embed_dim: usize,
}

// Implement Clone manually since EmbeddingLookup is not a burn Module
impl<B: Backend> Clone for PretrainedEmbedding<B> {
    fn clone(&self) -> Self {
        Self {
            lookup: self.lookup.clone(),
            layer_norm: self.layer_norm.clone(),
            embed_dim: self.embed_dim,
        }
    }
}

/// Configuration for pre-trained embedding layer.
#[derive(Debug, Clone)]
pub struct PretrainedEmbeddingConfig {
    /// Pre-computed embeddings lookup
    pub lookup: EmbeddingLookup,
    /// Maximum sequence length
    pub max_length: usize,
    /// Embedding dimension (384 for all-MiniLM-L6-v2)
    pub embed_dim: usize,
}

impl PretrainedEmbeddingConfig {
    /// Create a new configuration.
    pub fn new(lookup: EmbeddingLookup, max_length: usize) -> Self {
        let embed_dim = lookup.embed_dim();
        Self {
            lookup,
            max_length,
            embed_dim,
        }
    }

    /// Initialize the embedding layer.
    pub fn init<B: Backend>(&self, device: &B::Device) -> PretrainedEmbedding<B> {
        // Layer norm for the embedding dimension
        let layer_norm = LayerNormConfig::new(self.embed_dim).init(device);

        PretrainedEmbedding {
            lookup: self.lookup.clone(),
            layer_norm,
            embed_dim: self.embed_dim,
        }
    }
}

impl<B: Backend> PretrainedEmbedding<B> {
    /// Forward pass: convert text to embeddings using pre-trained vectors.
    ///
    /// # Arguments
    /// * `texts` - Text samples to embed
    ///
    /// # Returns
    /// Embedded tensor of shape [batch_size, 1, embed_dim]
    pub fn forward(&self, texts: &[String]) -> Result<Tensor<B, 3>> {
        let batch_size = texts.len();
        let device = self.layer_norm.gamma.device();

        // Get embeddings for each text
        let mut embeddings = Vec::new();

        for text in texts {
            if let Some(embedding) = self.lookup.get(text) {
                embeddings.push(embedding.clone());
            } else {
                // For unknown text, use zero vector (fallback)
                // In production, could use a hash-based mapping or compute on-the-fly
                embeddings.push(vec![0.0; self.embed_dim]);
            }
        }

        // Convert to tensor: [batch_size, embed_dim]
        let flat_embeddings: Vec<f32> = embeddings.iter().flat_map(|e| e.iter().cloned()).collect();

        let tensor = Tensor::<B, 2, _>::from_data(
            TensorData::new(flat_embeddings, [batch_size, self.embed_dim]),
            &device,
        );

        // Unsqueeze to [batch_size, 1, embed_dim] for sequence processing
        let unsqueezed = tensor.unsqueeze::<3>();

        // Apply layer norm
        Ok(self.layer_norm.forward(unsqueezed))
    }

    /// Get embedding dimension.
    pub fn embed_dim(&self) -> usize {
        self.embed_dim
    }

    /// Get number of cached embeddings.
    pub fn num_cached_embeddings(&self) -> usize {
        self.lookup.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_lookup() {
        let mut lookup = EmbeddingLookup::new(384);
        let embedding = vec![0.1; 384];

        lookup.insert("test text".to_string(), embedding.clone());

        assert_eq!(lookup.len(), 1);
        assert!(lookup.get("test text").is_some());
        assert_eq!(lookup.get("test text").unwrap(), &embedding);
        assert!(lookup.get("unknown").is_none());
    }

    #[test]
    fn test_embedding_lookup_dimension() {
        let lookup = EmbeddingLookup::new(384);
        assert_eq!(lookup.embed_dim(), 384);
    }

    #[test]
    fn test_pretrained_embedding_config() {
        let mut lookup = EmbeddingLookup::new(384);
        lookup.insert("test".to_string(), vec![0.1; 384]);

        let config = PretrainedEmbeddingConfig::new(lookup, 512);

        assert_eq!(config.embed_dim, 384);
        assert_eq!(config.max_length, 512);
    }
}
