//! Comprehensive evaluation framework for JailGuard
//!
//! Provides multi-dimensional evaluation of model performance:
//! - Binary classification metrics (accuracy, precision, recall, F1)
//! - Multi-class metrics (per-attack-type performance)
//! - Calibration analysis (ECE, MCE, Brier score)
//! - Adversarial robustness testing
//! - SOTA comparison capabilities

pub mod adversarial_evaluator;
pub mod calibration_evaluator;
pub mod multiclass_evaluator;

pub use adversarial_evaluator::{AdversarialEvaluator, AttackResult};
pub use calibration_evaluator::{CalibrationBin, CalibrationEvaluator, CalibrationMetrics};
pub use multiclass_evaluator::{ConfusionMatrix, MultiClassEvaluator, PerClassMetrics};
