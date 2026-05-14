//! Response types for the API.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Single inference API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceApiResponse {
    /// Request ID (for tracking)
    pub request_id: String,
    /// Detection result
    pub is_injection: bool,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Processing latency in milliseconds
    pub latency_ms: u64,
    /// Timestamp of response
    pub timestamp: String,
    /// Response status
    pub status: String,
}

impl InferenceApiResponse {
    /// Create a successful response
    pub fn success(
        request_id: String,
        is_injection: bool,
        confidence: f32,
        latency_ms: u64,
    ) -> Self {
        Self {
            request_id,
            is_injection,
            confidence,
            latency_ms,
            timestamp: chrono_timestamp(),
            status: "success".to_string(),
        }
    }

    /// Create an error response
    pub fn error(request_id: String, message: String, latency_ms: u64) -> Self {
        Self {
            request_id,
            is_injection: false,
            confidence: 0.0,
            latency_ms,
            timestamp: chrono_timestamp(),
            status: format!("error: {}", message),
        }
    }
}

/// Batch inference API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInferenceResponse {
    /// Batch ID (for tracking)
    pub batch_id: String,
    /// Individual responses
    pub responses: Vec<InferenceApiResponse>,
    /// Total batch processing time in milliseconds
    pub total_latency_ms: u64,
    /// Average latency per request
    pub avg_latency_ms: f32,
    /// Timestamp of response
    pub timestamp: String,
    /// Batch status
    pub status: String,
}

impl BatchInferenceResponse {
    /// Create a successful batch response
    pub fn success(
        batch_id: String,
        responses: Vec<InferenceApiResponse>,
        total_latency_ms: u64,
    ) -> Self {
        let avg_latency_ms = if !responses.is_empty() {
            total_latency_ms as f32 / responses.len() as f32
        } else {
            0.0
        };

        Self {
            batch_id,
            responses,
            total_latency_ms,
            avg_latency_ms,
            timestamp: chrono_timestamp(),
            status: "success".to_string(),
        }
    }

    /// Create an error response
    pub fn error(batch_id: String, message: String, latency_ms: u64) -> Self {
        Self {
            batch_id,
            responses: Vec::new(),
            total_latency_ms: latency_ms,
            avg_latency_ms: 0.0,
            timestamp: chrono_timestamp(),
            status: format!("error: {}", message),
        }
    }

    /// Get number of responses
    pub fn len(&self) -> usize {
        self.responses.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.responses.is_empty()
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status: "healthy" or "unhealthy"
    pub status: String,
    /// API version
    pub version: String,
    /// Timestamp
    pub timestamp: String,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
    /// Models loaded
    pub models_loaded: usize,
    /// Cache status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_info: Option<CacheInfo>,
}

/// Cache information in health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    /// Is cache enabled
    pub enabled: bool,
    /// Cache hit rate (0.0-1.0)
    pub hit_rate: f32,
    /// Current entries in cache
    pub entries: usize,
}

impl HealthResponse {
    /// Create a healthy response
    pub fn healthy(uptime_seconds: u64) -> Self {
        Self {
            status: "healthy".to_string(),
            version: super::API_VERSION.to_string(),
            timestamp: chrono_timestamp(),
            uptime_seconds,
            models_loaded: 1,
            cache_info: None,
        }
    }

    /// Create an unhealthy response
    pub fn unhealthy(reason: String) -> Self {
        Self {
            status: format!("unhealthy: {}", reason),
            version: super::API_VERSION.to_string(),
            timestamp: chrono_timestamp(),
            uptime_seconds: 0,
            models_loaded: 0,
            cache_info: None,
        }
    }

    /// Set cache info
    pub fn with_cache_info(mut self, cache_info: CacheInfo) -> Self {
        self.cache_info = Some(cache_info);
        self
    }
}

/// Helper function to get current timestamp in ISO 8601 format
fn chrono_timestamp() -> String {
    let now = SystemTime::now();
    let duration = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();

    // Simple ISO 8601 formatting without external dependencies
    let days_since_epoch = secs / 86400;
    let remaining_secs = secs % 86400;
    let hours = remaining_secs / 3600;
    let minutes = (remaining_secs % 3600) / 60;
    let seconds = remaining_secs % 60;

    // Approximate date calculation (good enough for timestamps)
    let years = 1970 + days_since_epoch / 365;
    let days = days_since_epoch % 365;
    let months = (days / 30).min(11);
    let day_of_month = (days % 30) + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
        years,
        months + 1,
        day_of_month,
        hours,
        minutes,
        seconds,
        nanos / 1000
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::API_VERSION;

    #[test]
    fn test_inference_response_success() {
        let resp = InferenceApiResponse::success("req-1".to_string(), true, 0.95, 10);

        assert_eq!(resp.request_id, "req-1");
        assert!(resp.is_injection);
        assert_eq!(resp.confidence, 0.95);
        assert_eq!(resp.status, "success");
    }

    #[test]
    fn test_batch_response_creation() {
        let responses = vec![
            InferenceApiResponse::success("req-1".to_string(), true, 0.95, 10),
            InferenceApiResponse::success("req-2".to_string(), false, 0.2, 10),
        ];

        let batch = BatchInferenceResponse::success("batch-1".to_string(), responses, 20);

        assert_eq!(batch.len(), 2);
        assert_eq!(batch.total_latency_ms, 20);
        assert_eq!(batch.avg_latency_ms, 10.0);
    }

    #[test]
    fn test_health_response() {
        let health = HealthResponse::healthy(3600);
        assert_eq!(health.status, "healthy");
        assert_eq!(health.uptime_seconds, 3600);
        assert_eq!(health.version, API_VERSION);
    }

    #[test]
    fn test_health_with_cache() {
        let cache_info = CacheInfo {
            enabled: true,
            hit_rate: 0.75,
            entries: 500,
        };

        let health = HealthResponse::healthy(3600).with_cache_info(cache_info);
        assert!(health.cache_info.is_some());
        assert_eq!(health.cache_info.unwrap().hit_rate, 0.75);
    }
}
