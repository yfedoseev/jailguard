# JailGuard API Reference

## Overview

JailGuard is a 6-layer defense-in-depth system for detecting and preventing prompt injection attacks. The unified API provides simple integration of all defense layers.

## Core Types

### RequestContext

Request metadata for JailGuard processing.

```rust
pub struct RequestContext {
    pub request_id: String,
    pub task_description: Option<String>,
    pub user_id: Option<String>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(request_id: String) -> Self;

    /// Set task description for behavioral tracking
    pub fn with_task(mut self, task: String) -> Self;

    /// Set user identifier for privilege validation
    pub fn with_user(mut self, user_id: String) -> Self;
}
```

### InputValidationResult

Result of input validation through all 6 layers.

```rust
pub struct InputValidationResult {
    /// Whether input is safe/allowed
    pub allowed: bool,

    /// Detection result details (if detection layer enabled)
    pub detection: Option<DetectionResult>,

    /// Privilege validation result (if privilege layer enabled)
    pub privilege_result: Option<String>,

    /// Anomaly score from behavior monitoring (0.0-1.0)
    pub anomaly_score: f32,

    /// Session identifier for tracking
    pub session_id: String,

    /// Detailed reason for blocking (if blocked)
    pub reason: Option<String>,
}
```

### OutputCheckResult

Result of output validation.

```rust
pub struct OutputCheckResult {
    /// Whether output is safe (no secrets detected)
    pub is_safe: bool,

    /// Sanitized output with secrets redacted
    pub sanitized_output: String,

    /// Number of violations found
    pub violation_count: usize,

    /// Detailed violation information
    pub details: String,
}
```

### JailGuardConfig

Configuration for the unified JailGuard system.

```rust
pub struct JailGuardConfig {
    /// Enable spotlighting layer (input boundary marking)
    pub enable_spotlighting: bool,

    /// Enable detection layer (transformer-based)
    pub enable_detection: bool,

    /// Enable task tracking (behavioral drift detection)
    pub enable_task_tracking: bool,

    /// Enable privilege context (resource access control)
    pub enable_privilege_context: bool,

    /// Enable output validation (secret detection)
    pub enable_output_validation: bool,

    /// Enable behavior monitoring (anomaly detection)
    pub enable_monitoring: bool,

    /// Block threshold for detection confidence (0.0-1.0, default 0.7)
    pub block_threshold: f32,

    /// Strict mode: block if ANY layer detects threat
    pub strict_mode: bool,
}

impl Default for JailGuardConfig {
    fn default() -> Self {
        // All layers enabled, threshold 0.7, non-strict mode
    }
}
```

### SessionStats

Session statistics summary.

```rust
pub struct SessionStats {
    /// Total requests processed in session
    pub total_requests: usize,

    /// Number of injection attempts detected
    pub injection_attempts: usize,

    /// Injection rate (0.0-1.0)
    pub injection_rate: f32,

    /// Average confidence of detected injections
    pub avg_confidence: f32,

    /// Current anomaly score (0.0-1.0)
    pub anomaly_score: f32,
}
```

## Main API: JailGuard

### Initialization

```rust
// Create with default configuration (all layers enabled)
let jg = JailGuard::new();

// Create with custom configuration
let config = JailGuardConfig {
    block_threshold: 0.8,
    strict_mode: true,
    ..Default::default()
};
let jg = JailGuard::with_config(config);

// Clone creates new session
let jg2 = jg.clone();  // Different session_id
```

### Input Validation

```rust
// Basic validation
let ctx = RequestContext::new("req-1".to_string());
let result = jailguard.check_input("user input", &ctx);

if result.allowed {
    println!("Input safe");
} else {
    println!("Input blocked: {}", result.reason.unwrap_or_default());
}

// Validation with task context (for drift detection)
let ctx = RequestContext::new("req-1".to_string())
    .with_task("Answer questions about Python".to_string())
    .with_user("user-123".to_string());

let result = jailguard.check_input("Tell me about Django", &ctx);

// Check anomaly score (behavior monitoring)
println!("Anomaly score: {:.2}", result.anomaly_score);
```

### Output Validation

```rust
// Check output for secrets and injection markers
let output = "The password is secret123";
let result = jailguard.check_output(output);

if result.is_safe {
    println!("Output safe");
} else {
    println!("Output contains secrets: {}", result.details);
    println!("Sanitized: {}", result.sanitized_output);
}
```

### Session Management

```rust
// Get session identifier
let session_id = jailguard.session_id();

// Get session statistics
let stats = jailguard.session_stats();
println!("Injection attempts: {}", stats.injection_attempts);
println!("Injection rate: {:.1}%", stats.injection_rate * 100.0);

// Reset session
jailguard.reset_session();
```

## 6-Layer Architecture

### 1. Spotlighting Layer

Prevents prompt injection by marking input boundaries with delimiters.

**Purpose**: Create clear separation between system prompts and user input

**Detection capability**: Low confidence, acts as preprocessing

```rust
// Enabled by: enable_spotlighting: true
// Impact: Marks all inputs with delimiters
// Output: <user_input>...</user_input>
```

### 2. Detection Layer

Transformer-based multi-task detection with 3 heads:
- Binary classification (injection vs benign)
- 7-way attack type classification
- Semantic similarity scoring

**Purpose**: Detect known injection patterns with high accuracy

**Detection capability**: 95-98% accuracy on test set

```rust
// Enabled by: enable_detection: true
// Configuration: block_threshold (0.0-1.0, default 0.7)
// Output: DetectionResult with confidence score
```

