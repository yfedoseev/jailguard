//! Transformer encoder architecture for prompt injection detection.
//!
//! This module implements a standard transformer encoder with multi-head attention
//! and position-wise feedforward networks, using a Pre-LN (Layer Normalization before)
//! architecture for improved training stability.

mod attention;
mod config;
mod encoder;
mod feedforward;

pub use attention::{MultiHeadAttention, MultiHeadAttentionConfig};
pub use config::TransformerConfig;
pub use encoder::{
    TransformerEncoder, TransformerEncoderBlock, TransformerEncoderBlockConfig,
    TransformerEncoderConfig,
};
pub use feedforward::{PositionWiseFeedForward, PositionWiseFeedForwardConfig};
