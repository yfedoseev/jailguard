//! Training metrics tracking.

/// Metrics collected during training.
#[derive(Debug, Clone, Default)]
pub struct TrainingMetrics {
    /// Average loss value
    pub loss: f32,
    /// Classification accuracy
    pub accuracy: f32,
    /// Number of true positives (correctly blocked injections)
    pub true_positives: usize,
    /// Number of true negatives (correctly allowed benign)
    pub true_negatives: usize,
    /// Number of false positives (incorrectly blocked benign)
    pub false_positives: usize,
    /// Number of false negatives (missed injections)
    pub false_negatives: usize,
}

impl TrainingMetrics {
    /// Compute precision (of all blocked, how many were actually injections).
    pub fn precision(&self) -> f32 {
        let total_blocked = self.true_positives + self.false_positives;
        if total_blocked == 0 {
            0.0
        } else {
            self.true_positives as f32 / total_blocked as f32
        }
    }

    /// Compute recall (of all injections, how many were blocked).
    pub fn recall(&self) -> f32 {
        let total_injections = self.true_positives + self.false_negatives;
        if total_injections == 0 {
            0.0
        } else {
            self.true_positives as f32 / total_injections as f32
        }
    }

    /// Compute F1 score.
    pub fn f1_score(&self) -> f32 {
        let p = self.precision();
        let r = self.recall();
        if p + r == 0.0 {
            0.0
        } else {
            2.0 * p * r / (p + r)
        }
    }

    /// Total number of samples processed.
    pub fn total_samples(&self) -> usize {
        self.true_positives + self.true_negatives + self.false_positives + self.false_negatives
    }

    /// Merge metrics from another instance.
    pub fn merge(&mut self, other: &TrainingMetrics) {
        self.true_positives += other.true_positives;
        self.true_negatives += other.true_negatives;
        self.false_positives += other.false_positives;
        self.false_negatives += other.false_negatives;

        // Update accuracy based on totals
        let total = self.total_samples();
        if total > 0 {
            self.accuracy = (self.true_positives + self.true_negatives) as f32 / total as f32;
        }
    }
}

impl std::fmt::Display for TrainingMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Loss: {:.4} | Acc: {:.2}% | P: {:.2}% | R: {:.2}% | F1: {:.2}%",
            self.loss,
            self.accuracy * 100.0,
            self.precision() * 100.0,
            self.recall() * 100.0,
            self.f1_score() * 100.0
        )
    }
}
