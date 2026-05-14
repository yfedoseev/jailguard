//! Metrics collection and reporting for the API.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// API metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Total requests processed
    pub total_requests: u64,
    /// Total responses sent
    pub total_responses: u64,
    /// Total errors occurred
    pub total_errors: u64,
    /// Total latency accumulated (milliseconds)
    pub total_latency_ms: u64,
    /// Average latency per request (milliseconds)
    pub avg_latency_ms: f32,
    /// Minimum latency observed (milliseconds)
    pub min_latency_ms: u64,
    /// Maximum latency observed (milliseconds)
    pub max_latency_ms: u64,
    /// Error rate (0.0-1.0)
    pub error_rate: f32,
    /// Injections detected
    pub injections_detected: u64,
    /// Benign requests
    pub benign_requests: u64,
}

/// API metrics collector
pub struct ApiMetrics {
    total_requests: Arc<AtomicU64>,
    total_responses: Arc<AtomicU64>,
    total_errors: Arc<AtomicU64>,
    total_latency_ms: Arc<AtomicU64>,
    min_latency_ms: Arc<AtomicU64>,
    max_latency_ms: Arc<AtomicU64>,
    injections_detected: Arc<AtomicU64>,
    benign_requests: Arc<AtomicU64>,
}

impl ApiMetrics {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            total_responses: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
            total_latency_ms: Arc::new(AtomicU64::new(0)),
            min_latency_ms: Arc::new(AtomicU64::new(u64::MAX)),
            max_latency_ms: Arc::new(AtomicU64::new(0)),
            injections_detected: Arc::new(AtomicU64::new(0)),
            benign_requests: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Record a request
    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a response
    pub fn record_response(&self, latency_ms: u64, is_injection: bool) {
        self.total_responses.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);

        // Update min/max latency
        let min = self.min_latency_ms.load(Ordering::Relaxed);
        if latency_ms < min {
            let _ = self.min_latency_ms.compare_exchange_weak(
                min,
                latency_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }

        let max = self.max_latency_ms.load(Ordering::Relaxed);
        if latency_ms > max {
            let _ = self.max_latency_ms.compare_exchange_weak(
                max,
                latency_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }

        // Update detection stats
        if is_injection {
            self.injections_detected.fetch_add(1, Ordering::Relaxed);
        } else {
            self.benign_requests.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record an error
    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let total_responses = self.total_responses.load(Ordering::Relaxed);
        let total_errors = self.total_errors.load(Ordering::Relaxed);
        let total_latency_ms = self.total_latency_ms.load(Ordering::Relaxed);
        let min_latency_ms = {
            let min = self.min_latency_ms.load(Ordering::Relaxed);
            if min == u64::MAX {
                0
            } else {
                min
            }
        };
        let max_latency_ms = self.max_latency_ms.load(Ordering::Relaxed);
        let injections_detected = self.injections_detected.load(Ordering::Relaxed);
        let benign_requests = self.benign_requests.load(Ordering::Relaxed);

        let avg_latency_ms = if total_responses > 0 {
            total_latency_ms as f32 / total_responses as f32
        } else {
            0.0
        };

        let error_rate = if total_requests > 0 {
            total_errors as f32 / total_requests as f32
        } else {
            0.0
        };

        MetricsSnapshot {
            total_requests,
            total_responses,
            total_errors,
            total_latency_ms,
            avg_latency_ms,
            min_latency_ms,
            max_latency_ms,
            error_rate,
            injections_detected,
            benign_requests,
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.total_responses.store(0, Ordering::Relaxed);
        self.total_errors.store(0, Ordering::Relaxed);
        self.total_latency_ms.store(0, Ordering::Relaxed);
        self.min_latency_ms.store(u64::MAX, Ordering::Relaxed);
        self.max_latency_ms.store(0, Ordering::Relaxed);
        self.injections_detected.store(0, Ordering::Relaxed);
        self.benign_requests.store(0, Ordering::Relaxed);
    }
}

impl Default for ApiMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = ApiMetrics::new();
        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.total_requests, 0);
        assert_eq!(snapshot.total_responses, 0);
        assert_eq!(snapshot.error_rate, 0.0);
    }

    #[test]
    fn test_record_response() {
        let metrics = ApiMetrics::new();

        metrics.record_request();
        metrics.record_response(100, true);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_responses, 1);
        assert_eq!(snapshot.total_latency_ms, 100);
        assert_eq!(snapshot.avg_latency_ms, 100.0);
        assert_eq!(snapshot.injections_detected, 1);
    }

    #[test]
    fn test_min_max_latency() {
        let metrics = ApiMetrics::new();

        metrics.record_request();
        metrics.record_response(10, false);
        metrics.record_response(100, false);
        metrics.record_response(50, false);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.min_latency_ms, 10);
        assert_eq!(snapshot.max_latency_ms, 100);
    }

    #[test]
    fn test_error_rate() {
        let metrics = ApiMetrics::new();

        metrics.record_request();
        metrics.record_response(10, false);
        metrics.record_request();
        metrics.record_error();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 2);
        assert_eq!(snapshot.total_errors, 1);
        assert_eq!(snapshot.error_rate, 0.5);
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = ApiMetrics::new();

        metrics.record_request();
        metrics.record_response(100, true);

        metrics.reset();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 0);
        assert_eq!(snapshot.total_responses, 0);
    }
}
