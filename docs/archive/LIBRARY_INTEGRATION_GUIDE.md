# JailGuard Library Integration Guide
## Production Documentation for Application Developers

**Version**: 1.0.0
**Status**: Production Ready
**Last Updated**: January 17, 2026

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Installation](#installation)
3. [Basic Usage](#basic-usage)
4. [Configuration](#configuration)
5. [Advanced Features](#advanced-features)
6. [Performance Tuning](#performance-tuning)
7. [Monitoring & Metrics](#monitoring--metrics)
8. [Error Handling](#error-handling)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)
11. [API Reference](#api-reference)

---

## Quick Start

### 1. Add JailGuard to Your Project

```toml
[dependencies]
jailguard = "1.0.0"
```

### 2. Basic Detection

```rust
use jailguard::{JailGuard, RequestContext};

fn main() {
    // Create JailGuard with default configuration
    let mut jailguard = JailGuard::new();

    // Create request context
    let ctx = RequestContext::new("req-001".to_string());

    // Check input
    let result = jailguard.check_input("user input here", &ctx);

    if result.allowed {
        println!("Input is safe ✓");
    } else {
        println!("Blocked: {}", result.reason.unwrap_or_default());
    }
}
```

### 3. Ensemble Detection (96-98% Accuracy)

```rust
use jailguard::JailGuardConfig;

let config = JailGuardConfig::with_ensemble();
let mut jailguard = JailGuard::with_config(config);

let result = jailguard.check_input("potentially malicious input", &ctx);
```

---

## Installation

### From crates.io

```toml
[dependencies]
jailguard = "1.0"
```

### From Git

```toml
[dependencies]
jailguard = { git = "https://github.com/your-org/jailguard", branch = "main" }
```

### Features

```toml
[dependencies]
jailguard = { version = "1.0", features = ["metrics", "profiling"] }
```

---

## Basic Usage

### Single-Model Detection (78.9% Accuracy)

```rust
use jailguard::{JailGuard, RequestContext, JailGuardConfig};

let config = JailGuardConfig {
    enable_ensemble: false,
    ..Default::default()
};

let mut jg = JailGuard::with_config(config);
let ctx = RequestContext::new("req-001".to_string());
let result = jg.check_input("What is 2+2?", &ctx);

match result.allowed {
    true => println!("Safe ✓"),
    false => println!("Blocked: {}", result.reason.unwrap()),
}
```

### Ensemble Detection (96-98% Accuracy)

```rust
use jailguard::JailGuardConfig;

// Option 1: Simple enablement
let config = JailGuardConfig::with_ensemble();
let mut jg = JailGuard::with_config(config);

// Option 2: Custom ensemble configuration
let config = JailGuardConfig::default()
    .set_ensemble_config(EnsembleConfig {
        jailguard_weight: 0.60,
        gentelshed_weight: 0.25,
        protect_ai_weight: 0.15,
        ..Default::default()
    });
let mut jg = JailGuard::with_config(config);
```

### Checking Output

```rust
let output_result = jg.check_output("LLM response here");

if output_result.is_safe {
    println!("Output is clean");
} else {
    println!("Violations found: {}", output_result.violation_count);
    println!("Sanitized: {}", output_result.sanitized_output);
}
```

### Session Tracking

```rust
let stats = jg.session_stats();

println!("Session: {}", jg.session_id());
println!("Total requests: {}", stats.total_requests);
println!("Injection attempts: {}", stats.injection_attempts);
println!("Injection rate: {:.1}%", stats.injection_rate * 100.0);
println!("Anomaly score: {:.3}", stats.anomaly_score);
```

---

## Configuration

### Default Configuration

```rust
JailGuardConfig {
    enable_spotlighting: true,        // Input boundary marking
    enable_detection: true,            // Multi-task detection
    enable_ensemble: false,            // Opt-in ensemble (disabled by default)
    ensemble_config: None,             // Ensemble configuration
    enable_task_tracking: true,        // Behavioral drift detection
    enable_privilege_context: true,    // Resource access control
    enable_output_validation: true,    // Secret detection
    enable_monitoring: true,           // Anomaly detection
    block_threshold: 0.7,              // Confidence threshold
    strict_mode: false,                // Fail if any layer detects threat
}
```

### Custom Configuration

```rust
use jailguard::{JailGuardConfig, EnsembleConfig};

let config = JailGuardConfig {
    enable_spotlighting: true,
    enable_detection: true,
    enable_ensemble: true,
    ensemble_config: Some(EnsembleConfig {
        jailguard_weight: 0.60,
        gentelshed_weight: 0.25,
        protect_ai_weight: 0.15,
        injection_threshold: 0.5,
        use_weighted_voting: true,
        agreement_threshold: 0.66,
    }),
    enable_task_tracking: true,
    enable_privilege_context: true,
    enable_output_validation: true,
    enable_monitoring: true,
    block_threshold: 0.8,
    strict_mode: true,
};

let mut jg = JailGuard::with_config(config);
```

### Layer-by-Layer Control

```rust
// Minimal configuration (single detection only)
let minimal = JailGuardConfig {
    enable_spotlighting: false,
    enable_detection: true,
    enable_task_tracking: false,
    enable_privilege_context: false,
    enable_output_validation: false,
    enable_monitoring: false,
    ..Default::default()
};

// Maximum security (all layers enabled)
let maximum = JailGuardConfig {
    enable_spotlighting: true,
    enable_detection: true,
    enable_ensemble: true,
    enable_task_tracking: true,
    enable_privilege_context: true,
    enable_output_validation: true,
    enable_monitoring: true,
    strict_mode: true,
    ..Default::default()
};
```

---

## Advanced Features

### 1. Performance Profiling

```rust
use jailguard::{EnsembleProfiler, performance::profiler::Timer};

let mut profiler = EnsembleProfiler::new(10000);

// Profile individual detections
let timer = Timer::start();
let result = jg.check_input(text, &ctx);
let latency_us = timer.elapsed_us();

// Analyze results
profiler.print_summary();
println!("Avg latency: {} µs", profiler.avg_total_us());
println!("P99 latency: {} µs", profiler.p99_total_us());
```

### 2. Response Caching

```rust
use jailguard::ResponseCache;

let mut cache = ResponseCache::with_config(1000, 300); // 1k items, 5 min TTL

// Check cache first
if let Some(cached) = cache.get(user_input) {
    return cached;
}

// Get detection
let result = detector.detect(user_input);

// Store in cache
cache.put(user_input.to_string(), result.clone());

// Check efficiency
let stats = cache.stats();
println!("Cache hit rate: {:.1}%", stats.hit_rate * 100.0);
```

### 3. Metrics Collection

```rust
use jailguard::PerformanceMetrics;

let mut metrics = PerformanceMetrics::new(10000);

// Collect metrics from ensemble detections
for result in ensemble_results {
    metrics.record(result);
}

// Analyze
metrics.print_summary();
let summary = metrics.summary();
println!("High agreement rate: {:.1}%", summary.high_agreement_rate * 100.0);
```

### 4. Custom Request Context

```rust
let ctx = RequestContext::new("req-123".to_string())
    .with_task("Answer technical questions".to_string())
    .with_user("user-456".to_string());

let result = jg.check_input(text, &ctx);
```

---

## Performance Tuning

### Latency Optimization

| Optimization | Impact | Trade-off |
|---|---|---|
| Disable non-essential layers | -30% | Reduced detection depth |
| Enable caching | -90% (on hits) | Memory usage |
| Single model vs ensemble | -50% | Accuracy (-17-19%) |
| P95 vs average latency | Varies | SLO strictness |

### Memory Optimization

```rust
// Smaller configurations
let cache = ResponseCache::with_config(100, 60);  // 100 items, 1 min
let metrics = PerformanceMetrics::new(1000);      // 1k samples
let profiler = EnsembleProfiler::new(1000);       // 1k profiles
```

### Throughput Optimization

```rust
// Batch processing recommendations
const BATCH_SIZE: usize = 100;
for batch in inputs.chunks(BATCH_SIZE) {
    // Process batch with shared JailGuard instance
    for input in batch {
        jg.check_input(input, &ctx);
    }
}
```

---

## Monitoring & Metrics

### Exporting Metrics

```rust
// Integration with monitoring systems (example)
fn export_metrics(metrics: &PerformanceMetrics) {
    let summary = metrics.summary();

    // Prometheus format
    println!("jailguard_agreement_score {}", summary.avg_agreement);
    println!("jailguard_injection_rate {}", summary.injection_rate);
    println!("jailguard_samples_total {}", summary.sample_count);
}
```

### Health Checks

```rust
fn health_check(metrics: &PerformanceMetrics) -> bool {
    let summary = metrics.summary();

    // Check ensemble health
    summary.high_agreement_rate > 0.90  // >90% high agreement
    && summary.low_variance_rate > 0.85 // >85% low variance
    && summary.avg_confidence > 0.5     // Average confidence > 50%
}
```

### Performance SLOs

```rust
use jailguard::EnsembleProfiler;

fn check_latency_slos(profiler: &EnsembleProfiler) {
    let avg = profiler.avg_total_us();
    let p99 = profiler.p99_total_us();

    assert!(avg < 500, "Average latency SLO: {} µs", avg);
    assert!(p99 < 2000, "P99 latency SLO: {} µs", p99);
}
```

---

## Error Handling

### Basic Error Handling

```rust
use jailguard::RequestContext;

let ctx = RequestContext::new("req-001".to_string());
let result = jg.check_input(user_input, &ctx);

if !result.allowed {
    // Log the block event
    eprintln!("Blocked input: {:?}", result.reason);

    // Return error to user
    return Err("Input validation failed");
}

// Continue processing
Ok(result.detection.unwrap().confidence)
```

### Graceful Degradation

```rust
// Ensemble falls back to single model if external APIs unavailable
let config = JailGuardConfig::with_ensemble();
let mut jg = JailGuard::with_config(config);

// If GenTel-Shield API fails, ensemble still works with other models
let result = jg.check_input(text, &ctx); // Won't panic
```

---

## Best Practices

### 1. Session Management

```rust
// Create new session for each user/conversation
let mut jg = JailGuard::new(); // New session ID generated

// Process multiple requests in same session
for msg in user_messages {
    let ctx = RequestContext::new(format!("msg-{}", msg.id));
    jg.check_input(&msg.content, &ctx);
}

// Get session statistics
let stats = jg.session_stats();
if stats.injection_rate > 0.1 {  // >10% injection rate
    alert!("Suspicious session activity");
}
```

### 2. Ensemble Deployment

```rust
// Start with single model, upgrade to ensemble
let config = if use_high_accuracy {
    JailGuardConfig::with_ensemble()  // 96-98% accuracy
} else {
    JailGuardConfig::default()         // 78.9% accuracy
};

let mut jg = JailGuard::with_config(config);
```

### 3. Caching Strategy

```rust
// Cache TTL should match your use case
let cache = match environment {
    "development" => ResponseCache::with_config(100, 60),  // 1 min cache
    "staging" => ResponseCache::with_config(1000, 300),    // 5 min cache
    "production" => ResponseCache::with_config(10000, 1800), // 30 min cache
    _ => ResponseCache::new(),
};
```

### 4. Monitoring

```rust
// Collect metrics regularly
let mut metrics = PerformanceMetrics::new(10000);

for request in incoming_requests {
    let result = jg.check_input(&request, &ctx);

    if let Some(ensemble_result) = ensemble_results {
        metrics.record(ensemble_result);
    }
}

// Export metrics hourly
if should_export_metrics {
    metrics.print_summary();
    export_to_monitoring_system(&metrics);
}
```

---

## Troubleshooting

### Issue: High False Positive Rate

**Symptom**: Legitimate inputs are being blocked

**Solution**:
1. Lower `block_threshold` (default: 0.7 → try 0.6)
2. Disable non-essential layers
3. Use single model instead of ensemble (accuracy vs speed)

```rust
let config = JailGuardConfig {
    block_threshold: 0.6,  // Lower threshold
    enable_task_tracking: false,
    enable_privilege_context: false,
    ..Default::default()
};
```

### Issue: Slow Performance

**Symptom**: Latency exceeds SLO

**Solution**:
1. Enable caching
2. Disable ensemble (drop to single model)
3. Profile to find bottleneck

```rust
let cache = ResponseCache::new();
let config = JailGuardConfig {
    enable_ensemble: false,  // Reduce to single model
    ..Default::default()
};
```

### Issue: Memory Usage Growing

**Symptom**: Cache/metrics consuming too much memory

**Solution**:
1. Reduce cache size / TTL
2. Reduce metrics window size
3. Clear periodically

```rust
let cache = ResponseCache::with_config(500, 60);  // Smaller
let metrics = PerformanceMetrics::new(1000);      // Smaller

// Clear periodically
cache.clear();
metrics.clear();
```

### Issue: Inconsistent Results

**Symptom**: Same input produces different results

**Solution**:
1. Check ensemble agreement score
2. Monitor for model drift
3. Verify environment variables for external APIs

```rust
if ensemble_result.agreement_score < 0.8 {
    warn!("Low agreement on detection: {}", text);
}
```

---

## API Reference

### JailGuard

Main unified defense system.

```rust
pub struct JailGuard {
    // Complete 6-layer defense system
}

impl JailGuard {
    pub fn new() -> Self;
    pub fn with_config(config: JailGuardConfig) -> Self;
    pub fn session_id(&self) -> &str;
    pub fn check_input(&mut self, text: &str, context: &RequestContext)
        -> InputValidationResult;
    pub fn check_output(&self, output: &str) -> OutputCheckResult;
    pub fn session_stats(&self) -> SessionStats;
    pub fn reset_session(&mut self);
    pub fn session_tracker(&self) -> &SessionTracker;
}
```

### RequestContext

Context for request processing.

```rust
pub struct RequestContext {
    pub request_id: String;
    pub task_description: Option<String>;
    pub user_id: Option<String>;
}

impl RequestContext {
    pub fn new(request_id: String) -> Self;
    pub fn with_task(self, task: String) -> Self;
    pub fn with_user(self, user_id: String) -> Self;
}
```

### InputValidationResult

Result from input validation.

```rust
pub struct InputValidationResult {
    pub allowed: bool;
    pub detection: Option<DetectionResult>;
    pub privilege_result: Option<String>;
    pub anomaly_score: f32;
    pub session_id: String;
    pub reason: Option<String>;
}
```

### JailGuardConfig

Configuration for unified system.

```rust
pub struct JailGuardConfig {
    pub enable_spotlighting: bool;
    pub enable_detection: bool;
    pub enable_ensemble: bool;
    pub ensemble_config: Option<EnsembleConfig>;
    pub enable_task_tracking: bool;
    pub enable_privilege_context: bool;
    pub enable_output_validation: bool;
    pub enable_monitoring: bool;
    pub block_threshold: f32;
    pub strict_mode: bool;
}

impl JailGuardConfig {
    pub fn with_ensemble() -> Self;
    pub fn set_ensemble_config(self, config: EnsembleConfig) -> Self;
}
```

---

## Examples

See `/examples/` directory for complete working examples:

- `unified_api_ensemble_demo.rs` - Ensemble usage patterns
- `performance_optimization_demo.rs` - Profiling and metrics

---

## Support & Feedback

For issues, feature requests, or questions:

1. Check [Troubleshooting](#troubleshooting) section
2. Review examples in `/examples/`
3. Check inline documentation: `cargo doc --open`
4. File issues on GitHub with reproduction steps

---

**Last Updated**: January 17, 2026
**Version**: 1.0.0
**Status**: Production Ready
