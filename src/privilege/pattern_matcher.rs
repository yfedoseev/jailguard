//! Pattern matching for detecting resource access attempts.

use crate::privilege::Resource;

/// A resource access pattern with associated keywords.
#[derive(Debug, Clone)]
pub struct ResourcePattern {
    /// The resource type
    pub resource: Resource,
    /// Keywords indicating access to this resource
    pub patterns: Vec<String>,
}

impl ResourcePattern {
    /// Create a new resource pattern.
    pub fn new(resource: Resource, patterns: Vec<String>) -> Self {
        Self { resource, patterns }
    }

    /// Check if text matches any pattern (case-insensitive).
    pub fn matches(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.patterns
            .iter()
            .any(|pattern| text_lower.contains(&pattern.to_lowercase()))
    }
}

/// Database access patterns.
pub fn database_patterns() -> ResourcePattern {
    ResourcePattern::new(
        Resource::Database,
        vec![
            "SELECT".to_string(),
            "INSERT".to_string(),
            "UPDATE".to_string(),
            "DELETE".to_string(),
            "DROP".to_string(),
            "CREATE TABLE".to_string(),
            "ALTER TABLE".to_string(),
            "sql".to_string(),
            "query".to_string(),
            "database".to_string(),
            "execute query".to_string(),
            "sql injection".to_string(),
            "table".to_string(),
        ],
    )
}

/// File system access patterns.
pub fn filesystem_patterns() -> ResourcePattern {
    ResourcePattern::new(
        Resource::FileSystem,
        vec![
            "read file".to_string(),
            "write file".to_string(),
            "delete file".to_string(),
            "open file".to_string(),
            "create file".to_string(),
            "rm ".to_string(),
            "cat ".to_string(),
            "ls ".to_string(),
            "chmod ".to_string(),
            "mkdir ".to_string(),
            "file system".to_string(),
            "directory".to_string(),
            "path".to_string(),
            "/etc".to_string(),
            "/home".to_string(),
        ],
    )
}

/// Network access patterns.
pub fn network_patterns() -> ResourcePattern {
    ResourcePattern::new(
        Resource::Network,
        vec![
            "http://".to_string(),
            "https://".to_string(),
            "fetch".to_string(),
            "curl".to_string(),
            "request".to_string(),
            "api".to_string(),
            "endpoint".to_string(),
            "socket".to_string(),
            "connection".to_string(),
            "server".to_string(),
            "ip address".to_string(),
            "domain".to_string(),
            "network request".to_string(),
        ],
    )
}

/// Credentials/secrets access patterns.
pub fn credentials_patterns() -> ResourcePattern {
    ResourcePattern::new(
        Resource::Credentials,
        vec![
            "password".to_string(),
            "api_key".to_string(),
            "api key".to_string(),
            "secret".to_string(),
            "token".to_string(),
            "credential".to_string(),
            "authentication".to_string(),
            "bearer".to_string(),
            "private key".to_string(),
            "access token".to_string(),
            "refresh token".to_string(),
            "ssh key".to_string(),
            "encryption key".to_string(),
        ],
    )
}

/// Get all resource patterns.
pub fn all_patterns() -> Vec<ResourcePattern> {
    vec![
        database_patterns(),
        filesystem_patterns(),
        network_patterns(),
        credentials_patterns(),
    ]
}

/// Detect resource access in text.
///
/// Returns the resource type if text contains access patterns, None otherwise.
pub fn detect_resource_access(text: &str) -> Option<Resource> {
    for pattern in all_patterns() {
        if pattern.matches(text) {
            return Some(pattern.resource);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_pattern_matches() {
        let pattern = database_patterns();
        assert!(pattern.matches("SELECT * FROM users"));
        assert!(pattern.matches("INSERT INTO table VALUES"));
        assert!(pattern.matches("execute query"));
        assert!(!pattern.matches("normal text"));
    }

    #[test]
    fn test_database_pattern_case_insensitive() {
        let pattern = database_patterns();
        assert!(pattern.matches("select * from users"));
        assert!(pattern.matches("Select * From Users"));
    }

    #[test]
    fn test_filesystem_pattern_matches() {
        let pattern = filesystem_patterns();
        assert!(pattern.matches("read file /etc/passwd"));
        assert!(pattern.matches("delete file config"));
        assert!(pattern.matches("rm -rf /home"));
        assert!(pattern.matches("cat /etc/shadow"));
        assert!(!pattern.matches("normal text"));
    }

    #[test]
    fn test_network_pattern_matches() {
        let pattern = network_patterns();
        assert!(pattern.matches("https://example.com"));
        assert!(pattern.matches("fetch data from API"));
        assert!(pattern.matches("curl request"));
        assert!(pattern.matches("connect to server"));
        assert!(!pattern.matches("normal text"));
    }

    #[test]
    fn test_credentials_pattern_matches() {
        let pattern = credentials_patterns();
        assert!(pattern.matches("password: secret123"));
        assert!(pattern.matches("api_key: xyz123"));
        assert!(pattern.matches("bearer token"));
        assert!(pattern.matches("private key content"));
        assert!(!pattern.matches("normal text"));
    }

    #[test]
    fn test_detect_database_access() {
        let resource = detect_resource_access("SELECT * FROM users");
        assert_eq!(resource, Some(Resource::Database));
    }

    #[test]
    fn test_detect_filesystem_access() {
        let resource = detect_resource_access("read file /etc/passwd");
        assert_eq!(resource, Some(Resource::FileSystem));
    }

    #[test]
    fn test_detect_network_access() {
        let resource = detect_resource_access("fetch from https://api.example.com");
        assert_eq!(resource, Some(Resource::Network));
    }

    #[test]
    fn test_detect_credentials_access() {
        let resource = detect_resource_access("show password");
        assert_eq!(resource, Some(Resource::Credentials));
    }

    #[test]
    fn test_detect_no_access() {
        let resource = detect_resource_access("what is the weather today?");
        assert_eq!(resource, None);
    }

    #[test]
    fn test_pattern_priority() {
        // First matching pattern wins
        let text = "SELECT password FROM users";
        let resource = detect_resource_access(text);
        // Database pattern appears first, so it should match
        assert_eq!(resource, Some(Resource::Database));
    }

    #[test]
    fn test_partial_word_match() {
        let pattern = database_patterns();
        // Should match even within larger words
        assert!(pattern.matches("query_results"));
    }
}
