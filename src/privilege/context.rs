//! Privilege context and access control validation.

use super::pattern_matcher::detect_resource_access;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, SystemTime};

/// Resource types that can be protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resource {
    /// Database access (queries, tables)
    Database,
    /// File system access (read, write, delete)
    FileSystem,
    /// Network access (HTTP, sockets)
    Network,
    /// Credentials/secrets (passwords, tokens, keys)
    Credentials,
}

impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Resource::Database => write!(f, "Database"),
            Resource::FileSystem => write!(f, "FileSystem"),
            Resource::Network => write!(f, "Network"),
            Resource::Credentials => write!(f, "Credentials"),
        }
    }
}

/// Rate limit for a resource.
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Maximum requests per time window
    pub max_requests: usize,
    /// Time window duration
    pub window: Duration,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            max_requests: 10,
            window: Duration::from_secs(60),
        }
    }
}

impl RateLimit {
    /// Create a new rate limit.
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
        }
    }
}

/// A scope rule restricting resource access to specific contexts.
#[derive(Debug, Clone)]
pub struct ScopeRule {
    /// Resource being restricted
    pub resource: Resource,
    /// Allowed scope (e.g., "`user_tables`" for database)
    pub allowed_scope: String,
}

impl ScopeRule {
    /// Create a new scope rule.
    pub fn new(resource: Resource, allowed_scope: String) -> Self {
        Self {
            resource,
            allowed_scope,
        }
    }

    /// Check if access matches this scope.
    pub fn matches(&self, text: &str) -> bool {
        text.contains(&self.allowed_scope)
    }
}

/// Access event for auditing.
#[derive(Debug, Clone)]
pub struct AccessEvent {
    /// Resource being accessed
    pub resource: Resource,
    /// Request text
    pub text: String,
    /// Whether access was allowed
    pub allowed: bool,
    /// Timestamp
    pub timestamp: SystemTime,
}

/// Result of privilege validation.
#[derive(Debug, Clone)]
pub struct PrivilegeResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Reason for denial (if denied)
    pub reason: Option<String>,
    /// Resource accessed (if any)
    pub resource: Option<Resource>,
}

impl PrivilegeResult {
    /// Create an allowed result.
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            reason: None,
            resource: None,
        }
    }

    /// Create a denied result.
    pub fn denied(reason: String, resource: Option<Resource>) -> Self {
        Self {
            allowed: false,
            reason: Some(reason),
            resource,
        }
    }
}

/// Configuration for privilege control.
#[derive(Debug, Clone)]
pub struct PrivilegeConfig {
    /// Resources to allow (empty = deny all)
    pub allowed_resources: HashSet<Resource>,
    /// Rate limits per resource
    pub rate_limits: HashMap<Resource, RateLimit>,
    /// Scope restrictions
    pub scope_restrictions: Vec<ScopeRule>,
    /// Enable rate limiting
    pub enable_rate_limits: bool,
    /// Enable scope restrictions
    pub enable_scopes: bool,
}

impl Default for PrivilegeConfig {
    fn default() -> Self {
        Self {
            allowed_resources: HashSet::new(),
            rate_limits: HashMap::new(),
            scope_restrictions: Vec::new(),
            enable_rate_limits: true,
            enable_scopes: true,
        }
    }
}

impl PrivilegeConfig {
    /// Allow a resource.
    pub fn allow(mut self, resource: Resource) -> Self {
        self.allowed_resources.insert(resource);
        self
    }

    /// Set rate limit for a resource.
    pub fn rate_limit(mut self, resource: Resource, limit: RateLimit) -> Self {
        self.rate_limits.insert(resource, limit);
        self
    }

    /// Add a scope restriction.
    pub fn with_scope(mut self, rule: ScopeRule) -> Self {
        self.scope_restrictions.push(rule);
        self
    }
}

/// Validates privilege requests.
#[derive(Debug)]
pub struct PrivilegeValidator {
    /// Configuration
    config: PrivilegeConfig,
    /// Access log (circular buffer)
    access_log: VecDeque<AccessEvent>,
    /// Max log size
    max_log_size: usize,
}

impl PrivilegeValidator {
    /// Create a new privilege validator.
    pub fn new(config: PrivilegeConfig) -> Self {
        Self {
            config,
            access_log: VecDeque::with_capacity(1000),
            max_log_size: 1000,
        }
    }

