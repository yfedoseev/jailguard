//! Online learning infrastructure for continuous model improvement from feedback.

pub mod feedback_collector;
pub mod incremental_trainer;

pub use feedback_collector::{
    FeedbackCollector, FeedbackCollectorConfig, FeedbackSample, FeedbackStatistics,
};
pub use incremental_trainer::{IncrementalMetrics, IncrementalTrainer, IncrementalTrainingConfig};
