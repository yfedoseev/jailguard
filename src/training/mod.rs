//! Training infrastructure for RL agents and multi-task learning.
//!
//! This module provides:
//! - Experience replay buffer
//! - Reward shaping
//! - Training loop orchestration
//! - Metrics collection
//! - Multi-task learning (binary + attack type + semantic)
//! - Adversarial training with robustness
//! - Confidence calibration
//! - Online learning from feedback
#![allow(clippy::unnecessary_wraps)]

pub mod adversarial;
pub mod adversarial_batch_mixer;
pub mod adversarial_trainer;
pub mod adversarial_training;
mod buffer;
pub mod calibration;
pub mod fine_tune;
pub mod gradient_descent;
mod metrics;
pub mod multilabel;
pub mod multilabel_trainer;
mod multitask;
pub mod multitask_learning;
pub mod multitask_trainer;
pub mod online;
mod reward;
pub mod robust_multilabel_trainer;

pub use adversarial::{
    AdversarialConfig, AdversarialGenerator, CharSubstitutionAttack, EncodingAttack,
    ParaphraseAttack,
};
pub use adversarial_batch_mixer::{
    AdversarialBatchConfig, AdversarialBatchMixer, AdversarialBatchStats,
};
pub use adversarial_trainer::{AdversarialMetrics, AdversarialTrainer, AdversarialTrainingConfig};
pub use adversarial_training::{
    AdversarialConfig as AdvConfig, AdversarialDatasetMixer, AdversarialGenerator as AdvGenerator,
    AugmentationStats,
};
pub use buffer::ExperienceBuffer;
pub use calibration::CalibrationConfig;
pub use calibration::{CalibrationMetrics, CalibrationValidator, TemperatureScaling};
pub use fine_tune::{FineTuneConfig, FineTuner, TrainingMetrics as FinetuneMetrics};
pub use gradient_descent::{EpochMetrics, GradientDescentTrainer};
pub use metrics::TrainingMetrics;
pub use multilabel::{MultiLabelLoss, MultiLabelLossConfig};
pub use multilabel_trainer::{
    MultiLabelTrainer, MultiLabelTrainingConfig, MultiLabelTrainingMetrics,
    MultiLabelTrainingSample,
};
pub use multitask::MultiTaskLoss;
pub use multitask_learning::{
    AttackType, MultiTaskConfig, MultiTaskLearner, MultiTaskResult, RiskLevel,
};
pub use multitask_trainer::{MultiTaskMetrics, MultiTaskTrainer, MultiTaskTrainingConfig};
pub use online::{
    FeedbackCollector, FeedbackCollectorConfig, FeedbackSample, FeedbackStatistics,
    IncrementalMetrics, IncrementalTrainer, IncrementalTrainingConfig,
};
pub use reward::{RewardConfig, RewardShaper};
pub use robust_multilabel_trainer::{
    RobustEpochMetrics, RobustMultiLabelTrainer, RobustTrainingConfig, RobustnessMetrics,
};

use crate::agent::Experience;
use crate::error::Result;

/// Configuration for the trainer.
#[derive(Debug, Clone)]
pub struct TrainerConfig {
    /// Batch size for training
    pub batch_size: usize,
    /// Number of epochs per training session
    pub epochs: usize,
    /// Whether to shuffle experiences
    pub shuffle: bool,
    /// Reward configuration
    pub reward_config: RewardConfig,
}

impl Default for TrainerConfig {
    fn default() -> Self {
        Self {
            batch_size: 32,
            epochs: 10,
            shuffle: true,
            reward_config: RewardConfig::default(),
        }
    }
}

/// Trainer for RL agents.
pub struct Trainer {
    /// Experience replay buffer
    buffer: ExperienceBuffer,
    /// Training configuration
    config: TrainerConfig,
    /// Reward shaper
    reward_shaper: RewardShaper,
}

impl Trainer {
    /// Create a new trainer.
    pub fn new(config: TrainerConfig) -> Self {
        Self {
            buffer: ExperienceBuffer::new(10000),
            reward_shaper: RewardShaper::new(config.reward_config.clone()),
            config,
        }
    }

    /// Add an experience to the buffer.
    pub fn add_experience(&mut self, experience: Experience) {
        self.buffer.push(experience);
    }

    /// Sample experiences from the buffer.
    pub fn sample_experiences(&self) -> Vec<Experience> {
        self.buffer.sample(self.config.batch_size)
    }

    /// Get the reward shaper.
    pub fn reward_shaper(&self) -> &RewardShaper {
        &self.reward_shaper
    }

    /// Get the current buffer size.
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// Clear the experience buffer.
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Save training state.
    pub fn save(&self, _path: &str) -> Result<()> {
        // TODO: Implement state saving
        Ok(())
    }
}
