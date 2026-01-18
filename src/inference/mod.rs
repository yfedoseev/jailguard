//! Inference optimization for production deployment.
//!
//! This module provides optimized inference capabilities:
//! - Efficient model loading and caching
//! - Batch inference support
//! - Latency optimization
//! - Memory-efficient detection

pub mod batch_inference;
pub mod inference_cache;
pub mod inference_config;

pub use batch_inference::{BatchInference, InferenceBatch, InferenceRequest, InferenceResponse};
pub use inference_cache::{CacheConfig, CacheStats, InferenceCache};
pub use inference_config::InferenceConfig;

/// Result type for inference operations
pub type InferenceResult<T> = std::result::Result<T, InferenceError>;

/// Errors that can occur during inference
#[derive(Debug, Clone)]
pub enum InferenceError {
    /// Model loading error
    ModelLoadError(String),
    /// Batch processing error
    BatchError(String),
    /// Cache error
    CacheError(String),
    /// Invalid input
    InvalidInput(String),
    /// IO error
    IoError(String),
}

impl std::fmt::Display for InferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModelLoadError(e) => write!(f, "Model load error: {}", e),
            Self::BatchError(e) => write!(f, "Batch error: {}", e),
            Self::CacheError(e) => write!(f, "Cache error: {}", e),
            Self::InvalidInput(e) => write!(f, "Invalid input: {}", e),
            Self::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for InferenceError {}

impl From<std::io::Error> for InferenceError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e.to_string())
    }
}
