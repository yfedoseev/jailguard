//! Value network for PPO advantage estimation.

use burn::nn::{Linear, LinearConfig, Relu};
use burn::prelude::*;
use burn::tensor::{backend::Backend, Tensor};

/// Value network that estimates state values for PPO.
///
/// Architecture:
/// - Two hidden layers with ReLU activation
/// - Single scalar value output
#[derive(Module, Debug)]
pub struct ValueNetwork<B: Backend> {
    /// First hidden layer
    hidden1: Linear<B>,
    /// Second hidden layer
    hidden2: Linear<B>,
    /// Value output layer
    value_head: Linear<B>,
    /// ReLU activation
    relu: Relu,
}

/// Configuration for the value network.
#[derive(Debug, Clone)]
pub struct ValueNetworkConfig {
    /// Input dimension (embedding size)
    pub input_dim: usize,
    /// Hidden layer dimension
    pub hidden_dim: usize,
}

impl ValueNetworkConfig {
    /// Create a new configuration.
    pub fn new(input_dim: usize, hidden_dim: usize) -> Self {
        Self {
            input_dim,
            hidden_dim,
        }
    }

    /// Initialize the value network.
    pub fn init<B: Backend>(&self, device: &B::Device) -> ValueNetwork<B> {
        let hidden1 = LinearConfig::new(self.input_dim, self.hidden_dim).init(device);
        let hidden2 = LinearConfig::new(self.hidden_dim, self.hidden_dim).init(device);
        let value_head = LinearConfig::new(self.hidden_dim, 1).init(device);
        let relu = Relu::new();

        ValueNetwork {
            hidden1,
            hidden2,
            value_head,
            relu,
        }
    }
}

impl<B: Backend> ValueNetwork<B> {
    /// Forward pass: estimate state value.
    ///
    /// # Arguments
    /// * `x` - Pooled embedding tensor of shape [batch_size, embed_dim]
    ///
    /// # Returns
    /// Value tensor of shape [batch_size, 1]
    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        // First hidden layer with ReLU
        let x = self.hidden1.forward(x);
        let x = self.relu.forward(x);

        // Second hidden layer with ReLU
        let x = self.hidden2.forward(x);
        let x = self.relu.forward(x);

        // Value head (no activation - can be negative)
        self.value_head.forward(x)
    }

    /// Get scalar value from batch.
    ///
    /// # Arguments
    /// * `x` - Pooled embedding tensor of shape [batch_size, embed_dim]
    ///
    /// # Returns
    /// Value tensor of shape [batch_size]
    pub fn forward_scalar(&self, x: Tensor<B, 2>) -> Tensor<B, 1> {
        self.forward(x).squeeze()
    }
}
