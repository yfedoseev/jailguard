//! Simplified Phase 6 binary classification network
//!
//! This is a corrected version that focuses ONLY on binary classification (injection vs benign).
//! Removes multi-task confusion and adds regularization for training stability.
//!
//! Architecture:
//! - Input: 384-dim embedding
//! - Hidden 1: 384 → 256 (ReLU)
//! - Dropout: 0.2
//! - Hidden 2: 256 → 128 (ReLU)
//! - Dropout: 0.2
//! - Output: 128 → 1 (sigmoid for binary classification)

/// Binary classification network with regularization
#[derive(Clone)]
pub struct NeuralBinaryNetwork {
    // Shared layers
    pub w_h1: Vec<Vec<f32>>, // 256 × 384
    pub b_h1: Vec<f32>,      // 256
    pub w_h2: Vec<Vec<f32>>, // 128 × 256
    pub b_h2: Vec<f32>,      // 128

    // Output layer
    pub w_out: Vec<Vec<f32>>, // 1 × 128
    pub b_out: Vec<f32>,      // 1

    pub learning_rate: f32,
    pub dropout_rate: f32, // 0.2
}

#[derive(Debug, Clone)]
pub struct NeuralForwardCache {
    pub h1: Vec<f32>,       // Hidden layer 1 activations
    pub h1_mask: Vec<bool>, // Dropout mask for h1
    pub h2: Vec<f32>,       // Hidden layer 2 activations
    pub h2_mask: Vec<bool>, // Dropout mask for h2
}

impl NeuralBinaryNetwork {
    /// Create a new binary classification network with Xavier initialization
    pub fn new(learning_rate: f32) -> Self {
        // Xavier initialization for each layer
        // For layer with fan_in inputs and fan_out outputs:
        // Initialize weights uniformly in [-sqrt(6 / (fan_in + fan_out)), sqrt(6 / (fan_in + fan_out))]

        let xavier_limit_h1 = ((6.0_f32) / (384.0_f32 + 256.0_f32)).sqrt();
        let xavier_limit_h2 = ((6.0_f32) / (256.0_f32 + 128.0_f32)).sqrt();
        let xavier_limit_out = ((6.0_f32) / (128.0_f32 + 1.0_f32)).sqrt();

        // Initialize w_h1: 256 × 384
        let w_h1 = (0..256)
            .map(|i| {
                (0..384)
                    .map(|j| {
                        // Deterministic pseudo-random from indices
                        let seed = ((i as usize * 37) ^ (j as usize * 127)) % 1000;
                        let val = seed as f32 / 1000.0;
                        (val - 0.5) * 2.0 * xavier_limit_h1
                    })
                    .collect()
            })
            .collect();

        // Initialize w_h2: 128 × 256
        let w_h2 = (0..128)
            .map(|i| {
                (0..256)
                    .map(|j| {
                        let seed = ((i as usize * 37) ^ (j as usize * 127)) % 1000;
                        let val = seed as f32 / 1000.0;
                        (val - 0.5) * 2.0 * xavier_limit_h2
                    })
                    .collect()
            })
            .collect();

        // Initialize w_out: 1 × 128
        let w_out = vec![(0..128)
            .map(|j| {
                let seed = (j as usize * 127) % 1000;
                let val = seed as f32 / 1000.0;
                (val - 0.5) * 2.0 * xavier_limit_out
            })
            .collect()];

        Self {
            w_h1,
            b_h1: vec![0.0; 256],
            w_h2,
            b_h2: vec![0.0; 128],
            w_out,
            b_out: vec![0.0],
            learning_rate,
            dropout_rate: 0.2,
        }
    }