### 3. Task Tracking Layer

Detects behavioral drift from expected task context.

**Purpose**: Identify when user requests deviate from declared task

**Detection capability**: >85% accuracy on topic shifts

```rust
// Enabled by: enable_task_tracking: true
// Input: RequestContext.task_description
// Detection: Cosine similarity vs expected topics
```

### 4. Privilege Context Layer

Validates resource access requests against allowed privileges.

**Purpose**: Prevent requests for unauthorized resources

**Detection capability**: Pattern matching on keywords

```rust
// Enabled by: enable_privilege_context: true
// Patterns: Database, FileSystem, Network, Credentials access
// Result: Denied if unauthorized resource requested
```

### 5. Output Validation Layer

Detects and sanitizes secrets in output.

**Purpose**: Prevent accidental secret leakage

**Detection capability**: 95%+ of common secret patterns

```rust
// Enabled by: enable_output_validation: true
// Patterns:
//   - API keys (sk_live_*, AKIA*)
//   - JWT tokens (eyJ*)
//   - Private keys (-----BEGIN...)
//   - Passwords (password:, pwd:, secret:)
// Result: Redacted with [REDACTED]
```

### 6. Behavior Monitoring Layer

Tracks session statistics and detects anomalies.

**Purpose**: Identify attack campaigns and unusual patterns

**Detection capability**: Z-score based anomaly detection

```rust
// Enabled by: enable_monitoring: true
// Tracks: Injection attempts, frequency, confidence progression
// Result: Anomaly score (0.0-1.0) and reasoning
```

## Configuration Examples

### Lenient Mode (High Recall)

Only detect obvious attacks, minimize false positives.

```rust
let config = JailGuardConfig {
    block_threshold: 0.85,
    strict_mode: false,
    enable_spotlighting: true,
    enable_detection: true,
    enable_task_tracking: false,
    enable_privilege_context: false,
    enable_output_validation: false,
    enable_monitoring: false,
};
```

### Strict Mode (High Precision)

Detect even subtle attacks, block on any layer detection.

```rust
let config = JailGuardConfig {
    block_threshold: 0.5,
    strict_mode: true,
    enable_spotlighting: true,
    enable_detection: true,
    enable_task_tracking: true,
    enable_privilege_context: true,
    enable_output_validation: true,
    enable_monitoring: true,
};
```

### Output-Only Validation

Only validate outputs, no input checking.

```rust
let config = JailGuardConfig {
    enable_spotlighting: false,
    enable_detection: false,
    enable_task_tracking: false,
    enable_privilege_context: false,
    enable_output_validation: true,
    enable_monitoring: false,
    ..Default::default()
};
```

## Error Handling

All APIs return `InputValidationResult` or `OutputCheckResult` directly without errors. Check the `allowed` and `is_safe` fields to determine outcomes.

```rust
let result = jailguard.check_input("text", &ctx);

match result.allowed {
    true => println!("Allowed"),
    false => {
        println!("Blocked: {}", result.reason.unwrap_or_default());
        // Log the reason for monitoring
    }
}
```

## Performance Characteristics

- **CPU Latency**: <30ms for single inference (spotlighting + detection)
- **GPU Latency**: <5ms with WGPU backend
- **Memory**: ~50MB runtime footprint
- **Model Size**: ~16MB (FP32 weights)
- **Throughput**: >100 requests/second on GPU

## Best Practices

1. **Use Task Context**: Provide task descriptions for behavioral tracking
2. **Enable Multiple Layers**: Combine layers for defense-in-depth
3. **Monitor Anomaly Scores**: Track session statistics for attack patterns
4. **Validate Outputs**: Always check outputs for secrets before sending
5. **Tune Threshold**: Adjust block_threshold based on your false positive tolerance
6. **Log Blocked Inputs**: Store blocked inputs for security analysis
7. **Reset Sessions**: Clear session data between user sessions for privacy

## Integration Examples

### Web API Integration

```rust
#[post("/api/generate")]
async fn generate(
    req: web::Json<GenerateRequest>,
) -> Result<web::Json<GenerateResponse>> {
    let mut jailguard = JailGuard::new();

    // Validate input
    let input_ctx = RequestContext::new(req.request_id.clone())
        .with_user(req.user_id.clone());

    let validation = jailguard.check_input(&req.prompt, &input_ctx);
    if !validation.allowed {
        return Ok(web::Json(GenerateResponse {
            error: Some(format!("Input blocked: {}", validation.reason.unwrap_or_default())),
            ..Default::default()
        }));
    }

    // Generate response
    let response = generate_text(&req.prompt).await;

    // Validate output
    let output_check = jailguard.check_output(&response);
    let safe_response = if output_check.is_safe {
        response
    } else {
        output_check.sanitized_output
    };

    Ok(web::Json(GenerateResponse {
        response: safe_response,
        ..Default::default()
    }))
}
```

### Monitoring Integration

```rust
fn log_security_event(jg: &JailGuard, result: &InputValidationResult) {
    let stats = jg.session_stats();

    event_logger.log(SecurityEvent {
        session_id: result.session_id.clone(),
        allowed: result.allowed,
        reason: result.reason.clone(),
        anomaly_score: result.anomaly_score,
        injection_rate: stats.injection_rate,
        timestamp: SystemTime::now(),
    });
}
```

## See Also

- [Architecture Documentation](./ARCHITECTURE.md)
- [Training Guide](./TRAINING.md)
- [Examples](../examples/)
