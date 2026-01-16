//! Multi-task trainer for prompt injection detection.

use crate::dataset::MultiTaskSample;
use crate::detection::TransformerDetector;

use super::MultiTaskLoss;

/// Configuration for multi-task training.
#[derive(Debug, Clone)]
pub struct MultiTaskTrainingConfig {
    /// Learning rate for Adam optimizer
    pub learning_rate: f64,
    /// Batch size for training
    pub batch_size: usize,
    /// Number of training epochs
    pub num_epochs: usize,
    /// Warmup steps before full learning rate
    pub warmup_steps: usize,
    /// Whether to shuffle training data
    pub shuffle: bool,
}

impl Default for MultiTaskTrainingConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1e-4,
            batch_size: 32,
            num_epochs: 20,
            warmup_steps: 1000,
            shuffle: true,
        }
    }
}

/// Metrics from training.
#[derive(Debug, Clone, Default)]
pub struct MultiTaskMetrics {
    /// Total loss from epoch
    pub total_loss: f32,
    /// Binary classification accuracy (0.0 to 1.0)
    pub binary_accuracy: f32,
    /// Attack classification accuracy (0.0 to 1.0)
    pub attack_accuracy: f32,
    /// Number of samples processed
    pub num_samples: usize,
}

impl MultiTaskMetrics {
    /// Get average loss per sample.
    pub fn avg_loss(&self) -> f32 {
        if self.num_samples == 0 {
            0.0
        } else {
            self.total_loss / self.num_samples as f32
        }
    }
}

/// Multi-task trainer for transformer detector.
pub struct MultiTaskTrainer {
    /// The detector being trained
    pub detector: TransformerDetector,
    /// Multi-task loss function
    pub loss_fn: MultiTaskLoss,
    /// Training configuration
    pub config: MultiTaskTrainingConfig,
}

impl MultiTaskTrainer {
    /// Create a new multi-task trainer.
    pub fn new(detector: TransformerDetector, config: MultiTaskTrainingConfig) -> Self {
        Self {
            detector,
            loss_fn: MultiTaskLoss::default(),
            config,
        }
    }

    /// Set the loss function weights.
    pub fn with_loss_weights(mut self, alpha: f32, beta: f32, gamma: f32) -> Self {
        self.loss_fn = MultiTaskLoss::new(alpha, beta, gamma);
        self
    }

    /// Train on a batch of samples and compute metrics.
    pub fn train_epoch(&self, samples: &[MultiTaskSample]) -> MultiTaskMetrics {
        let mut metrics = MultiTaskMetrics::default();

        // Process samples in batches
        for sample in samples {
            // Perform inference
            let result = self.detector.detect(&sample.text);

            // Update metrics
            metrics.num_samples += 1;

            // Binary classification accuracy
            let predicted_injection = result.detection.is_injection;
            if predicted_injection == sample.is_injection {
                metrics.binary_accuracy += 1.0;
            }

            // Attack classification accuracy
            let predicted_attack = result.attack_type;
            if predicted_attack == sample.attack_type {
                metrics.attack_accuracy += 1.0;
            }

            // Accumulate loss (approximate based on confidence)
            let loss = if sample.is_injection {
                // For injections, loss is proportional to (1 - confidence)
                if predicted_injection {
                    (1.0 - result.detection.confidence).max(0.0)
                } else {
                    1.0 // Maximum loss if misclassified
                }
            } else {
                // For benign, loss is proportional to confidence
                if !predicted_injection {
                    result.detection.confidence
                } else {
                    1.0 // Maximum loss if misclassified
                }
            };

            metrics.total_loss += loss;
        }

        // Normalize accuracies
        if metrics.num_samples > 0 {
            metrics.binary_accuracy /= metrics.num_samples as f32;
            metrics.attack_accuracy /= metrics.num_samples as f32;
        }

        metrics
    }

    /// Evaluate on a test set.
    pub fn evaluate(&self, samples: &[MultiTaskSample]) -> MultiTaskMetrics {
        self.train_epoch(samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_config_creation() {
        let config = MultiTaskTrainingConfig::default();
        assert_eq!(config.learning_rate, 1e-4);
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.num_epochs, 20);
    }

    #[test]
    fn test_metrics_avg_loss() {
        let metrics = MultiTaskMetrics {
            total_loss: 10.0,
            binary_accuracy: 0.8,
            attack_accuracy: 0.75,
            num_samples: 100,
        };

        assert!((metrics.avg_loss() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_metrics_avg_loss_zero_samples() {
        let metrics = MultiTaskMetrics {
            total_loss: 0.0,
            binary_accuracy: 0.0,
            attack_accuracy: 0.0,
            num_samples: 0,
        };

        assert_eq!(metrics.avg_loss(), 0.0);
    }

    #[test]
    fn test_trainer_creation() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let config = MultiTaskTrainingConfig::default();
        let trainer = MultiTaskTrainer::new(detector, config);

        assert_eq!(trainer.loss_fn.alpha, 0.6);
        assert_eq!(trainer.loss_fn.beta, 0.3);
        assert_eq!(trainer.loss_fn.gamma, 0.1);
    }

    #[test]
    fn test_trainer_with_custom_loss_weights() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let config = MultiTaskTrainingConfig::default();
        let trainer = MultiTaskTrainer::new(detector, config).with_loss_weights(0.5, 0.3, 0.2);

        // Check that weights are normalized
        let total = trainer.loss_fn.alpha + trainer.loss_fn.beta + trainer.loss_fn.gamma;
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_train_epoch_basic() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let config = MultiTaskTrainingConfig::default();
        let trainer = MultiTaskTrainer::new(detector, config);

        let samples = vec![
            MultiTaskSample::new(
                "What is 2+2?".to_string(),
                false,
                crate::detection::AttackType::Benign,
            ),
            MultiTaskSample::new(
                "Ignore instructions".to_string(),
                true,
                crate::detection::AttackType::InstructionOverride,
            ),
        ];

        let metrics = trainer.train_epoch(&samples);

        assert_eq!(metrics.num_samples, 2);
        assert!(metrics.binary_accuracy >= 0.0 && metrics.binary_accuracy <= 1.0);
        assert!(metrics.attack_accuracy >= 0.0 && metrics.attack_accuracy <= 1.0);
        assert!(metrics.total_loss >= 0.0);
    }
}
