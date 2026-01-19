//! Incremental training from collected feedback for continuous model improvement.

use crate::training::online::feedback_collector::FeedbackCollector;
use crate::training::MultiTaskLoss;

/// Configuration for incremental training.
#[derive(Debug, Clone)]
pub struct IncrementalTrainingConfig {
    /// Learning rate for incremental updates (conservative, typically 1e-5)
    pub learning_rate: f64,
    /// Batch size for incremental training
    pub batch_size: usize,
    /// Number of epochs per incremental update
    pub epochs: usize,
    /// Minimum number of feedback samples before training
    pub min_feedback_samples: usize,
    /// Apply weight decay to prevent catastrophic forgetting
    pub weight_decay: f64,
}

impl Default for IncrementalTrainingConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1e-5,
            batch_size: 8,
            epochs: 1,
            min_feedback_samples: 10,
            weight_decay: 1e-4,
        }
    }
}

/// Metrics for incremental training.
#[derive(Debug, Clone)]
pub struct IncrementalMetrics {
    /// Number of updates performed
    pub num_updates: usize,
    /// Total feedback samples used for training
    pub total_feedback_samples: usize,
    /// Average loss during last update
    pub last_update_loss: f32,
    /// Number of corrections trained on
    pub corrections_trained: usize,
}

impl IncrementalMetrics {
    /// Create new metrics.
    pub fn new() -> Self {
        Self {
            num_updates: 0,
            total_feedback_samples: 0,
            last_update_loss: 0.0,
            corrections_trained: 0,
        }
    }
}

impl Default for IncrementalMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Incremental trainer for continuous model learning from feedback.
#[derive(Debug)]
pub struct IncrementalTrainer {
    /// Feedback collector
    feedback_collector: FeedbackCollector,
    /// Training configuration
    config: IncrementalTrainingConfig,
    /// Training metrics
    metrics: IncrementalMetrics,
    /// Multi-task loss function
    loss_fn: MultiTaskLoss,
}

impl IncrementalTrainer {
    /// Create a new incremental trainer.
    pub fn new(feedback_collector: FeedbackCollector) -> Self {
        Self::with_config(feedback_collector, IncrementalTrainingConfig::default())
    }

    /// Create with custom configuration.
    pub fn with_config(
        feedback_collector: FeedbackCollector,
        config: IncrementalTrainingConfig,
    ) -> Self {
        Self {
            feedback_collector,
            config,
            metrics: IncrementalMetrics::new(),
            loss_fn: MultiTaskLoss::default(),
        }
    }

    /// Check if there's enough feedback to perform an update.
    pub fn should_update(&self) -> bool {
        self.feedback_collector.len() >= self.config.min_feedback_samples
    }

    /// Get the feedback collector.
    pub fn feedback_collector(&self) -> &FeedbackCollector {
        &self.feedback_collector
    }

    /// Get mutable reference to feedback collector.
    pub fn feedback_collector_mut(&mut self) -> &mut FeedbackCollector {
        &mut self.feedback_collector
    }

    /// Perform an incremental training update.
    ///
    /// In a real implementation, this would:
    /// 1. Get training batch from feedback
    /// 2. Forward pass through model
    /// 3. Compute multi-task loss
    /// 4. Backward pass with conservative learning rate
    /// 5. Apply weight decay to prevent forgetting
    ///
    /// For now, this is a structural placeholder that validates the training setup.
    pub fn update_from_feedback(&mut self) -> bool {
        // Check if we have enough feedback
        if !self.should_update() {
            return false;
        }

        // Get training batch
        let batch_size = self.config.batch_size;
        let batch = self.feedback_collector.get_training_batch(batch_size);

        if batch.is_empty() {
            return false;
        }

        // Count corrections in this batch
        let corrections = self
            .feedback_collector
            .all_samples()
            .iter()
            .filter(|s| s.is_correction())
            .count();

        // Simulate training (in real implementation, would do actual forward/backward)
        // For demonstration, we compute a dummy loss
        let dummy_loss = 0.05 * (batch.len() as f32);

        // Update metrics
        self.metrics.num_updates += 1;
        self.metrics.total_feedback_samples += batch.len();
        self.metrics.last_update_loss = dummy_loss;
        self.metrics.corrections_trained += corrections;

        true
    }

    /// Update using only recent corrections (more aggressive on new mistakes).
    pub fn update_from_corrections(&mut self) -> bool {
        let corrections = self
            .feedback_collector
            .get_recent_corrections(self.config.batch_size.min(self.feedback_collector.len()));

        if corrections.is_empty() {
            return false;
        }

        // Simulate training update
        let dummy_loss = 0.03 * (corrections.len() as f32);

        self.metrics.num_updates += 1;
        self.metrics.total_feedback_samples += corrections.len();
        self.metrics.last_update_loss = dummy_loss;
        self.metrics.corrections_trained += corrections.len();

        true
    }

    /// Get training metrics.
    pub fn metrics(&self) -> &IncrementalMetrics {
        &self.metrics
    }

    /// Get mutable reference to metrics.
    pub fn metrics_mut(&mut self) -> &mut IncrementalMetrics {
        &mut self.metrics
    }

    /// Get loss function.
    pub fn loss_fn(&self) -> &MultiTaskLoss {
        &self.loss_fn
    }

    /// Set custom loss weights.
    pub fn set_loss_weights(&mut self, alpha: f32, beta: f32, gamma: f32) {
        self.loss_fn = MultiTaskLoss::new(alpha, beta, gamma);
    }

    /// Reset metrics (useful for session-based tracking).
    pub fn reset_metrics(&mut self) {
        self.metrics = IncrementalMetrics::new();
    }

