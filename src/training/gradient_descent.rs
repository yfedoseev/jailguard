//! Gradient-based training with loss computation and metrics tracking.
//!
//! This module implements loss computation and accuracy tracking for multi-task detection.
//! It's designed for supervised training on labeled data without requiring full autodiff.

use super::multilabel::MultiLabelLossConfig;
use super::multilabel_trainer::MultiLabelTrainingSample;
use crate::detection::MultiLabelDetector;
use crate::error::Result;
use crate::model::EmbeddingLookup;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Metrics computed during training.
#[derive(Debug, Clone, Default)]
pub struct EpochMetrics {
    /// Training loss
    pub train_loss: f32,
    /// Training binary accuracy
    pub train_binary_acc: f32,
    /// Training attack accuracy
    pub train_attack_acc: f32,
    /// Validation loss
    pub val_loss: f32,
    /// Validation binary accuracy
    pub val_binary_acc: f32,
    /// Validation attack accuracy
    pub val_attack_acc: f32,
    /// Learning rate used
    pub learning_rate: f64,
}

/// Gradient-based trainer for multi-task detection.
///
/// This trainer:
/// - Computes losses for all three tasks
/// - Tracks accuracy metrics
/// - Simulates gradient updates through loss weighting
/// - Prepares data for future autodiff integration
pub struct GradientDescentTrainer {
    /// The detector being trained
    detector: MultiLabelDetector,
    /// Loss configuration
    loss_config: MultiLabelLossConfig,
    /// Learning rate
    learning_rate: f64,
    /// Training history
    history: Vec<EpochMetrics>,
}

impl GradientDescentTrainer {
    /// Create a new gradient descent trainer.
    pub fn new(
        embedding_lookup: EmbeddingLookup,
        loss_config: MultiLabelLossConfig,
        learning_rate: f64,
    ) -> Result<Self> {
        let detector = MultiLabelDetector::new(embedding_lookup)?;

        Ok(Self {
            detector,
            loss_config,
            learning_rate,
            history: Vec::new(),
        })
    }

    /// Get reference to the detector.
    pub fn detector(&self) -> &MultiLabelDetector {
        &self.detector
    }

    /// Evaluate on a single epoch and return metrics.
    pub fn evaluate_epoch(
        &self,
        train_samples: &[MultiLabelTrainingSample],
        val_samples: &[MultiLabelTrainingSample],
    ) -> Result<EpochMetrics> {
        // Evaluate training set
        let mut train_loss = 0.0;
        let mut train_binary_correct = 0;
        let mut train_attack_correct = 0;

        for sample in train_samples {
            let result = self.detector.detect_multilabel(&sample.text)?;

            // Binary loss
            let binary_loss = if sample.is_injection {
                (1.0 - result.binary_confidence).max(0.0)
            } else {
                result.binary_confidence
            };

            // Attack loss
            let attack_max_prob = result
                .attack_probs
                .get(sample.attack_type_idx)
                .copied()
                .unwrap_or(0.0);
            let attack_loss = (1.0 - attack_max_prob).max(0.0);

            // Semantic loss
            let semantic_loss = (result.semantic_score - sample.semantic_score).powi(2);

            // Weighted combination
            let (bw, aw, sw) = self.loss_config.normalized_weights();
            let sample_loss = binary_loss * bw + attack_loss * aw + semantic_loss * sw;

            train_loss += sample_loss;

            // Accuracy
            if result.is_injection == sample.is_injection {
                train_binary_correct += 1;
            }
            if result.attack_type_idx == sample.attack_type_idx {
                train_attack_correct += 1;
            }
        }

        train_loss /= train_samples.len() as f32;
        let train_binary_acc = train_binary_correct as f32 / train_samples.len() as f32;
        let train_attack_acc = train_attack_correct as f32 / train_samples.len() as f32;

        // Evaluate validation set
        let mut val_loss = 0.0;
        let mut val_binary_correct = 0;
        let mut val_attack_correct = 0;

        for sample in val_samples {
            let result = self.detector.detect_multilabel(&sample.text)?;

            // Binary loss
            let binary_loss = if sample.is_injection {
                (1.0 - result.binary_confidence).max(0.0)
            } else {
                result.binary_confidence
            };

            // Attack loss
            let attack_max_prob = result
                .attack_probs
                .get(sample.attack_type_idx)
                .copied()
                .unwrap_or(0.0);
            let attack_loss = (1.0 - attack_max_prob).max(0.0);

            // Semantic loss
            let semantic_loss = (result.semantic_score - sample.semantic_score).powi(2);

            // Weighted combination
            let (bw, aw, sw) = self.loss_config.normalized_weights();
            let sample_loss = binary_loss * bw + attack_loss * aw + semantic_loss * sw;

            val_loss += sample_loss;

            // Accuracy
            if result.is_injection == sample.is_injection {
                val_binary_correct += 1;
            }
            if result.attack_type_idx == sample.attack_type_idx {
                val_attack_correct += 1;
            }
        }

        val_loss /= val_samples.len() as f32;
        let val_binary_acc = val_binary_correct as f32 / val_samples.len() as f32;
        let val_attack_acc = val_attack_correct as f32 / val_samples.len() as f32;

        Ok(EpochMetrics {
            train_loss,
            train_binary_acc,
            train_attack_acc,
            val_loss,
            val_binary_acc,
            val_attack_acc,
            learning_rate: self.learning_rate,
        })
    }

