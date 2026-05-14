//! Anomaly detection for identifying attack campaigns and suspicious behavior patterns.

use super::session_tracker::{SessionStats, SessionTracker};

/// Result of anomaly detection.
#[derive(Debug, Clone)]
pub struct AnomalyResult {
    /// Whether the session is anomalous
    pub is_anomalous: bool,
    /// Anomaly score (0.0 to 1.0, higher = more anomalous)
    pub anomaly_score: f32,
    /// Reason for anomaly (if detected)
    pub reason: Option<String>,
    /// Confidence in the anomaly detection (0.0 to 1.0)
    pub confidence: f32,
}

impl AnomalyResult {
    /// Create a normal (non-anomalous) result.
    pub fn normal() -> Self {
        Self {
            is_anomalous: false,
            anomaly_score: 0.0,
            reason: None,
            confidence: 1.0,
        }
    }

    /// Create an anomalous result.
    pub fn anomalous(reason: String, score: f32, confidence: f32) -> Self {
        Self {
            is_anomalous: true,
            anomaly_score: score,
            reason: Some(reason),
            confidence,
        }
    }
}

/// Configuration for anomaly detection.
#[derive(Debug, Clone)]
pub struct AnomalyConfig {
    /// Z-score threshold for detecting outliers (typically 2.0-3.0)
    pub z_score_threshold: f32,
    /// Minimum events required for detection
    pub min_events: usize,
    /// Threshold for high injection rate (0.0 to 1.0)
    pub injection_rate_threshold: f32,
    /// Threshold for high confidence injections (0.0 to 1.0)
    pub confidence_threshold: f32,
    /// Topic drift threshold (cosine similarity)
    pub drift_threshold: f32,
    /// Escalation rate threshold (0.5 = 50% increase)
    pub escalation_threshold: f32,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            z_score_threshold: 2.5,
            min_events: 5,
            injection_rate_threshold: 0.4,
            confidence_threshold: 0.75,
            drift_threshold: 0.5,
            escalation_threshold: 0.5,
        }
    }
}

/// Detects anomalies in detection patterns using statistical methods.
#[derive(Debug)]
pub struct AnomalyDetector {
    config: AnomalyConfig,
}

impl AnomalyDetector {
    /// Create a new anomaly detector with default configuration.
    pub fn new() -> Self {
        Self {
            config: AnomalyConfig::default(),
        }
    }

    /// Create a new anomaly detector with custom configuration.
    pub fn with_config(config: AnomalyConfig) -> Self {
        Self { config }
    }

    /// Detect anomalies in a session.
    pub fn detect(&self, tracker: &mut SessionTracker) -> AnomalyResult {
        let stats = tracker.statistics();

        // Need minimum events
        if stats.total_requests < self.config.min_events {
            return AnomalyResult::normal();
        }

        // Check for high injection rate
        if stats.injection_rate > self.config.injection_rate_threshold {
            let score = (stats.injection_rate / self.config.injection_rate_threshold).min(1.0);
            tracker.anomaly_score = score;
            return AnomalyResult::anomalous(
                format!("High injection rate: {:.1}%", stats.injection_rate * 100.0),
                score,
                0.9,
            );
        }

        // Check for high confidence injections
        if stats.avg_injection_confidence > self.config.confidence_threshold {
            let score =
                (stats.avg_injection_confidence / self.config.confidence_threshold).min(1.0);
            tracker.anomaly_score = score;
            return AnomalyResult::anomalous(
                format!(
                    "High confidence injections: {:.2}",
                    stats.avg_injection_confidence
                ),
                score,
                0.85,
            );
        }

        // Check for topic drift
        if tracker.detect_topic_drift(self.config.drift_threshold) {
            tracker.anomaly_score = 0.6;
            return AnomalyResult::anomalous("Topic switching detected".to_string(), 0.6, 0.75);
        }

        // Check for escalation
        if stats.total_requests >= 10
            && tracker.detect_escalation(5, self.config.escalation_threshold)
        {
            tracker.anomaly_score = 0.7;
            return AnomalyResult::anomalous("Attack escalation detected".to_string(), 0.7, 0.8);
        }

        // Check for rapid succession attacks (short time between high-confidence injections)
        if self.detect_rapid_succession(tracker) {
            tracker.anomaly_score = 0.65;
            return AnomalyResult::anomalous(
                "Rapid injection attempts detected".to_string(),
                0.65,
                0.82,
            );
        }

        // All checks passed
        tracker.anomaly_score = 0.0;
        AnomalyResult::normal()
    }