    /// Validate a request for resource access.
    pub fn validate_request(&mut self, text: &str) -> PrivilegeResult {
        // Detect resource access
        let Some(resource) = detect_resource_access(text) else {
            return PrivilegeResult::allowed(); // No resource access detected
        };

        // Check if resource is allowed
        if !self.config.allowed_resources.contains(&resource) {
            let result = PrivilegeResult::denied(
                format!("Access to {} not allowed", resource),
                Some(resource),
            );
            self.log_access(resource, text, false);
            return result;
        }

        // Check rate limit
        if self.config.enable_rate_limits {
            if let Some(limit) = self.config.rate_limits.get(&resource) {
                if !self.check_rate_limit(resource, limit) {
                    let result = PrivilegeResult::denied(
                        format!("{} access rate limit exceeded", resource),
                        Some(resource),
                    );
                    self.log_access(resource, text, false);
                    return result;
                }
            }
        }

        // Check scope restrictions
        if self.config.enable_scopes {
            for rule in &self.config.scope_restrictions {
                if rule.resource == resource && !rule.matches(text) {
                    let result = PrivilegeResult::denied(
                        format!("{} access outside allowed scope", resource),
                        Some(resource),
                    );
                    self.log_access(resource, text, false);
                    return result;
                }
            }
        }

        // Access allowed
        self.log_access(resource, text, true);
        PrivilegeResult::allowed()
    }

    /// Check if request exceeds rate limit.
    fn check_rate_limit(&self, resource: Resource, limit: &RateLimit) -> bool {
        let now = SystemTime::now();
        let cutoff = now - limit.window;

        let recent_count = self
            .access_log
            .iter()
            .filter(|event| {
                event.resource == resource && event.allowed && event.timestamp >= cutoff
            })
            .count();

        recent_count < limit.max_requests
    }

    /// Log an access event.
    fn log_access(&mut self, resource: Resource, text: &str, allowed: bool) {
        if self.access_log.len() >= self.max_log_size {
            self.access_log.pop_front();
        }

        self.access_log.push_back(AccessEvent {
            resource,
            text: text.to_string(),
            allowed,
            timestamp: SystemTime::now(),
        });
    }

    /// Get access statistics.
    pub fn statistics(&self) -> PrivilegeStatistics {
        let total = self.access_log.len();
        let denied = self.access_log.iter().filter(|e| !e.allowed).count();
        let allowed = total - denied;

        PrivilegeStatistics {
            total_requests: total,
            allowed_requests: allowed,
            denied_requests: denied,
            denial_ratio: if total == 0 {
                0.0
            } else {
                denied as f32 / total as f32
            },
        }
    }

    /// Get access log.
    pub fn access_log(&self) -> Vec<&AccessEvent> {
        self.access_log.iter().collect()
    }

    /// Clear access log.
    pub fn clear_log(&mut self) {
        self.access_log.clear();
    }

    /// Allow a resource.
    pub fn allow_resource(&mut self, resource: Resource) {
        self.config.allowed_resources.insert(resource);
    }

    /// Deny a resource.
    pub fn deny_resource(&mut self, resource: Resource) {
        self.config.allowed_resources.remove(&resource);
    }

    /// Set rate limit.
    pub fn set_rate_limit(&mut self, resource: Resource, limit: RateLimit) {
        self.config.rate_limits.insert(resource, limit);
    }
}

/// Privilege statistics.
#[derive(Debug, Clone)]
pub struct PrivilegeStatistics {
    /// Total access requests
    pub total_requests: usize,
    /// Allowed requests
    pub allowed_requests: usize,
    /// Denied requests
    pub denied_requests: usize,
    /// Ratio of denials
    pub denial_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_display() {
        assert_eq!(Resource::Database.to_string(), "Database");
        assert_eq!(Resource::FileSystem.to_string(), "FileSystem");
        assert_eq!(Resource::Network.to_string(), "Network");
        assert_eq!(Resource::Credentials.to_string(), "Credentials");
    }

    #[test]
    fn test_rate_limit_creation() {
        let limit = RateLimit::new(5, Duration::from_secs(60));
        assert_eq!(limit.max_requests, 5);
        assert_eq!(limit.window, Duration::from_secs(60));
    }

    #[test]
    fn test_scope_rule_matches() {
        let rule = ScopeRule::new(Resource::Database, "user_tables".to_string());
        assert!(rule.matches("query user_tables"));
        assert!(!rule.matches("query admin_tables"));
    }

