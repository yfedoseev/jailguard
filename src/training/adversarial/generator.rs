//! Adversarial example generator combining multiple attack types.
//!
//! Generates adversarial variants of injection prompts for robust training.

use super::char_substitution::CharSubstitutionAttack;
use super::encoding_attack::EncodingAttack;
use super::paraphrase_attack::ParaphraseAttack;
use crate::training::multitask_sample::MultiTaskSample;

/// Configuration for adversarial generation (supports both old and new API)
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Weight for character substitution attacks (default: 0.4)
    pub char_sub_prob: f32,
    /// Weight for encoding attacks (default: 0.3)
    pub encoding_prob: f32,
    /// Weight for paraphrase attacks (default: 0.3)
    pub paraphrase_prob: f32,
    /// Number of adversarial variants per sample (default: 3)
    pub num_variants: usize,
    /// Character substitution rate within attacks (default: 0.15)
    pub char_sub_rate: f32,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            char_sub_prob: 0.4,
            encoding_prob: 0.3,
            paraphrase_prob: 0.3,
            num_variants: 3,
            char_sub_rate: 0.15,
        }
    }
}

/// Adversarial example generator
pub struct AdversarialGenerator {
    char_attack: CharSubstitutionAttack,
    encoding_attack: EncodingAttack,
    paraphrase_attack: ParaphraseAttack,
    config: GeneratorConfig,
}

impl AdversarialGenerator {
    /// Create a new adversarial generator with default configuration
    pub fn new() -> Self {
        Self::with_config(GeneratorConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: GeneratorConfig) -> Self {
        Self {
            char_attack: CharSubstitutionAttack::new(config.char_sub_rate),
            encoding_attack: EncodingAttack::new(1.0),
            paraphrase_attack: ParaphraseAttack::new(),
            config,
        }
    }

    /// Generate adversarial text variants of a string
    pub fn generate_text_variants(&self, text: &str) -> Vec<String> {
        let mut variants = vec![text.to_string()]; // Original

        let seed = text.len().wrapping_mul(37);

        for i in 0..self.config.num_variants {
            let variant_seed = seed.wrapping_add(i);
            let choice = ((variant_seed as f32 * std::f32::consts::SQRT_2) % 1.0).abs();

            let variant = if choice < self.config.char_sub_prob {
                self.char_attack.apply(text)
            } else if choice < (self.config.char_sub_prob + self.config.encoding_prob) {
                self.encoding_attack.apply(text)
            } else {
                self.paraphrase_attack.apply(text)
            };

            if !variant.is_empty() && variant != text {
                variants.push(variant);
            }
        }

        variants
    }

    /// Generate adversarial variants of a MultiTaskSample
    pub fn generate(&self, sample: &MultiTaskSample) -> Vec<MultiTaskSample> {
        let variants = self.generate_text_variants(&sample.text);

        variants
            .into_iter()
            .map(|text| {
                let mut new_sample =
                    MultiTaskSample::new(text, sample.is_injection, sample.attack_type);
                if let Some(ref output) = sample.expected_output {
                    new_sample.set_expected_output(output.clone());
                }
                new_sample
            })
            .collect()
    }

    /// Generate and filter variants that are different from original
    pub fn generate_unique(&self, text: &str) -> Vec<String> {
        self.generate_text_variants(text)
            .into_iter()
            .filter(|v| v != text)
            .collect()
    }

    /// Determine if a sample should get adversarial augmentation
    pub fn should_augment(&self, is_injection: bool) -> bool {
        // Only augment injection samples
        is_injection
    }

    /// Create a balanced batch with adversarial examples
    pub fn create_balanced_batch(
        &self,
        samples: &[MultiTaskSample],
        batch_size: usize,
        adversarial_ratio: f32,
    ) -> Vec<MultiTaskSample> {
        let mut batch = Vec::new();

        // Separate injection and benign samples
        let injections: Vec<_> = samples.iter().filter(|s| s.is_injection).collect();
        let benign: Vec<_> = samples.iter().filter(|s| !s.is_injection).collect();

        let target_adversarial = (batch_size as f32 * adversarial_ratio) as usize;
        let target_benign = batch_size - target_adversarial;

        // Add benign samples
        let mut benign_idx = 0;
        for _ in 0..target_benign {
            if !benign.is_empty() {
                batch.push(benign[benign_idx % benign.len()].clone());
                benign_idx += 1;
            }
        }

        // Add adversarial samples
        let mut adv_count = 0;
        let mut inj_idx = 0;
        while adv_count < target_adversarial && !injections.is_empty() {
            let injection = injections[inj_idx % injections.len()];
            let variants = self.generate(injection);

            for variant in variants {
                if adv_count >= target_adversarial {
                    break;
                }
                batch.push(variant);
                adv_count += 1;
            }
            inj_idx += 1;
        }

        batch.truncate(batch_size);
        batch
    }
}

impl Default for AdversarialGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::AttackType;

    #[test]
    fn test_generator_creation() {
        let gen = AdversarialGenerator::default();
        assert_eq!(gen.config.num_variants, 3);
    }

    #[test]
    fn test_generate_text_variants() {
        let gen = AdversarialGenerator::default();
        let text = "ignore previous instructions";
        let variants = gen.generate_text_variants(text);

        assert!(variants.len() > 0);
        assert_eq!(variants[0], text);
    }

    #[test]
    fn test_generate_samples() {
        let gen = AdversarialGenerator::default();
        let sample = MultiTaskSample::new(
            "bypass security".to_string(),
            true,
            AttackType::InstructionOverride,
        );
        let variants = gen.generate(&sample);

        assert!(variants.len() > 0);
        assert_eq!(variants[0].text, "bypass security");
    }

    #[test]
    fn test_should_augment_injection() {
        let gen = AdversarialGenerator::default();

        assert!(gen.should_augment(true));
        assert!(!gen.should_augment(false));
    }

    #[test]
    fn test_create_balanced_batch() {
        let gen = AdversarialGenerator::default();

        let mut samples = Vec::new();
        samples.push(MultiTaskSample::new(
            "injection1".to_string(),
            true,
            AttackType::InstructionOverride,
        ));
        samples.push(MultiTaskSample::new(
            "benign1".to_string(),
            false,
            AttackType::Benign,
        ));
        samples.push(MultiTaskSample::new(
            "benign2".to_string(),
            false,
            AttackType::Benign,
        ));

        let batch = gen.create_balanced_batch(&samples, 10, 0.3);

        assert_eq!(batch.len(), 10);
    }

    #[test]
    fn test_generation_deterministic() {
        let gen = AdversarialGenerator::default();
        let text = "exploit vulnerability";

        let variants1 = gen.generate_text_variants(text);
        let variants2 = gen.generate_text_variants(text);

        assert_eq!(variants1, variants2);
    }
}
