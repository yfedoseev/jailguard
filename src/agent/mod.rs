//! Reinforcement learning agents for prompt injection detection.
//!
//! This module provides RL agents that learn to classify text as injection or benign:
//! - PPO (Proximal Policy Optimization): Stable policy gradient method
//! - DQN (Deep Q-Network): Value-based method with experience replay

mod config;
mod dqn;
mod ppo;

pub use config::{AgentConfig, DQNConfig, PPOConfig};
pub use dqn::DQNAgent;
pub use ppo::PPOAgent;

/// Experience tuple for replay buffer.
#[derive(Debug, Clone)]
pub struct Experience {
    /// Current state (pooled embeddings)
    pub state: Vec<f32>,
    /// Action taken (0 = Block, 1 = Allow)
    pub action: usize,
    /// Reward received
    pub reward: f32,
    /// Next state (pooled embeddings)
    pub next_state: Vec<f32>,
    /// Whether episode is done
    pub done: bool,
    /// Log probability of action (for PPO)
    pub log_prob: f32,
}
