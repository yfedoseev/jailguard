//! Temperature scaling for calibrating confidence scores.

/// Configuration for temperature scaling.
#[derive(Debug, Clone)]
pub struct TemperatureScalingConfig {
    /// Initial temperature value
    pub initial_temperature: f32,
    /// Learning rate for optimization
    pub learning_rate: f32,
    /// Number of optimization steps
    pub num_steps: usize,
}

impl Default for TemperatureScalingConfig {
    fn default() -> Self {
        Self {
            initial_temperature: 1.0,
            learning_rate: 0.01,
            num_steps: 1000,
        }
    }
}

/// Temperature scaling for calibrating neural network confidence scores.
///
/// Temperature scaling is a post-hoc calibration method that learns a single
/// scalar temperature T to transform logits before softmax:
///
/// `calibrated_probs` = softmax(logits / T)
///
/// When T > 1, confidences are reduced (useful for overconfident models).
/// When T < 1, confidences are increased (useful for underconfident models).
#[derive(Debug, Clone)]
pub struct TemperatureScaling {
    /// Learned temperature value
    temperature: f32,
    /// Configuration
    #[allow(dead_code)]
    config: TemperatureScalingConfig,
}

impl TemperatureScaling {
    /// Create a new temperature scaling with default config.
    pub fn new() -> Self {
        Self::with_config(TemperatureScalingConfig::default())
    }

    /// Create with custom configuration.
    pub fn with_config(config: TemperatureScalingConfig) -> Self {
        Self {
            temperature: config.initial_temperature,
            config,
        }
    }

    /// Get the current temperature value.
    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Set the temperature value.
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = temperature;
    }

    /// Apply temperature scaling to raw confidence scores.
    ///
    /// Divides scores by temperature to calibrate confidences.
    pub fn scale_confidence(&self, confidence: f32) -> f32 {
        // Simple scaling: divide by temperature
        // In practice, for single-value confidence (binary case),
        // temperature scaling works by adjusting the logit before softmax
        (confidence / self.temperature).clamp(0.0, 1.0)
    }

    /// Apply temperature to logits (for full probability distribution).
    ///
    /// Returns scaled logits (before softmax).
    pub fn scale_logits(&self, logits: &[f32]) -> Vec<f32> {
        logits.iter().map(|&l| l / self.temperature).collect()
    }

    /// Calibrate temperature using validation data.
    ///
    /// This is a simplified version that optimizes temperature to minimize
    /// negative log-likelihood on the validation set.
    pub fn calibrate(&mut self, predictions: &[f32], targets: &[bool]) {
        if predictions.is_empty() {
            return;
        }

        // Simple optimization: use grid search or gradient-based method
        // For now, we'll use a simple exponential search
        let mut best_temp = self.temperature;
        let mut best_nll = self.compute_nll(predictions, targets, best_temp);

        // Search in range [0.1, 10.0]
        let mut current_temp = 0.1;
        while current_temp <= 10.0 {
            let nll = self.compute_nll(predictions, targets, current_temp);
            if nll < best_nll {
                best_nll = nll;
                best_temp = current_temp;
            }
            current_temp += 0.1;
        }

        self.temperature = best_temp;
    }

    /// Compute negative log-likelihood for given temperature.
    fn compute_nll(&self, predictions: &[f32], targets: &[bool], temperature: f32) -> f32 {
        let mut total_nll = 0.0;

        for (pred, &target) in predictions.iter().zip(targets.iter()) {
            let scaled = pred / temperature;

            // Clamp to avoid log(0)
            let clamped = scaled.clamp(1e-7, 1.0 - 1e-7);

            let nll = if target {
                -clamped.ln()
            } else {
                -(1.0 - clamped).ln()
            };

            total_nll += nll;
        }

        total_nll / predictions.len() as f32
    }

    /// Estimate calibration error using temperature scaling.
    ///
    /// Returns the estimated calibration error (lower is better).
    pub fn estimate_calibration_error(
        &self,
        predictions: &[f32],
        targets: &[bool],
        num_bins: usize,
    ) -> f32 {
        if predictions.is_empty() {
            return 0.0;
        }

        let bin_size = 1.0 / num_bins as f32;
        let mut ece = 0.0;

        for bin_idx in 0..num_bins {
            let bin_lower = bin_idx as f32 * bin_size;
            let bin_upper = (bin_idx + 1) as f32 * bin_size;

            // Find samples in this bin
            let mut bin_preds = Vec::new();
            let mut bin_targets = Vec::new();

            for (pred, &target) in predictions.iter().zip(targets.iter()) {
                let scaled = self.scale_confidence(*pred);
                if scaled >= bin_lower && scaled <= bin_upper {
                    bin_preds.push(scaled);
                    bin_targets.push(target);
                }
            }

            if bin_preds.is_empty() {
                continue;
            }

            // Compute accuracy in bin
            let correct = bin_targets.iter().filter(|&&t| t).count();
            let accuracy = correct as f32 / bin_preds.len() as f32;

            // Compute confidence in bin
            let avg_confidence = bin_preds.iter().sum::<f32>() / bin_preds.len() as f32;

            // Add to ECE
            let calibration_error = (avg_confidence - accuracy).abs();
            ece += (bin_preds.len() as f32 / predictions.len() as f32) * calibration_error;
        }

        ece
    }
}

