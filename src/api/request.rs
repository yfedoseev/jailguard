//! Request types for the API.

use serde::{Deserialize, Serialize};

/// Single inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceApiRequest {
    /// Input text to analyze
    pub text: String,
    /// Optional request ID for tracking
    #[serde(default)]
    pub request_id: Option<String>,
    /// Optional user/client identifier
    #[serde(default)]
    pub client_id: Option<String>,
}

impl InferenceApiRequest {
    /// Create a new inference request
    pub fn new(text: String) -> Self {
        Self {
            text,
            request_id: None,
            client_id: None,
        }
    }

    /// Set request ID
    pub fn with_id(mut self, id: String) -> Self {
        self.request_id = Some(id);
        self
    }

    /// Set client ID
    pub fn with_client(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    /// Validate request
    pub fn validate(&self) -> Result<(), String> {
        if self.text.is_empty() {
            return Err("text cannot be empty".to_string());
        }

        if self.text.len() > 1_000_000 {
            return Err("text exceeds maximum length (1MB)".to_string());
        }

        Ok(())
    }
}

/// Batch inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInferenceRequest {
    /// List of inference requests
    pub requests: Vec<InferenceApiRequest>,
    /// Optional batch ID for tracking
    #[serde(default)]
    pub batch_id: Option<String>,
    /// Parallel processing enabled
    #[serde(default)]
    pub parallel: bool,
}

impl BatchInferenceRequest {
    /// Create a new batch request
    pub fn new(requests: Vec<InferenceApiRequest>) -> Self {
        Self {
            requests,
            batch_id: None,
            parallel: false,
        }
    }

    /// Validate batch request
    pub fn validate(&self, max_batch_size: usize) -> Result<(), String> {
        if self.requests.is_empty() {
            return Err("requests cannot be empty".to_string());
        }

        if self.requests.len() > max_batch_size {
            return Err(format!(
                "batch size {} exceeds maximum {}",
                self.requests.len(),
                max_batch_size
            ));
        }

        for req in &self.requests {
            req.validate()?;
        }

        Ok(())
    }

    /// Get number of requests
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_request_creation() {
        let req = InferenceApiRequest::new("test".to_string());
        assert_eq!(req.text, "test");
        assert!(req.request_id.is_none());
    }

    #[test]
    fn test_inference_request_validation() {
        let valid = InferenceApiRequest::new("test".to_string());
        assert!(valid.validate().is_ok());

        let empty = InferenceApiRequest::new("".to_string());
        assert!(empty.validate().is_err());
    }

    #[test]
    fn test_batch_request_creation() {
        let req1 = InferenceApiRequest::new("test1".to_string());
        let req2 = InferenceApiRequest::new("test2".to_string());
        let batch = BatchInferenceRequest::new(vec![req1, req2]);

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_batch_request_validation() {
        let req = InferenceApiRequest::new("test".to_string());
        let batch = BatchInferenceRequest::new(vec![req]);

        assert!(batch.validate(32).is_ok());
    }

    #[test]
    fn test_batch_request_max_size() {
        let req1 = InferenceApiRequest::new("test1".to_string());
        let req2 = InferenceApiRequest::new("test2".to_string());
        let batch = BatchInferenceRequest::new(vec![req1, req2]);

        assert!(batch.validate(1).is_err());
        assert!(batch.validate(2).is_ok());
    }
}
