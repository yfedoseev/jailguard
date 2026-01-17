//! Collection Framework Error Types
//!
//! Unified error handling for all collection sources.

use std::fmt;

/// Collection framework error types
#[derive(Debug, Clone)]
pub enum CollectionError {
    /// API error (network, authentication, etc.)
    ApiError(String),
    /// Rate limit exceeded
    RateLimitExceeded {
        reset_time: Option<String>,
    },
    /// Parse error (invalid response format)
    ParseError(String),
    /// Validation error (sample doesn't meet criteria)
    ValidationError(String),
    /// Network error
    NetworkError(String),
    /// Data format error
    FormatError(String),
    /// Configuration error
    ConfigError(String),
}

impl fmt::Display for CollectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CollectionError::ApiError(msg) => write!(f, "API error: {}", msg),
            CollectionError::RateLimitExceeded { reset_time } => {
                write!(
                    f,
                    "Rate limit exceeded{}",
                    reset_time
                        .as_ref()
                        .map(|t| format!(" (resets at {})", t))
                        .unwrap_or_default()
                )
            }
            CollectionError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CollectionError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            CollectionError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            CollectionError::FormatError(msg) => write!(f, "Format error: {}", msg),
            CollectionError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for CollectionError {}

/// Result type for collection operations
pub type CollectionResult<T> = Result<T, CollectionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_error_display() {
        let err = CollectionError::ApiError("Connection timeout".to_string());
        assert!(err.to_string().contains("API error"));

        let err = CollectionError::RateLimitExceeded {
            reset_time: Some("2026-01-18 10:00:00".to_string()),
        };
        assert!(err.to_string().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_collection_error_types() {
        let _api_err: CollectionResult<()> =
            Err(CollectionError::ApiError("test".to_string()));
        let _parse_err: CollectionResult<()> =
            Err(CollectionError::ParseError("test".to_string()));
        let _val_err: CollectionResult<()> =
            Err(CollectionError::ValidationError("test".to_string()));

        // All should be constructible
        assert!(true);
    }
}
