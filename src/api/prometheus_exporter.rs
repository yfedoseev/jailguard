//! Prometheus metrics exporter for JailGuard API.

use super::metrics::ApiMetrics;

/// Prometheus metrics formatter
pub struct PrometheusExporter {
    metrics: ApiMetrics,
}

impl PrometheusExporter {
    /// Create new Prometheus exporter
    pub fn new(metrics: ApiMetrics) -> Self {
        Self { metrics }
    }

    /// Generate Prometheus text format metrics
    pub fn export(&self) -> String {
        let snapshot = self.metrics.snapshot();

        let mut output = String::new();

        // API Request Metrics
        output.push_str("# HELP jailguard_api_requests_total Total number of API requests\n");
        output.push_str("# TYPE jailguard_api_requests_total counter\n");
        output.push_str(&format!(
            "jailguard_api_requests_total {}\n",
            snapshot.total_requests
        ));

        output.push_str("# HELP jailguard_api_responses_total Total number of API responses\n");
        output.push_str("# TYPE jailguard_api_responses_total counter\n");
        output.push_str(&format!(
            "jailguard_api_responses_total {}\n",
            snapshot.total_responses
        ));

        output.push_str("# HELP jailguard_api_errors_total Total number of API errors\n");
        output.push_str("# TYPE jailguard_api_errors_total counter\n");
        output.push_str(&format!(
            "jailguard_api_errors_total {}\n",
            snapshot.total_errors
        ));

        // Latency Metrics
        output.push_str(
            "# HELP jailguard_api_latency_ms_total Total accumulated latency in milliseconds\n",
        );
        output.push_str("# TYPE jailguard_api_latency_ms_total counter\n");
        output.push_str(&format!(
            "jailguard_api_latency_ms_total {}\n",
            snapshot.total_latency_ms
        ));

        output.push_str("# HELP jailguard_api_latency_ms Average latency in milliseconds\n");
        output.push_str("# TYPE jailguard_api_latency_ms gauge\n");
        output.push_str(&format!(
            "jailguard_api_latency_ms {:.2}\n",
            snapshot.avg_latency_ms
        ));

        output.push_str("# HELP jailguard_api_latency_min_ms Minimum latency in milliseconds\n");
        output.push_str("# TYPE jailguard_api_latency_min_ms gauge\n");
        output.push_str(&format!(
            "jailguard_api_latency_min_ms {}\n",
            snapshot.min_latency_ms
        ));

        output.push_str("# HELP jailguard_api_latency_max_ms Maximum latency in milliseconds\n");
        output.push_str("# TYPE jailguard_api_latency_max_ms gauge\n");
        output.push_str(&format!(
            "jailguard_api_latency_max_ms {}\n",
            snapshot.max_latency_ms
        ));

        // Error Rate
        output.push_str("# HELP jailguard_api_error_rate Error rate (0.0-1.0)\n");
        output.push_str("# TYPE jailguard_api_error_rate gauge\n");
        output.push_str(&format!(
            "jailguard_api_error_rate {:.4}\n",
            snapshot.error_rate
        ));

        // Detection Metrics
        output.push_str("# HELP jailguard_detections_injections_total Total injections detected\n");
        output.push_str("# TYPE jailguard_detections_injections_total counter\n");
        output.push_str(&format!(
            "jailguard_detections_injections_total {}\n",
            snapshot.injections_detected
        ));

        output.push_str("# HELP jailguard_detections_benign_total Total benign requests\n");
        output.push_str("# TYPE jailguard_detections_benign_total counter\n");
        output.push_str(&format!(
            "jailguard_detections_benign_total {}\n",
            snapshot.benign_requests
        ));

        // Detection Rate
        let total_detections = snapshot.injections_detected + snapshot.benign_requests;
        let injection_rate = if total_detections > 0 {
            snapshot.injections_detected as f32 / total_detections as f32
        } else {
            0.0
        };

        output.push_str("# HELP jailguard_detections_injection_rate Percentage of requests detected as injections\n");
        output.push_str("# TYPE jailguard_detections_injection_rate gauge\n");
        output.push_str(&format!(
            "jailguard_detections_injection_rate {:.4}\n",
            injection_rate
        ));

        // System Metrics
        output.push_str("# HELP jailguard_info JailGuard system information\n");
        output.push_str("# TYPE jailguard_info gauge\n");
        output.push_str("jailguard_info{version=\"1.0.0\",detector=\"real_detector\"} 1\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_export() {
        let metrics = ApiMetrics::new();
        metrics.record_request();
        metrics.record_response(100, true);

        let exporter = PrometheusExporter::new(metrics);
        let output = exporter.export();

        // Verify key metrics are present
        assert!(output.contains("jailguard_api_requests_total 1"));
        assert!(output.contains("jailguard_api_responses_total 1"));
        assert!(output.contains("jailguard_detections_injections_total 1"));
        assert!(output.contains("jailguard_api_latency_ms 100"));
    }

    #[test]
    fn test_prometheus_format() {
        let metrics = ApiMetrics::new();
        let exporter = PrometheusExporter::new(metrics);
        let output = exporter.export();

        // Verify Prometheus format compliance
        assert!(output.contains("# HELP"));
        assert!(output.contains("# TYPE"));
        for line in output.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Each metric line should have format: metric_name metric_value
            let parts: Vec<&str> = line.split_whitespace().collect();
            assert!(parts.len() >= 2, "Invalid metric format: {}", line);
        }
    }
}
