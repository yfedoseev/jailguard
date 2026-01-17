//! Main detector implementation.

use burn::prelude::*;
use burn_ndarray::NdArray;

use super::result::{DetectionResult, InjectionRisk};
use crate::error::Result;
use crate::model::{PolicyNetwork, PolicyNetworkConfig, TextEmbedding, TextEmbeddingConfig};
use crate::tokenizer::{SimpleTokenizer, Tokenizer};

/// Backend type for CPU inference.
type B = NdArray;

/// Configuration for the detector.
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    /// Maximum sequence length
    pub max_length: usize,
    /// Embedding dimension
    pub embed_dim: usize,
    /// Hidden layer dimension
    pub hidden_dim: usize,
    /// Confidence threshold for blocking
    pub block_threshold: f32,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            max_length: 512,
            embed_dim: 128,
            hidden_dim: 256,
            block_threshold: 0.7,
        }
    }
}

/// Main detector for prompt injection detection.
pub struct Detector {
    /// Tokenizer for text processing
    tokenizer: SimpleTokenizer,
    /// Text embedding layer
    embedding: TextEmbedding<B>,
    /// Policy network for action selection
    policy: PolicyNetwork<B>,
    /// Configuration
    config: DetectorConfig,
    /// Device for computation
    device: <B as Backend>::Device,
}

impl Detector {
    /// Create a new detector with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(DetectorConfig::default())
    }

    /// Create a new detector with custom configuration.
    #[allow(clippy::unnecessary_wraps)]
    pub fn with_config(config: DetectorConfig) -> Result<Self> {
        let device = Default::default();

        // Initialize tokenizer
        let tokenizer = SimpleTokenizer::new();

        // Initialize embedding layer
        let embed_config = TextEmbeddingConfig::new(10000, config.embed_dim, config.max_length);
        let embedding = embed_config.init(&device);

        // Initialize policy network
        let policy_config = PolicyNetworkConfig::new(config.embed_dim, config.hidden_dim);
        let policy = policy_config.init(&device);

        Ok(Self {
            tokenizer,
            embedding,
            policy,
            config,
            device,
        })
    }

    /// Load a pre-trained detector.
    #[cfg(feature = "pretrained")]
    pub fn pretrained(name: &str) -> Result<Self> {
        crate::pretrained::load_detector(name)
    }

    /// Detect if text contains a prompt injection.
    pub fn detect(&self, text: &str) -> DetectionResult {
        use burn::tensor::TensorData;

        // Tokenize
        let tokens = self.tokenizer.tokenize(text);
        let seq_len = tokens.len();

        // Convert to tensor
        let tokens_i64: Vec<i64> = tokens.iter().map(|&t| t as i64).collect();
        let tokens_tensor =
            Tensor::<B, 2, Int>::from_data(TensorData::new(tokens_i64, [1, seq_len]), &self.device);

        // Get embeddings
        let embeddings = self.embedding.forward(tokens_tensor); // [1, seq_len, embed_dim]

        // Mean pooling over sequence - squeeze and reshape to [1, embed_dim]
        let pooled = embeddings.mean_dim(1).reshape([1, self.config.embed_dim]);

        // Get action probabilities
        let action_probs = self.policy.forward(pooled);
        let probs_data: Vec<f32> = action_probs.to_data().to_vec().unwrap();

        // Action 0 = Block (injection detected), Action 1 = Allow
        let block_prob = probs_data[0];
        let allow_prob = probs_data[1];

        let is_injection = block_prob > allow_prob;
        let confidence = if is_injection { block_prob } else { allow_prob };

        DetectionResult::new(is_injection, confidence, [block_prob, allow_prob])
    }

    /// Detect with detailed analysis.
    pub fn detect_detailed(&self, text: &str) -> (DetectionResult, DetectionAnalysis) {
        let result = self.detect(text);

        let analysis = DetectionAnalysis {
            text_length: text.len(),
            token_count: self.tokenizer.tokenize(text).len(),
            risk_level: result.risk_level,
            should_block: result.should_block(self.config.block_threshold),
        };

        (result, analysis)
    }

    /// Get the configuration.
    pub fn config(&self) -> &DetectorConfig {
        &self.config
    }

    /// Build vocabulary from sample texts.
    pub fn build_vocab(&self, texts: &[&str]) {
        self.tokenizer.build_vocab(texts, 1);
    }
}

impl Default for Detector {
    fn default() -> Self {
        Self::new().expect("Failed to create default detector")
    }
}

/// Detailed analysis of a detection.
#[derive(Debug, Clone)]
pub struct DetectionAnalysis {
    /// Length of input text in characters
    pub text_length: usize,
    /// Number of tokens after tokenization
    pub token_count: usize,
    /// Risk level
    pub risk_level: InjectionRisk,
    /// Whether this would be blocked with current threshold
    pub should_block: bool,
}
