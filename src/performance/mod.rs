//! Performance profiling and optimization for JailGuard ensemble detection
//!
//! This module provides:
//! - Performance profiling capabilities
//! - Ensemble voting optimization metrics
//! - Cache performance analysis
//! - Memory usage tracking

pub mod profiler;
pub mod cache;
pub mod metrics;

pub use cache::ResponseCache;
pub use metrics::{PerformanceMetrics, EnsembleProfile};
pub use profiler::EnsembleProfiler;
