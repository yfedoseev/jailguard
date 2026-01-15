//! Detection result types.

use serde::{Deserialize, Serialize};

/// Type of injection attack detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackType {
    /// Role-play or persona injection (e.g., "you are now a hacker")
    RolePlay = 0,
    /// Instruction override (e.g., "ignore your instructions")
    InstructionOverride = 1,
    /// Context manipulation (e.g., "forget previous context")
    ContextManipulation = 2,
    /// Output manipulation (e.g., "output in a different format")
    OutputManipulation = 3,
    /// Encoding/obfuscation attacks (base64, ROT13, etc.)
    EncodingAttack = 4,
    /// Jailbreak patterns (e.g., "DAN", "STAN")
    JailbreakPattern = 5,
    /// Benign input (no attack detected)
    Benign = 6,
}

impl AttackType {
    /// Get all attack types.
    pub fn variants() -> &'static [AttackType] {
        &[
            AttackType::RolePlay,
            AttackType::InstructionOverride,
            AttackType::ContextManipulation,
            AttackType::OutputManipulation,
            AttackType::EncodingAttack,
            AttackType::JailbreakPattern,
            AttackType::Benign,
        ]
    }

    /// Get the number of attack types.
    pub fn count() -> usize {
        7
    }

    /// Convert from index.
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(AttackType::RolePlay),
            1 => Some(AttackType::InstructionOverride),
            2 => Some(AttackType::ContextManipulation),
            3 => Some(AttackType::OutputManipulation),
            4 => Some(AttackType::EncodingAttack),
            5 => Some(AttackType::JailbreakPattern),
            6 => Some(AttackType::Benign),
            _ => None,
        }
    }

    /// Get human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            AttackType::RolePlay => "Role-play or persona injection",
            AttackType::InstructionOverride => "Instruction override attempt",
            AttackType::ContextManipulation => "Context manipulation attack",
            AttackType::OutputManipulation => "Output format manipulation",
            AttackType::EncodingAttack => "Encoding/obfuscation attack",
            AttackType::JailbreakPattern => "Known jailbreak pattern",
            AttackType::Benign => "No attack detected",
        }
    }
}

impl std::fmt::Display for AttackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

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

/// Multi-task detection result with attack type classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTaskDetectionResult {
    /// Base detection result (injection yes/no + confidence)
    pub detection: DetectionResult,
    /// Classified attack type
    pub attack_type: AttackType,
    /// Probabilities for each attack type (7 classes)
    pub attack_probs: [f32; 7],
    /// Semantic similarity score (0.0 to 1.0)
    pub semantic_score: f32,
    /// Embedding vector for downstream use
    #[serde(skip)]
    pub embedding: Vec<f32>,
}

impl MultiTaskDetectionResult {
    /// Create a new multi-task detection result.
    pub fn new(
        is_injection: bool,
        confidence: f32,
        action_probs: [f32; 2],
        attack_type: AttackType,
        attack_probs: [f32; 7],
        semantic_score: f32,
        embedding: Vec<f32>,
    ) -> Self {
        let detection = DetectionResult::new(is_injection, confidence, action_probs);

        Self {
            detection,
            attack_type,
            attack_probs,
            semantic_score,
            embedding,
        }
    }

    /// Check if this should be blocked based on a threshold.
    pub fn should_block(&self, threshold: f32) -> bool {
        self.detection.should_block(threshold)
    }

    /// Get the most likely attack type with its probability.
    pub fn top_attack(&self) -> (AttackType, f32) {
        let (idx, &prob) = self
            .attack_probs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        (
            AttackType::from_index(idx).unwrap_or(AttackType::Benign),
            prob,
        )
    }
}
