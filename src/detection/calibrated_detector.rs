//! Calibrated detector with temperature scaling for reliable confidence scores.

use crate::detection::TransformerDetector;

/// Temperature scaling for calibrating neural network confidence scores.
///
/// Divides raw confidence by a learned scalar T so that stated probabilities
/// match empirical accuracy (T > 1 reduces overconfidence, T < 1 increases it).
#[derive(Debug, Clone)]
pub struct TemperatureScaling {
    temperature: f32,
}

impl TemperatureScaling {
    /// Create with default temperature 1.0 (identity — no adjustment).
    pub fn new() -> Self {
        Self { temperature: 1.0 }
    }

    /// Create with a fixed temperature value.
    pub fn with_temperature(temperature: f32) -> Self {
        Self { temperature }
    }

    /// Return the current temperature.
    pub fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Override the temperature directly.
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = temperature;
    }

    /// Apply temperature scaling to a raw confidence value.
    pub fn scale_confidence(&self, confidence: f32) -> f32 {
        (confidence / self.temperature).clamp(0.0, 1.0)
    }

    /// Fit temperature via grid search to minimise NLL on a validation set.
    pub fn calibrate(&mut self, predictions: &[f32], targets: &[bool]) {
        if predictions.is_empty() {
            return;
        }
        let mut best_temp = self.temperature;
        let mut best_nll = Self::compute_nll(predictions, targets, best_temp);
        let mut t = 0.1f32;
        while t <= 10.0 {
            let nll = Self::compute_nll(predictions, targets, t);
            if nll < best_nll {
                best_nll = nll;
                best_temp = t;
            }
            t += 0.1;
        }
        self.temperature = best_temp;
    }

    fn compute_nll(predictions: &[f32], targets: &[bool], temperature: f32) -> f32 {
        let mut total = 0.0f32;
        for (pred, &target) in predictions.iter().zip(targets.iter()) {
            let scaled = (pred / temperature).clamp(1e-7, 1.0 - 1e-7);
            total += if target { -scaled.ln() } else { -(1.0 - scaled).ln() };
        }
        total / predictions.len() as f32
    }
}

impl Default for TemperatureScaling {
    fn default() -> Self {
        Self::new()
    }
}

/// Calibrated detector with temperature-scaled confidence scores.
///
/// Wraps a transformer detector and applies temperature scaling to produce
/// well-calibrated confidence estimates.
pub struct CalibratedDetector {
    /// Underlying transformer detector
    detector: TransformerDetector,
    /// Temperature scaling for confidence calibration
    temperature_scaling: TemperatureScaling,
}

impl CalibratedDetector {
    /// Create a new calibrated detector from a transformer detector.
    pub fn new(detector: TransformerDetector) -> Self {
        Self {
            detector,
            temperature_scaling: TemperatureScaling::new(),
        }
    }

    /// Create with custom temperature scaling.
    pub fn with_temperature_scaling(
        detector: TransformerDetector,
        temperature_scaling: TemperatureScaling,
    ) -> Self {
        Self {
            detector,
            temperature_scaling,
        }
    }

    /// Get the current temperature.
    pub fn temperature(&self) -> f32 {
        self.temperature_scaling.temperature()
    }

    /// Set the temperature.
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature_scaling.set_temperature(temperature);
    }

    /// Get a mutable reference to the temperature scaling.
    pub fn temperature_scaling_mut(&mut self) -> &mut TemperatureScaling {
        &mut self.temperature_scaling
    }

    /// Run detection with calibrated confidence scores.
    pub fn detect(&self, text: &str) -> CalibratedDetectionResult {
        // Get raw detection from transformer
        let raw_result = self.detector.detect(text);

        // Scale confidence
        let scaled_confidence = self
            .temperature_scaling
            .scale_confidence(raw_result.detection.confidence);

        // Create calibrated result
        CalibratedDetectionResult {
            raw: raw_result.clone(),
            scaled_confidence,
            temperature: self.temperature_scaling.temperature(),
        }
    }

    /// Calibrate temperature on validation data.
    ///
    /// Uses validation predictions and targets to optimize temperature
    /// for better calibration.
    pub fn calibrate_temperature(&mut self, predictions: &[f32], targets: &[bool]) {
        self.temperature_scaling.calibrate(predictions, targets);
    }

    /// Get underlying detector for advanced usage.
    pub fn inner_detector(&self) -> &TransformerDetector {
        &self.detector
    }

    /// Get mutable reference to underlying detector.
    pub fn inner_detector_mut(&mut self) -> &mut TransformerDetector {
        &mut self.detector
    }
}

