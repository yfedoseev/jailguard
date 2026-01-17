//! Dataset loading for training.
//!
//! This module provides loaders for prompt injection datasets:
//! - deepset/prompt-injections (primary dataset)
//! - Synthetic data generation

pub mod deduplication;
mod deepset;
pub mod external;
pub mod llm_augmentation;
mod multitask_sample;
pub mod phase1_pipeline;
mod synthetic;
pub mod synthetic_generator;

pub use deduplication::{DeduplicationConfig, Deduplicator, SampleWithEmbedding};
pub use deepset::DeepsetDataset;
pub use external::{ExpandedDataset, ExternalDatasetConfig};
pub use llm_augmentation::{
    AttackTypeSpec, LLMAugmentationConfig, LLMAugmentationGenerator, LLMAugmentedSample,
};
pub use multitask_sample::MultiTaskSample;
pub use phase1_pipeline::{ExtendedDataset, Phase1Config, Phase1Pipeline, Phase1Stats};
pub use synthetic::SyntheticDataset;
pub use synthetic_generator::{SyntheticDataGenerator, SyntheticGeneratorConfig, SyntheticSample};

/// A labeled sample for training.
#[derive(Debug, Clone)]
pub struct Sample {
    /// The text content
    pub text: String,
    /// Whether this is an injection (true) or benign (false)
    pub is_injection: bool,
}

/// Trait for datasets.
pub trait Dataset: Send + Sync {
    /// Get the number of samples.
    fn len(&self) -> usize;

    /// Check if the dataset is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a sample by index.
    fn get(&self, index: usize) -> Option<&Sample>;

    /// Get all samples.
    fn samples(&self) -> &[Sample];

    /// Get injection samples only.
    fn injections(&self) -> Vec<&Sample> {
        self.samples().iter().filter(|s| s.is_injection).collect()
    }

    /// Get benign samples only.
    fn benign(&self) -> Vec<&Sample> {
        self.samples().iter().filter(|s| !s.is_injection).collect()
    }
}
