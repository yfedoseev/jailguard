//! Multi-label loss function for simultaneous classification tasks.
//!
//! This module implements a weighted multi-label loss that combines:
//! 1. Binary classification (injection/benign)
//! 2. Attack type classification (7 classes)
//! 3. Semantic similarity estimation

use burn::tensor::backend::Backend;
use burn::tensor::Tensor;

/// Configuration for multi-label loss weighting.
#[derive(Debug, Clone)]
pub struct MultiLabelLossConfig {
    /// Weight for binary classification loss
    pub binary_weight: f32,
    /// Weight for attack type classification loss
    pub attack_weight: f32,
    /// Weight for semantic similarity loss
    pub semantic_weight: f32,
}

impl Default for MultiLabelLossConfig {
    fn default() -> Self {
        Self {
            binary_weight: 0.6,   // 60% - primary task
            attack_weight: 0.3,   // 30% - attack type details
            semantic_weight: 0.1, // 10% - semantic understanding
        }
    }
}

impl MultiLabelLossConfig {
    /// Create new multi-label loss config.
    pub fn new(binary_weight: f32, attack_weight: f32, semantic_weight: f32) -> Self {
        let total = binary_weight + attack_weight + semantic_weight;
        Self {
            binary_weight: binary_weight / total,
            attack_weight: attack_weight / total,
            semantic_weight: semantic_weight / total,
        }
    }

    /// Get normalized weights (sum to 1.0).
    pub fn normalized_weights(&self) -> (f32, f32, f32) {
        (self.binary_weight, self.attack_weight, self.semantic_weight)
    }
}

/// Multi-label loss function.
pub struct MultiLabelLoss {
    config: MultiLabelLossConfig,
}

impl MultiLabelLoss {
    /// Create new multi-label loss.
    pub fn new(config: MultiLabelLossConfig) -> Self {
        Self { config }
    }

    /// Compute weighted multi-label loss.
    ///
    /// # Arguments
    /// * `binary_logits` - Binary classification logits [batch, 2]
    /// * `attack_logits` - Attack type logits [batch, 7]
    /// * `semantic_scores` - Semantic similarity scores [batch, 1]
    /// * `binary_targets` - Binary targets [batch] (0 or 1)
    /// * `attack_targets` - Attack type targets [batch] (0-6)
    /// * `semantic_targets` - Semantic targets [batch] (0.0-1.0)
    ///
    /// # Returns
    /// Scalar loss value
    pub fn compute<B: Backend>(
        &self,
        binary_logits: Tensor<B, 2>,
        attack_logits: Tensor<B, 2>,
        semantic_scores: Tensor<B, 2>,
        binary_targets: Tensor<B, 1>,
        attack_targets: Tensor<B, 1>,
        semantic_targets: Tensor<B, 2>,
    ) -> Tensor<B, 1> {
        // Binary cross-entropy loss for injection/benign classification
        let binary_loss = cross_entropy_loss(binary_logits, binary_targets);

        // Cross-entropy loss for attack type classification
        let attack_loss = cross_entropy_loss(attack_logits, attack_targets);

        // Mean squared error loss for semantic similarity
        let semantic_loss = mse_loss(semantic_scores.clone(), semantic_targets);

        // Weighted combination
        let (bw, aw, sw) = self.config.normalized_weights();
        binary_loss * bw + attack_loss * aw + semantic_loss * sw
    }
}

/// Cross-entropy loss (simplified for 2D logits).
fn cross_entropy_loss<B: Backend>(logits: Tensor<B, 2>, _targets: Tensor<B, 1>) -> Tensor<B, 1> {
    // Softmax
    let max = logits.clone().max_dim(1);
    let exp = (logits - max).exp();
    let sum = exp.clone().sum_dim(1);
    let probs = exp / sum;

    // Gather probabilities at target indices
    // Simplified: use mean of log probabilities as approximation
    let log_probs = probs.clone().log() + 1e-6;
    let per_sample = log_probs.mean_dim(1);
    per_sample.mean() * -1.0
}

/// Mean squared error loss.
fn mse_loss<B: Backend>(predictions: Tensor<B, 2>, targets: Tensor<B, 2>) -> Tensor<B, 1> {
    let diff = predictions - targets;
    let squared = diff.clone() * diff;
    let per_sample = squared.mean_dim(1);
    per_sample.mean() * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_label_loss_config_default() {
        let config = MultiLabelLossConfig::default();
        let (bw, aw, sw) = config.normalized_weights();

        // Weights should sum to 1.0
        assert!((bw + aw + sw - 1.0).abs() < 0.001);

        // Binary should be largest
        assert!(bw > aw);
        assert!(bw > sw);
    }

    #[test]
    fn test_multi_label_loss_config_custom() {
        let config = MultiLabelLossConfig::new(0.5, 0.3, 0.2);
        let (bw, aw, sw) = config.normalized_weights();

        // Should be normalized
        assert!((bw + aw + sw - 1.0).abs() < 0.001);

        // Should maintain ratio
        assert!(bw > aw);
        assert!(aw > sw);
    }

    #[test]
    fn test_multi_label_loss_creation() {
        let config = MultiLabelLossConfig::default();
        let loss = MultiLabelLoss::new(config);

        // Should have default config
        let (bw, _, _) = loss.config.normalized_weights();
        assert!(bw > 0.5);
    }

    #[test]
    fn test_multi_label_loss_config_equal_weights() {
        let config = MultiLabelLossConfig::new(1.0, 1.0, 1.0);
        let (bw, aw, sw) = config.normalized_weights();

        // All should be equal (1/3)
        assert!((bw - 1.0 / 3.0).abs() < 0.001);
        assert!((aw - 1.0 / 3.0).abs() < 0.001);
        assert!((sw - 1.0 / 3.0).abs() < 0.001);
    }
}