    #[test]
    fn test_privilege_result_allowed() {
        let result = PrivilegeResult::allowed();
        assert!(result.allowed);
        assert!(result.reason.is_none());
    }

    #[test]
    fn test_privilege_result_denied() {
        let result =
            PrivilegeResult::denied("Access denied".to_string(), Some(Resource::Credentials));
        assert!(!result.allowed);
        assert!(result.reason.is_some());
        assert_eq!(result.resource, Some(Resource::Credentials));
    }

    #[test]
    fn test_config_allow_resource() {
        let config = PrivilegeConfig::default()
            .allow(Resource::Database)
            .allow(Resource::FileSystem);

        assert!(config.allowed_resources.contains(&Resource::Database));
        assert!(config.allowed_resources.contains(&Resource::FileSystem));
        assert!(!config.allowed_resources.contains(&Resource::Network));
    }

    #[test]
    fn test_validator_deny_not_allowed() {
        let config = PrivilegeConfig::default().allow(Resource::FileSystem); // Only allow FileSystem
        let mut validator = PrivilegeValidator::new(config);

        let result = validator.validate_request("SELECT * FROM users");
        assert!(!result.allowed);
        assert_eq!(result.resource, Some(Resource::Database));
    }

    #[test]
    fn test_validator_allow_allowed() {
        let config = PrivilegeConfig::default().allow(Resource::Database);
        let mut validator = PrivilegeValidator::new(config);

        let result = validator.validate_request("SELECT * FROM users");
        assert!(result.allowed);
    }

    #[test]
    fn test_validator_no_resource_access() {
        let config = PrivilegeConfig::default();
        let mut validator = PrivilegeValidator::new(config);

        let result = validator.validate_request("what is 2+2?");
        assert!(result.allowed); // No resource access detected
    }

    #[test]
    fn test_validator_rate_limit() {
        let config = PrivilegeConfig::default()
            .allow(Resource::Database)
            .rate_limit(
                Resource::Database,
                RateLimit::new(2, Duration::from_secs(60)),
            );
        let mut validator = PrivilegeValidator::new(config);

        // First two requests allowed
        assert!(validator.validate_request("SELECT 1").allowed);
        assert!(validator.validate_request("SELECT 2").allowed);

        // Third request denied
        let result = validator.validate_request("SELECT 3");
        assert!(!result.allowed);
    }

    #[test]
    fn test_validator_scope_restriction() {
        let config = PrivilegeConfig::default()
            .allow(Resource::Database)
            .with_scope(ScopeRule::new(Resource::Database, "users".to_string()));
        let mut validator = PrivilegeValidator::new(config);

        // Access to allowed scope
        assert!(validator.validate_request("SELECT * FROM users").allowed);

        // Access to denied scope
        let result = validator.validate_request("SELECT * FROM admin");
        assert!(!result.allowed);
    }

    #[test]
    fn test_validator_statistics() {
        let config = PrivilegeConfig::default().allow(Resource::Database);
        let mut validator = PrivilegeValidator::new(config);

        validator.validate_request("SELECT 1"); // allowed
        validator.validate_request("read file /etc/passwd"); // denied

        let stats = validator.statistics();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.allowed_requests, 1);
        assert_eq!(stats.denied_requests, 1);
        assert!((stats.denial_ratio - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_validator_access_log() {
        let config = PrivilegeConfig::default().allow(Resource::Database);
        let mut validator = PrivilegeValidator::new(config);

        validator.validate_request("SELECT 1");
        let log = validator.access_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].resource, Resource::Database);
        assert!(log[0].allowed);
    }

    #[test]
    fn test_validator_clear_log() {
        let config = PrivilegeConfig::default().allow(Resource::Database);
        let mut validator = PrivilegeValidator::new(config);

        validator.validate_request("SELECT 1");
        assert!(!validator.access_log().is_empty());

        validator.clear_log();
        assert!(validator.access_log().is_empty());
    }

    #[test]
    fn test_validator_allow_deny_resource() {
        let config = PrivilegeConfig::default();
        let mut validator = PrivilegeValidator::new(config);

        // Initially denied
        assert!(!validator.validate_request("SELECT 1").allowed);

        // Allow and try again
        validator.allow_resource(Resource::Database);
        assert!(validator.validate_request("SELECT 2").allowed);

        // Deny and try again
        validator.deny_resource(Resource::Database);
        assert!(!validator.validate_request("SELECT 3").allowed);
    }
}
