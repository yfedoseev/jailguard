//! Configuration for inference optimization.

/// Configuration for inference optimization
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Maximum batch size for inference
    pub max_batch_size: usize,
    /// Timeout in milliseconds for batch processing
    pub batch_timeout_ms: u64,
    /// Enable caching of results
    pub enable_caching: bool,
    /// Maximum cache size in number of entries
    pub max_cache_size: usize,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// Enable model quantization (future)
    pub enable_quantization: bool,
    /// Device to use: "cpu" or "gpu"
    pub device: String,
    /// Verbosity level: 0=silent, 1=errors, 2=warnings, 3=info
    pub verbosity: u8,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            batch_timeout_ms: 100,
            enable_caching: true,
            max_cache_size: 1000,
            cache_ttl_secs: 3600, // 1 hour
            enable_quantization: false,
            device: "cpu".to_string(),
            verbosity: 1,
        }
    }
}

impl InferenceConfig {
    /// Create a new inference config with custom batch size
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    /// Enable or disable caching
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.enable_caching = enabled;
        self
    }

    /// Set cache size
    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = size;
        self
    }

    /// Set batch timeout
    pub fn with_batch_timeout(mut self, ms: u64) -> Self {
        self.batch_timeout_ms = ms;
        self
    }

    /// Set device (cpu or gpu)
    pub fn with_device(mut self, device: String) -> Self {
        self.device = device;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_batch_size == 0 {
            return Err("max_batch_size must be > 0".to_string());
        }

        if self.batch_timeout_ms == 0 {
            return Err("batch_timeout_ms must be > 0".to_string());
        }

        if self.max_cache_size == 0 && self.enable_caching {
            return Err("max_cache_size must be > 0 if caching is enabled".to_string());
        }

        if self.device != "cpu" && self.device != "gpu" {
            return Err("device must be 'cpu' or 'gpu'".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = InferenceConfig::default();
        assert_eq!(config.max_batch_size, 32);
        assert!(config.enable_caching);
        assert_eq!(config.device, "cpu");
    }

    #[test]
    fn test_config_builder() {
        let config = InferenceConfig::default()
            .with_batch_size(64)
            .with_device("gpu".to_string())
            .with_caching(false);

        assert_eq!(config.max_batch_size, 64);
        assert_eq!(config.device, "gpu");
        assert!(!config.enable_caching);
    }

    #[test]
    fn test_config_validation() {
        let valid = InferenceConfig::default();
        assert!(valid.validate().is_ok());

        let invalid = InferenceConfig {
            max_batch_size: 0,
            ..Default::default()
        };
        assert!(invalid.validate().is_err());

        let invalid_device = InferenceConfig {
            device: "tpu".to_string(),
            ..Default::default()
        };
        assert!(invalid_device.validate().is_err());
    }
}
