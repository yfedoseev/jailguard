//! Reward shaping for prompt injection detection.

/// Configuration for reward values.
#[derive(Debug, Clone)]
pub struct RewardConfig {
    /// Reward for correctly blocking an injection (true positive)
    pub true_positive: f32,
    /// Reward for correctly allowing benign text (true negative)
    pub true_negative: f32,
    /// Penalty for incorrectly blocking benign text (false positive)
    pub false_positive: f32,
    /// Penalty for missing an injection (false negative) - SECURITY CRITICAL
    pub false_negative: f32,
}

impl Default for RewardConfig {
    fn default() -> Self {
        Self {
            true_positive: 1.0,
            true_negative: 1.0,
            false_positive: -1.0,
            // More severe penalty for missing injections (security risk)
            false_negative: -2.0,
        }
    }
}

/// Reward shaper that computes rewards based on detection outcomes.
pub struct RewardShaper {
    config: RewardConfig,
}

impl RewardShaper {
    /// Create a new reward shaper with the given configuration.
    pub fn new(config: RewardConfig) -> Self {
        Self { config }
    }

    /// Compute reward based on action and ground truth.
    ///
    /// # Arguments
    /// * `action` - 0 for Block, 1 for Allow
    /// * `is_injection` - Ground truth: true if the text is an injection
    ///
    /// # Returns
    /// The reward value based on the outcome.
    pub fn compute(&self, action: usize, is_injection: bool) -> f32 {
        match (action, is_injection) {
            (0, true) => self.config.true_positive, // Correctly blocked injection
            (1, false) => self.config.true_negative, // Correctly allowed benign
            (0, false) => self.config.false_positive, // Incorrectly blocked benign
            (1, true) => self.config.false_negative, // MISSED INJECTION (critical!)
            _ => 0.0,
        }
    }

    /// Compute reward with confidence scaling.
    ///
    /// Higher confidence correct decisions get slightly higher rewards.
    /// Lower confidence incorrect decisions get slightly lower penalties.
    pub fn compute_with_confidence(
        &self,
        action: usize,
        is_injection: bool,
        confidence: f32,
    ) -> f32 {
        let base_reward = self.compute(action, is_injection);
        let is_correct = (action == 0 && is_injection) || (action == 1 && !is_injection);

        if is_correct {
            // Bonus for high-confidence correct decisions
            base_reward * (0.5 + 0.5 * confidence)
        } else {
            // Full penalty regardless of confidence (we want to discourage mistakes)
            base_reward
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_values() {
        let shaper = RewardShaper::new(RewardConfig::default());

        // True positive: blocked injection
        assert!(shaper.compute(0, true) > 0.0);

        // True negative: allowed benign
        assert!(shaper.compute(1, false) > 0.0);

        // False positive: blocked benign
        assert!(shaper.compute(0, false) < 0.0);

        // False negative: missed injection (should be most severe)
        let fn_penalty = shaper.compute(1, true);
        let fp_penalty = shaper.compute(0, false);
        assert!(fn_penalty < fp_penalty); // FN is worse than FP
    }
}
