//! Dataset loading for training.
//!
//! This module provides loaders for prompt injection datasets:
//! - deepset/prompt-injections (primary dataset)
//! - Synthetic data generation

mod deepset;
mod multitask_sample;
mod synthetic;

pub use deepset::DeepsetDataset;
pub use multitask_sample::MultiTaskSample;
pub use synthetic::SyntheticDataset;

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
