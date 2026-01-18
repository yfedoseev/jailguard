//! Early stopping mechanism to prevent overfitting during training.
//!
//! Monitors validation metrics and stops training when no improvement is observed
//! for a specified number of steps (patience).

/// Early stopping configuration
#[derive(Debug, Clone)]
pub struct EarlyStoppingConfig {
    /// Number of evaluations without improvement before stopping (default: 3)
    pub patience: usize,
    /// Minimum relative improvement to count as improvement (default: 0.001 = 0.1%)
    pub min_delta: f32,
    /// Whether to restore best model on stop (default: true)
    pub restore_best: bool,
}

impl Default for EarlyStoppingConfig {
    fn default() -> Self {
        Self {
            patience: 3,
            min_delta: 0.001,
            restore_best: true,
        }
    }
}

impl EarlyStoppingConfig {
    /// Create with custom patience
    pub fn with_patience(mut self, patience: usize) -> Self {
        self.patience = patience;
        self
    }

    /// Create with custom min delta
    pub fn with_min_delta(mut self, min_delta: f32) -> Self {
        self.min_delta = min_delta;
        self
    }

    /// Create with custom restore best setting
    pub fn with_restore_best(mut self, restore: bool) -> Self {
        self.restore_best = restore;
        self
    }
}

/// Early stopper that monitors validation loss
#[derive(Debug, Clone)]
pub struct EarlyStopper {
    /// Best validation loss observed so far
    pub best_loss: f32,
    /// Current patience counter
    pub patience_counter: usize,
    /// Configuration
    config: EarlyStoppingConfig,
    /// Number of evaluations performed
    pub num_evals: usize,
    /// Step number when best loss was achieved
    pub best_step: usize,
}

impl EarlyStopper {
    /// Create a new early stopper
    pub fn new(config: EarlyStoppingConfig) -> Self {
        Self {
            best_loss: f32::MAX,
            patience_counter: 0,
            config,
            num_evals: 0,
            best_step: 0,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(EarlyStoppingConfig::default())
    }

    /// Check if training should stop
    ///
    /// Returns true if patience limit exceeded, false otherwise.
    pub fn should_stop(&mut self, val_loss: f32, step: usize) -> bool {
        self.num_evals += 1;

        // Check if loss improved
        let improvement_threshold = self.best_loss * (1.0 - self.config.min_delta);
        if val_loss < improvement_threshold {
            // New best loss found
            self.best_loss = val_loss;
            self.patience_counter = 0;
            self.best_step = step;
            false
        } else {
            // No improvement
            self.patience_counter += 1;
            self.patience_counter >= self.config.patience
        }
    }

    /// Get the number of steps without improvement
    pub fn steps_without_improvement(&self) -> usize {
        self.patience_counter
    }

    /// Get the best loss value
    pub fn best_loss(&self) -> f32 {
        self.best_loss
    }

    /// Reset the early stopper
    pub fn reset(&mut self) {
        self.best_loss = f32::MAX;
        self.patience_counter = 0;
        self.num_evals = 0;
        self.best_step = 0;
    }

    /// Check if patience limit is reached
    pub fn is_exhausted(&self) -> bool {
        self.patience_counter >= self.config.patience
    }

    /// Get configuration
    pub fn config(&self) -> &EarlyStoppingConfig {
        &self.config
    }
}

/// Checkpoint data for model saving/loading
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// Step number when checkpoint was saved
    pub step: usize,
    /// Validation loss at checkpoint
    pub val_loss: f32,
    /// Validation accuracy at checkpoint
    pub val_accuracy: f32,
    /// Training loss at checkpoint
    pub train_loss: f32,
    /// Training accuracy at checkpoint
    pub train_accuracy: f32,
    /// Epoch number
    pub epoch: usize,
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(
        step: usize,
        epoch: usize,
        val_loss: f32,
        val_accuracy: f32,
        train_loss: f32,
        train_accuracy: f32,
    ) -> Self {
        Self {
            step,
            val_loss,
            val_accuracy,
            train_loss,
            train_accuracy,
            epoch,
        }
    }
}

