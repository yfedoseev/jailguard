//! Feedback system for online learning.
//!
//! This module allows collecting user feedback on detection results
//! to continuously improve the model through online learning.

mod collector;

pub use collector::FeedbackCollector;

use serde::{Deserialize, Serialize};

/// Type of feedback for a detection result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// Detection was correct - it was an injection and was blocked
    TruePositive,
    /// Detection was correct - it was benign and was allowed
    TrueNegative,
    /// Detection was wrong - it was benign but was blocked
    FalsePositive,
    /// Detection was wrong - it was an injection but was allowed
    FalseNegative,
}

impl FeedbackType {
    /// Check if this feedback indicates the detection was correct.
    pub fn is_correct(&self) -> bool {
        matches!(
            self,
            FeedbackType::TruePositive | FeedbackType::TrueNegative
        )
    }

    /// Check if this feedback indicates an injection was present.
    pub fn is_injection(&self) -> bool {
        matches!(
            self,
            FeedbackType::TruePositive | FeedbackType::FalseNegative
        )
    }

    /// Get the ground truth label from this feedback.
    pub fn ground_truth(&self) -> bool {
        self.is_injection()
    }
}

/// A feedback entry for training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEntry {
    /// Detection result ID
    pub detection_id: String,
    /// The original text that was analyzed
    pub text: String,
    /// The action that was taken (0 = Block, 1 = Allow)
    pub action: usize,
    /// The feedback type
    pub feedback: FeedbackType,
    /// Timestamp of the feedback
    pub timestamp: u64,
}

impl FeedbackEntry {
    /// Create a new feedback entry.
    pub fn new(detection_id: String, text: String, action: usize, feedback: FeedbackType) -> Self {
        Self {
            detection_id,
            text,
            action,
            feedback,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
