//! Adversarial example generation for robust training.
//!
//! This module provides attack strategies to generate adversarial variants
//! of prompt injections, enabling the model to learn robust defenses.
//!
//! Supported attacks:
//! - Character substitution (homoglyphs, leetspeak, case variation)
//! - Encoding (Base64, URL, Unicode normalization)
//! - Paraphrasing (synonym substitution)

mod char_substitution;
mod encoding_attack;
pub mod generator;
mod paraphrase_attack;

pub use char_substitution::CharSubstitutionAttack;
pub use encoding_attack::EncodingAttack;
pub use generator::AdversarialGenerator;
pub use paraphrase_attack::ParaphraseAttack;

/// Configuration for adversarial example generation.
#[derive(Debug, Clone)]
pub struct AdversarialConfig {
    /// Ratio of samples that are adversarial (0.0 to 1.0)
    pub adversarial_ratio: f32,
    /// Number of variants to generate per sample
    pub num_variants: usize,
    /// Mix ratio for attack types: (`char_sub`, encoding, paraphrase)
    pub attack_mix: (f32, f32, f32),
}

impl Default for AdversarialConfig {
    fn default() -> Self {
        Self {
            adversarial_ratio: 0.3,      // 30% of batch
            num_variants: 3,             // 3 variants per sample
            attack_mix: (0.4, 0.3, 0.3), // 40% char, 30% encoding, 30% paraphrase
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AdversarialConfig::default();
        assert_eq!(config.adversarial_ratio, 0.3);
        assert_eq!(config.num_variants, 3);
        assert!(
            (config.attack_mix.0 + config.attack_mix.1 + config.attack_mix.2 - 1.0).abs() < 0.01
        );
    }

    #[test]
    fn test_custom_config() {
        let config = AdversarialConfig {
            adversarial_ratio: 0.5,
            num_variants: 5,
            attack_mix: (0.5, 0.3, 0.2),
        };
        assert_eq!(config.adversarial_ratio, 0.5);
        assert_eq!(config.num_variants, 5);
    }
}
