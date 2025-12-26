//! Neural network models for prompt injection detection.
//!
//! This module contains the core neural network architectures:
//! - Text embedding layer
//! - Policy network (action selection)
//! - Value network (for PPO advantage estimation)

mod embedding;
mod policy;
mod value;

pub use embedding::{TextEmbedding, TextEmbeddingConfig};
pub use policy::{PolicyNetwork, PolicyNetworkConfig};
pub use value::{ValueNetwork, ValueNetworkConfig};