    /// Forward pass with caching for backprop (with dropout)
    pub fn forward_train(&self, embedding: &[f32]) -> (NeuralForwardCache, f32) {
        // h1 = relu(w_h1 @ embedding + b_h1)
        let mut h1 = vec![0.0; 256];
        for i in 0..256 {
            h1[i] = self.b_h1[i];
            for j in 0..384 {
                h1[i] += self.w_h1[i][j] * embedding[j];
            }
            h1[i] = h1[i].max(0.0); // ReLU
        }

        // Apply dropout to h1 during training
        let h1_mask: Vec<bool> = (0..256)
            .map(|_| _random_bool(1.0 - self.dropout_rate))
            .collect();

        let h1_dropped: Vec<f32> = h1
            .iter()
            .zip(h1_mask.iter())
            .map(|(val, &keep)| {
                if keep {
                    val / (1.0 - self.dropout_rate)
                } else {
                    0.0
                }
            })
            .collect();

        // h2 = relu(w_h2 @ h1_dropped + b_h2)
        let mut h2 = vec![0.0; 128];
        for i in 0..128 {
            h2[i] = self.b_h2[i];
            for j in 0..256 {
                h2[i] += self.w_h2[i][j] * h1_dropped[j];
            }
            h2[i] = h2[i].max(0.0); // ReLU
        }

        // Apply dropout to h2 during training
        let h2_mask: Vec<bool> = (0..128)
            .map(|_| _random_bool(1.0 - self.dropout_rate))
            .collect();

        let h2_dropped: Vec<f32> = h2
            .iter()
            .zip(h2_mask.iter())
            .map(|(val, &keep)| {
                if keep {
                    val / (1.0 - self.dropout_rate)
                } else {
                    0.0
                }
            })
            .collect();

        // output = sigmoid(w_out @ h2_dropped + b_out)
        let mut logit = self.b_out[0];
        for j in 0..128 {
            logit += self.w_out[0][j] * h2_dropped[j];
        }

        let pred = _sigmoid(logit);

        let cache = NeuralForwardCache {
            h1: h1_dropped,
            h1_mask,
            h2: h2_dropped,
            h2_mask,
        };

        (cache, pred)
    }

    /// Forward pass without dropout (inference)
    pub fn forward_eval(&self, embedding: &[f32]) -> f32 {
        // h1 = relu(w_h1 @ embedding + b_h1)
        let mut h1 = vec![0.0; 256];
        for i in 0..256 {
            h1[i] = self.b_h1[i];
            for j in 0..384 {
                h1[i] += self.w_h1[i][j] * embedding[j];
            }
            h1[i] = h1[i].max(0.0); // ReLU
        }

        // h2 = relu(w_h2 @ h1 + b_h2)
        let mut h2 = vec![0.0; 128];
        for i in 0..128 {
            h2[i] = self.b_h2[i];
            for j in 0..256 {
                h2[i] += self.w_h2[i][j] * h1[j];
            }
            h2[i] = h2[i].max(0.0); // ReLU
        }

        // output = sigmoid(w_out @ h2 + b_out)
        let mut logit = self.b_out[0];
        for j in 0..128 {
            logit += self.w_out[0][j] * h2[j];
        }

        _sigmoid(logit)
    }

