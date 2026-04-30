#![allow(missing_docs)]
use crate::dataset::Sample;
/// Phase 1 Dataset Extension Pipeline
///
/// Orchestrates synthetic data generation and LLM augmentation to extend
/// the training dataset from 4,500 to 12,000+ samples, targeting:
/// - Synthetic generation: 3-5 variants per existing sample
/// - LLM augmentation: 5,000-7,000 novel samples
/// - Target improvement: +0.8-1.5% accuracy (95.9% → 96.7-97.4%)
/// - Timeline: 2-3 weeks
use crate::dataset::{
    AttackTypeSpec, DeduplicationConfig, Deduplicator, LLMAugmentationConfig,
    LLMAugmentationGenerator, SampleWithEmbedding, SyntheticDataGenerator,
    SyntheticGeneratorConfig,
};

/// Configuration for Phase 1 pipeline
#[derive(Clone, Debug)]
pub struct Phase1Config {
    /// Enable synthetic generation
    pub enable_synthetic: bool,
    /// Number of variants per injection sample
    pub synthetic_variants_per_sample: usize, // 3-5

    /// Enable LLM augmentation
    pub enable_llm_augmentation: bool,
    /// Target number of LLM-generated samples
    pub llm_target_samples: usize, // 5000-7000
    /// LLM config (API key, model, etc.)
    pub llm_config: Option<LLMAugmentationConfig>,

    /// Enable deduplication
    pub enable_deduplication: bool,
    /// Embedding dimension for similarity computation
    pub embedding_dim: usize,
    /// Similarity threshold for deduplication
    pub similarity_threshold: f32,

    /// Verbose logging
    pub verbose: bool,
}

impl Default for Phase1Config {
    fn default() -> Self {
        Self {
            enable_synthetic: true,
            synthetic_variants_per_sample: 4,
            enable_llm_augmentation: false, // Requires API key
            llm_target_samples: 6000,
            llm_config: None,
            enable_deduplication: true,
            embedding_dim: 768,
            similarity_threshold: 0.92,
            verbose: true,
        }
    }
}

/// Statistics from Phase 1 pipeline execution
#[derive(Clone, Debug)]
pub struct Phase1Stats {
    pub original_samples: usize,
    pub original_injections: usize,

    pub synthetic_generated: usize,
    pub synthetic_valid: usize,

    pub llm_generated: usize,
    pub llm_valid: usize,

    pub pre_dedup_total: usize,
    pub post_dedup_total: usize,
    pub duplicates_removed: usize,

    pub injections_after: usize,
    pub benign_after: usize,
    pub balance_ratio: f32,

    pub expected_accuracy_improvement: (f32, f32), // (min, max)
}

/// Extended dataset with Phase 1 augmentation
#[derive(Clone, Debug)]
pub struct ExtendedDataset {
    pub original: Vec<Sample>,
    pub synthetic: Vec<Sample>,
    pub llm_augmented: Vec<Sample>,
    pub all_samples: Vec<Sample>,
    pub stats: Phase1Stats,
}

impl ExtendedDataset {
    pub fn len(&self) -> usize {
        self.all_samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.all_samples.is_empty()
    }

    pub fn injections(&self) -> usize {
        self.all_samples.iter().filter(|s| s.is_injection).count()
    }

    pub fn benign(&self) -> usize {
        self.all_samples.iter().filter(|s| !s.is_injection).count()
    }

    pub fn balance_ratio(&self) -> f32 {
        let injections = self.injections();
        let benign = self.benign();
        if benign == 0 {
            0.0
        } else {
            injections as f32 / benign as f32
        }
    }
}

/// Phase 1 Pipeline orchestrator
pub struct Phase1Pipeline {
    config: Phase1Config,
    synthetic_gen: SyntheticDataGenerator,
}

