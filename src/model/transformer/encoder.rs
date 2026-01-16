//! Transformer encoder with multi-head attention and feedforward layers.

use super::attention::{MultiHeadAttention, MultiHeadAttentionConfig};
use super::feedforward::{PositionWiseFeedForward, PositionWiseFeedForwardConfig};
use burn::nn::{Dropout, DropoutConfig, LayerNorm, LayerNormConfig};
use burn::prelude::*;
use burn::tensor::backend::Backend;
use burn::tensor::Tensor;

/// A single transformer encoder block.
///
/// Implements: `LayerNorm` -> `MultiHeadAttention` -> Residual -> `LayerNorm` -> `FeedForward` -> Residual
#[derive(Module, Debug)]
pub struct TransformerEncoderBlock<B: Backend> {
    /// Layer normalization before attention
    norm1: LayerNorm<B>,
    /// Multi-head attention
    attention: MultiHeadAttention<B>,
    /// Layer normalization before feedforward
    norm2: LayerNorm<B>,
    /// Position-wise feedforward
    feedforward: PositionWiseFeedForward<B>,
    /// Dropout for residual connections
    dropout: Dropout,
}

impl<B: Backend> TransformerEncoderBlock<B> {
    /// Forward pass for a single encoder block (Pre-LN architecture).
    ///
    /// # Arguments
    /// * `x` - Input tensor of shape [`batch_size`, `seq_len`, `embed_dim`]
    /// * `mask` - Optional attention mask
    ///
    /// # Returns
    /// Output tensor of shape [`batch_size`, `seq_len`, `embed_dim`]
    pub fn forward(&self, x: Tensor<B, 3>, mask: Option<Tensor<B, 4>>) -> Tensor<B, 3> {
        // Self-attention with pre-LN
        let norm_x = self.norm1.forward(x.clone());
        let attn_out = self.attention.forward_self(norm_x, mask);
        let attn_out = self.dropout.forward(attn_out);
        let x = x + attn_out;

        // Feedforward with pre-LN
        let norm_x = self.norm2.forward(x.clone());
        let ff_out = self.feedforward.forward(norm_x);
        let ff_out = self.dropout.forward(ff_out);
        x + ff_out
    }
}

/// Configuration for a transformer encoder block.
#[derive(Debug, Clone)]
pub struct TransformerEncoderBlockConfig {
    /// Embedding dimension
    pub embed_dim: usize,
    /// Number of attention heads
    pub num_heads: usize,
    /// Feedforward dimension
    pub ff_dim: usize,
    /// Dropout rate
    pub dropout: f64,
}

impl TransformerEncoderBlockConfig {
    /// Create a new encoder block configuration.
    pub fn new(embed_dim: usize, num_heads: usize, ff_dim: usize) -> Self {
        Self {
            embed_dim,
            num_heads,
            ff_dim,
            dropout: 0.1,
        }
    }

    /// Set dropout rate.
    pub fn with_dropout(mut self, dropout: f64) -> Self {
        self.dropout = dropout;
        self
    }

    /// Initialize the encoder block.
    pub fn init<B: Backend>(&self, device: &B::Device) -> TransformerEncoderBlock<B> {
        TransformerEncoderBlock {
            norm1: LayerNormConfig::new(self.embed_dim).init(device),
            attention: MultiHeadAttentionConfig::new(self.embed_dim, self.num_heads).init(device),
            norm2: LayerNormConfig::new(self.embed_dim).init(device),
            feedforward: PositionWiseFeedForwardConfig::new(self.embed_dim, self.ff_dim)
                .with_dropout(self.dropout)
                .init(device),
            dropout: DropoutConfig::new(self.dropout).init(),
        }
    }
}

/// Complete transformer encoder with multiple stacked blocks.
#[derive(Module, Debug)]
pub struct TransformerEncoder<B: Backend> {
    /// Stack of encoder blocks
    blocks: Vec<TransformerEncoderBlock<B>>,
    /// Number of layers
    num_layers: usize,
}

