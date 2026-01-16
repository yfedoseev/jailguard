//! Calibration metrics: ECE, MCE, Brier score, reliability diagrams.

/// Calibration metrics for evaluating confidence calibration.
#[derive(Debug, Clone, Default)]
pub struct CalibrationMetrics {
    /// Expected Calibration Error (average calibration error across bins)
    pub ece: f32,
    /// Maximum Calibration Error (worst calibration error)
    pub mce: f32,
    /// Brier Score (mean squared error between confidence and accuracy)
    pub brier_score: f32,
    /// Number of samples
    pub num_samples: usize,
}

impl CalibrationMetrics {
    /// Check if model is well-calibrated (ECE < 0.05).
    pub fn is_well_calibrated(&self) -> bool {
        self.ece < 0.05
    }

    /// Get calibration quality rating.
    pub fn quality_rating(&self) -> &'static str {
        match self.ece {
            ece if ece < 0.05 => "Excellent",
            ece if ece < 0.10 => "Good",
            ece if ece < 0.15 => "Fair",
            _ => "Poor",
        }
    }
}

/// Compute Expected Calibration Error (ECE).
///
/// ECE measures the average difference between predicted confidence and actual accuracy
/// across binned confidence intervals.
///
/// ECE = `Σ(|N_i`| / N) * |`confidence_i` - `accuracy_i`|
///
/// where `N_i` is the number of samples in bin i, and `confidence_i` and `accuracy_i`
/// are the average confidence and accuracy in that bin.
pub fn compute_ece(predictions: &[f32], targets: &[bool], num_bins: usize) -> f32 {
    if predictions.is_empty() {
        return 0.0;
    }

    let bin_size = 1.0 / num_bins as f32;
    let mut ece = 0.0;

    for bin_idx in 0..num_bins {
        let bin_lower = bin_idx as f32 * bin_size;
        let bin_upper = (bin_idx + 1) as f32 * bin_size;

        // Find samples in this bin
        let bin_samples: Vec<(f32, bool)> = predictions
            .iter()
            .zip(targets.iter())
            .filter(|(pred, _)| {
                let p = pred.clamp(0.0, 1.0);
                p >= bin_lower && p <= bin_upper
            })
            .map(|(&p, &t)| (p.clamp(0.0, 1.0), t))
            .collect();

        if bin_samples.is_empty() {
            continue;
        }

        // Compute average confidence in bin
        let avg_confidence =
            bin_samples.iter().map(|(p, _)| p).sum::<f32>() / bin_samples.len() as f32;

        // Compute accuracy in bin
        let correct = bin_samples.iter().filter(|(_, t)| *t).count();
        let accuracy = correct as f32 / bin_samples.len() as f32;

        // Add weighted calibration error
        let calibration_error = (avg_confidence - accuracy).abs();
        let weight = bin_samples.len() as f32 / predictions.len() as f32;
        ece += weight * calibration_error;
    }

    ece
}

/// Compute Maximum Calibration Error (MCE).
///
/// MCE measures the worst-case calibration error across bins.
///
/// MCE = `max_i` |`confidence_i` - `accuracy_i`|
pub fn compute_mce(predictions: &[f32], targets: &[bool], num_bins: usize) -> f32 {
    if predictions.is_empty() {
        return 0.0;
    }

    let bin_size = 1.0 / num_bins as f32;
    let mut mce = 0.0;

    for bin_idx in 0..num_bins {
        let bin_lower = bin_idx as f32 * bin_size;
        let bin_upper = (bin_idx + 1) as f32 * bin_size;

        // Find samples in this bin
        let bin_samples: Vec<(f32, bool)> = predictions
            .iter()
            .zip(targets.iter())
            .filter(|(pred, _)| {
                let p = pred.clamp(0.0, 1.0);
                p >= bin_lower && p <= bin_upper
            })
            .map(|(&p, &t)| (p.clamp(0.0, 1.0), t))
            .collect();

        if bin_samples.is_empty() {
            continue;
        }

        // Compute average confidence and accuracy in bin
        let avg_confidence =
            bin_samples.iter().map(|(p, _)| p).sum::<f32>() / bin_samples.len() as f32;
        let correct = bin_samples.iter().filter(|(_, t)| *t).count();
        let accuracy = correct as f32 / bin_samples.len() as f32;

        // Update maximum
        let calibration_error = (avg_confidence - accuracy).abs();
        if calibration_error > mce {
            mce = calibration_error;
        }
    }

    mce
}

/// Compute Brier Score.
///
/// Brier Score measures the mean squared error between predicted probabilities
/// and actual binary outcomes.
///
/// BS = (1/N) * Σ(confidence - target)²
///
/// Lower Brier Score is better. Perfect predictions: BS = 0.0.
pub fn compute_brier_score(predictions: &[f32], targets: &[bool]) -> f32 {
    if predictions.is_empty() {
        return 0.0;
    }

    let mut sum_squared_error = 0.0;

    for (pred, &target) in predictions.iter().zip(targets.iter()) {
        let pred_clamped = pred.clamp(0.0, 1.0);
        let target_value = if target { 1.0 } else { 0.0 };
        let error = pred_clamped - target_value;
        sum_squared_error += error * error;
    }

    sum_squared_error / predictions.len() as f32
}

