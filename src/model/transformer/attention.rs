//! Multi-head scaled dot-product attention.

use burn::nn::{Linear, LinearConfig};
use burn::prelude::*;
use burn::tensor::backend::Backend;
use burn::tensor::Tensor;

/// Multi-head scaled dot-product attention.
#[derive(Module, Debug)]
pub struct MultiHeadAttention<B: Backend> {
    /// Query projection
    q_proj: Linear<B>,
    /// Key projection
    k_proj: Linear<B>,
    /// Value projection
    v_proj: Linear<B>,
    /// Output projection
    out_proj: Linear<B>,
    /// Number of attention heads
    num_heads: usize,
    /// Dimension per head
    head_dim: usize,
    /// Embedding dimension
    embed_dim: usize,
}

impl<B: Backend> MultiHeadAttention<B> {
    /// Forward pass for multi-head attention.
    ///
    /// # Arguments
    /// * `query` - Query tensor of shape [batch_size, seq_len, embed_dim]
    /// * `key` - Key tensor of shape [batch_size, seq_len, embed_dim]
    /// * `value` - Value tensor of shape [batch_size, seq_len, embed_dim]
    /// * `mask` - Optional attention mask [batch_size, 1, seq_len, seq_len]
    ///
    /// # Returns
    /// Output tensor of shape [batch_size, seq_len, embed_dim]
    pub fn forward(
        &self,
        query: Tensor<B, 3>,
        key: Tensor<B, 3>,
        value: Tensor<B, 3>,
        mask: Option<Tensor<B, 4>>,
    ) -> Tensor<B, 3> {
        let batch_size = query.shape().dims[0];
        let seq_len = query.shape().dims[1];

        // Project to multiple heads
        // [batch_size, seq_len, embed_dim] -> [batch_size, seq_len, embed_dim]
        let q = self.q_proj.forward(query);
        let k = self.k_proj.forward(key);
        let v = self.v_proj.forward(value);

        // Reshape for multi-head: [batch_size, seq_len, num_heads, head_dim]
        // -> [batch_size, num_heads, seq_len, head_dim]
        let q = q
            .reshape([batch_size, seq_len, self.num_heads, self.head_dim])
            .permute([0, 2, 1, 3]);
        let k = k
            .reshape([batch_size, seq_len, self.num_heads, self.head_dim])
            .permute([0, 2, 1, 3]);
        let v = v
            .reshape([batch_size, seq_len, self.num_heads, self.head_dim])
            .permute([0, 2, 1, 3]);

        // Compute attention scores: Q @ K^T / sqrt(head_dim)
        // [batch_size, num_heads, seq_len, head_dim] @ [batch_size, num_heads, head_dim, seq_len]
        // -> [batch_size, num_heads, seq_len, seq_len]
        let scale = (self.head_dim as f32).sqrt();
        let scores = q.clone().matmul(k.transpose());
        let scores = scores / scale;

        // Apply mask if provided
        let scores = if let Some(attention_mask) = mask {
            // mask should be [batch_size, 1, seq_len, seq_len]
            // Broadcast and add (negative infinity for masked positions)
            scores + attention_mask
        } else {
            scores
        };

        // Apply softmax: [batch_size, num_heads, seq_len, seq_len]
        let attn_weights = softmax_dim(scores, 3);

        // Apply to values: [batch_size, num_heads, seq_len, seq_len] @ [batch_size, num_heads, seq_len, head_dim]
        // -> [batch_size, num_heads, seq_len, head_dim]
        let context = attn_weights.matmul(v);

        // Reshape back: [batch_size, num_heads, seq_len, head_dim]
        // -> [batch_size, seq_len, num_heads, head_dim]
        // -> [batch_size, seq_len, embed_dim]
        let context = context.permute([0, 2, 1, 3]);
        let context = context.reshape([batch_size, seq_len, self.embed_dim]);

        // Final output projection
        self.out_proj.forward(context)
    }

