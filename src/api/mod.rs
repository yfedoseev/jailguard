//! REST API for production inference deployment.
//!
//! This module provides HTTP endpoints for:
//! - Single inference requests
//! - Batch inference processing
//! - Health checks
//! - Metrics collection
//! - Model information

pub mod endpoints;
pub mod error;
pub mod metrics;
pub mod prometheus_exporter;
pub mod real_detector;
pub mod request;
pub mod response;

pub use endpoints::ApiEndpoints;
pub use error::{ApiError, ApiResult};
pub use metrics::{ApiMetrics, MetricsSnapshot};
pub use prometheus_exporter::PrometheusExporter;
pub use real_detector::{AttackType, DetectionResult, RealDetector};
pub use request::{BatchInferenceRequest, InferenceApiRequest};
pub use response::{BatchInferenceResponse, HealthResponse, InferenceApiResponse};

/// API version constant
pub const API_VERSION: &str = "1.0.0";

/// API configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// Enable metrics endpoint
    pub enable_metrics: bool,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Max batch size
    pub max_batch_size: usize,
    /// Enable request logging
    pub enable_logging: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_cors: true,
            enable_metrics: true,
            request_timeout_ms: 5000,
            max_batch_size: 32,
            enable_logging: true,
        }
    }
}

impl ApiConfig {
    /// Get the server address
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Validate configuration
    pub fn validate(&self) -> ApiResult<()> {
        if self.port == 0 {
            return Err(ApiError::config_error("Port cannot be 0"));
        }

        if self.max_batch_size == 0 {
            return Err(ApiError::config_error("Max batch size must be > 0"));
        }

        if self.request_timeout_ms == 0 {
            return Err(ApiError::config_error("Request timeout must be > 0"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ApiConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.enable_cors);
    }

    #[test]
    fn test_config_address() {
        let config = ApiConfig::default();
        assert_eq!(config.address(), "127.0.0.1:8080");
    }

    #[test]
    fn test_config_validation() {
        let valid = ApiConfig::default();
        assert!(valid.validate().is_ok());

        let invalid_port = ApiConfig {
            port: 0,
            ..Default::default()
        };
        assert!(invalid_port.validate().is_err());
    }
}
