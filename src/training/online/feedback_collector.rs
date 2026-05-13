//! Feedback collection for online learning and continuous model improvement.

use crate::training::multitask_sample::MultiTaskSample;
use crate::detection::MultiTaskDetectionResult;
use std::collections::VecDeque;
use std::time::SystemTime;

/// A piece of feedback from the system or user.
#[derive(Debug, Clone)]
pub struct FeedbackSample {
    /// Original input text
    pub text: String,
    /// Model's prediction
    pub predicted: MultiTaskDetectionResult,
    /// Actual label (user correction)
    pub actual: Option<bool>,
    /// Timestamp of feedback
    pub timestamp: SystemTime,
    /// Optional detailed feedback on attack type
    pub actual_attack_type: Option<u32>,
}

impl FeedbackSample {
    /// Create a new feedback sample.
    pub fn new(text: String, predicted: MultiTaskDetectionResult, actual: Option<bool>) -> Self {
        Self {
            text,
            predicted,
            actual,
            timestamp: SystemTime::now(),
            actual_attack_type: None,
        }
    }

    /// Add attack type feedback.
    pub fn with_attack_type(mut self, attack_type: u32) -> Self {
        self.actual_attack_type = Some(attack_type);
        self
    }

    /// Check if this feedback represents a correction (predicted != actual).
    pub fn is_correction(&self) -> bool {
        if let Some(actual) = self.actual {
            self.predicted.detection.is_injection != actual
        } else {
            false
        }
    }

    /// Convert feedback sample to multi-task training sample.
    pub fn to_training_sample(&self) -> MultiTaskSample {
        let attack_type = if let Some(attack_idx) = self.actual_attack_type {
            crate::detection::AttackType::from_index(attack_idx as usize)
                .unwrap_or(self.predicted.attack_type)
        } else {
            self.predicted.attack_type
        };

        MultiTaskSample {
            text: self.text.clone(),
            is_injection: self.actual.unwrap_or(self.predicted.detection.is_injection),
            attack_type,
            expected_output: None,
        }
    }
}

/// Configuration for feedback collection.
#[derive(Debug, Clone)]
pub struct FeedbackCollectorConfig {
    /// Maximum number of feedback samples to store
    pub max_size: usize,
    /// Only collect feedback on confident predictions (above this threshold)
    pub confidence_threshold: f32,
    /// Collect feedback on all predictions regardless of confidence
    pub collect_all: bool,
}

impl Default for FeedbackCollectorConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            confidence_threshold: 0.5,
            collect_all: true,
        }
    }
}

/// Collects and manages feedback for continuous learning.
#[derive(Debug)]
pub struct FeedbackCollector {
    /// Circular buffer of feedback samples
    buffer: VecDeque<FeedbackSample>,
    /// Configuration
    config: FeedbackCollectorConfig,
    /// Total feedback received (for statistics)
    total_received: usize,
    /// Number of corrections received
    total_corrections: usize,
}

impl FeedbackCollector {
    /// Create a new feedback collector with default config.
    pub fn new() -> Self {
        Self::with_config(FeedbackCollectorConfig::default())
    }

    /// Create with custom configuration.
    pub fn with_config(config: FeedbackCollectorConfig) -> Self {
        Self {
            buffer: VecDeque::with_capacity(config.max_size),
            config,
            total_received: 0,
            total_corrections: 0,
        }
    }

    /// Add a feedback sample to the collection.
    pub fn add_feedback(&mut self, sample: FeedbackSample) {
        // Check if we should collect this feedback
        if !self.config.collect_all
            && sample.predicted.detection.confidence < self.config.confidence_threshold
        {
            return;
        }

        // Track statistics
        self.total_received += 1;
        if sample.is_correction() {
            self.total_corrections += 1;
        }

        // Add to buffer (FIFO, drop oldest if full)
        if self.buffer.len() >= self.config.max_size {
            self.buffer.pop_front();
        }
        self.buffer.push_back(sample);
    }

    /// Get current buffer size.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get feedback samples for training.
    pub fn get_training_batch(&self, size: usize) -> Vec<MultiTaskSample> {
        self.buffer
            .iter()
            .take(size)
            .map(FeedbackSample::to_training_sample)
            .collect()
    }

    /// Get recent corrections (last N feedback samples that are corrections).
    pub fn get_recent_corrections(&self, count: usize) -> Vec<MultiTaskSample> {
        self.buffer
            .iter()
            .rev()
            .filter(|s| s.is_correction())
            .take(count)
            .map(FeedbackSample::to_training_sample)
            .collect()
    }

    /// Get correction ratio (corrections / total received).
    pub fn correction_ratio(&self) -> f32 {
        if self.total_received == 0 {
            0.0
        } else {
            self.total_corrections as f32 / self.total_received as f32
        }
    }

    /// Clear all collected feedback.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.total_received = 0;
        self.total_corrections = 0;
    }

    /// Get all feedback samples (for inspection).
    pub fn all_samples(&self) -> Vec<&FeedbackSample> {
        self.buffer.iter().collect()
    }

    /// Get statistics about collected feedback.
    pub fn statistics(&self) -> FeedbackStatistics {
        let total_samples = self.buffer.len();
        let corrections = self.buffer.iter().filter(|s| s.is_correction()).count();

        FeedbackStatistics {
            total_received: self.total_received,
            stored_samples: total_samples,
            corrections_count: corrections,
            correction_ratio: self.correction_ratio(),
            avg_confidence: if total_samples == 0 {
                0.0
            } else {
                self.buffer
                    .iter()
                    .map(|s| s.predicted.detection.confidence)
                    .sum::<f32>()
                    / total_samples as f32
            },
        }
    }
}

