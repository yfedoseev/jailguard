//! Comprehensive evaluation framework for JailGuard
//!
//! Provides multi-dimensional evaluation of model performance:
//! - Binary classification metrics (accuracy, precision, recall, F1)
//! - Multi-class metrics (per-attack-type performance)
//! - Calibration analysis (ECE, MCE, Brier score)
//! - Adversarial robustness testing
//! - SOTA comparison capabilities

pub mod multiclass_evaluator;
pub mod calibration_evaluator;
pub mod adversarial_evaluator;

pub use multiclass_evaluator::{MultiClassEvaluator, PerClassMetrics, ConfusionMatrix};
pub use calibration_evaluator::{CalibrationEvaluator, CalibrationMetrics, CalibrationBin};
pub use adversarial_evaluator::{AdversarialEvaluator, AttackResult};