impl Phase1Pipeline {
    /// Create a new Phase 1 pipeline
    pub fn new(config: Phase1Config) -> Self {
        let synthetic_config = SyntheticGeneratorConfig {
            max_variants_per_sample: config.synthetic_variants_per_sample,
            include_expansion: true,
            include_synonym: true,
            include_pronoun_variation: true,
            include_structure_change: true,
            seed: 42,
        };

        let synthetic_gen = SyntheticDataGenerator::new(synthetic_config);

        Self {
            config,
            synthetic_gen,
        }
    }

    /// Execute Phase 1 pipeline: synthetic + LLM augmentation + deduplication
    pub async fn execute(&self, original_samples: &[Sample]) -> ExtendedDataset {
        if self.config.verbose {
            println!("=== Phase 1 Dataset Extension Pipeline ===");
            println!("Original dataset: {} samples", original_samples.len());
        }

        let mut all_samples = original_samples.to_vec();
        let original_count = all_samples.len();
        let original_injections = all_samples.iter().filter(|s| s.is_injection).count();

        // Phase 1a: Synthetic generation
        let (synthetic_samples, synthetic_stats) = if self.config.enable_synthetic {
            self.generate_synthetic(original_samples)
        } else {
            (vec![], (0, 0))
        };
        all_samples.extend(synthetic_samples.clone());

        // Phase 1b: LLM augmentation
        let (llm_samples, llm_stats) = if self.config.enable_llm_augmentation {
            self.generate_llm_augmented().await
        } else {
            (vec![], (0, 0))
        };
        all_samples.extend(llm_samples.clone());

        // Phase 1c: Deduplication
        let (deduplicated, dedup_stats) = if self.config.enable_deduplication {
            self.deduplicate_samples(&all_samples)
        } else {
            (all_samples.clone(), (all_samples.len(), 0))
        };

        let injections_after = deduplicated.iter().filter(|s| s.is_injection).count();
        let benign_after = deduplicated.len() - injections_after;

        let stats = Phase1Stats {
            original_samples: original_count,
            original_injections,

            synthetic_generated: synthetic_stats.0,
            synthetic_valid: synthetic_stats.1,

            llm_generated: llm_stats.0,
            llm_valid: llm_stats.1,

            pre_dedup_total: all_samples.len(),
            post_dedup_total: deduplicated.len(),
            duplicates_removed: dedup_stats.1,

            injections_after,
            benign_after,
            balance_ratio: if benign_after == 0 {
                0.0
            } else {
                injections_after as f32 / benign_after as f32
            },

            expected_accuracy_improvement: (0.008, 0.015), // 0.8-1.5%
        };

        if self.config.verbose {
            self.print_pipeline_results(&stats);
        }

        ExtendedDataset {
            original: original_samples.to_vec(),
            synthetic: synthetic_samples,
            llm_augmented: llm_samples,
            all_samples: deduplicated,
            stats,
        }
    }

    /// Generate synthetic variants from original samples
    fn generate_synthetic(&self, original: &[Sample]) -> (Vec<Sample>, (usize, usize)) {
        if self.config.verbose {
            println!("\n[Phase 1a] Synthetic Generation");
            println!(
                "  Processing {} injection samples",
                original.iter().filter(|s| s.is_injection).count()
            );
        }

        let mut synthetic = Vec::new();
        let mut total_generated = 0;
        let mut valid_count = 0;

        for sample in original {
            if sample.is_injection {
                let variants = self.synthetic_gen.generate_variants(
                    &sample.text,
                    true,
                    Some("injection".to_string()),
                );

                for variant in variants {
                    total_generated += 1;

                    // Validation: non-empty, reasonable length
                    if !variant.text.is_empty()
                        && variant.text.len() > 5
                        && variant.text.len() < 2000
                    {
                        synthetic.push(Sample {
                            text: variant.text,
                            is_injection: variant.is_injection,
                        });
                        valid_count += 1;
                    }
                }
            }
        }

        if self.config.verbose {
            println!("  Generated: {} variants", total_generated);
            println!(
                "  Valid: {} ({:.1}%)",
                valid_count,
                (valid_count as f32 / total_generated.max(1) as f32) * 100.0
            );
        }

        (synthetic, (total_generated, valid_count))
    }

