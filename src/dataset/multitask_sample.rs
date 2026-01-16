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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multitask_sample_creation() {
        let sample = MultiTaskSample::new(
            "Ignore instructions".to_string(),
            true,
            AttackType::InstructionOverride,
        );

        assert_eq!(sample.text, "Ignore instructions");
        assert!(sample.is_injection);
        assert_eq!(sample.attack_type, AttackType::InstructionOverride);
        assert!(sample.expected_output.is_none());
    }

    #[test]
    fn test_multitask_sample_with_output() {
        let sample = MultiTaskSample::with_output(
            "What is 2+2?".to_string(),
            false,
            AttackType::Benign,
            "The answer is 4".to_string(),
        );

        assert_eq!(sample.text, "What is 2+2?");
        assert!(!sample.is_injection);
        assert_eq!(sample.attack_type, AttackType::Benign);
        assert_eq!(sample.expected_output, Some("The answer is 4".to_string()));
    }

    #[test]
    fn test_multitask_sample_set_output() {
        let mut sample = MultiTaskSample::new("Test".to_string(), false, AttackType::Benign);

        assert!(sample.expected_output.is_none());

        sample.set_expected_output("Output".to_string());
        assert_eq!(sample.expected_output, Some("Output".to_string()));
    }
}
