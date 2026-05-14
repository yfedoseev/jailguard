//! Multi-task learning for comprehensive injection detection
//!
//! This module implements multi-task learning with three complementary tasks:
//!
//! 1. **Binary Classification** (60% weight)
//!    - Injection vs Benign
//!    - Primary security objective
//!
//! 2. **Attack Type Classification** (30% weight)
//!    - 7-way attack categorization
//!    - Enables defense-specific responses
//!    - Categories: instruction override, role-play, encoding, separator,
//!                 prompt leaking, output manipulation, novel/unknown
//!
//! 3. **Semantic Similarity** (10% weight)
//!    - Cosine similarity to known attack patterns
//!    - Detects novel variations
//!    - Transfers knowledge from similar attacks
//!
//! # Expected Improvements
//!
//! - +2-3% accuracy from auxiliary tasks
//! - Better feature learning through shared representations
//! - Interpretable attack classification
//! - Improved generalization to unseen attacks

use serde::{Deserialize, Serialize};

/// 7-way attack type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackType {
    /// Direct instruction overrides (weight: 0.15)
    InstructionOverride = 0,
    /// Role-play or context manipulation (weight: 0.15)
    RolePlay = 1,
    /// Encoding or obfuscation attacks (weight: 0.15)
    Encoding = 2,
    /// Separator or structural attacks (weight: 0.15)
    Separator = 3,
    /// Prompt leaking attempts (weight: 0.15)
    PromptLeaking = 4,
    /// Output manipulation attacks (weight: 0.15)
    OutputManipulation = 5,
    /// Novel or unknown attacks (weight: 0.1)
    Novel = 6,
}

impl AttackType {
    /// Get all attack types
    pub fn all() -> &'static [AttackType] {
        &[
            AttackType::InstructionOverride,
            AttackType::RolePlay,
            AttackType::Encoding,
            AttackType::Separator,
            AttackType::PromptLeaking,
            AttackType::OutputManipulation,
            AttackType::Novel,
        ]
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            AttackType::InstructionOverride => "Instruction Override",
            AttackType::RolePlay => "Role-play",
            AttackType::Encoding => "Encoding",
            AttackType::Separator => "Separator",
            AttackType::PromptLeaking => "Prompt Leaking",
            AttackType::OutputManipulation => "Output Manipulation",
            AttackType::Novel => "Novel",
        }
    }

    /// Get weight for this attack type in multi-task loss
    pub fn weight(&self) -> f32 {
        match self {
            AttackType::InstructionOverride => 0.15,
            AttackType::RolePlay => 0.15,
            AttackType::Encoding => 0.15,
            AttackType::Separator => 0.15,
            AttackType::PromptLeaking => 0.15,
            AttackType::OutputManipulation => 0.15,
            AttackType::Novel => 0.10,
        }
    }

    /// Convert from index
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(AttackType::InstructionOverride),
            1 => Some(AttackType::RolePlay),
            2 => Some(AttackType::Encoding),
            3 => Some(AttackType::Separator),
            4 => Some(AttackType::PromptLeaking),
            5 => Some(AttackType::OutputManipulation),
            6 => Some(AttackType::Novel),
            _ => None,
        }
    }

    /// Get index
    pub fn index(&self) -> usize {
        *self as usize
    }
}

/// Multi-task learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTaskConfig {
    /// Weight for binary classification loss (default: 0.6)
    pub binary_weight: f32,

    /// Weight for attack type classification loss (default: 0.3)
    pub attack_weight: f32,

    /// Weight for semantic similarity loss (default: 0.1)
    pub semantic_weight: f32,

    /// Threshold for classifying as injection (0.0-1.0)
    pub injection_threshold: f32,

    /// Threshold for attack type confidence
    pub attack_confidence_threshold: f32,

    /// Temperature for semantic similarity (higher = smoother)
    pub similarity_temperature: f32,
}

impl Default for MultiTaskConfig {
    fn default() -> Self {
        Self {
            binary_weight: 0.6,
            attack_weight: 0.3,
            semantic_weight: 0.1,
            injection_threshold: 0.5,
            attack_confidence_threshold: 0.3,
            similarity_temperature: 1.0,
        }
    }
}

impl MultiTaskConfig {
    /// Validate that weights sum to 1.0
    pub fn validate(&self) -> Result<(), String> {
        let total = self.binary_weight + self.attack_weight + self.semantic_weight;
        if (total - 1.0).abs() > 0.001 {
            return Err(format!("Weights must sum to 1.0, got {}", total));
        }
        Ok(())
    }

