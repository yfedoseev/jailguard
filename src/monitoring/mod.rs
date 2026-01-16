//! Behavior monitoring for detecting anomalies and attack campaigns.
//!
//! This module provides:
//! - Session tracking for detection events
//! - Anomaly detection with Z-score based statistical analysis
//! - Attack pattern recognition (rapid succession, escalation, topic drift)
//! - Behavior profiling and baseline comparison

pub mod anomaly_detector;
pub mod session_tracker;

pub use anomaly_detector::{AnomalyConfig, AnomalyDetector, AnomalyResult, BaselineStats};
pub use session_tracker::{DetectionEvent, SessionStats, SessionTracker};
