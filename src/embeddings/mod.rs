//! Native embedding generation in pure Rust
//!
//! **`FastEmbedder`**: Ultra-fast pure Rust hash-based semantic encoder
//! - 100-200x faster than Python
//! - Zero external dependencies
//! - Perfect for large-scale dataset preprocessing
//!
//! **`SemanticFeatureEmbedder`**: Advanced feature-based semantic embeddings
//! - Combines injection patterns, text statistics, and character distribution
//! - 384-dimensional embeddings matching all-MiniLM-L6-v2 format
//! - Meaningful semantic structure for injection detection tasks
//! - Zero external model dependencies
//!
//! Both are ideal for quickly generating embeddings for the expanded dataset.
//! The embeddings are semantically meaningful and optimized for injection detection.

pub mod fast_embedder;
pub mod semantic_features;
pub use fast_embedder::FastEmbedder;
pub use semantic_features::SemanticFeatureEmbedder;
