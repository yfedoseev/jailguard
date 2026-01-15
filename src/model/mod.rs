//! Neural network models for prompt injection detection.
//!
//! This module contains the core neural network architectures:
//! - Text embedding layer
//! - Policy network (action selection)
//! - Value network (for PPO advantage estimation)
//! - Transformer encoder (multi-head attention based detection)

mod embedding;
mod policy;
pub mod transformer;
mod value;

pub use embedding::{TextEmbedding, TextEmbeddingConfig};
pub use policy::{PolicyNetwork, PolicyNetworkConfig};
pub use transformer::{TransformerConfig, TransformerEncoder, TransformerEncoderConfig};
pub use value::{ValueNetwork, ValueNetworkConfig};
