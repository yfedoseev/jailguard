//! Adversarial training orchestration.
//!
//! Trains the detector on both clean and adversarial examples to improve robustness.

use crate::detection::TransformerDetector;
use crate::training::multitask_sample::MultiTaskSample;
use crate::training::{MultiTaskMetrics, MultiTaskTrainer, MultiTaskTrainingConfig};

use super::adversarial::{AdversarialConfig, AdversarialGenerator};

/// Configuration for adversarial training.
#[derive(Debug, Clone, Default)]
pub struct AdversarialTrainingConfig {
    /// Base training configuration
    pub base_config: MultiTaskTrainingConfig,
    /// Adversarial generation configuration
    pub adversarial_config: AdversarialConfig,
}

/// Metrics specific to adversarial training.
#[derive(Debug, Clone, Default)]
pub struct AdversarialMetrics {
    /// Metrics on clean samples
    pub clean_metrics: MultiTaskMetrics,
    /// Metrics on adversarial samples
    pub adversarial_metrics: MultiTaskMetrics,
    /// Combined metrics
    pub combined_metrics: MultiTaskMetrics,
}

impl AdversarialMetrics {
    /// Get average accuracy across clean and adversarial samples.
    pub fn avg_accuracy(&self) -> f32 {
        (self.clean_metrics.binary_accuracy + self.adversarial_metrics.binary_accuracy) / 2.0
    }

    /// Get robustness score (adversarial accuracy / clean accuracy).
    pub fn robustness_score(&self) -> f32 {
        if self.clean_metrics.binary_accuracy > 0.0 {
            self.adversarial_metrics.binary_accuracy / self.clean_metrics.binary_accuracy
        } else {
            0.0
        }
    }
}

/// Adversarial trainer orchestrating robust training.
pub struct AdversarialTrainer {
    /// Multi-task trainer
    pub trainer: MultiTaskTrainer,
    /// Adversarial generator
    pub generator: AdversarialGenerator,
    /// Configuration
    pub config: AdversarialTrainingConfig,
}

