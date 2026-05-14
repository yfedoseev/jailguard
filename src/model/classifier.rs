//! 7-way attack type classifier for prompt injection detection.

use burn::nn::{Dropout, DropoutConfig, Linear, LinearConfig, Relu};
use burn::prelude::*;
use burn::tensor::backend::Backend;
use burn::tensor::Tensor;

/// Classifies the type of injection attack (7 classes).
///
/// Attack types:
/// 0. Role-play/persona injection
/// 1. Instruction override
/// 2. Context manipulation
/// 3. Output manipulation
/// 4. Encoding/obfuscation
/// 5. Jailbreak patterns
/// 6. Benign (no attack)
#[derive(Module, Debug)]
pub struct AttackClassifier<B: Backend> {
    /// Hidden layer
    hidden: Linear<B>,
    /// Output layer (7 classes)
    output: Linear<B>,
    /// `ReLU` activation
    relu: Relu,
    /// Dropout for regularization
    dropout: Dropout,
}

impl<B: Backend> AttackClassifier<B> {
    /// Forward pass for attack type classification.
    ///
    /// # Arguments
    /// * `x` - Input tensor of shape [`batch_size`, `embed_dim`]
    ///
    /// # Returns
    /// Logits tensor of shape [`batch_size`, 7]
    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let x = self.hidden.forward(x);
        let x = self.relu.forward(x);
        let x = self.dropout.forward(x);
        self.output.forward(x)
    }

    /// Forward pass returning probabilities (softmax applied).
    pub fn forward_probs(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let logits = self.forward(x);
        softmax_dim(logits, 1)
    }
}

/// Configuration for attack type classifier.
#[derive(Debug, Clone)]
pub struct AttackClassifierConfig {
    /// Input dimension (embedding size)
    pub input_dim: usize,
    /// Hidden layer dimension
    pub hidden_dim: usize,
    /// Number of attack classes (always 7)
    pub num_classes: usize,
    /// Dropout rate
    pub dropout: f64,
}

impl AttackClassifierConfig {
    /// Create a new attack classifier configuration.
    pub fn new(input_dim: usize, hidden_dim: usize) -> Self {
        Self {
            input_dim,
            hidden_dim,
            num_classes: 7,
            dropout: 0.1,
        }
    }

    /// Set dropout rate.
    pub fn with_dropout(mut self, dropout: f64) -> Self {
        self.dropout = dropout;
        self
    }

    /// Initialize the attack classifier.
    pub fn init<B: Backend>(&self, device: &B::Device) -> AttackClassifier<B> {
        AttackClassifier {
            hidden: LinearConfig::new(self.input_dim, self.hidden_dim).init(device),
            output: LinearConfig::new(self.hidden_dim, self.num_classes).init(device),
            relu: Relu::new(),
            dropout: DropoutConfig::new(self.dropout).init(),
        }
    }
}

/// Compute softmax along a dimension.
fn softmax_dim<B: Backend>(tensor: Tensor<B, 2>, dim: usize) -> Tensor<B, 2> {
    let max = tensor.clone().max_dim(dim);
    let exp = (tensor - max).exp();
    let sum = exp.clone().sum_dim(dim);
    exp / sum
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::TensorData;
    use burn_ndarray::NdArray;

    #[test]
    fn test_classifier_output_shape() {
        let device = Default::default();
        let config = AttackClassifierConfig::new(256, 128);
        let classifier = config.init::<NdArray>(&device);

        let batch_size = 4;
        let embed_dim = 256;

        let data = vec![1.0; batch_size * embed_dim];
        let x = Tensor::<NdArray, 2>::from_data(
            TensorData::new(data, [batch_size, embed_dim]),
            &device,
        );

        let output = classifier.forward(x);
        assert_eq!(output.shape().dims, [batch_size, 7]);
    }

    #[test]
    fn test_classifier_probabilities() {
        let device = Default::default();
        let config = AttackClassifierConfig::new(256, 128);
        let classifier = config.init::<NdArray>(&device);

        let batch_size = 2;
        let embed_dim = 256;

        let data = vec![1.0; batch_size * embed_dim];
        let x = Tensor::<NdArray, 2>::from_data(
            TensorData::new(data, [batch_size, embed_dim]),
            &device,
        );

        let probs = classifier.forward_probs(x);
        assert_eq!(probs.shape().dims, [batch_size, 7]);

        // Each row should sum to approximately 1.0
        let probs_data = probs.to_data().to_vec::<f32>().unwrap();
        for row in probs_data.chunks(7) {
            let sum: f32 = row.iter().sum();
            assert!((sum - 1.0).abs() < 0.01, "Probabilities don't sum to 1.0");
        }
    }
}
