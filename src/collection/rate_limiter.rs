//! Rate Limiting for API Collection
//!
//! Manages API rate limits and quota to avoid getting blocked.

use super::error::{CollectionError, CollectionResult};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Rate limit configuration
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum requests per time window
    pub max_requests: u32,
    /// Time window in seconds
    pub window_secs: u64,
    /// Delay between requests in milliseconds
    pub request_delay_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_secs: 60,
            request_delay_ms: 100,
        }
    }
}

impl RateLimitConfig {
    /// Reddit API rate limit (60 requests per minute)
    pub fn reddit() -> Self {
        Self {
            max_requests: 60,
            window_secs: 60,
            request_delay_ms: 1000,
        }
    }

    /// GitHub API rate limit (60 requests per hour for unauthenticated)
    pub fn github_unauthenticated() -> Self {
        Self {
            max_requests: 60,
            window_secs: 3600,
            request_delay_ms: 500,
        }
    }

    /// GitHub API rate limit (5000 requests per hour for authenticated)
    pub fn github_authenticated() -> Self {
        Self {
            max_requests: 5000,
            window_secs: 3600,
            request_delay_ms: 50,
        }
    }

    /// Stack Overflow API rate limit (300 requests per day)
    pub fn stackoverflow() -> Self {
        Self {
            max_requests: 300,
            window_secs: 86400,
            request_delay_ms: 1000,
        }
    }

    /// arXiv API rate limit (3 requests per second)
    pub fn arxiv() -> Self {
        Self {
            max_requests: 3,
            window_secs: 1,
            request_delay_ms: 333,
        }
    }
}

/// Rate limiter for tracking API quota usage
pub struct RateLimiter {
    config: RateLimitConfig,
    request_times: Vec<SystemTime>,
    last_request_time: Option<SystemTime>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            request_times: Vec::new(),
            last_request_time: None,
        }
    }

    /// Check if a request can be made
    pub fn can_request(&self) -> CollectionResult<()> {
        // Check if enough time has passed since last request
        if let Some(last_time) = self.last_request_time {
            if let Ok(elapsed) = last_time.elapsed() {
                let delay_required = Duration::from_millis(self.config.request_delay_ms);
                if elapsed < delay_required {
                    let wait_ms = delay_required.as_millis() - elapsed.as_millis();
                    return Err(CollectionError::RateLimitExceeded {
                        reset_time: Some(format!("in {}ms", wait_ms)),
                    });
                }
            }
        }

        // Check quota in current window
        let now = SystemTime::now();
        let window_duration = Duration::from_secs(self.config.window_secs);

        // Remove old requests outside the window
        let cutoff_time = now - window_duration;
        let recent_requests = self
            .request_times
            .iter()
            .filter(|&&t| t > cutoff_time)
            .count();

        if recent_requests as u32 >= self.config.max_requests {
            return Err(CollectionError::RateLimitExceeded {
                reset_time: Some("after current window".to_string()),
            });
        }

        Ok(())
    }

    /// Record that a request was made
    pub fn record_request(&mut self) {
        let now = SystemTime::now();
        self.request_times.push(now);
        self.last_request_time = Some(now);

        // Clean up old entries
        let window_duration = Duration::from_secs(self.config.window_secs);
        let cutoff_time = now - window_duration;
        self.request_times.retain(|&t| t > cutoff_time);
    }

    /// Get current request count in window
    pub fn current_requests(&self) -> u32 {
        let now = SystemTime::now();
        let window_duration = Duration::from_secs(self.config.window_secs);
        let cutoff_time = now - window_duration;

        self.request_times.iter().filter(|&&t| t > cutoff_time).count() as u32
    }

    /// Get remaining requests in window
    pub fn remaining_requests(&self) -> u32 {
        self.config.max_requests.saturating_sub(self.current_requests())
    }

    /// Reset the rate limiter
    pub fn reset(&mut self) {
        self.request_times.clear();
        self.last_request_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(RateLimitConfig::default());
        assert_eq!(limiter.remaining_requests(), 100);
    }

    #[test]
    fn test_rate_limiter_can_request() {
        let config = RateLimitConfig {
            max_requests: 100,
            window_secs: 60,
            request_delay_ms: 0, // No delay for testing
        };
        let mut limiter = RateLimiter::new(config);
        assert!(limiter.can_request().is_ok());

        limiter.record_request();
        assert!(limiter.can_request().is_ok());
    }

    #[test]
    fn test_rate_limiter_tracks_requests() {
        let mut limiter = RateLimiter::new(RateLimitConfig::default());
        assert_eq!(limiter.current_requests(), 0);

        limiter.record_request();
        assert_eq!(limiter.current_requests(), 1);

        limiter.record_request();
        assert_eq!(limiter.current_requests(), 2);
    }

    #[test]
    fn test_reddit_config() {
        let config = RateLimitConfig::reddit();
        assert_eq!(config.max_requests, 60);
    }

    #[test]
    fn test_github_configs() {
        let unauth = RateLimitConfig::github_unauthenticated();
        let auth = RateLimitConfig::github_authenticated();

        assert_eq!(unauth.max_requests, 60);
        assert_eq!(auth.max_requests, 5000);
    }

    #[test]
    fn test_remaining_requests() {
        let mut limiter = RateLimiter::new(RateLimitConfig {
            max_requests: 10,
            window_secs: 60,
            request_delay_ms: 0,
        });

        assert_eq!(limiter.remaining_requests(), 10);

        limiter.record_request();
        assert_eq!(limiter.remaining_requests(), 9);

        limiter.record_request();
        assert_eq!(limiter.remaining_requests(), 8);
    }

    #[test]
    fn test_rate_limiter_reset() {
        let mut limiter = RateLimiter::new(RateLimitConfig::default());
        limiter.record_request();
        limiter.record_request();

        assert_eq!(limiter.current_requests(), 2);

        limiter.reset();
        assert_eq!(limiter.current_requests(), 0);
        assert_eq!(limiter.remaining_requests(), 100);
    }
}
