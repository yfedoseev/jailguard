//! Attention Tracker - Detection layer based on attention weight shifts
//!
//! This module implements the Attention Tracker method from arxiv 2411.00348,
//! which detects prompt injections by analyzing LLM attention weights.
//!
//! **Core Idea:** When an LLM is injected with a competing instruction,
//! its attention patterns shift away from the original instruction. This layer
//! detects that shift by comparing attention weights across the instruction region.
//!
//! # Example
//!
//! ```ignore
//! use jailguard::attention_tracker::{AttentionTracker, AttentionTrackerConfig};
//!
//! let tracker = AttentionTracker::new(
//!     vec![15, 27, 45],  // Important heads for Llama-2
//!     0.4,               // Threshold
//!     (5, 15),           // Instruction token range
//! );
//!
//! let attention_weights = [...]; // From LLM output
//! let result = tracker.detect(&attention_weights);
//!
//! if result.is_injection {
//!     println!("Injection detected!");
//! }
//! ```

use crate::error::Result;

/// Configuration for the Attention Tracker
#[derive(Debug, Clone)]
pub struct AttentionTrackerConfig {
    /// Model-specific important head indices
    /// These heads are most responsive to instruction vs injection shifts
    /// Examples:
    /// - Llama-2-70b: [15, 27, 45]
    /// - GPT-4: varies, see paper appendix
    /// - Claude: varies, see paper appendix
    pub important_heads: Vec<usize>,

    /// Threshold for injection detection (0.0-1.0)
    /// Lower threshold = higher recall, higher false positives
    /// Recommended: 0.3-0.5
    pub threshold: f32,

    /// Token range of original instruction (`start_idx`, `end_idx`)
    /// Marks where in the token sequence the original instruction appears
    pub instruction_range: (usize, usize),

    /// Maximum sequence length (for tensor shape validation)
    pub max_seq_len: usize,

    /// Number of attention heads in the model
    pub num_heads: usize,
}

impl Default for AttentionTrackerConfig {
    fn default() -> Self {
        Self {
            important_heads: vec![15, 27, 45], // Llama-2 defaults
            threshold: 0.4,
            instruction_range: (5, 15),
            max_seq_len: 512,
            num_heads: 32,
        }
    }
}

/// Attention Tracker detector
///
/// Analyzes attention weight patterns to detect when an LLM's focus shifts
/// from the original instruction to an injected instruction.
pub struct AttentionTracker {
    config: AttentionTrackerConfig,
}

/// Result from attention tracking detection
#[derive(Debug, Clone, PartialEq)]
pub struct AttentionTrackerResult {
    /// Whether injection was detected
    pub is_injection: bool,

    /// Average attention weight to instruction region (0.0-1.0)
    /// Higher score = model focusing on original instruction (benign)
    /// Lower score = model focusing away from instruction (injection detected)
    pub attention_score: f32,

    /// Confidence in the detection (0.0-1.0)
    /// For injection: higher = more confident
    pub confidence: f32,

    /// Per-head attention scores to instruction region
    /// Useful for debugging which heads triggered detection
    pub per_head_scores: Vec<f32>,
}

impl AttentionTracker {
    /// Create a new attention tracker with configuration
    pub fn new(config: AttentionTrackerConfig) -> Self {
        Self { config }
    }

