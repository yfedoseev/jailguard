//! Proximal Policy Optimization (PPO) agent.
#![allow(clippy::unnecessary_wraps)]

use burn::tensor::{backend::Backend, Tensor, TensorData};
use burn_ndarray::NdArray;

use super::Experience;
use crate::error::Result;
use crate::model::{PolicyNetwork, PolicyNetworkConfig, ValueNetwork, ValueNetworkConfig};

/// Backend type for PPO agent.
type B = NdArray;

/// PPO agent for prompt injection detection.
pub struct PPOAgent {
    /// Policy network
    policy: PolicyNetwork<B>,
    /// Value network
    value: ValueNetwork<B>,
    /// Device for computation
    device: <B as Backend>::Device,
    /// Clipping parameter
    #[allow(dead_code)]
    clip_epsilon: f32,
    /// Entropy coefficient
    entropy_coef: f32,
    /// Value loss coefficient
    value_coef: f32,
}

impl PPOAgent {
    /// Create a new PPO agent.
    pub fn new(embed_dim: usize, hidden_dim: usize) -> Self {
        let device = Default::default();
        let policy_config = PolicyNetworkConfig::new(embed_dim, hidden_dim);
        let value_config = ValueNetworkConfig::new(embed_dim, hidden_dim);

        Self {
            policy: policy_config.init(&device),
            value: value_config.init(&device),
            device,
            clip_epsilon: 0.2,
            entropy_coef: 0.01,
            value_coef: 0.5,
        }
    }

    /// Get policy network reference.
    pub fn policy(&self) -> &PolicyNetwork<B> {
        &self.policy
    }

    /// Get value network reference.
    pub fn value(&self) -> &ValueNetwork<B> {
        &self.value
    }

    /// Select an action given the current state.
    ///
    /// Returns (action, `log_probability`).
    pub fn select_action(&self, state: &[f32]) -> (usize, f32) {
        let state_dim = state.len();
        let state_tensor = Tensor::<B, 2>::from_data(
            TensorData::new(state.to_vec(), [1, state_dim]),
            &self.device,
        );

        // Get action probabilities
        let probs = self.policy.forward(state_tensor.clone());
        let log_probs = self.policy.log_probs(state_tensor);

        // Get probabilities as vec
        let probs_data: Vec<f32> = probs.to_data().to_vec().unwrap();

        // Determine action based on probabilities
        let action = if probs_data[0] > probs_data[1] { 0 } else { 1 };
        let log_probs_data: Vec<f32> = log_probs.to_data().to_vec().unwrap();
        let log_prob = log_probs_data[action];

        (action, log_prob)
    }

    /// Update the agent using a batch of experiences.
    ///
    /// Returns the loss value.
    pub fn update(&mut self, experiences: &[Experience]) -> f32 {
        if experiences.is_empty() {
            return 0.0;
        }

        // Extract data from experiences
        let rewards: Vec<f32> = experiences.iter().map(|e| e.reward).collect();
        let dones: Vec<bool> = experiences.iter().map(|e| e.done).collect();

        // Compute values for all states
        let states: Vec<f32> = experiences.iter().flat_map(|e| e.state.clone()).collect();
        let state_dim = experiences[0].state.len();
        let batch_size = experiences.len();

        let states_tensor = Tensor::<B, 2>::from_data(
            TensorData::new(states, [batch_size, state_dim]),
            &self.device,
        );

        let values = self.value.forward_scalar(states_tensor);
        let values_data: Vec<f32> = values.to_data().to_vec().unwrap();

        // Compute advantages
        let advantages = self.compute_gae(&rewards, &values_data, &dones, 0.99, 0.95);

        // Placeholder loss computation
        let mean_advantage = advantages.iter().sum::<f32>() / advantages.len() as f32;

        mean_advantage.abs() * self.value_coef + self.entropy_coef
    }

    /// Compute advantages using GAE.
    fn compute_gae(
        &self,
        rewards: &[f32],
        values: &[f32],
        dones: &[bool],
        gamma: f32,
        lambda: f32,
    ) -> Vec<f32> {
        let n = rewards.len();
        let mut advantages = vec![0.0; n];
        let mut gae = 0.0;

        for t in (0..n).rev() {
            let next_value = if t + 1 < n && !dones[t] {
                values[t + 1]
            } else {
                0.0
            };

            let delta = rewards[t] + gamma * next_value - values[t];
            let mask = if dones[t] { 0.0 } else { 1.0 };
            gae = delta + gamma * lambda * mask * gae;
            advantages[t] = gae;
        }

        advantages
    }

    /// Save the agent to a file.
    pub fn save(&self, _path: &str) -> Result<()> {
        // TODO: Implement model saving
        Ok(())
    }

    /// Load the agent from a file.
    pub fn load(_path: &str) -> Result<Self> {
        // TODO: Implement model loading
        Err(crate::error::Error::Model("Not implemented".into()))
    }
}