/// Checkpoint manager for saving best models
#[derive(Debug, Clone)]
pub struct CheckpointManager {
    /// Current best checkpoint
    best_checkpoint: Option<Checkpoint>,
    /// Checkpoints history
    history: Vec<Checkpoint>,
    /// Maximum checkpoints to keep
    max_checkpoints: usize,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(max_checkpoints: usize) -> Self {
        Self {
            best_checkpoint: None,
            history: Vec::new(),
            max_checkpoints,
        }
    }

    /// Create with default settings (keep 5 checkpoints)
    pub fn default() -> Self {
        Self::new(5)
    }

    /// Save a checkpoint if it's better than the best
    pub fn save_if_best(&mut self, checkpoint: Checkpoint) -> bool {
        let is_best = if let Some(ref best) = self.best_checkpoint {
            checkpoint.val_loss < best.val_loss
        } else {
            true
        };

        if is_best {
            self.best_checkpoint = Some(checkpoint.clone());
        }

        self.history.push(checkpoint);

        // Keep only the best N checkpoints
        if self.history.len() > self.max_checkpoints {
            self.history
                .sort_by(|a, b| a.val_loss.partial_cmp(&b.val_loss).unwrap());
            self.history.truncate(self.max_checkpoints);
        }

        is_best
    }

    /// Get the best checkpoint
    pub fn best(&self) -> Option<&Checkpoint> {
        self.best_checkpoint.as_ref()
    }

    /// Get checkpoint history
    pub fn history(&self) -> &[Checkpoint] {
        &self.history
    }

