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

pub mod adam_optimizer;
pub mod multitask_sample;
pub mod adversarial;
pub mod adversarial_batch_mixer;
pub mod adversarial_trainer;
pub mod adversarial_training;
mod buffer;
pub mod calibration;
pub mod early_stopping;
pub mod fine_tune;
pub mod gradient_descent;
mod metrics;
pub mod multilabel;
pub mod multilabel_trainer;
mod multitask;
pub mod multitask_learning;
pub mod multitask_trainer;
pub mod neural_binary_network;
pub mod neural_data_loader;
pub mod neural_multitask_network;
pub mod neural_trainer;
pub mod online;
mod reward;
pub mod robust_multilabel_trainer;
pub mod trainable_heads;

pub use adam_optimizer::{Adam, AdamConfig, LearningRateScheduler, ScheduleType};
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
pub use buffer::{Experience, ExperienceBuffer};
pub use calibration::CalibrationConfig;
pub use calibration::{CalibrationMetrics, CalibrationValidator, TemperatureScaling};
pub use early_stopping::{Checkpoint, CheckpointManager, EarlyStopper, EarlyStoppingConfig};
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
pub use multitask_sample::MultiTaskSample;
pub use neural_binary_network::{AdamState, NeuralBinaryNetwork};
pub use neural_data_loader::{NeuralDataLoader, NeuralEmbeddingSample};
#[allow(deprecated)]
pub use neural_multitask_network::NeuralMultitaskNetwork;
pub use neural_trainer::{
    NeuralLRSchedule, NeuralTrainer, NeuralTrainerConfig, NeuralTrainingMetrics,
};
pub use online::{
    FeedbackCollector, FeedbackCollectorConfig, FeedbackSample, FeedbackStatistics,
    IncrementalMetrics, IncrementalTrainer, IncrementalTrainingConfig,
};
pub use reward::{RewardConfig, RewardShaper};
pub use robust_multilabel_trainer::{
    RobustEpochMetrics, RobustMultiLabelTrainer, RobustTrainingConfig, RobustnessMetrics,
};
pub use trainable_heads::TrainableLinearHead;

