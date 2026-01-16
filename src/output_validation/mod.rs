//! Output validation for detecting secret leakage and injection markers.
//!
//! This module provides:
//! - Secret detection (API keys, passwords, tokens, etc.)
//! - Injection marker detection (jailbreak patterns, role-play, etc.)
//! - Output sanitization with configurable redaction
//! - Validation result tracking with position information

pub mod patterns;
pub mod validator;

pub use patterns::{InjectionMarkers, SecretPatterns};
pub use validator::{
    OutputValidationConfig, OutputValidator, ValidationResult, ValidationSummary, Violation,
    ViolationType,
};
