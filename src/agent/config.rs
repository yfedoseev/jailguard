//! Configuration for RL agents.

/// Common configuration for RL agents.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Embedding dimension from text encoder
    pub embed_dim: usize,
    /// Hidden layer dimension
    pub hidden_dim: usize,
    /// Learning rate
    pub learning_rate: f64,
    /// Discount factor (gamma)
    pub gamma: f64,
    /// Number of actions (2: Block, Allow)
    pub action_dim: usize,
}

impl AgentConfig {
    /// Create a new agent configuration.
    pub fn new(embed_dim: usize) -> Self {
        Self {
            embed_dim,
            hidden_dim: 256,
            learning_rate: 3e-4,
            gamma: 0.99,
            action_dim: 2,
        }
    }
}

/// PPO-specific configuration.
#[derive(Debug, Clone)]
pub struct PPOConfig {
    /// Base agent configuration
    pub base: AgentConfig,
    /// Clipping parameter for PPO
    pub clip_epsilon: f64,
    /// Entropy bonus coefficient
    pub entropy_coef: f64,
    /// Value loss coefficient
    pub value_coef: f64,
    /// GAE lambda for advantage estimation
    pub gae_lambda: f64,
    /// Number of epochs per update
    pub epochs: usize,
    /// Mini-batch size
    pub batch_size: usize,
}

impl PPOConfig {
    /// Create a new PPO configuration.
    pub fn new(embed_dim: usize) -> Self {
        Self {
            base: AgentConfig::new(embed_dim),
            clip_epsilon: 0.2,
            entropy_coef: 0.01,
            value_coef: 0.5,
            gae_lambda: 0.95,
            epochs: 4,
            batch_size: 64,
        }
    }
}

/// DQN-specific configuration.
#[derive(Debug, Clone)]
pub struct DQNConfig {
    /// Base agent configuration
    pub base: AgentConfig,
    /// Experience replay buffer size
    pub buffer_size: usize,
    /// Mini-batch size for training
    pub batch_size: usize,
    /// Initial exploration rate
    pub epsilon_start: f64,
    /// Final exploration rate
    pub epsilon_end: f64,
    /// Epsilon decay steps
    pub epsilon_decay_steps: usize,
    /// Target network update frequency
    pub target_update_freq: usize,
    /// Whether to use Double DQN
    pub double_dqn: bool,
}

impl DQNConfig {
    /// Create a new DQN configuration.
    pub fn new(embed_dim: usize) -> Self {
        Self {
            base: AgentConfig::new(embed_dim),
            buffer_size: 10000,
            batch_size: 32,
            epsilon_start: 1.0,
            epsilon_end: 0.01,
            epsilon_decay_steps: 10000,
            target_update_freq: 100,
            double_dqn: true,
        }
    }
}
