# JailGuard Configuration Guide
## Best Practices & Real-World Scenarios

**Version**: 1.0.0
**Status**: Production Ready

---

## Table of Contents

1. [Configuration Strategies](#configuration-strategies)
2. [Use Case Scenarios](#use-case-scenarios)
3. [Environment-Based Configuration](#environment-based-configuration)
4. [Performance vs Security Trade-offs](#performance-vs-security-trade-offs)
5. [Ensemble Configuration](#ensemble-configuration)
6. [Advanced Configuration](#advanced-configuration)
7. [Configuration Validation](#configuration-validation)

---

## Configuration Strategies

### Strategy 1: Default Configuration (Recommended for Most)

**Best for**: General-purpose applications, balanced security/performance

```rust
let mut jg = JailGuard::new();  // All defaults enabled except ensemble
```

**What you get**:
- ✅ All 6 defense layers active
- ✅ Good accuracy (78.9%)
- ✅ Fast latency (0.48ms)
- ✅ Minimal configuration needed

### Strategy 2: High-Accuracy (Enterprise)

**Best for**: High-security requirements, accuracy critical

```rust
let config = JailGuardConfig::with_ensemble();
let mut jg = JailGuard::with_config(config);
```

**What you get**:
- ✅ 96-98% accuracy (ensemble voting)
- ✅ Multiple models agree
- ⚠️ Slightly higher latency (~0.5ms)
- ✅ Better for critical systems

### Strategy 3: Performance-Optimized (High Volume)

**Best for**: High-traffic applications, latency-critical

```rust
let config = JailGuardConfig {
    enable_spotlighting: true,
    enable_detection: true,
    enable_ensemble: false,       // Single model only
    enable_task_tracking: false,  // Disable drift detection
    enable_privilege_context: false,  // Disable resource checking
    enable_output_validation: true,   // Keep secret detection
    enable_monitoring: false,      // Disable anomaly detection
    block_threshold: 0.7,
    strict_mode: false,
};
let mut jg = JailGuard::with_config(config);
```

**What you get**:
- ✅ Minimal latency
- ✅ Reduce CPU usage
- ✅ Lower memory footprint
- ⚠️ Fewer defense layers

### Strategy 4: Strict Security Mode

**Best for**: Security-critical environments, LLM systems

```rust
let config = JailGuardConfig {
    enable_spotlighting: true,
    enable_detection: true,
    enable_ensemble: true,
    ensemble_config: Some(EnsembleConfig::default()),
    enable_task_tracking: true,
    enable_privilege_context: true,
    enable_output_validation: true,
    enable_monitoring: true,
    block_threshold: 0.7,
    strict_mode: true,  // Fail if ANY layer detects threat
};
let mut jg = JailGuard::with_config(config);
```

**What you get**:
- ✅ Maximum security
- ✅ All layers active
- ✅ Any threat blocks request
- ⚠️ Highest latency
- ⚠️ Highest false positive potential

---

## Use Case Scenarios

### Scenario 1: Customer Support Chatbot

**Requirements**: Balance between security and user experience

```rust
let config = JailGuardConfig {
    enable_spotlighting: true,
    enable_detection: true,
    enable_ensemble: false,        // Single model sufficient
    enable_task_tracking: true,    // Detect off-topic
    enable_privilege_context: true, // Prevent credential requests
    enable_output_validation: true,
    enable_monitoring: true,
    block_threshold: 0.7,          // Standard threshold
    strict_mode: false,            // Allow minor detections
};

// Caching helps with repeated questions
let mut cache = ResponseCache::new();
let mut jg = JailGuard::with_config(config);
```

**Deployment**:
- Cache TTL: 5-10 minutes
- Monitoring: Track injection attempts
- Action: Log and rate-limit suspicious users

### Scenario 2: Document Processing API

**Requirements**: High volume, automated processing

```rust
let config = JailGuardConfig {
    enable_spotlighting: true,
    enable_detection: true,
    enable_ensemble: false,        // Performance critical
    enable_task_tracking: false,   // N/A for documents
    enable_privilege_context: false,  // N/A for documents
    enable_output_validation: true, // Check output for injection
    enable_monitoring: false,       // Volume too high
    block_threshold: 0.75,         // Stricter to reduce FP
    strict_mode: false,
};

let mut cache = ResponseCache::with_config(5000, 1800); // Large cache
```

**Deployment**:
- Cache: Critical for performance
- Profiling: Monitor latency
- Batching: Process documents in groups

### Scenario 3: Code Generation Service

**Requirements**: High security, creative prompts

```rust
let config = JailGuardConfig::with_ensemble();  // Maximum accuracy

let mut jg = JailGuard::with_config(config);
```

**Why Ensemble?**:
- Complex prompt injection techniques
- Users may craft sophisticated attacks
- Security > performance trade-off
- 96-98% accuracy critical

**Deployment**:
- Ensemble enabled for high accuracy
- Strict mode: block on any detection
- Metrics: Monitor agreement scores
- Action: Log all blocked attempts

### Scenario 4: Real-Time Moderation System

**Requirements**: Ultra-low latency, high accuracy

```rust
let config = JailGuardConfig {
    enable_spotlighting: true,
    enable_detection: true,
    enable_ensemble: false,         // Trade accuracy for speed
    enable_task_tracking: false,
    enable_privilege_context: true, // Still check resources
    enable_output_validation: true,
    enable_monitoring: false,       // Volume too high
    block_threshold: 0.65,          // Lower to catch more
    strict_mode: false,
};

let mut cache = ResponseCache::with_config(10000, 300); // Aggressive caching
```

**Optimization**:
- Large cache for repeated content
- Small profiler window (1k samples)
- No metrics collection (overhead)
- Batch processing where possible

---

## Environment-Based Configuration

### Development Environment

```rust
let config = if cfg!(debug_assertions) {
    JailGuardConfig {
        enable_spotlighting: true,
        enable_detection: true,
        enable_ensemble: false,  // Skip ensemble overhead
        enable_task_tracking: false,
        enable_privilege_context: false,
        enable_output_validation: true,
        enable_monitoring: false,
        block_threshold: 0.5,    // Lower threshold to catch issues
        strict_mode: false,
    }
} else {
    // Production config
    JailGuardConfig::with_ensemble()
};
```

### Staging Environment

```rust
let config = match std::env::var("ENVIRONMENT").as_deref() {
    Ok("staging") => {
        JailGuardConfig {
            enable_ensemble: true,
            block_threshold: 0.7,
            enable_monitoring: true,
            ..Default::default()
        }
    }
    _ => JailGuardConfig::default(),
};
```

### Production Environment

```rust
// Use external configuration
let ensemble_enabled = std::env::var("ENABLE_ENSEMBLE")
    .map(|v| v == "true")
    .unwrap_or(false);

let block_threshold: f32 = std::env::var("BLOCK_THRESHOLD")
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(0.7);

let config = JailGuardConfig {
    enable_ensemble: ensemble_enabled,
    block_threshold,
    ..Default::default()
};
```

---

## Performance vs Security Trade-offs

### Accuracy vs Latency

| Config | Accuracy | Latency | Best For |
|--------|----------|---------|----------|
| Single Model | 78.9% | 0.48ms | High volume |
| Ensemble | 96-98% | 0.5ms | High security |
| Minimal | 78.9% | 0.3ms | Ultra-low latency |
| Full Layers | 78.9% | 1-2ms | Maximum security |

### Memory vs Cache Size

```rust
// Development/Small scale
let cache = ResponseCache::with_config(100, 60);  // ~10MB

// Production/Medium scale
let cache = ResponseCache::with_config(1000, 300); // ~100MB

// Production/Large scale
let cache = ResponseCache::with_config(10000, 1800); // ~1GB
```

### Cost-Benefit Analysis

```rust
// Cost: Extra 0.02ms per request
// Benefit: +17-19% accuracy improvement
// ROI: High - worth the cost for most applications
let config = JailGuardConfig::with_ensemble();

// Cost: Extra memory for cache
// Benefit: 80-90% latency reduction on repeated inputs
// ROI: Very high - essential for production
let cache = ResponseCache::new();
```

---

## Ensemble Configuration

### Default Ensemble

```rust
let config = EnsembleConfig::default();
// JailGuard: 60% weight
// GenTel-Shield: 25% weight
// ProtectAI: 15% weight
// Threshold: 0.5
// Voting: Weighted average
```

### Conservative Ensemble (More JailGuard)

```rust
let config = EnsembleConfig {
    jailguard_weight: 0.70,      // Increased
    gentelshed_weight: 0.20,
    protect_ai_weight: 0.10,
    injection_threshold: 0.5,
    use_weighted_voting: true,
    agreement_threshold: 0.66,
};
// Use when: JailGuard model is well-trained on your domain
```

### Balanced Ensemble

```rust
let config = EnsembleConfig {
    jailguard_weight: 0.60,
    gentelshed_weight: 0.25,
    protect_ai_weight: 0.15,
    injection_threshold: 0.5,
    use_weighted_voting: true,
    agreement_threshold: 0.66,
};
// Use when: Good overall performance desired (default)
```

### Strict Ensemble (Majority Voting)

```rust
let config = EnsembleConfig {
    jailguard_weight: 0.33,
    gentelshed_weight: 0.33,
    protect_ai_weight: 0.34,
    injection_threshold: 0.5,
    use_weighted_voting: false,  // Majority voting
    agreement_threshold: 0.66,
};
// Use when: Want democratic voting, no model dominates
```

### Fine-Tuned Ensemble

```rust
let config = EnsembleConfig {
    jailguard_weight: 0.50,
    gentelshed_weight: 0.35,
    protect_ai_weight: 0.15,
    injection_threshold: 0.45,   // Lower to catch more
    use_weighted_voting: true,
    agreement_threshold: 0.70,    // Higher for confidence
};
// Use when: You've measured performance on your domain
```

---

## Advanced Configuration

### Configuration Template System

```rust
mod config_templates {
    use jailguard::{JailGuardConfig, EnsembleConfig};

    pub fn development() -> JailGuardConfig {
        JailGuardConfig {
            enable_ensemble: false,
            block_threshold: 0.5,
            ..Default::default()
        }
    }

    pub fn staging() -> JailGuardConfig {
        JailGuardConfig {
            enable_ensemble: true,
            block_threshold: 0.7,
            enable_monitoring: true,
            ..Default::default()
        }
    }

    pub fn production() -> JailGuardConfig {
        JailGuardConfig::with_ensemble()
    }

    pub fn high_security() -> JailGuardConfig {
        JailGuardConfig {
            enable_ensemble: true,
            ensemble_config: Some(EnsembleConfig {
                injection_threshold: 0.4,  // Stricter
                ..Default::default()
            }),
            strict_mode: true,
            block_threshold: 0.6,
            ..Default::default()
        }
    }

    pub fn high_throughput() -> JailGuardConfig {
        JailGuardConfig {
            enable_ensemble: false,
            enable_task_tracking: false,
            enable_privilege_context: false,
            enable_monitoring: false,
            block_threshold: 0.75,
            ..Default::default()
        }
    }
}

// Usage
let config = config_templates::production();
let mut jg = JailGuard::with_config(config);
```

### Dynamic Configuration

```rust
pub struct DynamicConfig {
    jailguard: JailGuard,
    ensemble_enabled: bool,
}

impl DynamicConfig {
    pub fn enable_ensemble(&mut self) {
        self.ensemble_enabled = true;
        self.jailguard = JailGuard::with_config(JailGuardConfig::with_ensemble());
    }

    pub fn disable_ensemble(&mut self) {
        self.ensemble_enabled = false;
        self.jailguard = JailGuard::with_config(JailGuardConfig::default());
    }

    pub fn check(&mut self, text: &str, ctx: &RequestContext) {
        self.jailguard.check_input(text, ctx)
    }
}
```

---

## Configuration Validation

### Validate Before Deployment

```rust
fn validate_production_config(config: &JailGuardConfig) -> Result<(), String> {
    // Ensemble weights must be valid
    if let Some(ensemble_cfg) = &config.ensemble_config {
        ensemble_cfg.validate()?;
    }

    // Block threshold must be reasonable
    if config.block_threshold < 0.5 || config.block_threshold > 0.9 {
        return Err("Block threshold out of range (0.5-0.9)".to_string());
    }

    // At least detection must be enabled
    if !config.enable_detection {
        return Err("Detection layer must be enabled".to_string());
    }

    // Strict mode has higher FP rate - warn
    if config.strict_mode && config.enable_ensemble {
        eprintln!("Warning: Strict mode + ensemble may cause high false positives");
    }

    Ok(())
}
```

### Configuration Testing

```rust
#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_development_config() {
        let cfg = config_templates::development();
        assert!(!cfg.enable_ensemble);
        assert_eq!(cfg.block_threshold, 0.5);
    }

    #[test]
    fn test_production_config() {
        let cfg = config_templates::production();
        assert!(cfg.enable_ensemble);
        validate_production_config(&cfg).unwrap();
    }

    #[test]
    fn test_high_security_config() {
        let cfg = config_templates::high_security();
        assert!(cfg.strict_mode);
        assert!(cfg.enable_ensemble);
    }
}
```

---

## Configuration Best Practices

### ✅ DO

1. **Use templates** - Leverage provided templates for common scenarios
2. **Validate configs** - Check before deployment
3. **Environment-specific** - Different configs for dev/staging/prod
4. **Document reasoning** - Why specific settings chosen
5. **Monitor metrics** - Track actual performance
6. **Plan caching** - Essential for performance

### ❌ DON'T

1. **Don't use default for high-security** - Enable ensemble explicitly
2. **Don't over-optimize** - Keep it simple unless metrics show need
3. **Don't hardcode thresholds** - Use environment variables
4. **Don't ignore monitoring** - Enable in production
5. **Don't skip testing** - Test all config combinations
6. **Don't disable detection** - Always enable at minimum

---

## Summary: Quick Reference

| Need | Config |
|------|--------|
| Quick start | `JailGuard::new()` |
| High accuracy | `JailGuardConfig::with_ensemble()` |
| High throughput | Disable non-essential layers |
| High security | All layers + ensemble + strict mode |
| Development | Low threshold, single model |
| Production | Ensemble, proper monitoring |
| Testing | Validate before use |

---

**Last Updated**: January 17, 2026
**Version**: 1.0.0
**Status**: Production Ready
