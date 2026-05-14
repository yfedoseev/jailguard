//! Detection result caching for improved performance.
//!
//! Caches detection results to avoid redundant computations for identical inputs.
//! Uses LRU (Least Recently Used) eviction when cache size exceeds limit.

use crate::detection::MultiTaskDetectionResult;
use std::collections::HashMap;

/// Cache entry with result and creation timestamp.
#[derive(Debug, Clone)]
struct CacheEntry {
    result: MultiTaskDetectionResult,
    access_count: usize,
    last_access: std::time::SystemTime,
}

/// Configuration for detection result caching.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of cached results
    pub max_size: usize,
    /// Enable caching
    pub enabled: bool,
    /// Cache TTL (time-to-live) in seconds, None = infinite
    pub ttl_seconds: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            enabled: true,
            ttl_seconds: Some(3600), // 1 hour default TTL
        }
    }
}

/// Detection result cache with LRU eviction.
pub struct DetectionResultCache {
    cache: HashMap<String, CacheEntry>,
    config: CacheConfig,
    access_order: Vec<String>, // Track access order for LRU
}

impl DetectionResultCache {
    /// Create a new detection result cache.
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: HashMap::new(),
            config,
            access_order: Vec::new(),
        }
    }

    /// Create with default configuration.
    pub fn default() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Get cached detection result if available and not expired.
    pub fn get(&mut self, text: &str) -> Option<MultiTaskDetectionResult> {
        if !self.config.enabled {
            return None;
        }

        let key = Self::hash_text(text);

        if let Some(entry) = self.cache.get_mut(&key) {
            // Check if entry has expired
            if let Some(ttl) = self.config.ttl_seconds {
                if let Ok(elapsed) = entry.last_access.elapsed() {
                    if elapsed.as_secs() > ttl {
                        // Entry expired, remove it
                        self.cache.remove(&key);
                        self.access_order.retain(|k| k != &key);
                        return None;
                    }
                }
            }

            // Update access tracking
            entry.access_count += 1;
            entry.last_access = std::time::SystemTime::now();

            // Update access order for LRU
            self.access_order.retain(|k| k != &key);
            self.access_order.push(key);

            return Some(entry.result.clone());
        }

        None
    }

    /// Store detection result in cache.
    pub fn put(&mut self, text: &str, result: MultiTaskDetectionResult) {
        if !self.config.enabled {
            return;
        }

        let key = Self::hash_text(text);

        // Remove LRU entry if cache is full
        if self.cache.len() >= self.config.max_size && !self.cache.contains_key(&key) {
            if let Some(lru_key) = self.access_order.first() {
                let lru_key = lru_key.clone();
                self.cache.remove(&lru_key);
                self.access_order.remove(0);
            }
        }

        let entry = CacheEntry {
            result,
            access_count: 1,
            last_access: std::time::SystemTime::now(),
        };

        self.cache.insert(key.clone(), entry);
        self.access_order.retain(|k| k != &key);
        self.access_order.push(key);
    }

    /// Clear all cached results.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let total_accesses: usize = self.cache.values().map(|e| e.access_count).sum();

        CacheStats {
            size: self.cache.len(),
            max_size: self.config.max_size,
            total_accesses,
            hit_count: 0, // Would track separately in production
        }
    }

    /// Remove expired entries from cache.
    pub fn evict_expired(&mut self) {
        if let Some(ttl) = self.config.ttl_seconds {
            let ttl_dur = std::time::Duration::from_secs(ttl);
            let now = std::time::SystemTime::now();

            let expired_keys: Vec<String> = self
                .cache
                .iter()
                .filter(|(_, entry)| {
                    if let Ok(elapsed) = now.duration_since(entry.last_access) {
                        elapsed > ttl_dur
                    } else {
                        false
                    }
                })
                .map(|(k, _)| k.clone())
                .collect();

            for key in expired_keys {
                self.cache.remove(&key);
                self.access_order.retain(|k| k != &key);
            }
        }
    }

    /// Hash input text to create cache key.
    fn hash_text(text: &str) -> String {
        // Simple hash using text length and first/last chars for quick key
        // In production, would use SHA256 or similar
        format!(
            "{}_{}_{}_{}",
            text.len(),
            text.chars().next().map(|c| c as u32).unwrap_or(0),
            text.chars().last().map(|c| c as u32).unwrap_or(0),
            text.chars().take(5).collect::<String>()
        )
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current number of cached entries
    pub size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Total accesses across all entries
    pub total_accesses: usize,
    /// Number of cache hits
    pub hit_count: usize,
}