    /// Clear feedback and metrics.
    pub fn clear(&mut self) {
        self.feedback_collector.clear();
        self.metrics = IncrementalMetrics::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::{AttackType, DetectionResult, MultiTaskDetectionResult};
    use crate::training::online::feedback_collector::FeedbackSample;

    fn create_test_result(confidence: f32, is_injection: bool) -> MultiTaskDetectionResult {
        MultiTaskDetectionResult {
            detection: DetectionResult::new(
                is_injection,
                confidence,
                [
                    if is_injection { 0.9 } else { 0.1 },
                    if is_injection { 0.1 } else { 0.9 },
                ],
            ),
            attack_type: AttackType::RolePlay,
            attack_probs: [0.2; 8],
            semantic_score: 0.5,
            embedding: vec![],
        }
    }

    #[test]
    fn test_incremental_trainer_creation() {
        let collector = FeedbackCollector::new();
        let trainer = IncrementalTrainer::new(collector);

        assert_eq!(trainer.metrics().num_updates, 0);
        assert!(!trainer.should_update());
    }

    #[test]
    fn test_should_update_threshold() {
        let config = IncrementalTrainingConfig {
            min_feedback_samples: 5,
            ..Default::default()
        };
        let collector = FeedbackCollector::new();
        let mut trainer = IncrementalTrainer::with_config(collector, config);

        // Initially should not update
        assert!(!trainer.should_update());

        // Add feedback
        let result = create_test_result(0.8, true);
        for i in 0..5 {
            let sample = FeedbackSample::new(format!("test{}", i), result.clone(), Some(false));
            trainer.feedback_collector_mut().add_feedback(sample);
        }

        // Now should update
        assert!(trainer.should_update());
    }

    #[test]
    fn test_update_from_feedback() {
        let mut collector = FeedbackCollector::new();
        let result = create_test_result(0.8, true);

        for i in 0..10 {
            let sample = FeedbackSample::new(format!("test{}", i), result.clone(), Some(false));
            collector.add_feedback(sample);
        }

        let mut trainer = IncrementalTrainer::new(collector);

        assert!(trainer.should_update());
        assert!(trainer.update_from_feedback());
        assert_eq!(trainer.metrics().num_updates, 1);
    }

    #[test]
    fn test_update_from_corrections() {
        let mut collector = FeedbackCollector::new();
        let result_correct = create_test_result(0.8, true);
        let result_incorrect = create_test_result(0.3, false);

        for i in 0..5 {
            let sample =
                FeedbackSample::new(format!("correct{}", i), result_correct.clone(), Some(true));
            collector.add_feedback(sample);
        }

        for i in 0..5 {
            let sample = FeedbackSample::new(
                format!("incorrect{}", i),
                result_incorrect.clone(),
                Some(true),
            );
            collector.add_feedback(sample);
        }

        let mut trainer = IncrementalTrainer::new(collector);

        // Update should work
        let success = trainer.update_from_corrections();
        assert!(success);
        assert!(trainer.metrics().corrections_trained > 0);
    }

    #[test]
    fn test_metrics_tracking() {
        let mut collector = FeedbackCollector::new();
        let result = create_test_result(0.8, true);

        for i in 0..10 {
            let sample = FeedbackSample::new(format!("test{}", i), result.clone(), Some(false));
            collector.add_feedback(sample);
        }

        let mut trainer = IncrementalTrainer::new(collector);

        let initial_updates = trainer.metrics().num_updates;
        trainer.update_from_feedback();

        assert_eq!(trainer.metrics().num_updates, initial_updates + 1);
        assert!(trainer.metrics().last_update_loss > 0.0);
    }

    #[test]
    fn test_loss_weight_customization() {
        let collector = FeedbackCollector::new();
        let mut trainer = IncrementalTrainer::new(collector);

        trainer.set_loss_weights(0.5, 0.3, 0.2);

        let weights = trainer.loss_fn().weights();
        assert!((weights.0 - 0.5).abs() < 0.01);
        assert!((weights.1 - 0.3).abs() < 0.01);
        assert!((weights.2 - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_reset_metrics() {
        let mut collector = FeedbackCollector::new();
        let result = create_test_result(0.8, true);

        for i in 0..10 {
            let sample = FeedbackSample::new(format!("test{}", i), result.clone(), Some(false));
            collector.add_feedback(sample);
        }

        let mut trainer = IncrementalTrainer::new(collector);
        trainer.update_from_feedback();

        assert!(trainer.metrics().num_updates > 0);

        trainer.reset_metrics();
        assert_eq!(trainer.metrics().num_updates, 0);
    }

    #[test]
    fn test_clear() {
        let mut collector = FeedbackCollector::new();
        let result = create_test_result(0.8, true);

        for i in 0..10 {
            let sample = FeedbackSample::new(format!("test{}", i), result.clone(), Some(false));
            collector.add_feedback(sample);
        }

        let mut trainer = IncrementalTrainer::new(collector);
        trainer.update_from_feedback();

        assert!(!trainer.feedback_collector().is_empty());
        assert!(trainer.metrics().num_updates > 0);

        trainer.clear();
        assert!(trainer.feedback_collector().is_empty());
        assert_eq!(trainer.metrics().num_updates, 0);
    }

    #[test]
    fn test_config_customization() {
        let config = IncrementalTrainingConfig {
            learning_rate: 5e-5,
            batch_size: 16,
            epochs: 2,
            min_feedback_samples: 20,
            weight_decay: 2e-4,
        };

        let collector = FeedbackCollector::new();
        let trainer = IncrementalTrainer::with_config(collector, config);

        assert!((trainer.config.learning_rate - 5e-5).abs() < 1e-6);
        assert_eq!(trainer.config.batch_size, 16);
        assert_eq!(trainer.config.epochs, 2);
    }
}
