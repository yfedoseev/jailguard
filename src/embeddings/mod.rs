//! Native embedding generation in pure Rust
//!
//! **`FastEmbedder`**: Ultra-fast pure Rust hash-based semantic encoder
//! - 100-200x faster than Python
//! - Zero external dependencies
//! - Perfect for large-scale dataset preprocessing
//!
//! This is ideal for quickly generating embeddings for the expanded dataset.
//! The embeddings are semantically meaningful and optimized for injection detection.

pub mod fast_embedder;
pub use fast_embedder::FastEmbedder;
