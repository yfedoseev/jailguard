//! Model serialization for saving and loading trained models.
//!
//! Provides infrastructure for persisting trained models to disk and
//! recovering them for inference in production environments.

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

/// Serialized model metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelMetadata {
    /// Model version
    pub version: String,
    /// Date the model was saved (ISO 8601 format)
    pub timestamp: String,
    /// Training epochs
    pub epochs_trained: u32,
    /// Final training accuracy
    pub train_accuracy: f32,
    /// Final validation accuracy
    pub val_accuracy: f32,
    /// Final validation loss
    pub val_loss: f32,
    /// Model architecture description
    pub architecture: String,
    /// Embedding dimension
    pub embedding_dim: usize,
    /// Number of trainable parameters
    pub num_parameters: usize,
}

impl ModelMetadata {
    /// Create new model metadata
    pub fn new(
        version: String,
        timestamp: String,
        epochs: u32,
        train_acc: f32,
        val_acc: f32,
        val_loss: f32,
        architecture: String,
        embed_dim: usize,
        num_params: usize,
    ) -> Self {
        Self {
            version,
            timestamp,
            epochs_trained: epochs,
            train_accuracy: train_acc,
            val_accuracy: val_acc,
            val_loss,
            architecture,
            embedding_dim: embed_dim,
            num_parameters: num_params,
        }
    }

    /// Serialize metadata to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize metadata from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Model checkpoint with weights and metadata
#[derive(Debug)]
pub struct ModelCheckpoint {
    /// Model weights (flattened)
    pub weights: Vec<f32>,
    /// Model metadata
    pub metadata: ModelMetadata,
}

impl ModelCheckpoint {
    /// Create new checkpoint
    pub fn new(weights: Vec<f32>, metadata: ModelMetadata) -> Self {
        Self { weights, metadata }
    }

    /// Save checkpoint to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize weights as binary (4 bytes per float)
        let weight_bytes: Vec<u8> = self
            .weights
            .iter()
            .flat_map(|w| w.to_le_bytes().to_vec())
            .collect();

        // Serialize metadata as JSON
        let metadata_json = self
            .metadata
            .to_json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Write metadata size (4 bytes)
        let mut file = fs::File::create(path)?;
        let metadata_size = metadata_json.len() as u32;
        file.write_all(&metadata_size.to_le_bytes())?;

        // Write metadata
        file.write_all(metadata_json.as_bytes())?;

        // Write weights
        file.write_all(&weight_bytes)?;

        Ok(())
    }

    /// Load checkpoint from file
    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let mut file = fs::File::open(path)?;

        // Read metadata size
        let mut size_bytes = [0u8; 4];
        file.read_exact(&mut size_bytes)?;
        let metadata_size = u32::from_le_bytes(size_bytes) as usize;

        // Read metadata
        let mut metadata_buf = vec![0u8; metadata_size];
        file.read_exact(&mut metadata_buf)?;
        let metadata_json = String::from_utf8(metadata_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let metadata = ModelMetadata::from_json(&metadata_json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Read weights
        let mut weight_bytes = Vec::new();
        file.read_to_end(&mut weight_bytes)?;

        // Convert bytes to floats
        let weights: Vec<f32> = weight_bytes
            .chunks_exact(4)
            .map(|chunk| {
                let mut bytes = [0u8; 4];
                bytes.copy_from_slice(chunk);
                f32::from_le_bytes(bytes)
            })
            .collect();

        Ok(ModelCheckpoint::new(weights, metadata))
    }

    /// Get model size in bytes
    pub fn size_bytes(&self) -> usize {
        // 4 bytes for metadata size + metadata JSON + weight bytes
        4 + self.metadata.to_json().unwrap_or_default().len() + (self.weights.len() * 4)
    }

    /// Get model size in MB
    pub fn size_mb(&self) -> f32 {
        self.size_bytes() as f32 / 1024.0 / 1024.0
    }
}

/// Model format enumerator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFormat {
    /// Binary format (compact)
    Binary,
    /// JSON format (human-readable)
    Json,
    /// ONNX format (cross-platform)
    Onnx,
}

impl std::fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ModelFormat::Binary => write!(f, "binary"),
            ModelFormat::Json => write!(f, "json"),
            ModelFormat::Onnx => write!(f, "onnx"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_metadata_creation() {
        let metadata = ModelMetadata::new(
            "1.0.0".to_string(),
            "2026-01-18T12:00:00Z".to_string(),
            10,
            0.92,
            0.90,
            0.15,
            "Transformer-based detector".to_string(),
            384,
            1_000_000,
        );

        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.epochs_trained, 10);
        assert_eq!(metadata.train_accuracy, 0.92);
    }

    #[test]
    fn test_metadata_json_serialization() {
        let metadata = ModelMetadata::new(
            "1.0.0".to_string(),
            "2026-01-18T12:00:00Z".to_string(),
            10,
            0.92,
            0.90,
            0.15,
            "Transformer-based detector".to_string(),
            384,
            1_000_000,
        );

        let json = metadata.to_json().unwrap();
        let restored = ModelMetadata::from_json(&json).unwrap();

        assert_eq!(restored.version, metadata.version);
        assert_eq!(restored.epochs_trained, metadata.epochs_trained);
    }

    #[test]
    fn test_checkpoint_creation() {
        let metadata = ModelMetadata::new(
            "1.0.0".to_string(),
            "2026-01-18T12:00:00Z".to_string(),
            10,
            0.92,
            0.90,
            0.15,
            "Transformer".to_string(),
            384,
            1_000_000,
        );

        let weights = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let checkpoint = ModelCheckpoint::new(weights.clone(), metadata);

        assert_eq!(checkpoint.weights, weights);
        assert_eq!(checkpoint.metadata.version, "1.0.0");
    }

    #[test]
    fn test_checkpoint_save_load() {
        let path = PathBuf::from("target/test_checkpoint_model.bin");

        // Clean up before test
        let _ = fs::remove_file(&path);

        let metadata = ModelMetadata::new(
            "1.0.0".to_string(),
            "2026-01-18T12:00:00Z".to_string(),
            10,
            0.92,
            0.90,
            0.15,
            "Transformer".to_string(),
            384,
            1_000_000,
        );

        let weights = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let checkpoint = ModelCheckpoint::new(weights.clone(), metadata);

        checkpoint.save(&path).unwrap();
        let restored = ModelCheckpoint::load(&path).unwrap();

        assert_eq!(restored.weights, weights);
        assert_eq!(restored.metadata.version, "1.0.0");
        assert_eq!(restored.metadata.epochs_trained, 10);
    }

    #[test]
    fn test_checkpoint_size() {
        let metadata = ModelMetadata::new(
            "1.0.0".to_string(),
            "2026-01-18T12:00:00Z".to_string(),
            10,
            0.92,
            0.90,
            0.15,
            "Transformer".to_string(),
            384,
            1_000_000,
        );

        let weights = vec![0.1; 1000];
        let checkpoint = ModelCheckpoint::new(weights, metadata);

        let size_bytes = checkpoint.size_bytes();
        let size_mb = checkpoint.size_mb();

        assert!(size_bytes > 4000); // At least weight bytes
        assert!(size_mb > 0.0);
    }

    #[test]
    fn test_model_format_display() {
        assert_eq!(ModelFormat::Binary.to_string(), "binary");
        assert_eq!(ModelFormat::Json.to_string(), "json");
        assert_eq!(ModelFormat::Onnx.to_string(), "onnx");
    }
}
