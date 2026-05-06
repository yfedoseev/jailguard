//! Error types for API operations.
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

/// API error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Request ID for tracing
    pub request_id: Option<String>,
}

impl ApiError {
    /// Create a new API error
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            request_id: None,
        }
    }

    /// Create validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    /// Create not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    /// Create config error
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::new("CONFIG_ERROR", message)
    }

    /// Create inference error
    pub fn inference(message: impl Into<String>) -> Self {
        Self::new("INFERENCE_ERROR", message)
    }

    /// Create timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new("TIMEOUT_ERROR", message)
    }

    /// Create internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }

    /// Set request ID
    pub fn with_request_id(mut self, id: String) -> Self {
        self.request_id = Some(id);
        self
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

/// API result type
pub type ApiResult<T> = Result<T, ApiError>;

// Convenience constructors as free functions for ergonomics
pub fn validation_error(msg: impl Into<String>) -> ApiError {
    ApiError::validation(msg)
}

pub fn not_found(msg: impl Into<String>) -> ApiError {
    ApiError::not_found(msg)
}

pub fn config_error(msg: impl Into<String>) -> ApiError {
    ApiError::config_error(msg)
}

pub fn inference_error(msg: impl Into<String>) -> ApiError {
    ApiError::inference(msg)
}

pub fn timeout_error(msg: impl Into<String>) -> ApiError {
    ApiError::timeout(msg)
}

pub fn internal_error(msg: impl Into<String>) -> ApiError {
    ApiError::internal(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ApiError::new("TEST_CODE", "Test message");
        assert_eq!(err.code, "TEST_CODE");
        assert_eq!(err.message, "Test message");
        assert!(err.request_id.is_none());
    }

    #[test]
    fn test_validation_error() {
        let err = ApiError::validation("Invalid input");
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert_eq!(err.message, "Invalid input");
    }

    #[test]
    fn test_error_with_request_id() {
        let err = ApiError::inference("Test").with_request_id("req-123".to_string());
        assert_eq!(err.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_error_display() {
        let err = ApiError::new("CODE", "message");
        assert_eq!(err.to_string(), "CODE: message");
    }
}
