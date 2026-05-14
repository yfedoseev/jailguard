//! Feedback collection and storage.
#![allow(clippy::unnecessary_wraps)]

use parking_lot::RwLock;
use std::collections::HashMap;

use super::{FeedbackEntry, FeedbackType};
use crate::detection::DetectionResult;
use crate::error::Result;

/// Collector for user feedback on detection results.
pub struct FeedbackCollector {
    /// Pending detection results awaiting feedback
    pending: RwLock<HashMap<String, PendingDetection>>,
    /// Collected feedback entries
    feedback: RwLock<Vec<FeedbackEntry>>,
    /// Maximum number of feedback entries to store
    max_entries: usize,
}

/// A detection result pending feedback.
#[derive(Debug, Clone)]
struct PendingDetection {
    text: String,
    action: usize,
    #[allow(dead_code)]
    timestamp: u64,
}

impl FeedbackCollector {
    /// Create a new feedback collector.
    pub fn new() -> Self {
        Self::with_capacity(10000)
    }

    /// Create a new feedback collector with custom capacity.
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            pending: RwLock::new(HashMap::new()),
            feedback: RwLock::new(Vec::new()),
            max_entries,
        }
    }

    /// Record a detection result for potential feedback.
    pub fn record_detection(&self, text: &str, result: &DetectionResult) {
        let pending = PendingDetection {
            text: text.to_string(),
            action: if result.is_injection { 0 } else { 1 },
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.pending.write().insert(result.id.clone(), pending);
    }

    /// Submit feedback for a detection result.
    pub fn submit_feedback(&self, detection_id: &str, feedback: FeedbackType) -> Result<()> {
        let pending = self.pending.write().remove(detection_id);

        if let Some(detection) = pending {
            let entry = FeedbackEntry::new(
                detection_id.to_string(),
                detection.text,
                detection.action,
                feedback,
            );

            let mut fb = self.feedback.write();

            // Remove oldest entries if at capacity
            if fb.len() >= self.max_entries {
                fb.remove(0);
            }

            fb.push(entry);
        }

        Ok(())
    }

    /// Get all feedback entries for training.
    pub fn get_feedback(&self) -> Vec<FeedbackEntry> {
        self.feedback.read().clone()
    }

    /// Get feedback entries since a timestamp.
    pub fn get_feedback_since(&self, timestamp: u64) -> Vec<FeedbackEntry> {
        self.feedback
            .read()
            .iter()
            .filter(|e| e.timestamp >= timestamp)
            .cloned()
            .collect()
    }

    /// Get statistics about collected feedback.
    pub fn stats(&self) -> FeedbackStats {
        let feedback = self.feedback.read();

        let mut stats = FeedbackStats::default();
        for entry in feedback.iter() {
            match entry.feedback {
                FeedbackType::TruePositive => stats.true_positives += 1,
                FeedbackType::TrueNegative => stats.true_negatives += 1,
                FeedbackType::FalsePositive => stats.false_positives += 1,
                FeedbackType::FalseNegative => stats.false_negatives += 1,
            }
        }

        stats.pending = self.pending.read().len();
        stats
    }

    /// Clear all feedback entries.
    pub fn clear(&self) {
        self.feedback.write().clear();
    }

    /// Save feedback to a file.
    pub fn save(&self, path: &str) -> Result<()> {
        let feedback = self.feedback.read();
        let json = serde_json::to_string_pretty(&*feedback)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load feedback from a file.
    pub fn load(&self, path: &str) -> Result<()> {
        let json = std::fs::read_to_string(path)?;
        let entries: Vec<FeedbackEntry> = serde_json::from_str(&json)?;
        *self.feedback.write() = entries;
        Ok(())
    }
}

impl Default for FeedbackCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about collected feedback.
#[derive(Debug, Default)]
pub struct FeedbackStats {
    /// Number of true positive feedbacks
    pub true_positives: usize,
    /// Number of true negative feedbacks
    pub true_negatives: usize,
    /// Number of false positive feedbacks
    pub false_positives: usize,
    /// Number of false negative feedbacks
    pub false_negatives: usize,
    /// Number of pending detections awaiting feedback
    pub pending: usize,
}

impl FeedbackStats {
    /// Total number of feedback entries.
    pub fn total(&self) -> usize {
        self.true_positives + self.true_negatives + self.false_positives + self.false_negatives
    }

    /// Accuracy based on feedback.
    pub fn accuracy(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.true_positives + self.true_negatives) as f32 / total as f32
        }
    }
}
