//! Detection API for prompt injection detection.
//!
//! This module provides the main detection interface that combines
//! tokenization, embedding, and the policy network to detect injections.

mod detector;
mod result;
pub mod transformer_detector;

pub use detector::{Detector, DetectorConfig};
pub use result::{AttackType, DetectionResult, InjectionRisk, MultiTaskDetectionResult};
pub use transformer_detector::{TransformerDetector, TransformerDetectorConfig};
