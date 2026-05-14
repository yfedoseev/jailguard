//! Pre-trained model loading and management.
//!
//! This module handles loading pre-trained weights for the detector,
//! allowing immediate use without training.

#[cfg(feature = "pretrained")]
use crate::detection::{Detector, DetectorConfig};
#[cfg(feature = "pretrained")]
use crate::error::{Error, Result};

/// Available pre-trained models.
#[derive(Debug, Clone, Copy)]
pub enum PretrainedModel {
    /// Version 1 - trained on deepset/prompt-injections
    V1,
}

impl PretrainedModel {
    /// Get the model identifier string.
    pub fn id(&self) -> &'static str {
        match self {
            PretrainedModel::V1 => "jailguard-v1",
        }
    }

    /// Get the expected file name for weights.
    pub fn weights_file(&self) -> &'static str {
        match self {
            PretrainedModel::V1 => "jailguard-v1.bin",
        }
    }
}

/// Load a pre-trained detector by name.
#[cfg(feature = "pretrained")]
pub fn load_detector(name: &str) -> Result<Detector> {
    match name {
        "jailguard-v1" | "v1" => load_v1(),
        _ => Err(Error::PretrainedNotFound(name.to_string())),
    }
}

/// Load the V1 pre-trained model.
#[cfg(feature = "pretrained")]
fn load_v1() -> Result<Detector> {
    // In a full implementation, this would load weights from embedded bytes
    // or from a file in a known location.

    // For now, return a fresh detector (weights would need to be trained)
    Detector::with_config(DetectorConfig::default())
}

/// Get the path where pre-trained models are stored.
pub fn models_dir() -> std::path::PathBuf {
    // Check for environment variable first
    if let Ok(dir) = std::env::var("BASTIQUE_MODELS_DIR") {
        return std::path::PathBuf::from(dir);
    }

    // Default to user's data directory
    if let Some(data_dir) = dirs::data_dir() {
        return data_dir.join("jailguard").join("models");
    }

    // Fallback to current directory
    std::path::PathBuf::from(".").join("models")
}

/// List available pre-trained models.
pub fn list_available() -> Vec<PretrainedModel> {
    vec![PretrainedModel::V1]
}

/// Check if a pre-trained model is available locally.
pub fn is_available(model: PretrainedModel) -> bool {
    let path = models_dir().join(model.weights_file());
    path.exists()
}

/// Download a pre-trained model.
#[cfg(feature = "pretrained")]
pub fn download(_model: PretrainedModel) -> Result<()> {
    // In a full implementation, this would download weights from a URL
    Err(Error::Model("Download not implemented yet".to_string()))
}
