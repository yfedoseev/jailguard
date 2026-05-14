/*!
Model Calibration Evaluator

Measures how well the model's confidence scores match actual correctness:
- Expected Calibration Error (ECE): Average difference between confidence and accuracy
- Maximum Calibration Error (MCE): Worst-case calibration error
- Brier Score: Mean squared error between predicted probabilities and ground truth
- Reliability Diagram: Calibration curve analysis

A well-calibrated model has:
- ECE < 0.05 (excellent)
- MCE < 0.10 (good)
- Brier Score < 0.10 (good)

Usage:
    let mut evaluator = CalibrationEvaluator::new(num_bins=10);
    evaluator.add_prediction(confidence=0.85, is_correct=true);
    let report = evaluator.evaluate();
*/

/// Calibration metrics for confidence-accuracy alignment
#[derive(Debug, Clone)]
pub struct CalibrationMetrics {
    /// Expected Calibration Error (ECE)
    /// Average absolute difference between confidence and accuracy
    pub expected_calibration_error: f32,

    /// Maximum Calibration Error (MCE)
    /// Worst-case absolute difference
    pub maximum_calibration_error: f32,

    /// Brier Score
    /// Mean squared difference between predicted probability and ground truth
    pub brier_score: f32,

    /// Overconfidence
    /// Average difference when predicted > actual
    pub overconfidence: f32,

    /// Underconfidence
    /// Average difference when predicted < actual
    pub underconfidence: f32,
}

/// Bin for reliability diagram
#[derive(Debug, Clone)]
pub struct CalibrationBin {
    /// Lower bound of confidence interval
    pub lower: f32,
    /// Upper bound of confidence interval
    pub upper: f32,
    /// Number of samples in this bin
    pub count: usize,
    /// Average confidence in this bin
    pub avg_confidence: f32,
    /// Accuracy in this bin
    pub accuracy: f32,
    /// Calibration gap (avg_confidence - accuracy)
    pub gap: f32,
}

/// Model calibration evaluator
pub struct CalibrationEvaluator {
    /// Confidence scores and correctness flags
    pub predictions: Vec<(f32, bool)>,
    /// Number of bins for reliability diagram
    pub num_bins: usize,
}

impl CalibrationEvaluator {
    /// Create new calibration evaluator
    pub fn new(num_bins: usize) -> Self {
        Self {
            predictions: Vec::new(),
            num_bins: num_bins.max(5).min(20), // Clamp between 5 and 20
        }
    }

    /// Add a prediction with confidence and correctness
    pub fn add_prediction(&mut self, confidence: f32, is_correct: bool) {
        let clamped_confidence = confidence.max(0.0).min(1.0);
        self.predictions.push((clamped_confidence, is_correct));
    }

    /// Add multiple predictions
    pub fn add_predictions(&mut self, predictions: Vec<(f32, bool)>) {
        for (confidence, is_correct) in predictions {
            self.add_prediction(confidence, is_correct);
        }
    }

    /// Compute Expected Calibration Error (ECE)
    fn compute_ece(&self) -> f32 {
        if self.predictions.is_empty() {
            return 0.0;
        }

        let bins = self.create_bins();
        let mut total_error = 0.0;

        for bin in bins {
            if bin.count > 0 {
                let bin_weight = bin.count as f32 / self.predictions.len() as f32;
                let bin_error = (bin.avg_confidence - bin.accuracy).abs();
                total_error += bin_weight * bin_error;
            }
        }

        total_error
    }

    /// Compute Maximum Calibration Error (MCE)
    fn compute_mce(&self) -> f32 {
        if self.predictions.is_empty() {
            return 0.0;
        }

        let bins = self.create_bins();
        let mut max_error: f32 = 0.0;

        for bin in bins {
            if bin.count > 0 {
                let bin_error = (bin.avg_confidence - bin.accuracy).abs();
                max_error = max_error.max(bin_error);
            }
        }

        max_error
    }

    /// Compute Brier Score
    fn compute_brier_score(&self) -> f32 {
        if self.predictions.is_empty() {
            return 0.0;
        }

        let mut sum_squared_error = 0.0;

        for (confidence, is_correct) in &self.predictions {
            let predicted_prob = *confidence;
            let actual_prob = if *is_correct { 1.0 } else { 0.0 };
            let error = predicted_prob - actual_prob;
            sum_squared_error += error * error;
        }

        sum_squared_error / self.predictions.len() as f32
    }

    /// Compute overconfidence and underconfidence
    fn compute_confidence_gaps(&self) -> (f32, f32) {
        if self.predictions.is_empty() {
            return (0.0, 0.0);
        }

        let mut over_errors = Vec::new();
        let mut under_errors = Vec::new();

        for (confidence, is_correct) in &self.predictions {
            let actual = if *is_correct { 1.0 } else { 0.0 };
            let gap = confidence - actual;

            if gap > 0.0 {
                over_errors.push(gap);
            } else if gap < 0.0 {
                under_errors.push(-gap);
            }
        }

        let overconfidence = if !over_errors.is_empty() {
            over_errors.iter().sum::<f32>() / over_errors.len() as f32
        } else {
            0.0
        };

        let underconfidence = if !under_errors.is_empty() {
            under_errors.iter().sum::<f32>() / under_errors.len() as f32
        } else {
            0.0
        };

        (overconfidence, underconfidence)
    }