/// Detection result from calibrated detector.
#[derive(Debug, Clone)]
pub struct CalibratedDetectionResult {
    /// Raw detection result (unscaled confidence)
    pub raw: crate::detection::MultiTaskDetectionResult,
    /// Scaled confidence (temperature-adjusted)
    pub scaled_confidence: f32,
    /// Temperature used for scaling
    pub temperature: f32,
}

impl CalibratedDetectionResult {
    /// Check if detection should block (scaled confidence above threshold).
    pub fn should_block(&self, threshold: f32) -> bool {
        self.scaled_confidence >= threshold
    }

    /// Get calibration improvement factor.
    ///
    /// Measures how much the scaling adjusted confidence.
    pub fn calibration_adjustment(&self) -> f32 {
        (self.raw.detection.confidence - self.scaled_confidence).abs()
    }

    /// Check if temperature is high (overconfidence correction).
    pub fn is_overconfidence_corrected(&self) -> bool {
        self.temperature > 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibrated_detector_creation() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let calibrated = CalibratedDetector::new(detector);

        assert_eq!(calibrated.temperature(), 1.0);
    }

    #[test]
    fn test_temperature_setting() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let mut calibrated = CalibratedDetector::new(detector);

        calibrated.set_temperature(2.0);
        assert_eq!(calibrated.temperature(), 2.0);
    }

    #[test]
    fn test_detection_with_scaling() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let mut calibrated = CalibratedDetector::new(detector);

        // Set high temperature to reduce confidence
        calibrated.set_temperature(2.0);

        let result = calibrated.detect("Test input");

        // Scaled confidence should be less than or equal to raw confidence
        assert!(result.scaled_confidence <= result.raw.detection.confidence);
    }

    #[test]
    fn test_calibrated_result_blocking() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let mut calibrated = CalibratedDetector::new(detector);

        // Set temperature to make confidence very low
        calibrated.set_temperature(10.0);

        let result = calibrated.detect("Potential injection");

        // Even if raw confidence is high, scaled should be low
        assert!(!result.should_block(0.5));
    }

    #[test]
    fn test_calibration_adjustment() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let mut calibrated = CalibratedDetector::new(detector);

        // Set temperature
        calibrated.set_temperature(2.0);

        let result = calibrated.detect("Test");

        // Adjustment should be positive
        assert!(result.calibration_adjustment() >= 0.0);
    }

    #[test]
    fn test_overconfidence_correction_detection() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let mut calibrated = CalibratedDetector::new(detector);

        // Set high temperature
        calibrated.set_temperature(2.0);
        let result_high = calibrated.detect("Test");
        assert!(result_high.is_overconfidence_corrected());

        // Set low temperature
        calibrated.set_temperature(0.5);
        let result_low = calibrated.detect("Test");
        assert!(!result_low.is_overconfidence_corrected());
    }

    #[test]
    fn test_temperature_calibration() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let mut calibrated = CalibratedDetector::new(detector);

        // Create sample predictions and targets
        let predictions = vec![0.9, 0.8, 0.95, 0.85];
        let targets = vec![true, true, false, false];

        let initial_temp = calibrated.temperature();
        calibrated.calibrate_temperature(&predictions, &targets);
        let final_temp = calibrated.temperature();

        // Temperature should change after calibration
        assert_ne!(initial_temp, final_temp);
    }

    #[test]
    fn test_custom_temperature_scaling() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let ts = TemperatureScaling::with_temperature(1.5);
        let calibrated = CalibratedDetector::with_temperature_scaling(detector, ts);
        assert_eq!(calibrated.temperature(), 1.5);
    }
}
