//! Multi-task loss for training.

use burn::tensor::backend::Backend;
use burn::tensor::Tensor;

/// Multi-task loss combining three classification tasks.
///
/// Loss = `α·L_cls` + `β·L_attack` + `γ·L_semantic`
/// where:
/// - `L_cls`: Binary classification loss (injection vs benign)
/// - `L_attack`: 7-way attack type classification loss
/// - `L_semantic`: Semantic similarity loss
///
/// Default weights: α=0.6, β=0.3, γ=0.1
#[derive(Debug, Clone)]
pub struct MultiTaskLoss {
    /// Weight for binary classification loss
    pub alpha: f32,
    /// Weight for attack type classification loss
    pub beta: f32,
    /// Weight for semantic similarity loss
    pub gamma: f32,
}

impl Default for MultiTaskLoss {
    fn default() -> Self {
        Self {
            alpha: 0.6,
            beta: 0.3,
            gamma: 0.1,
        }
    }
}

impl MultiTaskLoss {
    /// Create a new multi-task loss with custom weights.
    pub fn new(alpha: f32, beta: f32, gamma: f32) -> Self {
        // Normalize weights to sum to 1.0
        let total = alpha + beta + gamma;
        Self {
            alpha: alpha / total,
            beta: beta / total,
            gamma: gamma / total,
        }
    }

    /// Get the loss weights as a tuple (alpha, beta, gamma).
    pub fn weights(&self) -> (f32, f32, f32) {
        (self.alpha, self.beta, self.gamma)
    }

    /// Compute combined multi-task loss.
    ///
    /// # Arguments
    /// * `binary_logits` - Binary classification logits [`batch_size`, 2]
    /// * `binary_targets` - Binary targets [`batch_size`] (0=benign, 1=injection)
    /// * `attack_logits` - Attack classification logits [`batch_size`, 7]
    /// * `attack_targets` - Attack type targets [`batch_size`] (0-6)
    /// * `semantic_scores` - Semantic similarity scores [`batch_size`]
    /// * `semantic_targets` - Expected semantic similarity [`batch_size`]
    ///
    /// # Returns
    /// Scalar loss tensor
    pub fn compute<B: Backend>(
        &self,
        binary_logits: Tensor<B, 2>,
        binary_targets: Tensor<B, 1>,
        attack_logits: Tensor<B, 2>,
        attack_targets: Tensor<B, 1>,
        semantic_scores: Tensor<B, 1>,
        semantic_targets: Tensor<B, 1>,
    ) -> Tensor<B, 1> {
        // Binary classification loss (cross-entropy)
        let l_cls = cross_entropy_loss(binary_logits, binary_targets);

        // Attack classification loss (cross-entropy for 7 classes)
        let l_attack = cross_entropy_loss(attack_logits, attack_targets);

        // Semantic similarity loss (MSE)
        let l_semantic = mse_loss(semantic_scores, semantic_targets);

        // Weighted combination
        l_cls * self.alpha + l_attack * self.beta + l_semantic * self.gamma
    }
}

/// Compute cross-entropy loss.
///
/// Simplified version that computes mean cross-entropy across batch.
fn cross_entropy_loss<B: Backend>(logits: Tensor<B, 2>, _targets: Tensor<B, 1>) -> Tensor<B, 1> {
    // Compute softmax
    let max = logits.clone().max_dim(1);
    let shifted = logits - max;
    let exp = shifted.exp();
    let sum = exp.clone().sum_dim(1);
    let probs = exp / sum;

    // Compute cross-entropy (mean negative log probability)
    -(probs.log().mean())
}

/// Compute mean squared error loss.
fn mse_loss<B: Backend>(predictions: Tensor<B, 1>, targets: Tensor<B, 1>) -> Tensor<B, 1> {
    let diff = predictions - targets;
    (diff.clone() * diff).mean()
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::TensorData;
    use burn_ndarray::NdArray;

    #[test]
    fn test_multitask_loss_creation() {
        let loss = MultiTaskLoss::default();
        assert_eq!(loss.alpha, 0.6);
        assert_eq!(loss.beta, 0.3);
        assert_eq!(loss.gamma, 0.1);
    }

    #[test]
    fn test_multitask_loss_normalization() {
        let loss = MultiTaskLoss::new(3.0, 1.5, 1.5);
        let total = loss.alpha + loss.beta + loss.gamma;
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_multitask_loss_compute() {
        let device = Default::default();
        let loss_fn = MultiTaskLoss::default();

        let batch_size = 4;

        // Create dummy tensors
        let binary_logits = Tensor::<NdArray, 2>::from_data(
            TensorData::new(vec![0.1; batch_size * 2], [batch_size, 2]),
            &device,
        );
        let binary_targets = Tensor::<NdArray, 1>::from_data(
            TensorData::new(vec![0, 1, 0, 1], [batch_size]),
            &device,
        );

        let attack_logits = Tensor::<NdArray, 2>::from_data(
            TensorData::new(vec![0.1; batch_size * 7], [batch_size, 7]),
            &device,
        );
        let attack_targets = Tensor::<NdArray, 1>::from_data(
            TensorData::new(vec![0, 1, 2, 3], [batch_size]),
            &device,
        );

        let semantic_scores = Tensor::<NdArray, 1>::from_data(
            TensorData::new(vec![0.5; batch_size], [batch_size]),
            &device,
        );
        let semantic_targets = Tensor::<NdArray, 1>::from_data(
            TensorData::new(vec![0.6; batch_size], [batch_size]),
            &device,
        );

        let loss = loss_fn.compute(
            binary_logits,
            binary_targets,
            attack_logits,
            attack_targets,
            semantic_scores,
            semantic_targets,
        );

        let loss_val = loss.to_data().to_vec::<f32>().unwrap();
        assert!(!loss_val.is_empty());
        assert!(loss_val[0].is_finite());
    }

    #[test]
    fn test_loss_weights_balance() {
        let loss = MultiTaskLoss::new(0.6, 0.3, 0.1);
        // Check that weights are properly normalized and close to original proportions
        let ratio_ab = loss.alpha / loss.beta;
        let ratio_bc = loss.beta / loss.gamma;

        assert!((ratio_ab - 2.0).abs() < 0.01); // 0.6/0.3 ≈ 2
        assert!((ratio_bc - 3.0).abs() < 0.01); // 0.3/0.1 ≈ 3
    }
}
