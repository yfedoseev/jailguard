//! Policy network for action selection in RL.

use burn::nn::{Linear, LinearConfig, Relu};
use burn::prelude::*;
use burn::tensor::{backend::Backend, Tensor};

/// Policy network that outputs action probabilities.
///
/// Architecture:
/// - Mean pooling over sequence
/// - Two hidden layers with `ReLU` activation
/// - Softmax output for action probabilities
#[derive(Module, Debug)]
pub struct PolicyNetwork<B: Backend> {
    /// First hidden layer
    hidden1: Linear<B>,
    /// Second hidden layer
    hidden2: Linear<B>,
    /// Action output layer
    action_head: Linear<B>,
    /// `ReLU` activation
    relu: Relu,
}

/// Configuration for the policy network.
#[derive(Debug, Clone)]
pub struct PolicyNetworkConfig {
    /// Input dimension (embedding size)
    pub input_dim: usize,
    /// Hidden layer dimension
    pub hidden_dim: usize,
    /// Number of actions (2 for Block/Allow)
    pub action_dim: usize,
}

impl PolicyNetworkConfig {
    /// Create a new configuration.
    pub fn new(input_dim: usize, hidden_dim: usize) -> Self {
        Self {
            input_dim,
            hidden_dim,
            action_dim: 2,
        }
    }

    /// Initialize the policy network.
    pub fn init<B: Backend>(&self, device: &B::Device) -> PolicyNetwork<B> {
        let hidden1 = LinearConfig::new(self.input_dim, self.hidden_dim).init(device);
        let hidden2 = LinearConfig::new(self.hidden_dim, self.hidden_dim).init(device);
        let action_head = LinearConfig::new(self.hidden_dim, self.action_dim).init(device);
        let relu = Relu::new();

        PolicyNetwork {
            hidden1,
            hidden2,
            action_head,
            relu,
        }
    }
}

impl<B: Backend> PolicyNetwork<B> {
    /// Forward pass: compute action probabilities.
    ///
    /// # Arguments
    /// * `x` - Pooled embedding tensor of shape [`batch_size`, `embed_dim`]
    ///
    /// # Returns
    /// Action probabilities tensor of shape [`batch_size`, `action_dim`]
    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        // First hidden layer with ReLU
        let x = self.hidden1.forward(x);
        let x = self.relu.forward(x);

        // Second hidden layer with ReLU
        let x = self.hidden2.forward(x);
        let x = self.relu.forward(x);

        // Action head with softmax
        let logits = self.action_head.forward(x);
        softmax(logits, 1)
    }

    /// Get action log probabilities (for PPO loss calculation).
    ///
    /// # Arguments
    /// * `x` - Pooled embedding tensor of shape [`batch_size`, `embed_dim`]
    ///
    /// # Returns
    /// Log probabilities tensor of shape [`batch_size`, `action_dim`]
    pub fn log_probs(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        // First hidden layer with ReLU
        let x = self.hidden1.forward(x);
        let x = self.relu.forward(x);

        // Second hidden layer with ReLU
        let x = self.hidden2.forward(x);
        let x = self.relu.forward(x);

        // Action head with log softmax
        let logits = self.action_head.forward(x);
        log_softmax(logits, 1)
    }
}

/// Compute softmax along a dimension.
fn softmax<B: Backend, const D: usize>(tensor: Tensor<B, D>, dim: usize) -> Tensor<B, D> {
    let max = tensor.clone().max_dim(dim);
    let exp = (tensor - max).exp();
    let sum = exp.clone().sum_dim(dim);
    exp / sum
}

/// Compute log softmax along a dimension.
fn log_softmax<B: Backend, const D: usize>(tensor: Tensor<B, D>, dim: usize) -> Tensor<B, D> {
    let max = tensor.clone().max_dim(dim);
    let shifted = tensor - max.clone();
    let log_sum_exp = shifted.clone().exp().sum_dim(dim).log();
    shifted - log_sum_exp
}
