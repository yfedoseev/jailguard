//! Semantic similarity scoring head for multi-task learning.

use burn::nn::Linear;
use burn::prelude::*;
use burn::tensor::backend::Backend;
use burn::tensor::Tensor;

/// Semantic similarity head for detecting semantic shifts in output.
///
/// Used to compare expected vs actual semantic meaning and detect
/// attempts to manipulate output semantics while evading detection.
#[derive(Module, Debug)]
pub struct SemanticSimilarityHead<B: Backend> {
    /// Projection layer to semantic space
    projection: Linear<B>,
}

impl<B: Backend> SemanticSimilarityHead<B> {
    /// Compute semantic similarity score.
    ///
    /// # Arguments
    /// * `x` - Input embedding of shape [`batch_size`, `embed_dim`]
    ///
    /// # Returns
    /// Similarity scores of shape [`batch_size`, 1]
    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        // Project to 1D score (will be passed through sigmoid later)
        self.projection.forward(x)
    }

    /// Compute cosine similarity between two embeddings.
    ///
    /// # Arguments
    /// * `embedding1` - First embedding [`batch_size`, `embed_dim`]
    /// * `embedding2` - Second embedding [`batch_size`, `embed_dim`]
    ///
    /// # Returns
    /// Cosine similarity scores [`batch_size`, 1]
    pub fn cosine_similarity(
        &self,
        embedding1: Tensor<B, 2>,
        embedding2: Tensor<B, 2>,
    ) -> Tensor<B, 2> {
        // Compute dot product: [batch_size]
        let dot = (embedding1.clone() * embedding2.clone()).sum_dim(1);

        // Compute norms: [batch_size]
        let norm1 = (embedding1.clone() * embedding1).sum_dim(1).sqrt();
        let norm2 = (embedding2.clone() * embedding2).sum_dim(1).sqrt();

        // Cosine similarity = dot / (norm1 * norm2)
        let similarity = dot / (norm1 * norm2 + 1e-8);

        // Reshape to [batch_size, 1]
        let batch_size = similarity.shape().dims[0];
        similarity.reshape([batch_size, 1])
    }
}

/// Configuration for semantic similarity head.
#[derive(Debug, Clone)]
pub struct SemanticSimilarityHeadConfig {
    /// Input dimension (embedding size)
    pub input_dim: usize,
}

impl SemanticSimilarityHeadConfig {
    /// Create a new semantic similarity head configuration.
    pub fn new(input_dim: usize) -> Self {
        Self { input_dim }
    }

    /// Initialize the semantic similarity head.
    pub fn init<B: Backend>(&self, device: &B::Device) -> SemanticSimilarityHead<B> {
        SemanticSimilarityHead {
            projection: burn::nn::LinearConfig::new(self.input_dim, 1).init(device),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use burn::tensor::TensorData;
    use burn_ndarray::NdArray;

    #[test]
    fn test_semantic_head_output_shape() {
        let device = Default::default();
        let config = SemanticSimilarityHeadConfig::new(256);
        let head = config.init::<NdArray>(&device);

        let batch_size = 4;
        let embed_dim = 256;

        let data = vec![1.0; batch_size * embed_dim];
        let x = Tensor::<NdArray, 2>::from_data(
            TensorData::new(data, [batch_size, embed_dim]),
            &device,
        );

        let output = head.forward(x);
        assert_eq!(output.shape().dims, [batch_size, 1]);
    }

    #[test]
    fn test_cosine_similarity() {
        let device = Default::default();
        let config = SemanticSimilarityHeadConfig::new(256);
        let head = config.init::<NdArray>(&device);

        let embed_dim = 256;

        // Create identical embeddings
        let data = vec![1.0; embed_dim];
        let emb1 =
            Tensor::<NdArray, 2>::from_data(TensorData::new(data.clone(), [1, embed_dim]), &device);
        let emb2 = Tensor::<NdArray, 2>::from_data(TensorData::new(data, [1, embed_dim]), &device);

        let similarity = head.cosine_similarity(emb1, emb2);
        let sim_data = similarity.to_data().to_vec::<f32>().unwrap();

        // Identical embeddings should have similarity close to 1.0
        assert!((sim_data[0] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let device = Default::default();
        let config = SemanticSimilarityHeadConfig::new(2);
        let head = config.init::<NdArray>(&device);

        // Create orthogonal embeddings [1, 0] and [0, 1]
        let emb1_data = vec![1.0, 0.0];
        let emb2_data = vec![0.0, 1.0];

        let emb1 = Tensor::<NdArray, 2>::from_data(TensorData::new(emb1_data, [1, 2]), &device);
        let emb2 = Tensor::<NdArray, 2>::from_data(TensorData::new(emb2_data, [1, 2]), &device);

        let similarity = head.cosine_similarity(emb1, emb2);
        let sim_data = similarity.to_data().to_vec::<f32>().unwrap();

        // Orthogonal embeddings should have similarity close to 0.0
        // Result is [batch_size, 1] = [1, 1]
        assert!(sim_data[0].abs() < 0.1);
    }
}
