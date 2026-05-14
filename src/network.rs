//! Inference-only neural network for the embedded detector.
//!
//! Contains the minimal code for forward inference:
//! - [`NeuralBinaryNetwork`]: 384→256→128→1 MLP with `ReLU` activations
//!
//! No training code, no filesystem access, no heavy dependencies.

use serde::{Deserialize, Serialize};

// ============================================================================
// NeuralBinaryNetwork — inference only
// ============================================================================

/// Binary classification network (inference only).
///
/// Architecture: 384→256 (`ReLU`) → 128 (`ReLU`) → 1 (sigmoid)
///
/// Deserialized from the embedded JSON model weights.
#[derive(Clone, Serialize, Deserialize)]
pub struct NeuralBinaryNetwork {
    /// Hidden layer 1 weights (256 × 384)
    pub w_h1: Vec<Vec<f32>>,
    /// Hidden layer 1 biases (256)
    pub b_h1: Vec<f32>,
    /// Hidden layer 2 weights (128 × 256)
    pub w_h2: Vec<Vec<f32>>,
    /// Hidden layer 2 biases (128)
    pub b_h2: Vec<f32>,
    /// Output layer weights (1 × 128)
    pub w_out: Vec<Vec<f32>>,
    /// Output layer biases (1)
    pub b_out: Vec<f32>,

    /// Learning rate (stored in model JSON, unused at inference)
    pub learning_rate: f32,
    /// Dropout rate (stored in model JSON, unused at inference)
    pub dropout_rate: f32,
}

impl NeuralBinaryNetwork {
    /// Forward pass without dropout (inference mode).
    ///
    /// Returns a probability in `[0.0, 1.0]` where higher means more likely injection.
    pub fn forward_eval(&self, embedding: &[f32]) -> f32 {
        // h1 = relu(w_h1 @ embedding + b_h1)
        let mut h1 = vec![0.0; 256];
        for (i, h1_val) in h1.iter_mut().enumerate() {
            *h1_val = self.b_h1[i];
            for (j, emb_val) in embedding.iter().enumerate().take(384) {
                *h1_val += self.w_h1[i][j] * emb_val;
            }
            *h1_val = h1_val.max(0.0); // ReLU
        }

        // h2 = relu(w_h2 @ h1 + b_h2)
        let mut h2 = vec![0.0; 128];
        for (i, h2_val) in h2.iter_mut().enumerate() {
            *h2_val = self.b_h2[i];
            for (j, h1_val) in h1.iter().enumerate() {
                *h2_val += self.w_h2[i][j] * h1_val;
            }
            *h2_val = h2_val.max(0.0); // ReLU
        }

        // output = sigmoid(w_out @ h2 + b_out)
        let mut logit = self.b_out[0];
        for (j, h2_val) in h2.iter().enumerate() {
            logit += self.w_out[0][j] * h2_val;
        }

        sigmoid(logit)
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}
