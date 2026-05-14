//! Multi-task sample for training.

use crate::detection::AttackType;

/// A sample for multi-task learning during training.
#[derive(Debug, Clone)]
pub struct MultiTaskSample {
    /// The input text
    pub text: String,
    /// Whether this is an injection attempt (binary label)
    pub is_injection: bool,
    /// Type of attack (7-way classification)
    pub attack_type: AttackType,
    /// Expected output (for semantic similarity training)
    pub expected_output: Option<String>,
}

impl MultiTaskSample {
    /// Create a new multi-task sample.
    pub fn new(text: String, is_injection: bool, attack_type: AttackType) -> Self {
        Self {
            text,
            is_injection,
            attack_type,
            expected_output: None,
        }
    }

    /// Create a new multi-task sample with expected output.
    pub fn with_output(
        text: String,
        is_injection: bool,
        attack_type: AttackType,
        expected_output: String,
    ) -> Self {
        Self {
            text,
            is_injection,
            attack_type,
            expected_output: Some(expected_output),
        }
    }

    /// Set the expected output.
    pub fn set_expected_output(&mut self, output: String) {
        self.expected_output = Some(output);
    }
}
