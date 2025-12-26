//! Training infrastructure for RL agents.
//!
//! This module provides:
//! - Experience replay buffer
//! - Reward shaping
//! - Training loop orchestration
//! - Metrics collection

mod buffer;
mod metrics;
mod reward;

pub use buffer::ExperienceBuffer;
pub use metrics::TrainingMetrics;
pub use reward::{RewardConfig, RewardShaper};

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
