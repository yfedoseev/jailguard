//! Detection API for prompt injection detection.
//!
//! This module provides the main detection interface that combines
//! tokenization, embedding, and the policy network to detect injections.

pub mod calibrated_detector;
mod detector;
pub mod ensemble_detector;
mod result;
pub mod transformer_detector;

pub use calibrated_detector::{CalibratedDetectionResult, CalibratedDetector};
pub use detector::{Detector, DetectorConfig};
pub use ensemble_detector::{EnsembleConfig, EnsembleDetectionResult, EnsembleDetector};
pub use result::{AttackType, DetectionResult, InjectionRisk, MultiTaskDetectionResult};
pub use transformer_detector::{TransformerDetector, TransformerDetectorConfig};