    /// Get number of checkpoints
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Clear all checkpoints
    pub fn clear(&mut self) {
        self.best_checkpoint = None;
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_early_stopper_creation() {
        let stopper = EarlyStopper::default();
        assert_eq!(stopper.best_loss, f32::MAX);
        assert_eq!(stopper.patience_counter, 0);
        assert_eq!(stopper.num_evals, 0);
    }

    #[test]
    fn test_early_stopper_improvement() {
        let mut stopper = EarlyStopper::default();

        let should_stop1 = stopper.should_stop(0.5, 0);
        assert!(!should_stop1);
        assert_eq!(stopper.best_loss, 0.5);
        assert_eq!(stopper.patience_counter, 0);

        let should_stop2 = stopper.should_stop(0.4, 1);
        assert!(!should_stop2);
        assert_eq!(stopper.best_loss, 0.4);
        assert_eq!(stopper.patience_counter, 0);
    }

    #[test]
    fn test_early_stopper_no_improvement() {
        let mut stopper = EarlyStopper::default();

        stopper.should_stop(0.5, 0);

        let should_stop1 = stopper.should_stop(0.5, 1);
        assert!(!should_stop1);
        assert_eq!(stopper.patience_counter, 1);

        let should_stop2 = stopper.should_stop(0.5, 2);
        assert!(!should_stop2);
        assert_eq!(stopper.patience_counter, 2);

        let should_stop3 = stopper.should_stop(0.5, 3);
        assert!(should_stop3);
        assert_eq!(stopper.patience_counter, 3);
    }

    #[test]
    fn test_early_stopper_min_delta() {
        let config = EarlyStoppingConfig::default().with_min_delta(0.01); // 1%
        let mut stopper = EarlyStopper::new(config);

        stopper.should_stop(1.0, 0);

        // 0.991 is 0.9% improvement, below 1% threshold
        let should_stop = stopper.should_stop(0.991, 1);
        assert!(!should_stop);
        assert_eq!(stopper.patience_counter, 1); // Still counts as no improvement

        // 0.98 is 2% improvement, above 1% threshold
        let should_stop = stopper.should_stop(0.98, 2);
        assert!(!should_stop);
        assert_eq!(stopper.patience_counter, 0); // Reset on significant improvement
    }

    #[test]
    fn test_early_stopper_custom_patience() {
        let config = EarlyStoppingConfig::default().with_patience(5);
        let mut stopper = EarlyStopper::new(config);

        stopper.should_stop(0.5, 0);

        for i in 1..5 {
            let should_stop = stopper.should_stop(0.5, i);
            assert!(!should_stop);
        }

        let should_stop = stopper.should_stop(0.5, 5);
        assert!(should_stop);
    }

    #[test]
    fn test_early_stopper_reset() {
        let mut stopper = EarlyStopper::default();

        stopper.should_stop(0.5, 0);
        assert_eq!(stopper.best_loss, 0.5);

        stopper.reset();
        assert_eq!(stopper.best_loss, f32::MAX);
        assert_eq!(stopper.patience_counter, 0);
    }

    #[test]
    fn test_checkpoint_creation() {
        let checkpoint = Checkpoint::new(0, 0, 0.5, 0.8, 0.6, 0.75);

        assert_eq!(checkpoint.step, 0);
        assert_eq!(checkpoint.val_loss, 0.5);
        assert_eq!(checkpoint.val_accuracy, 0.8);
        assert_eq!(checkpoint.epoch, 0);
    }

    #[test]
    fn test_checkpoint_manager_best() {
        let mut manager = CheckpointManager::default();

        let cp1 = Checkpoint::new(0, 0, 0.5, 0.8, 0.6, 0.75);
        manager.save_if_best(cp1);

        assert!(manager.best().is_some());
        assert_eq!(manager.best().unwrap().val_loss, 0.5);
    }

    #[test]
    fn test_checkpoint_manager_better_replaces() {
        let mut manager = CheckpointManager::default();

        let cp1 = Checkpoint::new(0, 0, 0.5, 0.8, 0.6, 0.75);
        let is_best1 = manager.save_if_best(cp1);
        assert!(is_best1);

        let cp2 = Checkpoint::new(1, 1, 0.4, 0.85, 0.45, 0.8);
        let is_best2 = manager.save_if_best(cp2);
        assert!(is_best2);

        assert_eq!(manager.best().unwrap().val_loss, 0.4);
    }

    #[test]
    fn test_checkpoint_manager_worse_not_best() {
        let mut manager = CheckpointManager::default();

        let cp1 = Checkpoint::new(0, 0, 0.5, 0.8, 0.6, 0.75);
        manager.save_if_best(cp1);

        let cp2 = Checkpoint::new(1, 1, 0.6, 0.75, 0.7, 0.7);
        let is_best = manager.save_if_best(cp2);
        assert!(!is_best);

        assert_eq!(manager.best().unwrap().val_loss, 0.5);
    }

    #[test]
    fn test_checkpoint_manager_history() {
        let mut manager = CheckpointManager::default();

        for i in 0..3 {
            let cp = Checkpoint::new(i, i, (i as f32 + 1.0) * 0.1, 0.8, 0.6, 0.75);
            manager.save_if_best(cp);
        }

        assert_eq!(manager.len(), 3);
        assert_eq!(manager.history().len(), 3);
    }

    #[test]
    fn test_checkpoint_manager_max_checkpoints() {
        let mut manager = CheckpointManager::new(2);

        for i in 0..5 {
            let cp = Checkpoint::new(i, i, (i as f32 + 1.0) * 0.1, 0.8, 0.6, 0.75);
            manager.save_if_best(cp);
        }

        assert!(manager.len() <= 2);
    }

    #[test]
    fn test_checkpoint_manager_clear() {
        let mut manager = CheckpointManager::default();

        let cp = Checkpoint::new(0, 0, 0.5, 0.8, 0.6, 0.75);
        manager.save_if_best(cp);
        assert_eq!(manager.len(), 1);

        manager.clear();
        assert_eq!(manager.len(), 0);
        assert!(manager.best().is_none());
    }

    #[test]
    fn test_early_stopper_is_exhausted() {
        let mut stopper = EarlyStopper::default();

        stopper.should_stop(0.5, 0);
        assert!(!stopper.is_exhausted());

        stopper.should_stop(0.5, 1);
        stopper.should_stop(0.5, 2);
        stopper.should_stop(0.5, 3);
        assert!(stopper.is_exhausted());
    }
}
