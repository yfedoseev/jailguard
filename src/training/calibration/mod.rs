//! Confidence calibration for reliable uncertainty estimates.
//!
//! This module provides tools to calibrate model confidence scores using
//! temperature scaling and metric computation.
//!
//! Supported metrics:
//! - Expected Calibration Error (ECE)
//! - Maximum Calibration Error (MCE)
//! - Brier Score
//! - Reliability diagrams

pub mod metrics;
pub mod temperature_scaling;
pub mod validator;

pub use metrics::{compute_brier_score, compute_ece, compute_mce, CalibrationMetrics};
pub use temperature_scaling::{TemperatureScaling, TemperatureScalingConfig};
pub use validator::CalibrationValidator;

/// Configuration for calibration.
#[derive(Debug, Clone)]
pub struct CalibrationConfig {
    /// Number of bins for ECE computation
    pub num_bins: usize,
    /// Learning rate for temperature scaling optimization
    pub learning_rate: f32,
    /// Number of optimization steps
    pub num_steps: usize,
    /// Whether to use validation set for calibration
    pub use_validation: bool,
}

impl Default for CalibrationConfig {
    fn default() -> Self {
        Self {
            num_bins: 10,
            learning_rate: 0.01,
            num_steps: 1000,
            use_validation: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CalibrationConfig::default();
        assert_eq!(config.num_bins, 10);
        assert_eq!(config.learning_rate, 0.01);
        assert_eq!(config.num_steps, 1000);
        assert!(config.use_validation);
    }

    #[test]
    fn test_custom_config() {
        let config = CalibrationConfig {
            num_bins: 20,
            learning_rate: 0.001,
            num_steps: 500,
            use_validation: false,
        };

        assert_eq!(config.num_bins, 20);
        assert_eq!(config.learning_rate, 0.001);
    }
}
