//! Detection API for prompt injection detection.
//!
//! This module provides the main detection interface that combines
//! tokenization, embedding, and the policy network to detect injections.

mod detector;
mod result;

pub use detector::{Detector, DetectorConfig};
pub use result::{DetectionResult, InjectionRisk};
