//! Text embedding layer for converting token sequences to dense vectors.

use burn::nn::{Embedding, EmbeddingConfig, LayerNorm, LayerNormConfig};
use burn::prelude::*;
use burn::tensor::{backend::Backend, Int, Tensor};

/// Text embedding layer that converts token IDs to dense vectors.
#[derive(Module, Debug)]
pub struct TextEmbedding<B: Backend> {
    /// Token embedding lookup table
    token_embedding: Embedding<B>,
    /// Positional encoding
    position_embedding: Embedding<B>,
    /// Layer normalization
    layer_norm: LayerNorm<B>,
    /// Embedding dimension
    embed_dim: usize,
}

/// Configuration for text embedding layer.
#[derive(Debug, Clone)]
pub struct TextEmbeddingConfig {
    /// Vocabulary size
    pub vocab_size: usize,
    /// Embedding dimension
    pub embed_dim: usize,
    /// Maximum sequence length
    pub max_length: usize,
}

impl TextEmbeddingConfig {
    /// Create a new configuration.
    pub fn new(vocab_size: usize, embed_dim: usize, max_length: usize) -> Self {
        Self {
            vocab_size,
            embed_dim,
            max_length,
        }
    }

    /// Initialize the embedding layer.
    pub fn init<B: Backend>(&self, device: &B::Device) -> TextEmbedding<B> {
        let token_embedding = EmbeddingConfig::new(self.vocab_size, self.embed_dim).init(device);
        let position_embedding = EmbeddingConfig::new(self.max_length, self.embed_dim).init(device);
        let layer_norm = LayerNormConfig::new(self.embed_dim).init(device);

        TextEmbedding {
            token_embedding,
            position_embedding,
            layer_norm,
            embed_dim: self.embed_dim,
        }
    }
}

impl<B: Backend> TextEmbedding<B> {
    /// Forward pass: convert token IDs to embeddings.
    ///
    /// # Arguments
    /// * `tokens` - Token IDs tensor of shape [`batch_size`, `seq_len`]
    ///
    /// # Returns
    /// Embedded tensor of shape [`batch_size`, `seq_len`, `embed_dim`]
    pub fn forward(&self, tokens: Tensor<B, 2, Int>) -> Tensor<B, 3> {
        let [batch_size, seq_len] = tokens.dims();
        let device = tokens.device();

        // Get token embeddings
        let token_embeds = self.token_embedding.forward(tokens);

        // Create position indices [0, 1, 2, ..., seq_len-1]
        let positions: Vec<i64> = (0..seq_len as i64).collect();
        let position_ids = Tensor::<B, 1, Int>::from_data(
            burn::tensor::TensorData::new(positions, [seq_len]),
            &device,
        )
        .unsqueeze::<2>()
        .expand([batch_size, seq_len]);

        // Get position embeddings
        let position_embeds = self.position_embedding.forward(position_ids);

        // Combine embeddings
        let embeddings = token_embeds + position_embeds;

        // Apply layer norm
        self.layer_norm.forward(embeddings)
    }

    /// Get the embedding dimension.
    pub fn embed_dim(&self) -> usize {
        self.embed_dim
    }
}
