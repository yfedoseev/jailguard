#![warn(missing_docs)]
#![warn(clippy::print_stdout, clippy::print_stderr)]

//! # `JailGuard` — Prompt Injection Detection
//!
//! Fast, accurate prompt injection detection with a simple API.
//! The 200K-trained model (99.07% accuracy) is embedded in the library —
//! no external files or setup required.
//!
//! ## Quick Start
//!
//! ```rust
//! use jailguard::{detect, is_injection};
//!
//! // Simple boolean check
//! if is_injection("ignore previous instructions") {
//!     println!("Blocked!");
//! }
//!
//! // Get detailed result with confidence score
//! let result = detect("What is the capital of France?");
//! println!("Safe: {}, Confidence: {:.1}%", !result.is_injection, result.confidence * 100.0);
//! ```
//!
//! ## Features
//!
//! - **99.07% Accuracy**: Trained on 200K samples from 14 real datasets
//! - **Real ML**: ONNX embeddings (all-MiniLM-L6-v2) + neural classifier
//! - **Auto-setup**: ONNX model auto-downloaded on first use (~90 MB, cached)
//! - **Simple API**: `is_injection()`, `detect()`, `score()`
//!
//! ## API Overview
//!
//! | Function | Returns | Use Case |
//! |----------|---------|----------|
//! | `is_injection(text)` | `bool` | Quick yes/no check |
//! | `detect(text)` | `DetectionOutput` | Full details with confidence |
//! | `score(text)` | `f32` | Raw probability (0.0-1.0) |
//! | `detect_batch(texts)` | `Vec<DetectionOutput>` | Process multiple inputs |

// ============================================================================
// Core — always compiled
// ============================================================================

pub mod embedded;
mod error;
pub(crate) mod model_manager;
pub(crate) mod network;

/// C ABI surface — Go (cgo) and Node.js (napi-rs) bindings link against
/// these `extern "C"` functions. Compiled unconditionally so the
/// `cdylib` / `staticlib` artifact always exposes the symbols; the
/// `c-api` feature only gates the cbindgen header regeneration in
/// `build.rs`.
pub mod c_api;

/// Node.js native module via napi-rs / N-API. Compiled only when the
/// `napi` feature is enabled (typically via `npx napi build`).
#[cfg(feature = "napi")]
pub mod napi;

/// WASM bindings via wasm-bindgen. Compiled only when the `wasm` feature
/// is enabled (typically via `wasm-pack build`). Status: alpha — see
/// `src/wasm.rs` for the gap explanation.
#[cfg(feature = "wasm")]
pub mod wasm;

// Primary API at crate root
pub use embedded::{detect, detect_batch, is_injection, score, DetectionOutput, RiskLevel};
pub use error::{Error, Result};
pub use model_manager::download_model;

/// Deprecated alias for [`download_model`]. Retained so the companion
/// `jailguard-datasets/pipeline` and other downstream code that pinned the
/// 0.1 API keep compiling. New code should call `download_model` directly.
#[deprecated(since = "0.2.0", note = "use `download_model` instead")]
pub fn ensure_model() -> Result<std::path::PathBuf> {
    download_model()
}

// ============================================================================
// Feature-gated modules
// ============================================================================

#[cfg(feature = "python")]
mod python;

#[cfg(feature = "full")]
pub mod advanced_ensemble;
#[cfg(feature = "full")]
pub mod agent;
#[cfg(feature = "full")]
pub mod api;
#[cfg(feature = "full")]
pub mod attention_tracker;
#[cfg(feature = "full")]
pub mod collection;
#[cfg(feature = "full")]
pub mod dataset;
#[cfg(feature = "full")]
pub mod detection;
#[cfg(feature = "full")]
pub mod embeddings;
#[cfg(feature = "full")]
pub mod ensemble;
#[cfg(feature = "full")]
pub mod evaluation;
#[cfg(feature = "full")]
pub mod feedback;
#[cfg(feature = "full")]
pub mod heuristics;
#[cfg(feature = "full")]
pub mod inference;
#[cfg(feature = "full")]
pub mod jailguard;
#[cfg(feature = "full")]
pub mod model;
#[cfg(feature = "full")]
pub mod monitoring;
#[cfg(feature = "full")]
pub mod output_validation;
#[cfg(feature = "full")]
pub mod performance;
#[cfg(feature = "full")]
pub mod pretrained;
#[cfg(feature = "full")]
pub mod privilege;
#[cfg(feature = "full")]
pub mod spotlighting;
#[cfg(feature = "full")]
pub mod task_tracking;
#[cfg(feature = "full")]
pub mod tokenizer;
#[cfg(feature = "full")]
pub mod training;
#[cfg(feature = "full")]
pub mod validation;

// ============================================================================
// Feature-gated re-exports
// ============================================================================

#[cfg(feature = "full")]
pub use heuristics::{HeuristicDetector, HeuristicResult, HeuristicRule, RuleCategory};

#[cfg(feature = "full")]
pub use detection::{DetectionResult, Detector, DetectorConfig, InjectionRisk};

#[cfg(feature = "full")]
pub use advanced_ensemble::{AdvancedDetectionResult, AdvancedEnsemble, LayerScores};

#[cfg(feature = "full")]
pub use attention_tracker::{AttentionTracker, AttentionTrackerConfig, AttentionTrackerResult};

#[cfg(feature = "full")]
pub use ensemble::{EnsembleDetectionResult, EnsembleDetector, ModelWeights};

#[cfg(feature = "full")]
pub use evaluation::{
    AdversarialEvaluator, AttackResult, CalibrationBin, CalibrationEvaluator, CalibrationMetrics,
    ConfusionMatrix, MultiClassEvaluator, PerClassMetrics,
};

#[cfg(feature = "full")]
pub use feedback::{FeedbackCollector, FeedbackType};

#[cfg(feature = "full")]
pub use spotlighting::{Spotlighting, SpotlightingConfig};

#[cfg(feature = "full")]
pub use jailguard::{
    InputValidationResult, JailGuard, JailGuardConfig, OutputCheckResult, RequestContext,
    SessionStats,
};

#[cfg(feature = "full")]
pub use agent::{AgentConfig, DQNAgent, DQNConfig, Experience, PPOAgent, PPOConfig};

#[cfg(feature = "full")]
pub use training::{Trainer, TrainerConfig, TrainingMetrics};

#[cfg(feature = "full")]
pub use monitoring::{
    AnomalyConfig, AnomalyDetector, AnomalyResult, DetectionEvent, SessionTracker,
};

#[cfg(feature = "full")]
pub use performance::{EnsembleProfile, EnsembleProfiler, PerformanceMetrics, ResponseCache};

#[cfg(feature = "full")]
pub use validation::{
    BenchmarkDataset, ModelComparison, SOTAValidator, SecurityAssessment, ValidationMetrics,
    ValidationReport,
};
