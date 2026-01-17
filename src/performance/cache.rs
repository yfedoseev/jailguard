//! Response cache for ensemble detection results
//!
//! Caches detection results to avoid redundant model invocations.
//! Useful for repeated checks on the same or similar inputs.

use std::collections::HashMap;
use std::time::{Instant, Duration};
use crate::detection::DetectionResult;

/// Cache entry with timestamp and TTL
#[derive(Clone, Debug)]
struct CacheEntry {
    result: DetectionResult,
    created: Instant,
    ttl: Duration,
}

impl CacheEntry {
    /// Check if entry is still valid
    fn is_valid(&self) -> bool {
        self.created.elapsed() < self.ttl
    }
}

/// Response cache for detection results
///
/// Implements:
/// - Hash-based caching on text input
/// - TTL-based expiration
/// - Configurable max size
/// - Automatic cleanup of expired entries
#[derive(Debug, Clone)]
pub struct ResponseCache {
    /// Cache storage (text -> result + metadata)
    cache: HashMap<String, CacheEntry>,
    /// Maximum cache size
    max_size: usize,
    /// Default TTL for cache entries
    default_ttl: Duration,
    /// Cache hits
    hits: usize,
    /// Cache misses
    misses: usize,
}

impl ResponseCache {
    /// Create new cache with default settings
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_size: 1000,
            default_ttl: Duration::from_secs(300), // 5 minutes default
            hits: 0,
            misses: 0,
        }
    }

    /// Create cache with custom settings
    pub fn with_config(max_size: usize, ttl_secs: u64) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            default_ttl: Duration::from_secs(ttl_secs),
            hits: 0,
            misses: 0,
        }
    }

    /// Get cached result if available and valid
    pub fn get(&mut self, text: &str) -> Option<DetectionResult> {
        self.cleanup_expired();

        if let Some(entry) = self.cache.get(text) {
            if entry.is_valid() {
                self.hits += 1;
                return Some(entry.result.clone());
            }
        }

        self.misses += 1;
        None
    }

    /// Store result in cache
    pub fn put(&mut self, text: String, result: DetectionResult) {
        if self.cache.len() >= self.max_size {
            self.evict_oldest();
        }

        self.cache.insert(
            text,
            CacheEntry {
                result,
                created: Instant::now(),
                ttl: self.default_ttl,
            },
        );
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Get cache hit rate (0.0-1.0)
    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f32 / total as f32
        }
    }

    /// Get number of entries in cache
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            hit_rate: self.hit_rate(),
            size: self.cache.len(),
            capacity: self.max_size,
        }
    }

    /// Remove expired entries
    fn cleanup_expired(&mut self) {
        self.cache.retain(|_, entry| entry.is_valid());
    }

    /// Remove oldest entry (simple LRU eviction)
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .cache
            .iter()
            .min_by_key(|(_, entry)| entry.created)
            .map(|(key, _)| key.clone())
        {
            self.cache.remove(&oldest_key);
        }
    }
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: usize,
    /// Total cache misses
    pub misses: usize,
    /// Hit rate (0.0-1.0)
    pub hit_rate: f32,
    /// Current cache size
    pub size: usize,
    /// Cache capacity
    pub capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit_miss() {
        let mut cache = ResponseCache::new();
        let result = DetectionResult::new(true, 0.9, [0.9, 0.1]);

        // Miss
        assert!(cache.get("test").is_none());
        assert_eq!(cache.misses, 1);

        // Put and hit
        cache.put("test".to_string(), result.clone());
        assert!(cache.get("test").is_some());
        assert_eq!(cache.hits, 1);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = ResponseCache::with_config(100, 0); // 0 second TTL
        let result = DetectionResult::new(false, 0.1, [0.9, 0.1]);

        cache.put("test".to_string(), result);
        std::thread::sleep(Duration::from_millis(10));

        // Should be expired
        assert!(cache.get("test").is_none());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = ResponseCache::new();
        let result = DetectionResult::new(true, 0.8, [0.8, 0.2]);

        cache.put("test1".to_string(), result.clone());
        cache.put("test2".to_string(), result.clone());

        cache.get("test1"); // hit
        cache.get("test3"); // miss

        let stats = cache.stats();
        assert_eq!(stats.size, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!(stats.hit_rate > 0.4 && stats.hit_rate < 0.6);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = ResponseCache::with_config(2, 3600); // 2 item max
        let result = DetectionResult::new(false, 0.2, [0.8, 0.2]);

        cache.put("test1".to_string(), result.clone());
        cache.put("test2".to_string(), result.clone());
        cache.put("test3".to_string(), result); // Should evict test1

        assert!(cache.get("test1").is_none()); // Evicted
        assert!(cache.get("test2").is_some()); // Still there
        assert!(cache.get("test3").is_some()); // Still there
    }
}
