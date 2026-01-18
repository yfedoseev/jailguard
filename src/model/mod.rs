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
mod pretrained_embedding;
mod pretrained_embedding_loader;
#[cfg(feature = "semantic-embeddings")]
pub mod semantic_embedder;
mod semantic_head;
pub mod serialization;
pub mod transformer;
mod value;

pub use classifier::{AttackClassifier, AttackClassifierConfig};
pub use embedding::{TextEmbedding, TextEmbeddingConfig};
pub use policy::{PolicyNetwork, PolicyNetworkConfig};
pub use pretrained_embedding::{EmbeddingLookup, PretrainedEmbedding, PretrainedEmbeddingConfig};
pub use pretrained_embedding_loader::{EmbeddingLoader, EmbeddingSample};
#[cfg(feature = "semantic-embeddings")]
pub use semantic_embedder::SemanticEmbedder;
pub use semantic_head::{SemanticSimilarityHead, SemanticSimilarityHeadConfig};
pub use serialization::{ModelCheckpoint, ModelFormat, ModelMetadata};
pub use transformer::{TransformerConfig, TransformerEncoder, TransformerEncoderConfig};
pub use value::{ValueNetwork, ValueNetworkConfig};