    /// Create calibration bins for reliability diagram
    fn create_bins(&self) -> Vec<CalibrationBin> {
        let bin_size = 1.0 / self.num_bins as f32;
        let mut bins: Vec<CalibrationBin> = (0..self.num_bins)
            .map(|i| {
                let lower = i as f32 * bin_size;
                let upper = (i + 1) as f32 * bin_size;
                CalibrationBin {
                    lower,
                    upper,
                    count: 0,
                    avg_confidence: 0.0,
                    accuracy: 0.0,
                    gap: 0.0,
                }
            })
            .collect();

        // Assign predictions to bins
        for (confidence, is_correct) in &self.predictions {
            let bin_idx = (*confidence / bin_size).floor() as usize;
            let bin_idx = bin_idx.min(self.num_bins - 1);

            bins[bin_idx].count += 1;
            bins[bin_idx].avg_confidence += confidence;
            if *is_correct {
                bins[bin_idx].accuracy += 1.0;
            }
        }

        // Compute averages
        for bin in &mut bins {
            if bin.count > 0 {
                bin.avg_confidence /= bin.count as f32;
                bin.accuracy /= bin.count as f32;
                bin.gap = bin.avg_confidence - bin.accuracy;
            }
        }

        bins
    }

    /// Evaluate calibration
    pub fn evaluate(&self) -> CalibrationMetrics {
        let (overconfidence, underconfidence) = self.compute_confidence_gaps();

        CalibrationMetrics {
            expected_calibration_error: self.compute_ece(),
            maximum_calibration_error: self.compute_mce(),
            brier_score: self.compute_brier_score(),
            overconfidence,
            underconfidence,
        }
    }

    /// Generate detailed calibration report
    pub fn generate_report(&self) -> String {
        let metrics = self.evaluate();
        let bins = self.create_bins();

        let mut report = String::new();
        report.push_str(&"=".repeat(80));
        report.push_str("\n📊 MODEL CALIBRATION REPORT\n");
        report.push_str(&"=".repeat(80));
        report.push('\n');

        // Summary metrics
        report.push_str("\n🎯 Calibration Metrics:\n");
        report.push_str(&format!(
            "  ECE (Expected Calibration Error):  {:.4}\n",
            metrics.expected_calibration_error
        ));
        report.push_str(&format!(
            "  MCE (Maximum Calibration Error):   {:.4}\n",
            metrics.maximum_calibration_error
        ));
        report.push_str(&format!(
            "  Brier Score:                       {:.4}\n",
            metrics.brier_score
        ));

        // Interpretation
        report.push_str("\n📈 Calibration Quality:\n");

        if metrics.expected_calibration_error < 0.05 {
            report.push_str("  ✅ EXCELLENT: ECE < 0.05 - Very well calibrated\n");
        } else if metrics.expected_calibration_error < 0.10 {
            report.push_str("  ✓ GOOD: ECE < 0.10 - Well calibrated\n");
        } else if metrics.expected_calibration_error < 0.15 {
            report.push_str("  ⚠️  FAIR: ECE < 0.15 - Moderately calibrated\n");
        } else {
            report.push_str("  ❌ POOR: ECE >= 0.15 - Poorly calibrated\n");
        }

        if metrics.brier_score < 0.05 {
            report.push_str("  ✅ EXCELLENT: Brier < 0.05\n");
        } else if metrics.brier_score < 0.10 {
            report.push_str("  ✓ GOOD: Brier < 0.10\n");
        } else {
            report.push_str("  ⚠️  NEEDS IMPROVEMENT: Brier >= 0.10\n");
        }

        // Confidence gaps
        report.push_str("\n📊 Confidence Patterns:\n");
        report.push_str(&format!(
            "  Overconfidence (avg):   {:.4}\n",
            metrics.overconfidence
        ));
        report.push_str(&format!(
            "  Underconfidence (avg):  {:.4}\n",
            metrics.underconfidence
        ));

        // Reliability diagram table
        report.push_str("\n📈 Reliability Diagram (by confidence bin):\n");
        report.push_str("  Confidence  Count  Accuracy  Gap\n");
        report.push_str(&format!("  {}\n", "-".repeat(40)));

        for bin in bins {
            if bin.count > 0 {
                report.push_str(&format!(
                    "  [{:.2}-{:.2}]  {:>5}   {:.4}   {:.4}\n",
                    bin.lower, bin.upper, bin.count, bin.accuracy, bin.gap
                ));
            }
        }

        report.push('\n');
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_calibration() {
        let mut eval = CalibrationEvaluator::new(10);
        // High confidence predictions that are correct
        eval.add_prediction(0.95, true);
        eval.add_prediction(0.90, true);
        // Low confidence predictions that are incorrect
        eval.add_prediction(0.1, false);
        eval.add_prediction(0.05, false);

        let metrics = eval.evaluate();
        // Should have low ECE for perfect calibration
        assert!(metrics.expected_calibration_error < 0.2);
    }

    #[test]
    fn test_overconfident_model() {
        let mut eval = CalibrationEvaluator::new(10);
        // High confidence but incorrect
        eval.add_prediction(0.9, false);
        eval.add_prediction(0.85, false);

        let metrics = eval.evaluate();
        // Should have high overconfidence
        assert!(metrics.overconfidence > 0.7);
    }

    #[test]
    fn test_brier_score() {
        let mut eval = CalibrationEvaluator::new(10);
        // Perfect predictions
        eval.add_prediction(1.0, true);
        eval.add_prediction(0.0, false);

        let metrics = eval.evaluate();
        // Brier score should be 0 for perfect predictions
        assert!(metrics.brier_score < 0.01);
    }
}
