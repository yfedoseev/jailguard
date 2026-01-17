//! Performance metrics and statistics collection
//!
//! Tracks:
//! - Ensemble detection performance
//! - Model agreement statistics
//! - Confidence calibration metrics

use crate::detection::EnsembleDetectionResult;
use std::collections::VecDeque;

/// Ensemble voting profile
#[derive(Debug, Clone)]
pub struct EnsembleProfile {
    /// Ensemble accuracy (estimated from ground truth)
    pub accuracy: f32,
    /// Average agreement score
    pub avg_agreement: f32,
    /// Average confidence
    pub avg_confidence: f32,
    /// Average confidence variance
    pub avg_variance: f32,
    /// False positive rate
    pub false_positive_rate: f32,
    /// False negative rate
    pub false_negative_rate: f32,
    /// Samples analyzed
    pub sample_count: usize,
}

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Recent ensemble results (for windowed analysis)
    results: VecDeque<EnsembleDetectionResult>,
    window_size: usize,
}

impl PerformanceMetrics {
    /// Create new metrics collector
    pub fn new(window_size: usize) -> Self {
        Self {
            results: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    /// Record an ensemble detection result
    pub fn record(&mut self, result: EnsembleDetectionResult) {
        if self.results.len() >= self.window_size {
            self.results.pop_front();
        }
        self.results.push_back(result);
    }

    /// Get average agreement score over window
    pub fn avg_agreement_score(&self) -> f32 {
        if self.results.is_empty() {
            0.0
        } else {
            self.results.iter().map(|r| r.agreement_score).sum::<f32>() / self.results.len() as f32
        }
    }

    /// Get average ensemble confidence
    pub fn avg_ensemble_confidence(&self) -> f32 {
        if self.results.is_empty() {
            0.0
        } else {
            self.results
                .iter()
                .map(|r| r.ensemble_confidence)
                .sum::<f32>()
                / self.results.len() as f32
        }
    }

    /// Get average confidence variance
    pub fn avg_confidence_variance(&self) -> f32 {
        if self.results.is_empty() {
            0.0
        } else {
            self.results
                .iter()
                .map(|r| r.confidence_variance)
                .sum::<f32>()
                / self.results.len() as f32
        }
    }

    /// Get injection rate (estimated)
    pub fn injection_rate(&self) -> f32 {
        if self.results.is_empty() {
            0.0
        } else {
            let injections = self
                .results
                .iter()
                .filter(|r| r.result.is_injection)
                .count();
            injections as f32 / self.results.len() as f32
        }
    }

    /// Get high-agreement rate (agreement >= 0.8)
    pub fn high_agreement_rate(&self) -> f32 {
        if self.results.is_empty() {
            0.0
        } else {
            let high = self
                .results
                .iter()
                .filter(|r| r.agreement_score >= 0.8)
                .count();
            high as f32 / self.results.len() as f32
        }
    }

    /// Get low-variance rate (variance < 0.05)
    pub fn low_variance_rate(&self) -> f32 {
        if self.results.is_empty() {
            0.0
        } else {
            let low = self
                .results
                .iter()
                .filter(|r| r.confidence_variance < 0.05)
                .count();
            low as f32 / self.results.len() as f32
        }
    }

    /// Get sample count
    pub fn sample_count(&self) -> usize {
        self.results.len()
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.results.clear();
    }

    /// Get summary metrics
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            sample_count: self.sample_count(),
            avg_agreement: self.avg_agreement_score(),
            avg_confidence: self.avg_ensemble_confidence(),
            avg_variance: self.avg_confidence_variance(),
            injection_rate: self.injection_rate(),
            high_agreement_rate: self.high_agreement_rate(),
            low_variance_rate: self.low_variance_rate(),
        }
    }

    /// Print metrics summary
    pub fn print_summary(&self) {
        if self.results.is_empty() {
            println!("No metrics recorded");
            return;
        }

        let summary = self.summary();
        println!("\n=== Performance Metrics Summary ===");
        println!("Samples:              {}", summary.sample_count);
        println!(
            "Injection Rate:       {:.1}%",
            summary.injection_rate * 100.0
        );
        println!("\nAgreement Metrics:");
        println!("  Avg Agreement:      {:.3}", summary.avg_agreement);
        println!(
            "  High Agreement:     {:.1}%",
            summary.high_agreement_rate * 100.0
        );
        println!("\nConfidence Metrics:");
        println!("  Avg Confidence:     {:.3}", summary.avg_confidence);
        println!("  Avg Variance:       {:.5}", summary.avg_variance);
        println!(
            "  Low Variance:       {:.1}%",
            summary.low_variance_rate * 100.0
        );
        println!("====================================\n");
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new(10000)
    }
}

/// Metrics summary snapshot
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    /// Number of samples analyzed
    pub sample_count: usize,
    /// Average agreement score
    pub avg_agreement: f32,
    /// Average ensemble confidence
    pub avg_confidence: f32,
    /// Average confidence variance
    pub avg_variance: f32,
    /// Injection detection rate
    pub injection_rate: f32,
    /// High agreement rate (>= 0.8)
    pub high_agreement_rate: f32,
    /// Low variance rate (< 0.05)
    pub low_variance_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::DetectionResult;

    fn create_ensemble_result(
        agreement: f32,
        confidence: f32,
        variance: f32,
        is_injection: bool,
    ) -> EnsembleDetectionResult {
        EnsembleDetectionResult {
            result: DetectionResult::new(is_injection, confidence, [confidence, 1.0 - confidence]),
            detector_votes: vec![],
            agreement_score: agreement,
            ensemble_confidence: confidence,
            confidence_variance: variance,
        }
    }

    #[test]
    fn test_metrics_collection() {
        let mut metrics = PerformanceMetrics::new(100);

        metrics.record(create_ensemble_result(0.95, 0.9, 0.01, true));
        metrics.record(create_ensemble_result(0.95, 0.85, 0.02, true));
        metrics.record(create_ensemble_result(0.8, 0.3, 0.05, false));

        assert_eq!(metrics.sample_count(), 3);
        assert!(metrics.avg_agreement_score() > 0.8);
        assert!(metrics.avg_ensemble_confidence() > 0.5);
    }

    #[test]
    fn test_injection_rate() {
        let mut metrics = PerformanceMetrics::new(100);

        metrics.record(create_ensemble_result(0.9, 0.9, 0.01, true));
        metrics.record(create_ensemble_result(0.9, 0.9, 0.01, true));
        metrics.record(create_ensemble_result(0.8, 0.3, 0.05, false));

        assert!((metrics.injection_rate() - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_agreement_rates() {
        let mut metrics = PerformanceMetrics::new(100);

        metrics.record(create_ensemble_result(0.85, 0.9, 0.01, true));
        metrics.record(create_ensemble_result(0.75, 0.8, 0.1, true));

        assert_eq!(metrics.high_agreement_rate(), 0.5); // 1 out of 2
        assert_eq!(metrics.low_variance_rate(), 0.5); // 1 out of 2
    }
}