    /// Generate LLM-augmented samples
    async fn generate_llm_augmented(&self) -> (Vec<Sample>, (usize, usize)) {
        if self.config.verbose {
            println!("\n[Phase 1b] LLM Augmentation");
        }

        let config = match &self.config.llm_config {
            Some(cfg) => cfg.clone(),
            None => LLMAugmentationConfig::default(),
        };

        let llm_gen = LLMAugmentationGenerator::new(config);

        let mut augmented = Vec::new();
        let mut total_generated = 0;
        let mut valid_count = 0;

        // Generate samples for each attack type
        let samples_per_type = self.config.llm_target_samples / 6;
        let attack_types = vec![
            AttackTypeSpec::RolePlay,
            AttackTypeSpec::InstructionOverride,
            AttackTypeSpec::ContextManipulation,
            AttackTypeSpec::OutputManipulation,
            AttackTypeSpec::EncodingObfuscation,
            AttackTypeSpec::JailbreakPatterns,
        ];

        for attack_type in attack_types {
            if self.config.verbose {
                println!(
                    "  Generating {} samples for {:?}...",
                    samples_per_type, attack_type
                );
            }

            match llm_gen
                .generate_samples(attack_type, samples_per_type)
                .await
            {
                Ok(samples) => {
                    for sample in samples {
                        total_generated += 1;

                        // Validation
                        if !sample.text.is_empty()
                            && sample.text.len() > 5
                            && sample.confidence >= 0.7
                        {
                            augmented.push(Sample {
                                text: sample.text,
                                is_injection: true,
                            });
                            valid_count += 1;
                        }
                    }
                }
                Err(e) => {
                    if self.config.verbose {
                        println!("  Warning: LLM generation failed: {}", e);
                    }
                }
            }
        }

        if self.config.verbose {
            println!("  Generated: {} samples", total_generated);
            println!(
                "  Valid: {} ({:.1}%)",
                valid_count,
                (valid_count as f32 / total_generated.max(1) as f32) * 100.0
            );
        }

        (augmented, (total_generated, valid_count))
    }

    /// Deduplicate samples using embedding similarity
    fn deduplicate_samples(&self, samples: &[Sample]) -> (Vec<Sample>, (usize, usize)) {
        if self.config.verbose {
            println!("\n[Phase 1c] Deduplication");
            println!("  Samples to deduplicate: {}", samples.len());
        }

        // Convert samples to embeddings (stub: using simple hash-based embeddings for now)
        let samples_with_embeddings: Vec<SampleWithEmbedding> = samples
            .iter()
            .enumerate()
            .map(|(i, sample)| {
                // Simple embedding: hash-based for demonstration
                // In production, would use actual text embeddings
                let embedding = Self::compute_embedding(&sample.text, self.config.embedding_dim);

                SampleWithEmbedding {
                    text: sample.text.clone(),
                    embedding,
                    is_injection: sample.is_injection,
                    attack_type: if sample.is_injection {
                        Some("injection".to_string())
                    } else {
                        None
                    },
                    source: format!("sample_{}", i),
                }
            })
            .collect();

        let dedup_config = DeduplicationConfig {
            similarity_threshold: self.config.similarity_threshold,
            canonical_selection: crate::dataset::deduplication::CanonicalSelectionMethod::Longest,
            verbose: self.config.verbose,
        };

        let deduplicator = Deduplicator::new(dedup_config);
        let (deduplicated_embeddings, dedup_stats) =
            deduplicator.deduplicate_and_select(samples_with_embeddings, None);

        let deduplicated_samples: Vec<Sample> = deduplicated_embeddings
            .into_iter()
            .map(|e| Sample {
                text: e.text,
                is_injection: e.is_injection,
            })
            .collect();

        (
            deduplicated_samples,
            (samples.len(), dedup_stats.total_removed),
        )
    }