    /// Detect rapid succession of high-confidence injection attempts.
    fn detect_rapid_succession(&self, tracker: &SessionTracker) -> bool {
        use std::time::Duration;

        let recent = tracker.events_in_window(Duration::from_secs(60));
        if recent.len() < 3 {
            return false;
        }

        let high_conf_count = recent
            .iter()
            .filter(|e| e.is_injection && e.confidence > self.config.confidence_threshold)
            .count();

        high_conf_count >= 3
    }

    /// Get baseline statistics for comparison.
    pub fn compute_baseline(&self, trackers: &[&SessionTracker]) -> BaselineStats {
        if trackers.is_empty() {
            return BaselineStats::empty();
        }

        let stats: Vec<SessionStats> = trackers.iter().map(|t| t.statistics()).collect();

        let injection_rates: Vec<f32> = stats.iter().map(|s| s.injection_rate).collect();
        let confidences: Vec<f32> = stats.iter().map(|s| s.avg_injection_confidence).collect();

        BaselineStats {
            avg_injection_rate: self.mean(&injection_rates),
            std_injection_rate: self.std_dev(&injection_rates),
            avg_confidence: self.mean(&confidences),
            std_confidence: self.std_dev(&confidences),
            session_count: trackers.len(),
        }
    }

    /// Calculate z-score for a value against a distribution.
    pub fn z_score(&self, value: f32, mean: f32, std_dev: f32) -> f32 {
        if std_dev == 0.0 {
            return 0.0;
        }
        (value - mean) / std_dev
    }

    /// Detect outliers using z-score method.
    pub fn is_outlier(&self, value: f32, mean: f32, std_dev: f32) -> bool {
        if std_dev == 0.0 {
            // If no standard deviation, check if value is significantly different from mean
            return (value - mean).abs() > 0.2;
        }
        self.z_score(value, mean, std_dev).abs() >= self.config.z_score_threshold
    }

    /// Calculate mean of values.
    fn mean(&self, values: &[f32]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f32>() / values.len() as f32
    }

