//! Performance profiling and optimization for JailGuard ensemble detection
//!
//! This module provides:
//! - Performance profiling capabilities
//! - Ensemble voting optimization metrics
//! - Cache performance analysis
//! - Memory usage tracking

pub mod cache;
pub mod metrics;
pub mod profiler;

pub use cache::ResponseCache;
pub use metrics::{EnsembleProfile, PerformanceMetrics};
pub use profiler::EnsembleProfiler;