    /// Training step with binary cross-entropy loss
    pub fn train_step(&mut self, embedding: &[f32], is_injection: bool) {
        let target = if is_injection { 1.0 } else { 0.0 };

        // Forward pass with dropout
        let (cache, pred) = self.forward_train(embedding);

        // Binary cross-entropy gradient: d_loss/d_pred = pred - target
        let grad_pred = pred - target;

        // Backprop through sigmoid: d_loss/d_logit = (pred - target) * pred * (1 - pred)
        let grad_logit = grad_pred * pred * (1.0 - pred);

        // Update output layer weights and bias
        for j in 0..128 {
            self.w_out[0][j] -= self.learning_rate * grad_logit * cache.h2[j];
        }
        self.b_out[0] -= self.learning_rate * grad_logit;

        // Backprop to h2: d_loss/d_h2 = (d_loss/d_logit) * w_out
        let mut grad_h2 = vec![0.0; 128];
        for j in 0..128 {
            grad_h2[j] = grad_logit * self.w_out[0][j];
            // Apply ReLU backprop (only propagate if activation was positive)
            if cache.h2[j] <= 0.0 {
                grad_h2[j] = 0.0;
            }
            // Apply dropout mask
            if !cache.h2_mask[j] {
                grad_h2[j] = 0.0;
            }
        }

        // Update h2 layer weights and bias
        for i in 0..128 {
            for j in 0..256 {
                self.w_h2[i][j] -= self.learning_rate * grad_h2[i] * cache.h1[j];
            }
            self.b_h2[i] -= self.learning_rate * grad_h2[i];
        }

        // Backprop to h1: d_loss/d_h1 = (d_loss/d_h2) @ w_h2^T
        let mut grad_h1 = vec![0.0; 256];
        for j in 0..256 {
            for i in 0..128 {
                grad_h1[j] += grad_h2[i] * self.w_h2[i][j];
            }
            // Apply ReLU backprop
            if cache.h1[j] <= 0.0 {
                grad_h1[j] = 0.0;
            }
            // Apply dropout mask
            if !cache.h1_mask[j] {
                grad_h1[j] = 0.0;
            }
        }

        // Update h1 layer weights and bias
        for i in 0..256 {
            for j in 0..384 {
                self.w_h1[i][j] -= self.learning_rate * grad_h1[i] * embedding[j];
            }
            self.b_h1[i] -= self.learning_rate * grad_h1[i];
        }
    }

    /// Evaluate binary cross-entropy loss
    pub fn evaluate_loss(&self, embedding: &[f32], is_injection: bool) -> f32 {
        let pred = self.forward_eval(embedding);
        let target = if is_injection { 1.0 } else { 0.0 };

        // Clamp to prevent log(0)
        let pred_clamped = pred.clamp(1e-7, 1.0 - 1e-7);
        -target * pred_clamped.ln() - (1.0 - target) * (1.0 - pred_clamped).ln()
    }
}

// Utility functions
fn _sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

fn _val_seed() -> u32 {
    // Simple deterministic seed based on internal state
    // In real code, use a proper RNG
    42u32
}

fn _random_bool(probability: f32) -> bool {
    // Simple pseudo-random
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u32;
    (seed as f32 / u32::MAX as f32) < probability
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_network_creation() {
        let net = NeuralBinaryNetwork::new(0.001);
        assert_eq!(net.w_h1.len(), 256);
        assert_eq!(net.w_h2.len(), 128);
        assert_eq!(net.w_out.len(), 1);
    }

    #[test]
    fn test_forward_eval() {
        let net = NeuralBinaryNetwork::new(0.001);
        let embedding = vec![0.1; 384];
        let pred = net.forward_eval(&embedding);
        assert!(pred >= 0.0 && pred <= 1.0);
    }

    #[test]
    fn test_train_step_updates_weights() {
        let mut net = NeuralBinaryNetwork::new(0.5); // Larger learning rate for test
        let embedding = vec![1.0; 384];

        // Track multiple weight changes since initial may be zero
        let mut w_before: Vec<f32> = (0..256).map(|i| net.w_h1[i][0]).collect();

        // Train multiple steps to ensure weight changes
        for _ in 0..5 {
            net.train_step(&embedding, true);
        }

        let w_after: Vec<f32> = (0..256).map(|i| net.w_h1[i][0]).collect();

        // At least some weights should have changed
        let changed_count = w_before
            .iter()
            .zip(w_after.iter())
            .filter(|(b, a)| b != a)
            .count();
        assert!(
            changed_count > 0,
            "At least some weights should change after training"
        );
    }

    #[test]
    fn test_loss_decreases_on_convergence() {
        let mut net = NeuralBinaryNetwork::new(0.1);
        let embedding = vec![1.0; 384];

        let loss_before = net.evaluate_loss(&embedding, true);

        // Train multiple steps
        for _ in 0..10 {
            net.train_step(&embedding, true);
        }

        let loss_after = net.evaluate_loss(&embedding, true);
        assert!(loss_after < loss_before, "Loss should decrease");
    }
}
