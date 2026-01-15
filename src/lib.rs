//! JailGuard: RL-Based Prompt Injection Defense
//!
//! A reinforcement learning approach to detecting prompt injection attacks,
//! using the Burn deep learning framework.
//!
//! # Features
//!
//! - **PPO Agent**: Proximal Policy Optimization for stable policy learning
//! - **DQN Agent**: Deep Q-Network with experience replay for value-based learning
//! - **Online Learning**: Continuous improvement from user feedback
//! - **Pre-trained Models**: Ready-to-use models trained on prompt injection datasets
//!
//! # Example
//!
//! ```rust,ignore
//! use jailguard::{Detector, DetectorConfig};
//!
//! let detector = Detector::pretrained("jailguard-v1")?;
//! let result = detector.detect("Ignore previous instructions");
//!
//! if result.is_injection {
//!     println!("Blocked injection with {:.1}% confidence", result.confidence * 100.0);
//! }
//! ```

pub mod agent;
pub mod dataset;
pub mod detection;
pub mod error;
pub mod feedback;
pub mod model;
pub mod pretrained;
pub mod spotlighting;
pub mod tokenizer;
pub mod training;

// Re-exports for convenience
pub use detection::{DetectionResult, Detector, DetectorConfig, InjectionRisk};
pub use error::{Error, Result};
pub use feedback::{FeedbackCollector, FeedbackType};
pub use spotlighting::{Spotlighting, SpotlightingConfig};

// Agent re-exports
pub use agent::{AgentConfig, DQNAgent, DQNConfig, Experience, PPOAgent, PPOConfig};

// Training re-exports
pub use training::{Trainer, TrainerConfig, TrainingMetrics};