impl<B: Backend> TransformerEncoder<B> {
    /// Forward pass through all encoder blocks.
    ///
    /// # Arguments
    /// * `x` - Input tensor of shape [`batch_size`, `seq_len`, `embed_dim`]
    /// * `mask` - Optional attention mask
    ///
    /// # Returns
    /// Output tensor of shape [`batch_size`, `seq_len`, `embed_dim`]
    #[allow(clippy::needless_pass_by_value)]
    pub fn forward(&self, mut x: Tensor<B, 3>, mask: Option<Tensor<B, 4>>) -> Tensor<B, 3> {
        for block in &self.blocks {
            x = block.forward(x, mask.clone());
        }
        x
    }
}

/// Configuration for the complete transformer encoder.
#[derive(Debug, Clone)]
pub struct TransformerEncoderConfig {
    /// Embedding dimension
    pub embed_dim: usize,
    /// Number of attention heads
    pub num_heads: usize,
    /// Feedforward dimension
    pub ff_dim: usize,
    /// Number of encoder layers
    pub num_layers: usize,
    /// Dropout rate
    pub dropout: f64,
}

impl TransformerEncoderConfig {
    /// Create a new transformer encoder configuration.
    pub fn new(embed_dim: usize, num_heads: usize, ff_dim: usize, num_layers: usize) -> Self {
        Self {
            embed_dim,
            num_heads,
            ff_dim,
            num_layers,
            dropout: 0.1,
        }
    }

    /// Set dropout rate.
    pub fn with_dropout(mut self, dropout: f64) -> Self {
        self.dropout = dropout;
        self
    }

    /// Initialize the transformer encoder.
    pub fn init<B: Backend>(&self, device: &B::Device) -> TransformerEncoder<B> {
        let block_config =
            TransformerEncoderBlockConfig::new(self.embed_dim, self.num_heads, self.ff_dim)
                .with_dropout(self.dropout);

        let blocks = (0..self.num_layers)
            .map(|_| block_config.clone().init(device))
            .collect();

        TransformerEncoder {
            blocks,
            num_layers: self.num_layers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::TensorData;
    use burn_ndarray::NdArray;

    #[test]
    fn test_encoder_block_shapes() {
        let device = Default::default();
        let config = TransformerEncoderBlockConfig::new(256, 4, 1024);
        let block = config.init::<NdArray>(&device);

        let batch_size = 2;
        let seq_len = 10;
        let embed_dim = 256;

        let data = vec![1.0; batch_size * seq_len * embed_dim];
        let x = Tensor::<NdArray, 3>::from_data(
            TensorData::new(data, [batch_size, seq_len, embed_dim]),
            &device,
        );

        let output = block.forward(x, None);
        assert_eq!(output.shape().dims, [batch_size, seq_len, embed_dim]);
    }

    #[test]
    fn test_encoder_stack_shapes() {
        let device = Default::default();
        let config = TransformerEncoderConfig::new(256, 4, 1024, 3);
        let encoder = config.init::<NdArray>(&device);

        let batch_size = 2;
        let seq_len = 10;
        let embed_dim = 256;

        let data = vec![1.0; batch_size * seq_len * embed_dim];
        let x = Tensor::<NdArray, 3>::from_data(
            TensorData::new(data, [batch_size, seq_len, embed_dim]),
            &device,
        );

        let output = encoder.forward(x, None);
        assert_eq!(output.shape().dims, [batch_size, seq_len, embed_dim]);
    }

    #[test]
    fn test_encoder_with_different_layers() {
        let device = Default::default();

        for num_layers in [1, 2, 3, 6] {
            let config = TransformerEncoderConfig::new(256, 4, 1024, num_layers);
            let encoder = config.init::<NdArray>(&device);

            let batch_size = 2;
            let seq_len = 10;
            let embed_dim = 256;

            let data = vec![1.0; batch_size * seq_len * embed_dim];
            let x = Tensor::<NdArray, 3>::from_data(
                TensorData::new(data, [batch_size, seq_len, embed_dim]),
                &device,
            );

            let output = encoder.forward(x, None);
            assert_eq!(output.shape().dims, [batch_size, seq_len, embed_dim]);
        }
    }
}
