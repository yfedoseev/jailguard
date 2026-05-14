//! Trainable detection heads with manual gradient computation.
//!
//! This module implements simple neural network layers that can be trained
//! without requiring full autodiff support. It uses manual gradient computation
//! and SGD-style weight updates.

use crate::error::Result;

/// A simple trainable linear layer for classification.
#[derive(Debug, Clone)]
pub struct TrainableLinearHead {
    /// Weight matrix: input_dim × output_dim
    pub weights: Vec<Vec<f32>>,
    /// Bias vector: output_dim
    pub bias: Vec<f32>,
    /// Learning rate for weight updates
    pub learning_rate: f32,
    /// Gradient accumulator for weights
    weight_gradients: Vec<Vec<f32>>,
    /// Gradient accumulator for bias
    bias_gradients: Vec<f32>,
    /// Number of weight updates applied
    updates_count: usize,
}

impl TrainableLinearHead {
    /// Create a new trainable linear head.
    pub fn new(input_dim: usize, output_dim: usize, learning_rate: f32) -> Self {
        // Initialize weights with small random values
        let weights: Vec<Vec<f32>> = (0..input_dim)
            .map(|_| {
                (0..output_dim)
                    .map(|_| {
                        // Xavier initialization: std = 1.0 / sqrt(fan_in)
                        let scale = 1.0 / (input_dim as f32).sqrt();
                        (rand::random::<f32>() - 0.5) * 2.0 * scale
                    })
                    .collect()
            })
            .collect();

        let bias = vec![0.0; output_dim];
        let weight_gradients = vec![vec![0.0; output_dim]; input_dim];
        let bias_gradients = vec![0.0; output_dim];

        Self {
            weights,
            bias,
            learning_rate,
            weight_gradients,
            bias_gradients,
            updates_count: 0,
        }
    }

    /// Forward pass: compute output = input @ weights + bias.
    pub fn forward(&self, input: &[f32]) -> Result<Vec<f32>> {
        let output_dim = self.bias.len();
        let mut output = self.bias.clone();

        // Matrix-vector multiplication
        for j in 0..output_dim {
            let mut sum = 0.0;
            for i in 0..input.len() {
                sum += input[i] * self.weights[i][j];
            }
            output[j] += sum;
        }

        Ok(output)
    }

    /// Compute softmax probabilities from logits.
    pub fn softmax(logits: &[f32]) -> Vec<f32> {
        let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let exps: Vec<f32> = logits.iter().map(|&x| (x - max).exp()).collect();
        let sum: f32 = exps.iter().sum();
        exps.iter().map(|&x| x / sum).collect()
    }

    /// Compute cross-entropy loss and gradient.
    pub fn cross_entropy_loss_and_grad(logits: &[f32], target_idx: usize) -> (f32, Vec<f32>) {
        let probs = Self::softmax(logits);
        let target_prob = probs[target_idx].max(1e-10);
        let loss = -target_prob.ln();

        // Gradient of cross-entropy w.r.t. softmax input
        let mut grad = probs.clone();
        grad[target_idx] -= 1.0;

        (loss, grad)
    }

    /// Accumulate gradients for a single sample.
    pub fn accumulate_gradients(&mut self, input: &[f32], output_grad: &[f32]) -> Result<()> {
        let input_dim = input.len();
        let output_dim = output_grad.len();

        // Gradient w.r.t. weights: outer product of input and output_grad
        for i in 0..input_dim {
            for j in 0..output_dim {
                self.weight_gradients[i][j] += input[i] * output_grad[j];
            }
        }

        // Gradient w.r.t. bias: just the output gradient
        for j in 0..output_dim {
            self.bias_gradients[j] += output_grad[j];
        }

        Ok(())
    }

    /// Apply accumulated gradients and reset for next batch.
    pub fn apply_gradients(&mut self, batch_size: usize) -> Result<()> {
        let scale = self.learning_rate / batch_size as f32;

        let input_dim = self.weights.len();
        let output_dim = self.weights[0].len();

        // Update weights using SGD
        for i in 0..input_dim {
            for j in 0..output_dim {
                self.weights[i][j] -= scale * self.weight_gradients[i][j];
                self.weight_gradients[i][j] = 0.0;
            }
        }

        // Update bias
        for j in 0..output_dim {
            self.bias[j] -= scale * self.bias_gradients[j];
            self.bias_gradients[j] = 0.0;
        }

        self.updates_count += 1;
        Ok(())
    }

    /// Get the number of weight updates applied.
    pub fn updates_count(&self) -> usize {
        self.updates_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_head_creation() {
        let head = TrainableLinearHead::new(384, 2, 0.01);
        assert_eq!(head.weights.len(), 384);
        assert_eq!(head.weights[0].len(), 2);
        assert_eq!(head.bias.len(), 2);
    }

    #[test]
    fn test_forward_pass() {
        let head = TrainableLinearHead::new(10, 5, 0.01);
        let input = vec![0.1; 10];
        let output = head.forward(&input).unwrap();
        assert_eq!(output.len(), 5);
    }

    #[test]
    fn test_softmax() {
        let logits = vec![1.0, 2.0, 3.0];
        let probs = TrainableLinearHead::softmax(&logits);

        // Sum should be close to 1.0
        let sum: f32 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);

        // All probabilities should be in [0, 1]
        for p in probs {
            assert!(p >= 0.0 && p <= 1.0);
        }
    }

    #[test]
    fn test_cross_entropy_loss() {
        let logits = vec![1.0, 2.0, 3.0];
        let (loss, grad) = TrainableLinearHead::cross_entropy_loss_and_grad(&logits, 2);

        // Loss should be positive
        assert!(loss > 0.0);

        // Gradient should have same length as logits
        assert_eq!(grad.len(), logits.len());
    }

    #[test]
    fn test_gradient_accumulation() {
        let mut head = TrainableLinearHead::new(10, 5, 0.01);
        let input = vec![0.1; 10];
        let output_grad = vec![0.01; 5];

        head.accumulate_gradients(&input, &output_grad).unwrap();
        head.apply_gradients(1).unwrap();

        assert_eq!(head.updates_count(), 1);
    }
}