    /// Adjust weights for different training phases
    pub fn for_phase(phase: usize) -> Self {
        match phase {
            // Phase 1: Balanced learning
            1 => Self {
                binary_weight: 0.33,
                attack_weight: 0.33,
                semantic_weight: 0.34,
                ..Default::default()
            },
            // Phase 2: Focus on primary task
            2 => Self {
                binary_weight: 0.6,
                attack_weight: 0.2,
                semantic_weight: 0.2,
                ..Default::default()
            },
            // Phase 3 (default): Fine-tune secondary tasks
            _ => Self::default(),
        }
    }
}

/// Multi-task prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTaskResult {
    /// Binary classification: is injection
    pub is_injection: bool,

    /// Binary classification confidence (0.0-1.0)
    pub binary_confidence: f32,

    /// Predicted attack type
    pub attack_type: AttackType,

    /// Confidence for attack type (0.0-1.0)
    pub attack_confidence: f32,

    /// Attack type probabilities for all 7 types
    pub attack_probabilities: Vec<f32>,

    /// Semantic similarity score (0.0-1.0)
    pub semantic_similarity: f32,

    /// Combined score (weighted average)
    pub combined_score: f32,

    /// Risk level based on combined score
    pub risk_level: RiskLevel,
}

/// Risk level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// >=0.9: Immediate threat
    Critical,
    /// >=0.75: High risk
    High,
    /// >=0.6: Medium risk
    Medium,
    /// >=0.4: Low risk
    Low,
    /// <0.4: Safe
    Safe,
}

impl RiskLevel {
    /// Get recommended action
    pub fn recommended_action(&self) -> &'static str {
        match self {
            RiskLevel::Critical => "BLOCK: Immediate action required",
            RiskLevel::High => "BLOCK: High-confidence threat",
            RiskLevel::Medium => "CAUTION: Review required",
            RiskLevel::Low => "MONITOR: Possible threat",
            RiskLevel::Safe => "ALLOW: Safe to process",
        }
    }
}

/// Multi-task training metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MultiTaskMetrics {
    /// Binary classification accuracy
    pub binary_accuracy: f32,

    /// Attack type classification accuracy
    pub attack_accuracy: f32,

    /// Attack type classification F1 score (macro-averaged)
    pub attack_f1: f32,

    /// Semantic similarity correlation
    pub semantic_correlation: f32,

    /// Combined multi-task loss
    pub combined_loss: f32,

    /// Per-attack-type precision
    pub attack_precision_per_type: Vec<f32>,

    /// Per-attack-type recall
    pub attack_recall_per_type: Vec<f32>,

    /// Per-attack-type F1
    pub attack_f1_per_type: Vec<f32>,
}

/// Multi-task learning orchestrator
pub struct MultiTaskLearner {
    config: MultiTaskConfig,
}

