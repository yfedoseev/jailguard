//! Error types for the jailguard crate.

use thiserror::Error;

/// Main error type for jailguard operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Error during model operations (loading, inference, etc.)
    #[error("Model error: {0}")]
    Model(String),

    /// Error during training
    #[error("Training error: {0}")]
    Training(String),

    /// Error during tokenization
    #[error("Tokenization error: {0}")]
    Tokenization(String),

    /// Error during dataset loading or processing
    #[error("Dataset error: {0}")]
    Dataset(String),

    /// I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Pre-trained model not found
    #[error("Pre-trained model not found: {0}")]
    PretrainedNotFound(String),
}

/// Result type alias for jailguard operations.
pub type Result<T> = std::result::Result<T, Error>;

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}
