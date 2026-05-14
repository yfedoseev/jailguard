//! Configuration for transformer models.

/// Configuration for the transformer encoder.
#[derive(Debug, Clone)]
pub struct TransformerConfig {
    /// Vocabulary size
    pub vocab_size: usize,
    /// Embedding dimension
    pub embed_dim: usize,
    /// Number of attention heads
    pub num_heads: usize,
    /// Number of encoder layers
    pub num_layers: usize,
    /// Feedforward dimension (usually 4x `embed_dim`)
    pub ff_dim: usize,
    /// Maximum sequence length
    pub max_seq_len: usize,
    /// Dropout rate
    pub dropout: f64,
}

impl TransformerConfig {
    /// Create a new transformer configuration with default values.
    ///
    /// Default: 256 dim, 4 heads, 3 layers, 1024 FF dim, 512 max length
    pub fn new(vocab_size: usize) -> Self {
        Self {
            vocab_size,
            embed_dim: 256,
            num_heads: 4,
            num_layers: 3,
            ff_dim: 1024,
            max_seq_len: 512,
            dropout: 0.1,
        }
    }

    /// Set embedding dimension.
    pub fn with_embed_dim(mut self, embed_dim: usize) -> Self {
        self.embed_dim = embed_dim;
        self
    }

    /// Set number of attention heads.
    pub fn with_num_heads(mut self, num_heads: usize) -> Self {
        self.num_heads = num_heads;
        self
    }

    /// Set number of encoder layers.
    pub fn with_num_layers(mut self, num_layers: usize) -> Self {
        self.num_layers = num_layers;
        self
    }

    /// Set feedforward dimension.
    pub fn with_ff_dim(mut self, ff_dim: usize) -> Self {
        self.ff_dim = ff_dim;
        self
    }

    /// Set maximum sequence length.
    pub fn with_max_seq_len(mut self, max_seq_len: usize) -> Self {
        self.max_seq_len = max_seq_len;
        self
    }

    /// Set dropout rate.
    pub fn with_dropout(mut self, dropout: f64) -> Self {
        self.dropout = dropout;
        self
    }

    /// Validate configuration parameters.
    pub fn validate(&self) -> Result<(), String> {
        if !self.embed_dim.is_multiple_of(self.num_heads) {
            return Err(format!(
                "embed_dim ({}) must be divisible by num_heads ({})",
                self.embed_dim, self.num_heads
            ));
        }

        if self.ff_dim == 0 {
            return Err("ff_dim must be > 0".to_string());
        }

        if self.dropout < 0.0 || self.dropout >= 1.0 {
            return Err("dropout must be in [0, 1)".to_string());
        }

        Ok(())
    }

    /// Get the head dimension (`embed_dim` / `num_heads`).
    pub fn head_dim(&self) -> usize {
        self.embed_dim / self.num_heads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = TransformerConfig::new(10000);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_invalid_head_dim() {
        let config = TransformerConfig::new(10000)
            .with_embed_dim(256)
            .with_num_heads(3); // 256 not divisible by 3
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_head_dim() {
        let config = TransformerConfig::new(10000);
        assert_eq!(config.head_dim(), 64); // 256 / 4
    }
}
