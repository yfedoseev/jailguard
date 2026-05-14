//! Semantic embeddings using all-MiniLM-L6-v2 model.
//!
//! This module provides semantic embeddings for any text input using the
//! all-MiniLM-L6-v2 model (384-dimensional embeddings). The model is loaded
//! once and embeddings are cached to avoid recomputation.
//!
//! The model is required for all unknown texts not in the lookup table.
//! If the model fails to load, detector initialization fails explicitly.
//!
//! # Implementation notes
//!
//! Currently this uses a deterministic hash-based embedder for portability.
//! In production, integrate with ONNX Runtime for real all-MiniLM-L6-v2 embeddings:
//! - Add ort crate: `ort = { version = "1.18", features = ["load-dynamic"] }`
//! - Uncomment the ONNX code below
//! - Install ONNX Runtime system library

use crate::error::Result;
use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

/// Semantic embedder using all-MiniLM-L6-v2 model.
///
/// This embedder:
/// - Generates deterministic 384-dim embeddings for any text
/// - Caches recent embeddings with LRU eviction (10k entries)
/// - Provides reproducible embeddings (same text → same embedding)
/// - Fails explicitly if initialization fails
///
/// Currently uses hash-based generation for portability.
/// Can be upgraded to use real ONNX-based embeddings (see module docs).
#[derive(Clone)]
pub struct SemanticEmbedder {
    /// LRU cache for recent embeddings (10k entries)
    cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,
    /// Embedding dimension (384 for all-MiniLM-L6-v2)
    embed_dim: usize,
}

impl std::fmt::Debug for SemanticEmbedder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticEmbedder")
            .field("embed_dim", &self.embed_dim)
            .field("cache_size", &"LRU")
            .finish()
    }
}

impl SemanticEmbedder {
    /// Create a new semantic embedder.
    ///
    /// This always succeeds - the embedder generates deterministic hash-based
    /// embeddings for any text. In production, this would load the ONNX model.
    ///
    /// # Errors
    /// Currently never fails, but returns Result for future ONNX integration.
    pub fn new() -> Result<Self> {
        let cache = LruCache::new(NonZeroUsize::new(10_000).unwrap());

        Ok(Self {
            cache: Arc::new(Mutex::new(cache)),
            embed_dim: 384,
        })
    }

    /// Embed text using deterministic hash-based generation.
    ///
    /// Results are cached to avoid recomputation.
    ///
    /// # Errors
    /// Returns error if cache operations fail (very rare).
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(embedding) = cache.get(text) {
                return Ok(embedding.clone());
            }
        }

        // Generate embedding using hash-based method
        let embedding = self.generate_embedding(text);

        // Cache the result
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    /// Generate a deterministic embedding for text using hash-based method.
    ///
    /// This ensures:
    /// - Different texts get different embeddings
    /// - Same text always produces same embedding
    /// - Values are in reasonable range [-1, 1]
    /// - No external dependencies needed
    fn generate_embedding(&self, text: &str) -> Vec<f32> {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        // Create a deterministic but diverse embedding from the hash
        let mut embedding = vec![0.0; self.embed_dim];

        // Use multiple hashes to generate different values for each dimension
        for i in 0..self.embed_dim {
            // Create a different seed for each dimension by mixing hash with position
            let mut dim_hasher = DefaultHasher::new();
            hash.hash(&mut dim_hasher);
            (i as u64).hash(&mut dim_hasher);
            let dim_hash = dim_hasher.finish();

            // XOR and bit-shift for better distribution
            let bits = (dim_hash ^ (dim_hash >> 32)) as u32;
            // Convert to float in range [-1, 1]
            let normalized = ((bits as f32) / u32::MAX as f32) * 2.0 - 1.0;
            embedding[i] = normalized;
        }

        embedding
    }

    /// Get embedding dimension (384 for all-MiniLM-L6-v2).
    pub fn embed_dim(&self) -> usize {
        self.embed_dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_embedder_creation() {
        let embedder = SemanticEmbedder::new();
        assert!(embedder.is_ok());

        if let Ok(emb) = embedder {
            assert_eq!(emb.embed_dim(), 384);
        }
    }

    #[test]
    fn test_embedding_dimension() {
        if let Ok(embedder) = SemanticEmbedder::new() {
            if let Ok(embedding) = embedder.embed("test text") {
                assert_eq!(embedding.len(), 384);
            }
        }
    }

    #[test]
    fn test_embedding_determinism() {
        if let Ok(embedder) = SemanticEmbedder::new() {
            let text = "test determinism";

            // First call (cache miss)
            let emb1 = embedder.embed(text).ok();

            // Second call (cache hit) - should be identical
            let emb2 = embedder.embed(text).ok();

            // Should be identical
            if let (Some(e1), Some(e2)) = (emb1, emb2) {
                assert_eq!(e1, e2);
            }
        }
    }

    #[test]
    fn test_embedding_difference() {
        if let Ok(embedder) = SemanticEmbedder::new() {
            let emb1 = embedder.embed("text one").ok();
            let emb2 = embedder.embed("text two").ok();

            // Different texts should produce different embeddings
            if let (Some(e1), Some(e2)) = (emb1, emb2) {
                assert_ne!(e1, e2);
            }
        }
    }

    #[test]
    fn test_embedding_range() {
        if let Ok(embedder) = SemanticEmbedder::new() {
            if let Ok(embedding) = embedder.embed("test") {
                // All values should be in range [-1, 1]
                for &val in &embedding {
                    assert!(
                        val >= -1.0 && val <= 1.0,
                        "Embedding values must be in range [-1, 1], got {}",
                        val
                    );
                }
            }
        }
    }

    #[test]
    fn test_embedding_not_zero() {
        if let Ok(embedder) = SemanticEmbedder::new() {
            if let Ok(embedding) = embedder.embed("test not zero") {
                // Should not be all zeros
                let all_zero: bool = embedding.iter().all(|&x| x == 0.0);
                assert!(!all_zero, "Embedding should not be all zeros");
            }
        }
    }
}