    /// Create a tracker with simple parameters
    pub fn with_params(
        important_heads: Vec<usize>,
        threshold: f32,
        instruction_range: (usize, usize),
    ) -> Self {
        let mut config = AttentionTrackerConfig::default();
        config.important_heads = important_heads;
        config.threshold = threshold;
        config.instruction_range = instruction_range;
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &AttentionTrackerConfig {
        &self.config
    }

    /// Detect injection by analyzing attention weights
    ///
    /// # Arguments
    ///
    /// * `attention_weights` - Flattened attention weights from final token
    ///   Expected shape: (`num_heads`, `seq_len`) flattened to 1D
    ///   Values should be normalized (sum to 1.0 per head)
    ///
    /// # Returns
    ///
    /// `AttentionTrackerResult` with detection status and confidence
    pub fn detect(&self, attention_weights: &[f32]) -> Result<AttentionTrackerResult> {
        let seq_len = self.config.max_seq_len;
        let num_heads = self.config.num_heads;

        // Validate input size
        let expected_size = num_heads * seq_len;
        if attention_weights.len() != expected_size {
            return Err(crate::error::Error::Config(format!(
                "Expected {} attention weights, got {}",
                expected_size,
                attention_weights.len()
            )));
        }

        let (inst_start, inst_end) = self.config.instruction_range;
        if inst_end > seq_len {
            return Err(crate::error::Error::Config(format!(
                "Instruction range ({}, {}) exceeds max_seq_len {}",
                inst_start, inst_end, seq_len
            )));
        }

        let mut per_head_scores = Vec::new();
        let mut total_attention = 0.0f32;

        // For each important head, sum attention to instruction region
        for &head_idx in &self.config.important_heads {
            if head_idx >= num_heads {
                return Err(crate::error::Error::Config(format!(
                    "Head index {} exceeds num_heads {}",
                    head_idx, num_heads
                )));
            }

            let mut head_attention = 0.0f32;
            let head_start = head_idx * seq_len;

            // Sum attention weights in instruction region for this head
            for token_idx in inst_start..inst_end {
                if head_start + token_idx < attention_weights.len() {
                    head_attention += attention_weights[head_start + token_idx];
                }
            }

            // Normalize by instruction range size
            let range_size = (inst_end - inst_start) as f32;
            if range_size > 0.0 {
                head_attention /= range_size;
            }

            per_head_scores.push(head_attention);
            total_attention += head_attention;
        }

        // Average across important heads
        let avg_attention = if !self.config.important_heads.is_empty() {
            total_attention / self.config.important_heads.len() as f32
        } else {
            0.5 // Default to neutral if no heads specified
        };

        // Clamp to [0.0, 1.0]
        let attention_score = avg_attention.clamp(0.0, 1.0);

        // If attention to instruction region is LOW, likely injection
        let is_injection = attention_score < self.config.threshold;
        let confidence = if is_injection {
            // Confidence increases as score decreases below threshold
            ((self.config.threshold - attention_score) / self.config.threshold).min(1.0)
        } else {
            // Confidence increases as score increases above threshold
            ((attention_score - self.config.threshold) / (1.0 - self.config.threshold)).min(1.0)
        };

        Ok(AttentionTrackerResult {
            is_injection,
            attention_score,
            confidence,
            per_head_scores,
        })
    }

    /// Detect from a 2D attention matrix
    ///
    /// # Arguments
    ///
    /// * `attention_matrix` - 2D matrix: [`num_heads`][seq_len]
    ///   (`num_heads` rows, `seq_len` columns)
    pub fn detect_from_matrix(
        &self,
        attention_matrix: &[Vec<f32>],
    ) -> Result<AttentionTrackerResult> {
        // Flatten matrix to 1D
        let mut flattened = Vec::new();
        for row in attention_matrix {
            flattened.extend_from_slice(row);
        }
        self.detect(&flattened)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create mock attention weights
    fn create_mock_attention(
        num_heads: usize,
        seq_len: usize,
        instruction_range: (usize, usize),
        benign_weight: f32, // Weight for benign case
    ) -> Vec<f32> {
        let mut weights = vec![0.0; num_heads * seq_len];

        for head in 0..num_heads {
            for token in 0..seq_len {
                let is_in_instruction = token >= instruction_range.0 && token < instruction_range.1;

                let weight = if is_in_instruction {
                    benign_weight // High weight = model focusing on instruction
                } else {
                    (1.0 - benign_weight)
                        / ((seq_len - (instruction_range.1 - instruction_range.0)) as f32)
                };

                weights[head * seq_len + token] = weight;
            }
        }

        weights
    }

    #[test]
    fn test_benign_detection() {
        let tracker = AttentionTracker::with_params(
            vec![10, 15, 20], // Valid head indices for 32 heads
            0.4,
            (5, 15),
        );
        let weights = create_mock_attention(32, 512, (5, 15), 0.8); // High attention to instruction

        let result = tracker.detect(&weights).unwrap();
        assert!(
            !result.is_injection,
            "Should not detect injection for benign case"
        );
        assert!(
            result.attention_score > 0.5,
            "Attention score should be high"
        );
    }

    #[test]
    fn test_injection_detection() {
        let tracker = AttentionTracker::with_params(
            vec![10, 15, 20], // Valid head indices for 32 heads
            0.4,
            (5, 15),
        );
        let weights = create_mock_attention(32, 512, (5, 15), 0.2); // Low attention to instruction

        let result = tracker.detect(&weights).unwrap();
        assert!(
            result.is_injection,
            "Should detect injection when attention to instruction is low"
        );
        assert!(result.confidence >= 0.4, "Confidence should be reasonable");
    }

    #[test]
    fn test_threshold_boundary() {
        let tracker = AttentionTracker::with_params(
            vec![0, 1, 2],
            0.5, // Threshold at 0.5
            (5, 15),
        );

        // Just above threshold (benign)
        let weights = create_mock_attention(32, 512, (5, 15), 0.51);
        let result = tracker.detect(&weights).unwrap();
        assert!(!result.is_injection);

        // Just below threshold (injection)
        let weights = create_mock_attention(32, 512, (5, 15), 0.49);
        let result = tracker.detect(&weights).unwrap();
        assert!(result.is_injection);
    }

    #[test]
    fn test_per_head_scores() {
        let tracker = AttentionTracker::with_params(
            vec![10, 15, 20], // Valid head indices for 32 heads
            0.4,
            (5, 15),
        );
        let weights = create_mock_attention(32, 512, (5, 15), 0.6);

        let result = tracker.detect(&weights).unwrap();
        assert_eq!(
            result.per_head_scores.len(),
            tracker.config.important_heads.len()
        );
        assert!(result.per_head_scores.iter().all(|&s| s >= 0.0 && s <= 1.0));
    }

    #[test]
    fn test_invalid_input_size() {
        let tracker = AttentionTracker::new(AttentionTrackerConfig::default());
        let weights = vec![0.5; 100]; // Wrong size

        let result = tracker.detect(&weights);
        assert!(result.is_err());
    }

    #[test]
    fn test_confidence_scaling() {
        let tracker = AttentionTracker::with_params(vec![0, 1, 2], 0.5, (5, 15));

        // Very benign (attention = 0.9)
        let weights = create_mock_attention(32, 512, (5, 15), 0.9);
        let result = tracker.detect(&weights).unwrap();
        let benign_confidence = result.confidence;

        // Slightly benign (attention = 0.6)
        let weights = create_mock_attention(32, 512, (5, 15), 0.6);
        let result = tracker.detect(&weights).unwrap();
        assert!(
            benign_confidence > result.confidence,
            "More benign should have higher confidence"
        );
    }
}
