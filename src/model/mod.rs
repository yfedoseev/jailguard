//! Neural network models for prompt injection detection.
//!
//! This module contains the core neural network architectures:
//! - Text embedding layer
//! - Policy network (action selection)
//! - Value network (for PPO advantage estimation)
//! - Transformer encoder (multi-head attention based detection)

mod classifier;
mod embedding;
mod policy;
mod pretrained_embedding_loader;
mod semantic_head;
pub mod transformer;
mod value;

pub use classifier::{AttackClassifier, AttackClassifierConfig};
pub use embedding::{TextEmbedding, TextEmbeddingConfig};
pub use policy::{PolicyNetwork, PolicyNetworkConfig};
pub use pretrained_embedding_loader::{EmbeddingLoader, EmbeddingSample};
pub use semantic_head::{SemanticSimilarityHead, SemanticSimilarityHeadConfig};
pub use transformer::{TransformerConfig, TransformerEncoder, TransformerEncoderConfig};
pub use value::{ValueNetwork, ValueNetworkConfig};