impl Default for TemperatureScaling {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_scaling_creation() {
        let ts = TemperatureScaling::new();
        assert_eq!(ts.temperature(), 1.0);
    }

    #[test]
    fn test_temperature_setting() {
        let mut ts = TemperatureScaling::new();
        ts.set_temperature(2.0);
        assert_eq!(ts.temperature(), 2.0);
    }

    #[test]
    fn test_confidence_scaling() {
        let ts = TemperatureScaling::with_config(TemperatureScalingConfig {
            initial_temperature: 2.0,
            ..Default::default()
        });

        let confidence = 0.9;
        let scaled = ts.scale_confidence(confidence);

        // With T=2.0, confidence should be reduced
        assert!(scaled < confidence);
        assert!((0.0..=1.0).contains(&scaled));
    }

    #[test]
    fn test_confidence_scaling_high_temp() {
        let ts = TemperatureScaling::with_config(TemperatureScalingConfig {
            initial_temperature: 0.5,
            ..Default::default()
        });

        let confidence = 0.7;
        let scaled = ts.scale_confidence(confidence);

        // With T=0.5, confidence should be increased
        assert!(scaled > confidence);
        assert!(scaled <= 1.0);
    }

    #[test]
    fn test_logits_scaling() {
        let ts = TemperatureScaling::with_config(TemperatureScalingConfig {
            initial_temperature: 2.0,
            ..Default::default()
        });

        let logits = vec![1.0, 2.0, 3.0];
        let scaled = ts.scale_logits(&logits);

        assert_eq!(scaled.len(), 3);
        assert_eq!(scaled[0], 0.5);
        assert_eq!(scaled[1], 1.0);
        assert_eq!(scaled[2], 1.5);
    }

    #[test]
    fn test_calibration() {
        let mut ts = TemperatureScaling::new();

        // Create overconfident predictions
        let predictions = vec![0.9, 0.8, 0.95, 0.85];
        let targets = vec![false, false, false, false];

        ts.calibrate(&predictions, &targets);

        // Temperature should increase to reduce confidence
        assert!(ts.temperature() > 1.0);
    }

    #[test]
    fn test_calibration_error_estimation() {
        let ts = TemperatureScaling::with_config(TemperatureScalingConfig {
            initial_temperature: 1.0,
            ..Default::default()
        });

        let predictions = vec![0.9, 0.1, 0.8, 0.2, 0.7, 0.3];
        let targets = vec![true, false, true, false, true, false];

        let ece = ts.estimate_calibration_error(&predictions, &targets, 10);

        assert!(ece >= 0.0);
        assert!(ece <= 1.0);
    }

    #[test]
    fn test_perfect_calibration() {
        let ts = TemperatureScaling::default();

        // Perfect calibration: confidence matches accuracy
        let predictions = vec![0.5, 0.5, 0.5, 0.5];
        let targets = vec![true, true, false, false];

        let ece = ts.estimate_calibration_error(&predictions, &targets, 10);

        // Should have very low calibration error
        assert!(ece < 0.1);
    }

    #[test]
    fn test_overconfidence_detection() {
        let ts = TemperatureScaling::default();

        // Overconfident: high confidence on all wrong predictions
        let predictions = vec![0.99, 0.99, 0.99, 0.99];
        let targets = vec![false, false, false, false];

        let ece = ts.estimate_calibration_error(&predictions, &targets, 10);

        // Should have high calibration error
        assert!(ece > 0.5);
    }

    #[test]
    fn test_custom_config() {
        let config = TemperatureScalingConfig {
            initial_temperature: 1.5,
            learning_rate: 0.001,
            num_steps: 2000,
        };

        let ts = TemperatureScaling::with_config(config);
        assert_eq!(ts.temperature(), 1.5);
    }

    #[test]
    fn test_confidence_bounds() {
        let ts = TemperatureScaling::with_config(TemperatureScalingConfig {
            initial_temperature: 0.1, // Very low temperature, would cause scaling > 1.0
            ..Default::default()
        });

        let confidence = 0.9;
        let scaled = ts.scale_confidence(confidence);

        // Should be clamped to [0.0, 1.0]
        assert!((0.0..=1.0).contains(&scaled));
    }
}