impl Default for FeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about collected feedback.
#[derive(Debug, Clone)]
pub struct FeedbackStatistics {
    /// Total feedback items ever received
    pub total_received: usize,
    /// Currently stored samples
    pub stored_samples: usize,
    /// Number of corrections in buffer
    pub corrections_count: usize,
    /// Ratio of corrections to total received
    pub correction_ratio: f32,
    /// Average confidence of stored predictions
    pub avg_confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::{AttackType, DetectionResult};

    fn create_test_result(confidence: f32, is_injection: bool) -> MultiTaskDetectionResult {
        MultiTaskDetectionResult {
            detection: DetectionResult::new(
                is_injection,
                confidence,
                [
                    if is_injection { 0.9 } else { 0.1 },
                    if is_injection { 0.1 } else { 0.9 },
                ],
            ),
            attack_type: AttackType::RolePlay,
            attack_probs: [0.2; 8],
            semantic_score: 0.5,
            embedding: vec![],
        }
    }

    #[test]
    fn test_feedback_sample_creation() {
        let result = create_test_result(0.8, true);
        let sample = FeedbackSample::new("test".to_string(), result.clone(), Some(false));

        assert_eq!(sample.text, "test");
        assert!(sample.is_correction());
    }

    #[test]
    fn test_feedback_collector_creation() {
        let collector = FeedbackCollector::new();
        assert_eq!(collector.len(), 0);
        assert!(collector.is_empty());
    }

    #[test]
    fn test_add_feedback() {
        let mut collector = FeedbackCollector::new();
        let result = create_test_result(0.8, true);
        let sample = FeedbackSample::new("test".to_string(), result, Some(false));

        collector.add_feedback(sample);
        assert_eq!(collector.len(), 1);
    }

    #[test]
    fn test_correction_ratio() {
        let mut collector = FeedbackCollector::new();
        let result_correct = create_test_result(0.8, true);
        let result_incorrect = create_test_result(0.3, false);

        // Correct prediction (no correction)
        let sample1 = FeedbackSample::new("test1".to_string(), result_correct, Some(true));
        collector.add_feedback(sample1);

        // Incorrect prediction (is a correction)
        let sample2 = FeedbackSample::new("test2".to_string(), result_incorrect, Some(true));
        collector.add_feedback(sample2);

        assert_eq!(collector.total_received, 2);
        assert_eq!(collector.total_corrections, 1);
        assert!((collector.correction_ratio() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_get_training_batch() {
        let mut collector = FeedbackCollector::new();
        let result = create_test_result(0.8, true);

        for i in 0..5 {
            let sample = FeedbackSample::new(format!("test{}", i), result.clone(), Some(false));
            collector.add_feedback(sample);
        }

        let batch = collector.get_training_batch(3);
        assert_eq!(batch.len(), 3);
    }

    #[test]
    fn test_buffer_overflow() {
        let config = FeedbackCollectorConfig {
            max_size: 5,
            ..Default::default()
        };
        let mut collector = FeedbackCollector::with_config(config);
        let result = create_test_result(0.8, true);

        for i in 0..10 {
            let sample = FeedbackSample::new(format!("test{}", i), result.clone(), Some(false));
            collector.add_feedback(sample);
        }

        // Should keep only last 5
        assert_eq!(collector.len(), 5);
        assert_eq!(collector.buffer.front().unwrap().text, "test5");
        assert_eq!(collector.buffer.back().unwrap().text, "test9");
    }

    #[test]
    fn test_get_recent_corrections() {
        let mut collector = FeedbackCollector::new();
        let result_correct = create_test_result(0.8, true);
        let result_incorrect = create_test_result(0.3, false);

        for i in 0..3 {
            let sample =
                FeedbackSample::new(format!("correct{}", i), result_correct.clone(), Some(true));
            collector.add_feedback(sample);
        }

        for i in 0..2 {
            let sample = FeedbackSample::new(
                format!("incorrect{}", i),
                result_incorrect.clone(),
                Some(true),
            );
            collector.add_feedback(sample);
        }

        let corrections = collector.get_recent_corrections(1);
        assert_eq!(corrections.len(), 1);
    }

    #[test]
    fn test_statistics() {
        let mut collector = FeedbackCollector::new();
        let result = create_test_result(0.8, true);

        for i in 0..3 {
            let sample = FeedbackSample::new(format!("test{}", i), result.clone(), Some(false));
            collector.add_feedback(sample);
        }

        let stats = collector.statistics();
        assert_eq!(stats.stored_samples, 3);
        assert_eq!(stats.total_received, 3);
        assert!(stats.avg_confidence > 0.7);
    }

    #[test]
    fn test_feedback_with_attack_type() {
        let result = create_test_result(0.8, true);
        let sample =
            FeedbackSample::new("test".to_string(), result, Some(true)).with_attack_type(3);

        assert_eq!(sample.actual_attack_type, Some(3));
    }

    #[test]
    fn test_to_training_sample() {
        let result = create_test_result(0.8, true);
        let sample = FeedbackSample::new("inject prompt".to_string(), result, Some(true))
            .with_attack_type(2);

        let training_sample = sample.to_training_sample();
        assert_eq!(training_sample.text, "inject prompt");
        assert!(training_sample.is_injection);
        assert_eq!(training_sample.attack_type, AttackType::InstructionOverride);
    }

    #[test]
    fn test_clear_feedback() {
        let mut collector = FeedbackCollector::new();
        let result = create_test_result(0.8, true);

        for i in 0..5 {
            let sample = FeedbackSample::new(format!("test{}", i), result.clone(), Some(false));
            collector.add_feedback(sample);
        }

        assert_eq!(collector.len(), 5);
        collector.clear();
        assert_eq!(collector.len(), 0);
        assert_eq!(collector.total_received, 0);
    }
}
