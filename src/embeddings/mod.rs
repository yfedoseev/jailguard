//! Embedding generation modules.
//!
//! **`SemanticFeatureEmbedder`**: Advanced feature-based semantic embeddings
//! - Combines injection patterns, text statistics, and character distribution
//! - 384-dimensional embeddings matching all-MiniLM-L6-v2 format
//! - Zero external model dependencies
//!
//! **`OnnxEmbedder`**: Real neural embeddings via ONNX Runtime
//! - all-MiniLM-L6-v2 sentence transformer
//! - 384-dimensional L2-normalized vectors
//! - Proper BERT tokenization + mean pooling
//! - Requires ONNX model file + tokenizer.json

pub mod semantic_features;
pub use semantic_features::SemanticFeatureEmbedder;

pub mod onnx_embedder;
pub use onnx_embedder::OnnxEmbedder;