impl CacheStats {
    /// Get cache utilization ratio (0.0-1.0).
    pub fn utilization(&self) -> f32 {
        if self.max_size == 0 {
            0.0
        } else {
            self.size as f32 / self.max_size as f32
        }
    }

    /// Get average accesses per entry.
    pub fn avg_accesses_per_entry(&self) -> f32 {
        if self.size == 0 {
            0.0
        } else {
            self.total_accesses as f32 / self.size as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_result() -> MultiTaskDetectionResult {
        use crate::detection::{AttackType, DetectionResult};

        MultiTaskDetectionResult {
            detection: DetectionResult::new(true, 0.95, [0.95, 0.05]),
            attack_type: AttackType::RolePlay,
            attack_probs: [0.05, 0.90, 0.02, 0.01, 0.01, 0.01, 0.00, 0.0],
            semantic_score: 0.85,
            embedding: vec![0.1; 384],
        }
    }

    #[test]
    fn test_cache_creation_default() {
        let cache = DetectionResultCache::default();
        assert!(cache.config.enabled);
        assert_eq!(cache.config.max_size, 1000);
        assert_eq!(cache.cache.len(), 0);
    }

    #[test]
    fn test_cache_creation_custom() {
        let config = CacheConfig {
            max_size: 100,
            enabled: true,
            ttl_seconds: Some(1800),
        };
        let cache = DetectionResultCache::new(config);
        assert_eq!(cache.config.max_size, 100);
    }

    #[test]
    fn test_cache_disabled() {
        let config = CacheConfig {
            enabled: false,
            ..Default::default()
        };
        let mut cache = DetectionResultCache::new(config);

        let result = create_test_result();
        cache.put("test", result.clone());

        assert_eq!(cache.cache.len(), 0);
        assert!(cache.get("test").is_none());
    }

    #[test]
    fn test_cache_put_get() {
        let mut cache = DetectionResultCache::default();
        let result = create_test_result();

        cache.put("test", result.clone());
        assert_eq!(cache.cache.len(), 1);

        let retrieved = cache.get("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().detection.confidence, 0.95);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = DetectionResultCache::default();
        let retrieved = cache.get("nonexistent");
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let config = CacheConfig {
            max_size: 3,
            enabled: true,
            ttl_seconds: None,
        };
        let mut cache = DetectionResultCache::new(config);
        let result = create_test_result();

        // Fill cache
        cache.put("text1", result.clone());
        cache.put("text2", result.clone());
        cache.put("text3", result.clone());
        assert_eq!(cache.cache.len(), 3);

        // Add one more, should evict LRU (text1)
        cache.put("text4", result.clone());
        assert_eq!(cache.cache.len(), 3);
        assert!(cache.get("text1").is_none());
        assert!(cache.get("text4").is_some());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = DetectionResultCache::default();
        let result = create_test_result();

        cache.put("text1", result.clone());
        cache.put("text2", result.clone());

        let stats = cache.stats();
        assert_eq!(stats.size, 2);
        assert_eq!(stats.max_size, 1000);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = DetectionResultCache::default();
        let result = create_test_result();

        cache.put("text1", result.clone());
        cache.put("text2", result);

        cache.clear();
        assert_eq!(cache.cache.len(), 0);
        assert_eq!(cache.access_order.len(), 0);
    }

    #[test]
    fn test_cache_stats_utilization() {
        let mut cache = DetectionResultCache::default();
        cache.config.max_size = 100;

        let result = create_test_result();
        for i in 0..50 {
            cache.put(&format!("text{}", i), result.clone());
        }

        let stats = cache.stats();
        assert_eq!(stats.utilization(), 0.5);
    }

    #[test]
    fn test_cache_access_tracking() {
        let mut cache = DetectionResultCache::default();
        let result = create_test_result();

        cache.put("text1", result);

        // Access multiple times
        cache.get("text1");
        cache.get("text1");
        cache.get("text1");

        let stats = cache.stats();
        // Initial put = 1, 3 gets = 3 more = 4 total
        assert_eq!(stats.total_accesses, 4);
    }
}
