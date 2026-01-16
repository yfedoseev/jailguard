//! Position-wise feed-forward network.

use burn::nn::{Dropout, DropoutConfig, Gelu, Linear, LinearConfig};
use burn::prelude::*;
use burn::tensor::backend::Backend;
use burn::tensor::Tensor;

/// Position-wise feed-forward network.
///
/// Implements the feedforward sub-layer: Linear -> GELU -> Dropout -> Linear
#[derive(Module, Debug)]
pub struct PositionWiseFeedForward<B: Backend> {
    /// First linear layer (`embed_dim` -> `ff_dim`)
    linear1: Linear<B>,
    /// Second linear layer (`ff_dim` -> `embed_dim`)
    linear2: Linear<B>,
    /// GELU activation
    gelu: Gelu,
    /// Dropout layer
    dropout: Dropout,
}

impl<B: Backend> PositionWiseFeedForward<B> {
    /// Forward pass for position-wise feedforward.
    ///
    /// # Arguments
    /// * `x` - Input tensor of shape [`batch_size`, `seq_len`, `embed_dim`]
    ///
    /// # Returns
    /// Output tensor of shape [`batch_size`, `seq_len`, `embed_dim`]
    pub fn forward(&self, x: Tensor<B, 3>) -> Tensor<B, 3> {
        // First linear transformation: embed_dim -> ff_dim
        let x = self.linear1.forward(x);

        // GELU activation
        let x = self.gelu.forward(x);

        // Dropout
        let x = self.dropout.forward(x);

        // Second linear transformation: ff_dim -> embed_dim
        self.linear2.forward(x)
    }
}

/// Configuration for position-wise feedforward network.
#[derive(Debug, Clone)]
pub struct PositionWiseFeedForwardConfig {
    /// Input/output dimension (embedding dimension)
    pub embed_dim: usize,
    /// Intermediate dimension (typically 4x `embed_dim`)
    pub ff_dim: usize,
    /// Dropout rate
    pub dropout: f64,
}

impl PositionWiseFeedForwardConfig {
    /// Create a new position-wise feedforward configuration.
    pub fn new(embed_dim: usize, ff_dim: usize) -> Self {
        Self {
            embed_dim,
            ff_dim,
            dropout: 0.1,
        }
    }

    /// Set dropout rate.
    pub fn with_dropout(mut self, dropout: f64) -> Self {
        self.dropout = dropout;
        self
    }

    /// Initialize the position-wise feedforward layer.
    pub fn init<B: Backend>(&self, device: &B::Device) -> PositionWiseFeedForward<B> {
        PositionWiseFeedForward {
            linear1: LinearConfig::new(self.embed_dim, self.ff_dim).init(device),
            linear2: LinearConfig::new(self.ff_dim, self.embed_dim).init(device),
            gelu: Gelu::new(),
            dropout: DropoutConfig::new(self.dropout).init(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::TensorData;
    use burn_ndarray::NdArray;

    #[test]
    fn test_feedforward_shapes() {
        let device = Default::default();
        let config = PositionWiseFeedForwardConfig::new(256, 1024);
        let ff = config.init::<NdArray>(&device);

        let batch_size = 2;
        let seq_len = 10;
        let embed_dim = 256;

        let data = vec![1.0; batch_size * seq_len * embed_dim];
        let x = Tensor::<NdArray, 3>::from_data(
            TensorData::new(data, [batch_size, seq_len, embed_dim]),
            &device,
        );

        let output = ff.forward(x);
        assert_eq!(output.shape().dims, [batch_size, seq_len, embed_dim]);
    }

    #[test]
    fn test_feedforward_with_dropout() {
        let device = Default::default();
        let config = PositionWiseFeedForwardConfig::new(256, 1024).with_dropout(0.5);
        let ff = config.init::<NdArray>(&device);

        let batch_size = 2;
        let seq_len = 10;
        let embed_dim = 256;

        let data = vec![1.0; batch_size * seq_len * embed_dim];
        let x = Tensor::<NdArray, 3>::from_data(
            TensorData::new(data, [batch_size, seq_len, embed_dim]),
            &device,
        );

        let output = ff.forward(x);
        assert_eq!(output.shape().dims, [batch_size, seq_len, embed_dim]);
    }
}
