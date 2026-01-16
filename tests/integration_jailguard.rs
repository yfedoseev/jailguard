//! Integration tests for the unified `JailGuard` API across all 6 layers.

use jailguard::{JailGuard, JailGuardConfig, RequestContext};

#[test]
fn test_basic_jailguard_initialization() {
    let jg = JailGuard::new();
    assert!(!jg.session_id().is_empty());
}

#[test]
fn test_jailguard_with_custom_config() {
    let config = JailGuardConfig {
        block_threshold: 0.8,
        strict_mode: false,
        ..Default::default()
    };
    let jg = JailGuard::with_config(config);
    assert!(!jg.session_id().is_empty());
}

#[test]
fn test_input_validation_flow() {
    let mut jg = JailGuard::with_config(JailGuardConfig {
        enable_spotlighting: true,
        enable_detection: false,
        enable_task_tracking: false,
        enable_privilege_context: false,
        enable_output_validation: false,
        enable_monitoring: false,
        ..Default::default()
    });

    let ctx = RequestContext::new("req-1".to_string());
    let result = jg.check_input("What is 2+2?", &ctx);
    assert!(result.allowed);
    assert!(result.reason.is_none());
}

#[test]
fn test_output_validation_flow() {
    let jg = JailGuard::with_config(JailGuardConfig {
        enable_output_validation: true,
        ..Default::default()
    });

    let result = jg.check_output("Normal output text");
    assert!(result.is_safe);
    assert_eq!(result.violation_count, 0);
}

#[test]
fn test_session_stats_accessible() {
    let jg = JailGuard::with_config(JailGuardConfig {
        enable_spotlighting: true,
        enable_detection: false,
        enable_task_tracking: false,
        enable_privilege_context: false,
        enable_output_validation: false,
        enable_monitoring: false,
        ..Default::default()
    });

    let stats = jg.session_stats();
    // Session stats should be accessible and have default values
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.injection_attempts, 0);
}

#[test]
fn test_disabled_layers() {
    let config = JailGuardConfig {
        enable_spotlighting: false,
        enable_detection: false,
        enable_task_tracking: false,
        enable_privilege_context: false,
        enable_output_validation: false,
        enable_monitoring: false,
        ..Default::default()
    };
    let mut jg = JailGuard::with_config(config);

    let ctx = RequestContext::new("req-1".to_string());
    let result = jg.check_input("Any input", &ctx);
    // Should pass with all layers disabled
    assert!(result.allowed);
}

#[test]
fn test_request_context_builder() {
    let ctx = RequestContext::new("req-123".to_string())
        .with_task("Answer questions".to_string())
        .with_user("user-456".to_string());

    assert_eq!(ctx.request_id, "req-123");
    assert_eq!(ctx.task_description, Some("Answer questions".to_string()));
    assert_eq!(ctx.user_id, Some("user-456".to_string()));
}

#[test]
fn test_multiple_output_checks() {
    let jg = JailGuard::with_config(JailGuardConfig {
        enable_output_validation: true,
        enable_spotlighting: false,
        enable_detection: false,
        enable_task_tracking: false,
        enable_privilege_context: false,
        enable_monitoring: false,
        ..Default::default()
    });

    // Test safe output
    let safe_result = jg.check_output("Safe text");
    assert!(safe_result.is_safe, "Safe text should be safe");

    // Test another safe output
    let safe_result2 = jg.check_output("Another safe message");
    assert!(safe_result2.is_safe, "Another safe text should be safe");
}

#[test]
fn test_session_reset() {
    let config = JailGuardConfig {
        enable_spotlighting: true,
        enable_detection: false,
        ..Default::default()
    };
    let mut jg = JailGuard::with_config(config);

    // Reset should work even with no prior requests
    jg.reset_session();
    let stats_after = jg.session_stats();
    assert_eq!(stats_after.total_requests, 0);
}

#[test]
fn test_jailguard_clone_creates_different_sessions() {
    let jg1 = JailGuard::new();
    let jg2 = jg1.clone();

    assert_ne!(jg1.session_id(), jg2.session_id());
}

#[test]
fn test_strict_mode_configuration() {
    let config = JailGuardConfig {
        strict_mode: true,
        block_threshold: 0.5,
        enable_spotlighting: true,
        enable_detection: false,
        ..Default::default()
    };

    let jg = JailGuard::with_config(config);
    assert!(jg.session_stats().anomaly_score <= 1.0);
    assert!(jg.session_stats().anomaly_score >= 0.0);
}

#[test]
fn test_sequential_requests_minimal() {
    let mut jg = JailGuard::with_config(JailGuardConfig {
        enable_spotlighting: true,
        enable_detection: false,
        enable_task_tracking: false,
        enable_privilege_context: false,
        enable_output_validation: false,
        enable_monitoring: false,
        ..Default::default()
    });

    for i in 0..3 {
        let ctx = RequestContext::new(format!("req-{}", i));
        let result = jg.check_input(&format!("Request {}", i), &ctx);
        assert!(result.allowed, "Request {} should be allowed", i);
    }
}
