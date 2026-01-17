//! `JailGuard`: RL-Based Prompt Injection Defense
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

pub mod advanced_ensemble;
pub mod agent;
pub mod attention_tracker;
pub mod collection;
pub mod dataset;
pub mod detection;
pub mod embeddings;
pub mod ensemble;
pub mod error;
pub mod feedback;
pub mod heuristics;
pub mod jailguard;
pub mod model;
pub mod monitoring;
pub mod output_validation;
pub mod performance;
pub mod pretrained;
pub mod privilege;
pub mod spotlighting;
pub mod task_tracking;
pub mod tokenizer;
pub mod training;
pub mod validation;

// Re-exports for convenience
pub use advanced_ensemble::{AdvancedDetectionResult, AdvancedEnsemble, LayerScores};
pub use attention_tracker::{AttentionTracker, AttentionTrackerConfig, AttentionTrackerResult};
pub use detection::{DetectionResult, Detector, DetectorConfig, InjectionRisk};
pub use ensemble::{EnsembleDetectionResult, EnsembleDetector, ModelWeights};
pub use error::{Error, Result};
pub use feedback::{FeedbackCollector, FeedbackType};
pub use heuristics::{HeuristicDetector, HeuristicResult, HeuristicRule, RuleCategory};
pub use spotlighting::{Spotlighting, SpotlightingConfig};

// Unified API re-exports
pub use jailguard::{
    InputValidationResult, JailGuard, JailGuardConfig, OutputCheckResult, RequestContext,
    SessionStats,
};

// Agent re-exports
pub use agent::{AgentConfig, DQNAgent, DQNConfig, Experience, PPOAgent, PPOConfig};

// Training re-exports
pub use training::{Trainer, TrainerConfig, TrainingMetrics};

// Monitoring re-exports
pub use monitoring::{
    AnomalyConfig, AnomalyDetector, AnomalyResult, DetectionEvent, SessionTracker,
};

// Performance re-exports
pub use performance::{EnsembleProfile, EnsembleProfiler, PerformanceMetrics, ResponseCache};

// Validation re-exports
pub use validation::{
    BenchmarkDataset, ModelComparison, SOTAValidator, SecurityAssessment, ValidationMetrics,
    ValidationReport,
};