    /// Compute simple embedding from text (stub for demonstration)
    /// In production, would use actual text embeddings from the model
    fn compute_embedding(text: &str, dim: usize) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Create deterministic random embedding based on text hash
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        let mut embedding = vec![0.0; dim];
        let mut rng_state = hash;

        for i in 0..dim {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let value = ((rng_state >> 16) & 0x7fff) as f32 / 32768.0;
            embedding[i] = value * 2.0 - 1.0;
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for e in &mut embedding {
                *e /= norm;
            }
        }

        embedding
    }

    /// Print pipeline results
    fn print_pipeline_results(&self, stats: &Phase1Stats) {
        println!("\n=== Phase 1 Results ===");
        println!(
            "Original dataset: {} samples ({} injections)",
            stats.original_samples, stats.original_injections
        );

        if stats.synthetic_generated > 0 {
            println!("\nSynthetic generation:");
            println!("  Generated: {}", stats.synthetic_generated);
            println!("  Valid: {}", stats.synthetic_valid);
        }

        if stats.llm_generated > 0 {
            println!("\nLLM augmentation:");
            println!("  Generated: {}", stats.llm_generated);
            println!("  Valid: {}", stats.llm_valid);
        }

        if stats.duplicates_removed > 0 {
            println!("\nDeduplication:");
            println!("  Pre-dedup: {}", stats.pre_dedup_total);
            println!("  Post-dedup: {}", stats.post_dedup_total);
            println!(
                "  Removed: {} ({:.1}%)",
                stats.duplicates_removed,
                (stats.duplicates_removed as f32 / stats.pre_dedup_total as f32) * 100.0
            );
        }

        println!("\nFinal dataset:");
        println!("  Total samples: {}", stats.post_dedup_total);
        println!(
            "  Injections: {} ({:.1}%)",
            stats.injections_after,
            (stats.injections_after as f32 / stats.post_dedup_total as f32) * 100.0
        );
        println!(
            "  Benign: {} ({:.1}%)",
            stats.benign_after,
            (stats.benign_after as f32 / stats.post_dedup_total as f32) * 100.0
        );
        println!("  Balance ratio: {:.2}", stats.balance_ratio);

        println!("\nExpected accuracy improvement:");
        println!(
            "  Range: {:.1}% → +{:.1}% (95.9% base)",
            stats.expected_accuracy_improvement.0 * 100.0,
            stats.expected_accuracy_improvement.1 * 100.0
        );
        println!("  Target: 96.7% - 97.4%");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase1_config_default() {
        let config = Phase1Config::default();
        assert!(config.enable_synthetic);
        assert_eq!(config.synthetic_variants_per_sample, 4);
    }

    #[test]
    fn test_embedding_computation() {
        let embedding1 = Phase1Pipeline::compute_embedding("ignore instructions", 768);
        let embedding2 = Phase1Pipeline::compute_embedding("ignore instructions", 768);

        // Same text should produce same embedding
        assert_eq!(embedding1, embedding2);
        assert_eq!(embedding1.len(), 768);

        // Embedding should be normalized
        let norm: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_extended_dataset_stats() {
        let samples = vec![
            Sample {
                text: "injection".to_string(),
                is_injection: true,
            },
            Sample {
                text: "benign".to_string(),
                is_injection: false,
            },
        ];

        let stats = Phase1Stats {
            original_samples: 2,
            original_injections: 1,
            synthetic_generated: 0,
            synthetic_valid: 0,
            llm_generated: 0,
            llm_valid: 0,
            pre_dedup_total: 2,
            post_dedup_total: 2,
            duplicates_removed: 0,
            injections_after: 1,
            benign_after: 1,
            balance_ratio: 1.0,
            expected_accuracy_improvement: (0.008, 0.015),
        };

        let extended = ExtendedDataset {
            original: samples,
            synthetic: vec![],
            llm_augmented: vec![],
            all_samples: vec![],
            stats,
        };

        assert_eq!(extended.stats.original_samples, 2);
        assert_eq!(extended.stats.balance_ratio, 1.0);
    }
}