    /// Self-attention forward pass (query = key = value).
    ///
    /// # Arguments
    /// * `x` - Input tensor of shape [batch_size, seq_len, embed_dim]
    /// * `mask` - Optional attention mask
    ///
    /// # Returns
    /// Output tensor of shape [batch_size, seq_len, embed_dim]
    pub fn forward_self(&self, x: Tensor<B, 3>, mask: Option<Tensor<B, 4>>) -> Tensor<B, 3> {
        self.forward(x.clone(), x.clone(), x, mask)
    }
}

/// Compute softmax along a specified dimension (last dimension, dim=3).
fn softmax_dim<B: Backend>(tensor: Tensor<B, 4>, _dim: usize) -> Tensor<B, 4> {
    let max = tensor.clone().max_dim(3);
    let exp = (tensor - max).exp();
    let sum = exp.clone().sum_dim(3);
    exp / sum
}

/// Configuration for multi-head attention.
#[derive(Debug, Clone)]
pub struct MultiHeadAttentionConfig {
    /// Embedding dimension
    pub embed_dim: usize,
    /// Number of attention heads
    pub num_heads: usize,
}

impl MultiHeadAttentionConfig {
    /// Create a new multi-head attention configuration.
    pub fn new(embed_dim: usize, num_heads: usize) -> Self {
        Self {
            embed_dim,
            num_heads,
        }
    }

    /// Initialize the multi-head attention layer.
    pub fn init<B: Backend>(&self, device: &B::Device) -> MultiHeadAttention<B> {
        assert_eq!(
            self.embed_dim % self.num_heads,
            0,
            "embed_dim must be divisible by num_heads"
        );

        let head_dim = self.embed_dim / self.num_heads;

        MultiHeadAttention {
            q_proj: LinearConfig::new(self.embed_dim, self.embed_dim).init(device),
            k_proj: LinearConfig::new(self.embed_dim, self.embed_dim).init(device),
            v_proj: LinearConfig::new(self.embed_dim, self.embed_dim).init(device),
            out_proj: LinearConfig::new(self.embed_dim, self.embed_dim).init(device),
            num_heads: self.num_heads,
            head_dim,
            embed_dim: self.embed_dim,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::TensorData;
    use burn_ndarray::NdArray;

    #[test]
    fn test_attention_shapes() {
        let device = Default::default();
        let config = MultiHeadAttentionConfig::new(256, 4);
        let attention = config.init::<NdArray>(&device);

        let batch_size = 2;
        let seq_len = 10;
        let embed_dim = 256;

        // Create dummy tensors with ones
        let data = vec![1.0; batch_size * seq_len * embed_dim];
        let q = Tensor::<NdArray, 3>::from_data(
            TensorData::new(data.clone(), [batch_size, seq_len, embed_dim]),
            &device,
        );
        let k = Tensor::<NdArray, 3>::from_data(
            TensorData::new(data.clone(), [batch_size, seq_len, embed_dim]),
            &device,
        );
        let v = Tensor::<NdArray, 3>::from_data(
            TensorData::new(data, [batch_size, seq_len, embed_dim]),
            &device,
        );

        let output = attention.forward(q, k, v, None);
        assert_eq!(output.shape().dims, [batch_size, seq_len, embed_dim]);
    }

    #[test]
    fn test_self_attention_shapes() {
        let device = Default::default();
        let config = MultiHeadAttentionConfig::new(256, 4);
        let attention = config.init::<NdArray>(&device);

        let batch_size = 2;
        let seq_len = 10;
        let embed_dim = 256;

        let data = vec![1.0; batch_size * seq_len * embed_dim];
        let x = Tensor::<NdArray, 3>::from_data(
            TensorData::new(data, [batch_size, seq_len, embed_dim]),
            &device,
        );

        let output = attention.forward_self(x, None);
        assert_eq!(output.shape().dims, [batch_size, seq_len, embed_dim]);
    }
}
