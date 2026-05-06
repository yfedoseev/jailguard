//! Deep Q-Network (DQN) agent.
#![allow(clippy::unnecessary_wraps)]

use burn::tensor::{backend::Backend, Tensor, TensorData};
use burn_ndarray::NdArray;
use rand::Rng;

use super::Experience;
use crate::error::Result;
use crate::model::{PolicyNetwork, PolicyNetworkConfig};

/// Backend type for DQN agent.
type B = NdArray;

/// DQN agent for prompt injection detection.
pub struct DQNAgent {
    /// Q-network
    q_network: PolicyNetwork<B>,
    /// Target Q-network (for stable training)
    target_network: PolicyNetwork<B>,
    /// Device for computation
    device: <B as Backend>::Device,
    /// Exploration rate
    epsilon: f32,
    /// Minimum exploration rate
    epsilon_min: f32,
    /// Epsilon decay rate
    epsilon_decay: f32,
    /// Target network update frequency
    target_update_freq: usize,
    /// Step counter
    steps: usize,
    /// Whether to use Double DQN
    double_dqn: bool,
}

impl DQNAgent {
    /// Create a new DQN agent.
    pub fn new(embed_dim: usize, hidden_dim: usize) -> Self {
        let device = Default::default();
        let config = PolicyNetworkConfig::new(embed_dim, hidden_dim);

        Self {
            q_network: config.init(&device),
            target_network: config.init(&device),
            device,
            epsilon: 1.0,
            epsilon_min: 0.01,
            epsilon_decay: 0.995,
            target_update_freq: 100,
            steps: 0,
            double_dqn: true,
        }
    }

    /// Get Q-network reference.
    pub fn q_network(&self) -> &PolicyNetwork<B> {
        &self.q_network
    }

    /// Update target network with current Q-network weights.
    pub fn update_target_network(&mut self) {
        // In a full implementation, we would copy weights here
        // For now, this is a placeholder
    }

    /// Decay exploration rate.
    pub fn decay_epsilon(&mut self) {
        self.epsilon = (self.epsilon * self.epsilon_decay).max(self.epsilon_min);
    }

    /// Select an action given the current state.
    ///
    /// Returns (action, `log_probability`).
    pub fn select_action(&self, state: &[f32]) -> (usize, f32) {
        let mut rng = rand::rng();

        // Epsilon-greedy exploration
        if rng.random::<f32>() < self.epsilon {
            // Random action
            let action = rng.random_range(0..2);
            (action, 0.0) // Log prob not meaningful for random action
        } else {
            // Greedy action
            let state_dim = state.len();
            let state_tensor = Tensor::<B, 2>::from_data(
                TensorData::new(state.to_vec(), [1, state_dim]),
                &self.device,
            );

            let q_values = self.q_network.forward(state_tensor);
            let q_data: Vec<f32> = q_values.to_data().to_vec().unwrap();

            let action = if q_data[0] > q_data[1] { 0 } else { 1 };
            let log_prob = q_data[action].ln();

            (action, log_prob)
        }
    }

    /// Update the agent using a batch of experiences.
    ///
    /// Returns the loss value.
    pub fn update(&mut self, experiences: &[Experience]) -> f32 {
        if experiences.is_empty() {
            return 0.0;
        }

        self.steps += 1;

        // Update target network periodically
        if self.steps.is_multiple_of(self.target_update_freq) {
            self.update_target_network();
        }

        // Decay exploration
        self.decay_epsilon();

        // Extract data from experiences
        let batch_size = experiences.len();
        let state_dim = experiences[0].state.len();

        let states: Vec<f32> = experiences.iter().flat_map(|e| e.state.clone()).collect();
        let next_states: Vec<f32> = experiences
            .iter()
            .flat_map(|e| e.next_state.clone())
            .collect();
        let actions: Vec<usize> = experiences.iter().map(|e| e.action).collect();
        let rewards: Vec<f32> = experiences.iter().map(|e| e.reward).collect();
        let dones: Vec<bool> = experiences.iter().map(|e| e.done).collect();

        // Create tensors
        let states_tensor = Tensor::<B, 2>::from_data(
            TensorData::new(states, [batch_size, state_dim]),
            &self.device,
        );

        let next_states_tensor = Tensor::<B, 2>::from_data(
            TensorData::new(next_states, [batch_size, state_dim]),
            &self.device,
        );

        // Get current Q-values
        let current_q = self.q_network.forward(states_tensor);

        // Get target Q-values
        let target_q = if self.double_dqn {
            // Double DQN: use online network to select action, target network to evaluate
            let next_q_online = self.q_network.forward(next_states_tensor.clone());
            let _next_q_target = self.target_network.forward(next_states_tensor);
            next_q_online // Simplified for now
        } else {
            self.target_network.forward(next_states_tensor)
        };

        // Compute TD error (simplified)
        let current_q_data: Vec<f32> = current_q.to_data().to_vec().unwrap();
        let target_q_data: Vec<f32> = target_q.to_data().to_vec().unwrap();

        let gamma = 0.99f32;
        let mut total_loss = 0.0;

        for i in 0..batch_size {
            let current = current_q_data[i * 2 + actions[i]];
            let next_max = target_q_data[i * 2].max(target_q_data[i * 2 + 1]);
            let target = rewards[i] + if dones[i] { 0.0 } else { gamma * next_max };
            let td_error = target - current;
            total_loss += td_error * td_error;
        }

        total_loss / batch_size as f32
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