impl AdversarialTrainer {
    /// Create a new adversarial trainer.
    pub fn new(detector: TransformerDetector, config: AdversarialTrainingConfig) -> Self {
        let trainer = MultiTaskTrainer::new(detector, config.base_config.clone());

        // Convert AdversarialConfig to GeneratorConfig
        let gen_config = super::adversarial::generator::GeneratorConfig {
            char_sub_prob: config.adversarial_config.attack_mix.0,
            encoding_prob: config.adversarial_config.attack_mix.1,
            paraphrase_prob: config.adversarial_config.attack_mix.2,
            num_variants: config.adversarial_config.num_variants,
            char_sub_rate: 0.15,
        };
        let generator = AdversarialGenerator::with_config(gen_config);

        Self {
            trainer,
            generator,
            config,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults(detector: TransformerDetector) -> Self {
        Self::new(detector, AdversarialTrainingConfig::default())
    }

    /// Train on a batch with adversarial augmentation.
    ///
    /// Generates adversarial examples for injection samples and trains on both
    /// clean and adversarial variants.
    #[allow(clippy::field_reassign_with_default)]
    pub fn train_epoch(&self, samples: &[MultiTaskSample]) -> AdversarialMetrics {
        // Split into clean and injection samples
        let clean_samples: Vec<_> = samples.iter().filter(|s| !s.is_injection).collect();
        let injection_samples: Vec<_> = samples.iter().filter(|s| s.is_injection).collect();

        // Generate adversarial variants
        let mut adversarial_samples = Vec::new();
        for sample in &injection_samples {
            let variants = self.generator.generate(sample);
            adversarial_samples.extend(variants);
        }

        // Train on clean samples
        let clean_batch: Vec<_> = clean_samples.iter().map(|s| (*s).clone()).collect();
        let clean_batch_all: Vec<_> = clean_batch
            .iter()
            .chain(injection_samples.iter().copied())
            .cloned()
            .collect();

        let clean_metrics = self.trainer.train_epoch(&clean_batch_all);

        // Train on adversarial samples
        let adversarial_metrics = if !adversarial_samples.is_empty() {
            self.trainer.train_epoch(&adversarial_samples)
        } else {
            MultiTaskMetrics::default()
        };

        // Combined metrics
        let mut combined = MultiTaskMetrics::default();
        combined.num_samples = clean_metrics.num_samples + adversarial_metrics.num_samples;
        if combined.num_samples > 0 {
            combined.binary_accuracy = (clean_metrics.binary_accuracy
                * clean_metrics.num_samples as f32
                + adversarial_metrics.binary_accuracy * adversarial_metrics.num_samples as f32)
                / combined.num_samples as f32;
            combined.attack_accuracy = (clean_metrics.attack_accuracy
                * clean_metrics.num_samples as f32
                + adversarial_metrics.attack_accuracy * adversarial_metrics.num_samples as f32)
                / combined.num_samples as f32;
            combined.total_loss = clean_metrics.total_loss + adversarial_metrics.total_loss;
        }

        AdversarialMetrics {
            clean_metrics,
            adversarial_metrics,
            combined_metrics: combined,
        }
    }

    /// Evaluate on both clean and adversarial samples.
    #[allow(clippy::field_reassign_with_default)]
    pub fn evaluate(&self, samples: &[MultiTaskSample]) -> AdversarialMetrics {
        // Split samples
        let clean_samples: Vec<_> = samples.iter().filter(|s| !s.is_injection).collect();
        let injection_samples: Vec<_> = samples.iter().filter(|s| s.is_injection).collect();

        // Evaluate on clean set
        let clean_batch: Vec<_> = clean_samples.iter().map(|s| (*s).clone()).collect();
        let clean_batch_all: Vec<_> = clean_batch
            .iter()
            .chain(injection_samples.iter().copied())
            .cloned()
            .collect();

        let clean_metrics = self.trainer.evaluate(&clean_batch_all);

        // Generate and evaluate on adversarial samples
        let mut adversarial_samples = Vec::new();
        for sample in &injection_samples {
            let variants = self.generator.generate(sample);
            adversarial_samples.extend(variants);
        }

        let adversarial_metrics = if !adversarial_samples.is_empty() {
            self.trainer.evaluate(&adversarial_samples)
        } else {
            MultiTaskMetrics::default()
        };

        // Combined metrics
        let mut combined = MultiTaskMetrics::default();
        combined.num_samples = clean_metrics.num_samples + adversarial_metrics.num_samples;
        if combined.num_samples > 0 {
            combined.binary_accuracy = (clean_metrics.binary_accuracy
                * clean_metrics.num_samples as f32
                + adversarial_metrics.binary_accuracy * adversarial_metrics.num_samples as f32)
                / combined.num_samples as f32;
            combined.attack_accuracy = (clean_metrics.attack_accuracy
                * clean_metrics.num_samples as f32
                + adversarial_metrics.attack_accuracy * adversarial_metrics.num_samples as f32)
                / combined.num_samples as f32;
            combined.total_loss = clean_metrics.total_loss + adversarial_metrics.total_loss;
        }

        AdversarialMetrics {
            clean_metrics,
            adversarial_metrics,
            combined_metrics: combined,
        }
    }

    /// Create a balanced batch with adversarial examples.
    pub fn create_adversarial_batch(
        &self,
        samples: &[MultiTaskSample],
        batch_size: usize,
    ) -> Vec<MultiTaskSample> {
        self.generator.create_balanced_batch(
            samples,
            batch_size,
            self.config.adversarial_config.adversarial_ratio,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::AttackType;

    #[test]
    fn test_adversarial_trainer_creation() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let config = AdversarialTrainingConfig::default();
        let trainer = AdversarialTrainer::new(detector, config);

        assert_eq!(trainer.config.adversarial_config.adversarial_ratio, 0.3);
    }

    #[test]
    fn test_adversarial_metrics() {
        let mut metrics = AdversarialMetrics::default();

        metrics.clean_metrics.binary_accuracy = 0.9;
        metrics.clean_metrics.num_samples = 10;
        metrics.adversarial_metrics.binary_accuracy = 0.7;
        metrics.adversarial_metrics.num_samples = 30;

        let avg = metrics.avg_accuracy();
        assert!((avg - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_robustness_score() {
        let mut metrics = AdversarialMetrics::default();

        metrics.clean_metrics.binary_accuracy = 0.9;
        metrics.adversarial_metrics.binary_accuracy = 0.6;

        let robustness = metrics.robustness_score();
        assert!((robustness - (0.6 / 0.9)).abs() < 0.01);
    }

    #[test]
    fn test_train_epoch_with_adversarial() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let config = AdversarialTrainingConfig::default();
        let trainer = AdversarialTrainer::new(detector, config);

        let samples = vec![
            MultiTaskSample::new("Normal".to_string(), false, AttackType::Benign),
            MultiTaskSample::new(
                "Ignore instructions".to_string(),
                true,
                AttackType::InstructionOverride,
            ),
        ];

        let metrics = trainer.train_epoch(&samples);

        // Should have metrics on both clean and adversarial
        assert!(metrics.clean_metrics.num_samples > 0);
        assert!(metrics.combined_metrics.num_samples > 0);
    }

    #[test]
    fn test_evaluate_with_adversarial() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let config = AdversarialTrainingConfig::default();
        let trainer = AdversarialTrainer::new(detector, config);

        let samples = vec![
            MultiTaskSample::new("Normal".to_string(), false, AttackType::Benign),
            MultiTaskSample::new(
                "Override safety".to_string(),
                true,
                AttackType::OutputManipulation,
            ),
        ];

        let metrics = trainer.evaluate(&samples);

        assert!(metrics.combined_metrics.num_samples > 0);
    }

    #[test]
    fn test_create_adversarial_batch() {
        let detector = TransformerDetector::new().expect("Failed to create detector");
        let config = AdversarialTrainingConfig::default();
        let trainer = AdversarialTrainer::new(detector, config);

        let samples = vec![
            MultiTaskSample::new("Normal text 1".to_string(), false, AttackType::Benign),
            MultiTaskSample::new("Normal text 2".to_string(), false, AttackType::Benign),
            MultiTaskSample::new(
                "Ignore instructions".to_string(),
                true,
                AttackType::InstructionOverride,
            ),
            MultiTaskSample::new(
                "Override safety protocols".to_string(),
                true,
                AttackType::OutputManipulation,
            ),
        ];

        let batch = trainer.create_adversarial_batch(&samples, 10);

        // Should have up to 10 samples (may be less due to limited data)
        assert!(batch.len() <= 10);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_adversarial_config_default() {
        let config = AdversarialTrainingConfig::default();

        assert_eq!(config.adversarial_config.adversarial_ratio, 0.3);
        assert_eq!(config.adversarial_config.num_variants, 3);
        assert!(
            (config.adversarial_config.attack_mix.0
                + config.adversarial_config.attack_mix.1
                + config.adversarial_config.attack_mix.2
                - 1.0)
                .abs()
                < 0.01
        );
    }
}