/// Compute reliability diagram data.
///
/// Returns bins with (`avg_confidence`, accuracy, `bin_count`).
pub fn compute_reliability_diagram(
    predictions: &[f32],
    targets: &[bool],
    num_bins: usize,
) -> Vec<(f32, f32, usize)> {
    if predictions.is_empty() {
        return vec![];
    }

    let bin_size = 1.0 / num_bins as f32;
    let mut bins = Vec::new();

    for bin_idx in 0..num_bins {
        let bin_lower = bin_idx as f32 * bin_size;
        let bin_upper = (bin_idx + 1) as f32 * bin_size;

        // Find samples in this bin
        let bin_samples: Vec<(f32, bool)> = predictions
            .iter()
            .zip(targets.iter())
            .filter(|(pred, _)| {
                let p = pred.clamp(0.0, 1.0);
                p >= bin_lower && p <= bin_upper
            })
            .map(|(&p, &t)| (p.clamp(0.0, 1.0), t))
            .collect();

        if bin_samples.is_empty() {
            continue;
        }

        let avg_confidence =
            bin_samples.iter().map(|(p, _)| p).sum::<f32>() / bin_samples.len() as f32;
        let correct = bin_samples.iter().filter(|(_, t)| *t).count();
        let accuracy = correct as f32 / bin_samples.len() as f32;

        bins.push((avg_confidence, accuracy, bin_samples.len()));
    }

    bins
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibration_metrics() {
        let mut metrics = CalibrationMetrics::default();
        metrics.ece = 0.03;
        metrics.num_samples = 100;

        assert!(metrics.is_well_calibrated());
        assert_eq!(metrics.quality_rating(), "Excellent");
    }

    #[test]
    fn test_quality_rating() {
        let mut metrics = CalibrationMetrics::default();

        metrics.ece = 0.02;
        assert_eq!(metrics.quality_rating(), "Excellent");

        metrics.ece = 0.08;
        assert_eq!(metrics.quality_rating(), "Good");

        metrics.ece = 0.12;
        assert_eq!(metrics.quality_rating(), "Fair");

        metrics.ece = 0.20;
        assert_eq!(metrics.quality_rating(), "Poor");
    }

    #[test]
    fn test_perfect_calibration_ece() {
        // Perfect calibration: confidence matches accuracy
        let predictions = vec![0.5, 0.5, 0.5, 0.5];
        let targets = vec![true, true, false, false];

        let ece = compute_ece(&predictions, &targets, 10);

        // Should have very low ECE
        assert!(ece < 0.1);
    }

    #[test]
    fn test_overconfident_ece() {
        // Overconfident: all predictions at 0.9, but only 50% accurate
        let predictions = vec![0.9, 0.9, 0.9, 0.9];
        let targets = vec![true, true, false, false];

        let ece = compute_ece(&predictions, &targets, 10);

        // Should have high ECE
        assert!(ece > 0.3);
    }

    #[test]
    fn test_underconfident_ece() {
        // Underconfident: moderate confidence but perfect accuracy
        let predictions = vec![0.6, 0.6, 0.6, 0.6];
        let targets = vec![true, true, true, true];

        let ece = compute_ece(&predictions, &targets, 10);

        // Should have some calibration error (underconfidence)
        assert!((0.0..=1.0).contains(&ece));
    }

    #[test]
    fn test_mce_computation() {
        let predictions = vec![0.9, 0.9, 0.1, 0.1];
        let targets = vec![true, false, true, false];

        let mce = compute_mce(&predictions, &targets, 10);

        assert!((0.0..=1.0).contains(&mce));
    }

    #[test]
    fn test_brier_score_perfect() {
        // Perfect predictions
        let predictions = vec![1.0, 0.0, 1.0, 0.0];
        let targets = vec![true, false, true, false];

        let bs = compute_brier_score(&predictions, &targets);

        // Should be zero (perfect predictions)
        assert!(bs < 0.001);
    }

    #[test]
    fn test_brier_score_worst() {
        // Worst predictions (anti-predictions)
        let predictions = vec![0.0, 1.0, 0.0, 1.0];
        let targets = vec![true, false, true, false];

        let bs = compute_brier_score(&predictions, &targets);

        // Should be 1.0 (worst possible)
        assert!((bs - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_brier_score_average() {
        // Average predictions (50% confidence)
        let predictions = vec![0.5, 0.5, 0.5, 0.5];
        let targets = vec![true, true, false, false];

        let bs = compute_brier_score(&predictions, &targets);

        // Should be 0.25 (perfect average)
        assert!((bs - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_reliability_diagram() {
        let predictions = vec![0.1, 0.2, 0.5, 0.8, 0.9];
        let targets = vec![false, false, true, true, true];

        let diagram = compute_reliability_diagram(&predictions, &targets, 5);

        // Should have multiple bins
        assert!(!diagram.is_empty());

        // Each entry should be (avg_confidence, accuracy, count)
        for (conf, acc, count) in diagram {
            assert!((0.0..=1.0).contains(&conf));
            assert!((0.0..=1.0).contains(&acc));
            assert!(count > 0);
        }
    }

    #[test]
    fn test_empty_predictions() {
        let predictions: Vec<f32> = vec![];
        let targets: Vec<bool> = vec![];

        let ece = compute_ece(&predictions, &targets, 10);
        let mce = compute_mce(&predictions, &targets, 10);
        let bs = compute_brier_score(&predictions, &targets);

        assert_eq!(ece, 0.0);
        assert_eq!(mce, 0.0);
        assert_eq!(bs, 0.0);
    }

    #[test]
    fn test_confidence_bounds() {
        // Test that values outside [0, 1] are properly handled
        let predictions = vec![1.5, -0.5, 0.5];
        let targets = vec![true, false, true];

        let ece = compute_ece(&predictions, &targets, 10);
        let mce = compute_mce(&predictions, &targets, 10);
        let bs = compute_brier_score(&predictions, &targets);

        assert!((0.0..=1.0).contains(&ece));
        assert!((0.0..=1.0).contains(&mce));
        assert!((0.0..=2.0).contains(&bs)); // Max error is 1.0 per sample
    }
}
