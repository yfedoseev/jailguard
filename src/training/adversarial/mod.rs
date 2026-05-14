//! Adversarial training augmentation for robust prompt injection detection.
//!
//! Implements three types of adversarial attacks for augmenting training data:
//! - Character substitution (homoglyphs, leetspeak, case variation)
//! - Encoding attacks (Base64, URL, Unicode obfuscation)
//! - Paraphrase attacks (synonym substitution, reordering, templates)
//!
//! Usage:
//! ```ignore
//! let gen = AdversarialGenerator::default();
//! let variants = gen.generate_text_variants("ignore previous instructions");
//! // variants includes original + 3 adversarial variants
//! ```

pub mod char_substitution;
pub mod encoding_attack;
pub mod generator;
pub mod paraphrase_attack;

pub use char_substitution::CharSubstitutionAttack;
pub use encoding_attack::EncodingAttack;
pub use generator::{AdversarialGenerator, GeneratorConfig};
pub use paraphrase_attack::ParaphraseAttack;

/// Configuration for adversarial example generation and training
#[derive(Debug, Clone)]
pub struct AdversarialConfig {
    /// Mix ratio of (char_substitution, encoding, paraphrase) attacks
    pub attack_mix: (f32, f32, f32),
    /// Number of adversarial variants to generate per injection sample
    pub num_variants: usize,
    /// Ratio of adversarial examples in augmented batches (0.0 to 1.0)
    pub adversarial_ratio: f32,
}

impl Default for AdversarialConfig {
    fn default() -> Self {
        Self {
            attack_mix: (0.4, 0.3, 0.3), // 40% char, 30% encoding, 30% paraphrase
            num_variants: 3,
            adversarial_ratio: 0.3, // 30% of batch should be adversarial
        }
    }
}