    /// Calculate standard deviation of values.
    fn std_dev(&self, values: &[f32]) -> f32 {
        if values.len() < 2 {
            return 0.0;
        }
        let mean = self.mean(values);
        let variance =
            values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / (values.len() - 1) as f32;
        variance.sqrt()
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Baseline statistics for comparison across sessions.
#[derive(Debug, Clone)]
pub struct BaselineStats {
    /// Average injection rate across sessions
    pub avg_injection_rate: f32,
    /// Standard deviation of injection rates
    pub std_injection_rate: f32,
    /// Average confidence score across sessions
    pub avg_confidence: f32,
    /// Standard deviation of confidence scores
    pub std_confidence: f32,
    /// Number of sessions in baseline
    pub session_count: usize,
}

impl BaselineStats {
    /// Create empty baseline stats.
    pub fn empty() -> Self {
        Self {
            avg_injection_rate: 0.0,
            std_injection_rate: 0.0,
            avg_confidence: 0.0,
            std_confidence: 0.0,
            session_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::session_tracker::DetectionEvent;

    fn create_test_tracker(injections: Vec<bool>, confidences: Vec<f32>) -> SessionTracker {
        let mut tracker = SessionTracker::new("test-session".to_string());
        for (i, &is_injection) in injections.iter().enumerate() {
            let confidence = confidences.get(i).copied().unwrap_or(0.1);
            tracker.add_event(DetectionEvent::new(
                format!("event-{}", i),
                is_injection,
                confidence,
                vec![0.1, 0.2, 0.3],
            ));
        }
        tracker
    }

    #[test]
    fn test_anomaly_detector_creation() {
        let detector = AnomalyDetector::new();
        assert_eq!(detector.config.z_score_threshold, 2.5);
    }

    #[test]
    fn test_anomaly_result_normal() {
        let result = AnomalyResult::normal();
        assert!(!result.is_anomalous);
        assert_eq!(result.anomaly_score, 0.0);
        assert!(result.reason.is_none());
    }

    #[test]
    fn test_anomaly_result_anomalous() {
        let result = AnomalyResult::anomalous("Test anomaly".to_string(), 0.8, 0.9);
        assert!(result.is_anomalous);
        assert_eq!(result.anomaly_score, 0.8);
        assert!(result.reason.is_some());
    }

    #[test]
    fn test_detect_normal_session() {
        let mut tracker = create_test_tracker(
            vec![false, false, true, false, false],
            vec![0.1, 0.1, 0.6, 0.1, 0.1],
        );
        let detector = AnomalyDetector::new();
        let result = detector.detect(&mut tracker);
        assert!(!result.is_anomalous);
    }

    #[test]
    fn test_detect_high_injection_rate() {
        let mut tracker = create_test_tracker(vec![true, true, true, true, false], vec![0.9; 5]);
        let detector = AnomalyDetector::new();
        let result = detector.detect(&mut tracker);
        assert!(result.is_anomalous);
        assert!(result.reason.is_some());
        assert!(result.reason.unwrap().contains("injection rate"));
    }

    #[test]
    fn test_detect_high_confidence() {
        let mut tracker = create_test_tracker(
            vec![false, false, false, false, false, true, true],
            vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.9, 0.95],
        );
        let detector = AnomalyDetector::new();
        let result = detector.detect(&mut tracker);
        assert!(result.is_anomalous);
        assert!(result.reason.is_some());
        assert!(result.reason.unwrap().contains("confidence"));
    }

    #[test]
    fn test_insufficient_events() {
        let mut tracker = create_test_tracker(vec![true, true], vec![0.9, 0.9]);
        let detector = AnomalyDetector::new();
        let result = detector.detect(&mut tracker);
        assert!(!result.is_anomalous);
    }

    #[test]
    fn test_mean_calculation() {
        let detector = AnomalyDetector::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = detector.mean(&values);
        assert!((mean - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_std_dev_calculation() {
        let detector = AnomalyDetector::new();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let std = detector.std_dev(&values);
        assert!(std > 1.0 && std < 2.0);
    }

    #[test]
    fn test_z_score_calculation() {
        let detector = AnomalyDetector::new();
        let z = detector.z_score(10.0, 5.0, 2.0);
        assert!((z - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_outlier_detection() {
        let detector = AnomalyDetector::new();
        // Value 10 is 2.5 std devs away from mean 5 (outlier)
        assert!(detector.is_outlier(10.0, 5.0, 2.0));
        // Value 6 is only 0.5 std devs away (not outlier)
        assert!(!detector.is_outlier(6.0, 5.0, 2.0));
    }

    #[test]
    fn test_outlier_detection_zero_std() {
        let detector = AnomalyDetector::new();
        // With zero std dev, use absolute difference threshold
        assert!(detector.is_outlier(5.3, 5.0, 0.0));
        assert!(!detector.is_outlier(5.1, 5.0, 0.0));
    }

    #[test]
    fn test_baseline_stats() {
        let detector = AnomalyDetector::new();
        let tracker1 = create_test_tracker(
            vec![true, false, true, false, true],
            vec![0.8, 0.1, 0.8, 0.1, 0.8],
        );
        let tracker2 = create_test_tracker(
            vec![false, false, false, false, true],
            vec![0.1, 0.1, 0.1, 0.1, 0.8],
        );

        let baseline = detector.compute_baseline(&[&tracker1, &tracker2]);
        assert!(baseline.avg_injection_rate > 0.0);
        assert!(baseline.avg_confidence > 0.0);
        assert_eq!(baseline.session_count, 2);
    }

    #[test]
    fn test_custom_config() {
        let config = AnomalyConfig {
            z_score_threshold: 3.0,
            min_events: 10,
            injection_rate_threshold: 0.5,
            confidence_threshold: 0.8,
            drift_threshold: 0.6,
            escalation_threshold: 0.7,
        };
        let detector = AnomalyDetector::with_config(config);
        assert_eq!(detector.config.z_score_threshold, 3.0);
    }

    #[test]
    fn test_rapid_succession_detection() {
        let mut tracker = create_test_tracker(
            vec![true, true, true, true, true],
            vec![0.9, 0.85, 0.88, 0.1, 0.1],
        );
        let detector = AnomalyDetector::new();
        let result = detector.detect(&mut tracker);
        // This should be detected as anomalous due to high confidence
        assert!(result.is_anomalous);
    }

    #[test]
    fn test_anomaly_score_updated() {
        let mut tracker = create_test_tracker(vec![true, true, true, true, true], vec![0.9; 5]);
        let detector = AnomalyDetector::new();
        let result = detector.detect(&mut tracker);
        assert!(result.is_anomalous);
        assert!(tracker.anomaly_score > 0.0);
    }

    #[test]
    fn test_empty_baseline() {
        let detector = AnomalyDetector::new();
        let baseline = detector.compute_baseline(&[]);
        assert_eq!(baseline.session_count, 0);
    }
}
