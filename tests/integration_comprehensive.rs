#![cfg(feature = "full")]

//! Comprehensive integration tests spanning multiple test categories.
//!
//! This module organizes all testing categories:
//! - Robustness tests for adversarial attacks
//! - Scenario tests for real-world attack patterns
//! - Calibration tests for confidence reliability

// Test utilities and fixtures
mod common;

// Robustness tests: Adversarial attack patterns
#[path = "robustness/mod.rs"]
mod robustness;

// Scenario tests: Real-world attack campaigns
#[path = "scenarios/mod.rs"]
mod scenarios;

// Calibration tests: Confidence reliability
#[path = "calibration/mod.rs"]
mod calibration;

// Re-export modules for easier access
pub use common::*;

#[doc = "\
Summary of comprehensive test suite

Total test categories:
- Unit Tests: 274+ inline tests across all modules
- Integration Tests: 12 unified API tests + 80+ comprehensive tests
- Robustness Tests: 20+ adversarial pattern tests
- Scenario Tests: 30+ real-world attack scenario tests
- Calibration Tests: 15+ confidence reliability tests

Total estimated: 430+ tests across all categories
"]
mod test_suite_summary {}
