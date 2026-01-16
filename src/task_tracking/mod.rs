//! Task tracking and behavioral drift detection.
//!
//! This module provides:
//! - Task context management with expected topics
//! - Behavioral drift detection via embedding similarity
//! - Session history tracking
//! - Anomaly detection based on drift patterns

pub mod embedding_similarity;
pub mod task_context;

pub use embedding_similarity::{
    cosine_similarity, detect_drift, drift_score, max_similarity_to_references,
};
pub use task_context::{Action, TaskContext, TaskEvent, TaskStatistics, TaskTracker};
