//! Detection result types.

use serde::{Deserialize, Serialize};

/// Result of injection detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// Whether the text is classified as an injection
    pub is_injection: bool,
    /// Confidence of the classification (0.0 to 1.0)
    pub confidence: f32,
    /// Risk level based on confidence
    pub risk_level: InjectionRisk,
    /// Action probabilities: [block_prob, allow_prob]
    pub action_probabilities: [f32; 2],
    /// Unique ID for this detection (for feedback tracking)
    pub id: String,
}

impl DetectionResult {
    /// Create a new detection result.
    pub fn new(is_injection: bool, confidence: f32, action_probs: [f32; 2]) -> Self {
        let risk_level = if is_injection {
            InjectionRisk::from_confidence(confidence)
        } else {
            InjectionRisk::None
        };

        Self {
            is_injection,
            confidence,
            risk_level,
            action_probabilities: action_probs,
            id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Check if this should be blocked based on a threshold.
    pub fn should_block(&self, threshold: f32) -> bool {
        self.is_injection && self.confidence >= threshold
    }
}

/// Injection risk level based on detection confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InjectionRisk {
    /// No injection detected
    None,
    /// Low confidence injection (50-70%)
    Low,
    /// Medium confidence injection (70-85%)
    Medium,
    /// High confidence injection (85-95%)
    High,
    /// Very high confidence injection (95%+)
    Critical,
}

impl InjectionRisk {
    /// Convert confidence score to risk level.
    pub fn from_confidence(confidence: f32) -> Self {
        match confidence {
            c if c >= 0.95 => InjectionRisk::Critical,
            c if c >= 0.85 => InjectionRisk::High,
            c if c >= 0.70 => InjectionRisk::Medium,
            c if c >= 0.50 => InjectionRisk::Low,
            _ => InjectionRisk::None,
        }
    }

    /// Check if this risk level should trigger blocking.
    pub fn should_block(&self) -> bool {
        matches!(self, InjectionRisk::High | InjectionRisk::Critical)
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            InjectionRisk::None => "No injection detected",
            InjectionRisk::Low => "Low confidence - likely benign",
            InjectionRisk::Medium => "Medium confidence - review recommended",
            InjectionRisk::High => "High confidence - likely injection",
            InjectionRisk::Critical => "Critical - almost certainly injection",
        }
    }
}

impl std::fmt::Display for InjectionRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InjectionRisk::None => write!(f, "NONE"),
            InjectionRisk::Low => write!(f, "LOW"),
            InjectionRisk::Medium => write!(f, "MEDIUM"),
            InjectionRisk::High => write!(f, "HIGH"),
            InjectionRisk::Critical => write!(f, "CRITICAL"),
        }
    }
}
