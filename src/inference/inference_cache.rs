//! Caching layer for inference results.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Statistics about cache performance
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Number of evictions
    pub evictions: usize,
    /// Total entries currently in cache
    pub current_entries: usize,
}

impl CacheStats {
    /// Get cache hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f32 {
        let total = (self.hits + self.misses) as f32;
        if total == 0.0 {
            0.0
        } else {
            self.hits as f32 / total
        }
    }
}

/// Configuration for the inference cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in cache
    pub max_entries: usize,
    /// TTL in seconds
    pub ttl_secs: u64,
    /// Enable cache (can be disabled for testing)
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl_secs: 3600,
            enabled: true,
        }
    }
}

/// Cached inference result
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached result (serialized as string for simplicity)
    result: String,
    /// Timestamp when entry was created
    created_at: u64,
    /// TTL in seconds
    ttl_secs: u64,
}

impl CacheEntry {
    /// Check if entry is expired
    fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.created_at + self.ttl_secs
    }
}

/// Cache for inference results
pub struct InferenceCache {
    config: CacheConfig,
    cache: HashMap<String, CacheEntry>,
    stats: CacheStats,
}

impl InferenceCache {
    /// Create a new inference cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
            stats: CacheStats::default(),
        }
    }

    /// Get a cached result
    pub fn get(&mut self, key: &str) -> Option<String> {
        if !self.config.enabled {
            return None;
        }

        if let Some(entry) = self.cache.get(key) {
            if entry.is_expired() {
                self.cache.remove(key);
                self.stats.misses += 1;
                None
            } else {
                self.stats.hits += 1;
                Some(entry.result.clone())
            }
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Set a cached result
    pub fn set(&mut self, key: String, result: String) {
        if !self.config.enabled {
            return;
        }

        // Evict if cache is full
        if self.cache.len() >= self.config.max_entries {
            self.evict_oldest();
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.cache.insert(
            key,
            CacheEntry {
                result,
                created_at: now,
                ttl_secs: self.config.ttl_secs,
            },
        );
    }

    /// Evict the oldest entry from cache
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .cache
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(k, _)| k.clone())
        {
            self.cache.remove(&oldest_key);
            self.stats.evictions += 1;
        }
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = self.stats.clone();
        stats.current_entries = self.cache.len();
        stats
    }

    /// Remove expired entries (cleanup)
    pub fn cleanup(&mut self) {
        let expired: Vec<String> = self
            .cache
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired {
            self.cache.remove(&key);
        }
    }

    /// Get number of entries in cache
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_set_and_get() {
        let config = CacheConfig::default();
        let mut cache = InferenceCache::new(config);

        cache.set("key1".to_string(), "result1".to_string());
        assert_eq!(cache.get("key1"), Some("result1".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let config = CacheConfig::default();
        let mut cache = InferenceCache::new(config);

        assert_eq!(cache.get("nonexistent"), None);
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_cache_hit_rate() {
        let config = CacheConfig::default();
        let mut cache = InferenceCache::new(config);

        cache.set("key1".to_string(), "result1".to_string());
        cache.get("key1"); // hit
        cache.get("key2"); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 0.5);
    }

    #[test]
    fn test_cache_disabled() {
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };
        let mut cache = InferenceCache::new(config);

        cache.set("key1".to_string(), "result1".to_string());
        assert_eq!(cache.get("key1"), None);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_max_entries() {
        let config = CacheConfig {
            max_entries: 2,
            ..Default::default()
        };
        let mut cache = InferenceCache::new(config);

        cache.set("key1".to_string(), "result1".to_string());
        cache.set("key2".to_string(), "result2".to_string());
        cache.set("key3".to_string(), "result3".to_string()); // Should evict one

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn test_cache_clear() {
        let config = CacheConfig::default();
        let mut cache = InferenceCache::new(config);

        cache.set("key1".to_string(), "result1".to_string());
        cache.set("key2".to_string(), "result2".to_string());
        cache.clear();

        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let config = CacheConfig::default();
        let mut cache = InferenceCache::new(config);

        cache.set("key1".to_string(), "result1".to_string());
        cache.get("key1"); // hit
        cache.get("key2"); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.current_entries, 1);
    }
}
