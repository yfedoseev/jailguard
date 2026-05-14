//! Privilege context and access control for resource protection.
//!
//! This module provides:
//! - Resource access control (Database, `FileSystem`, Network, Credentials)
//! - Rate limiting for resource access
//! - Scope-based access restrictions
//! - Access logging and auditing

pub mod context;
pub mod pattern_matcher;

pub use context::{
    AccessEvent, PrivilegeConfig, PrivilegeResult, PrivilegeStatistics, PrivilegeValidator,
    RateLimit, Resource, ScopeRule,
};
pub use pattern_matcher::{all_patterns, detect_resource_access, ResourcePattern};