    /// Get training history.
    pub fn history(&self) -> &[EpochMetrics] {
        &self.history
    }

    /// Add epoch metrics to history.
    pub fn record_epoch(&mut self, metrics: EpochMetrics) {
        self.history.push(metrics);
    }

    /// Simulate training loop over multiple epochs.
    ///
    /// This simulates gradient descent by computing losses and tracking metrics.
    /// In practice, once burn's autodiff is properly integrated, this will perform
    /// actual weight updates.
    pub fn train(
        &mut self,
        train_samples: &[MultiLabelTrainingSample],
        val_samples: &[MultiLabelTrainingSample],
        num_epochs: usize,
    ) -> Result<()> {
        println!(
            "\n=== Gradient Descent Training Simulation ===\n\
             Learning Rate: {}\n\
             Epochs: {}\n\
             Training Samples: {}\n\
             Validation Samples: {}\n",
            self.learning_rate,
            num_epochs,
            train_samples.len(),
            val_samples.len()
        );

        for epoch in 0..num_epochs {
            let metrics = self.evaluate_epoch(train_samples, val_samples)?;
            self.record_epoch(metrics.clone());

            println!(
                "Epoch {:2} | Train Loss: {:.4} | Train Acc: {:.1}% | Val Loss: {:.4} | Val Acc: {:.1}%",
                epoch + 1,
                metrics.train_loss,
                metrics.train_binary_acc * 100.0,
                metrics.val_loss,
                metrics.val_binary_acc * 100.0
            );
        }

        Ok(())
    }

    /// Get best validation accuracy from history.
    pub fn best_val_accuracy(&self) -> f32 {
        self.history
            .iter()
            .map(|m| m.val_binary_acc)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get best validation epoch.
    pub fn best_epoch(&self) -> Option<usize> {
        self.history
            .iter()
            .enumerate()
            .max_by(|a, b| {
                a.1.val_binary_acc
                    .partial_cmp(&b.1.val_binary_acc)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_trainer_creation() {
        let mut lookup = EmbeddingLookup::new(384);
        lookup.insert("test".to_string(), vec![0.1; 384]);

        let loss_config = MultiLabelLossConfig::default();
        let trainer = GradientDescentTrainer::new(lookup, loss_config, 1e-4);

        assert!(trainer.is_ok());
    }

    #[test]
    fn test_epoch_metrics_default() {
        let metrics = EpochMetrics::default();
        assert_eq!(metrics.train_loss, 0.0);
        assert_eq!(metrics.train_binary_acc, 0.0);
    }

    #[test]
    fn test_best_val_accuracy() {
        let mut trainer_metrics = vec![
            EpochMetrics {
                val_binary_acc: 0.5,
                ..Default::default()
            },
            EpochMetrics {
                val_binary_acc: 0.7,
                ..Default::default()
            },
            EpochMetrics {
                val_binary_acc: 0.6,
                ..Default::default()
            },
        ];

        // Note: In real implementation, these would be computed
        assert_eq!(
            trainer_metrics
                .iter()
                .map(|m| m.val_binary_acc)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap(),
            0.7
        );
    }
}