impl MultiTaskLearner {
    /// Create new multi-task learner
    pub fn new(config: MultiTaskConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Compute multi-task loss
    pub fn compute_loss(
        &self,
        binary_logit: f32,
        attack_logits: &[f32],
        semantic_score: f32,
        target_binary: bool,
        target_attack: AttackType,
        target_similarity: f32,
    ) -> f32 {
        // Binary classification loss (cross-entropy)
        let target_prob = if target_binary { 1.0 } else { 0.0 };
        let binary_pred = 1.0 / (1.0 + (-binary_logit).exp()); // Sigmoid
        let binary_loss =
            -(target_prob * binary_pred.ln() + (1.0 - target_prob) * (1.0 - binary_pred).ln());

        // Attack type classification loss (cross-entropy)
        let attack_idx = target_attack.index();
        let attack_sum: f32 = attack_logits.iter().map(|x| x.exp()).sum();
        let attack_prob = attack_logits[attack_idx].exp() / attack_sum;
        let attack_loss = -attack_prob.ln();

        // Semantic similarity loss (MSE)
        let semantic_loss = (semantic_score - target_similarity).powi(2);

        // Weighted combination
        self.config.binary_weight * binary_loss
            + self.config.attack_weight * attack_loss
            + self.config.semantic_weight * semantic_loss
    }

    /// Generate multi-task result from predictions
    pub fn result_from_predictions(
        &self,
        binary_logit: f32,
        attack_logits: &[f32],
        semantic_score: f32,
    ) -> MultiTaskResult {
        // Binary classification
        let binary_prob = 1.0 / (1.0 + (-binary_logit).exp()); // Sigmoid
        let is_injection = binary_prob >= self.config.injection_threshold;

        // Attack type classification
        let attack_sum: f32 = attack_logits.iter().map(|x| x.exp()).sum();
        let attack_probs: Vec<f32> = attack_logits.iter().map(|x| x.exp() / attack_sum).collect();

        let attack_idx = attack_probs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map_or(0, |(idx, _)| idx);

        let attack_type = AttackType::from_index(attack_idx).unwrap_or(AttackType::Novel);
        let attack_confidence = attack_probs[attack_idx];

        // Combined score (weighted average)
        let combined_score = self.config.binary_weight * binary_prob
            + self.config.attack_weight * attack_confidence
            + self.config.semantic_weight * semantic_score;

        // Risk level
        let risk_level = match combined_score {
            x if x >= 0.9 => RiskLevel::Critical,
            x if x >= 0.75 => RiskLevel::High,
            x if x >= 0.6 => RiskLevel::Medium,
            x if x >= 0.4 => RiskLevel::Low,
            _ => RiskLevel::Safe,
        };

        MultiTaskResult {
            is_injection,
            binary_confidence: binary_prob,
            attack_type,
            attack_confidence,
            attack_probabilities: attack_probs,
            semantic_similarity: semantic_score,
            combined_score,
            risk_level,
        }
    }

    /// Get configuration
    pub fn config(&self) -> &MultiTaskConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_type_all() {
        assert_eq!(AttackType::all().len(), 7);
    }

    #[test]
    fn test_attack_type_weights_sum() {
        let total: f32 = AttackType::all().iter().map(|t| t.weight()).sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_attack_type_string_conversion() {
        assert_eq!(
            AttackType::InstructionOverride.as_str(),
            "Instruction Override"
        );
        assert_eq!(AttackType::Novel.as_str(), "Novel");
    }

    #[test]
    fn test_attack_type_index_conversion() {
        for (i, attack_type) in AttackType::all().iter().enumerate() {
            assert_eq!(AttackType::from_index(i), Some(*attack_type));
        }
    }

    #[test]
    fn test_multitask_config_default() {
        let config = MultiTaskConfig::default();
        config.validate().expect("Default config should be valid");
    }

    #[test]
    fn test_multitask_config_validate() {
        let mut config = MultiTaskConfig::default();
        config.binary_weight = 0.5;
        config.attack_weight = 0.3;
        config.semantic_weight = 0.3; // Sum = 1.1

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_multitask_config_phases() {
        let phase1 = MultiTaskConfig::for_phase(1);
        let phase2 = MultiTaskConfig::for_phase(2);
        let phase3 = MultiTaskConfig::for_phase(3);

        phase1.validate().expect("Phase 1 should be valid");
        phase2.validate().expect("Phase 2 should be valid");
        phase3.validate().expect("Phase 3 should be valid");

        // Phase 1 should be balanced
        assert!((phase1.binary_weight - 0.33).abs() < 0.01);

        // Phase 2 should emphasize binary
        assert!(phase2.binary_weight > phase2.attack_weight);
    }

    #[test]
    fn test_risk_level_recommendations() {
        assert_eq!(
            RiskLevel::Critical.recommended_action(),
            "BLOCK: Immediate action required"
        );
        assert_eq!(
            RiskLevel::Safe.recommended_action(),
            "ALLOW: Safe to process"
        );
    }

    #[test]
    fn test_multitask_result_generation() {
        let config = MultiTaskConfig::default();
        let learner = MultiTaskLearner::new(config).expect("Should create learner");

        let binary_logit = 1.0;
        let attack_logits = vec![0.5; 7];
        let semantic_score = 0.7;

        let result = learner.result_from_predictions(binary_logit, &attack_logits, semantic_score);

        assert!(result.is_injection);
        assert!(result.binary_confidence > 0.5);
        assert_eq!(result.attack_probabilities.len(), 7);
        assert!(result.semantic_similarity >= 0.0 && result.semantic_similarity <= 1.0);
    }

    #[test]
    fn test_multitask_loss_computation() {
        let config = MultiTaskConfig::default();
        let learner = MultiTaskLearner::new(config).expect("Should create learner");

        let binary_logit = 1.0;
        let attack_logits = vec![1.0, 0.5, 0.5, 0.5, 0.5, 0.5, 0.0];
        let semantic_score = 0.8;
        let target_binary = true;
        let target_attack = AttackType::InstructionOverride;
        let target_similarity = 0.8;

        let loss = learner.compute_loss(
            binary_logit,
            &attack_logits,
            semantic_score,
            target_binary,
            target_attack,
            target_similarity,
        );

        assert!(loss >= 0.0, "Loss should be non-negative");
        assert!(loss < 5.0, "Loss should be reasonable");
    }
}
