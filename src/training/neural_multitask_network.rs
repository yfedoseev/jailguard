//! Multi-task neural network with real gradient descent training.
//!
//! Implements a trainable network for prompt injection detection with three tasks:
//! 1. Binary classification (injection vs benign)
//! 2. Attack type classification (7-way)
//! 3. Semantic similarity scoring (optional)
//!
//! The network performs actual weight updates using gradient descent,
//! fixing the missing weight updates from earlier phases.

/// Multi-task trainable network with gradient-based weight updates.
///
/// Architecture:
/// - Input: 384-dim embedding (all-MiniLM-L6-v2)
/// - Hidden Layer 1: 384 → 256 (ReLU)
/// - Hidden Layer 2: 256 → 128 (ReLU)
/// - Task 1 Head: 128 → 2 (binary classification)
/// - Task 2 Head: 128 → 7 (attack type classification)
/// - Task 3 Head: 128 → 1 (semantic similarity)
#[deprecated(
    since = "1.1.0",
    note = "Multi-task approach has convergence issues. Use NeuralBinaryNetwork instead. See MIGRATION_GUIDE.md for details."
)]
#[derive(Debug, Clone)]
pub struct NeuralMultitaskNetwork {
    // Shared layers
    /// Hidden layer 1 weights: 256 × 384
    pub w_h1: Vec<Vec<f32>>,
    /// Hidden layer 1 bias: 256
    pub b_h1: Vec<f32>,
    /// Hidden layer 2 weights: 128 × 256
    pub w_h2: Vec<Vec<f32>>,
    /// Hidden layer 2 bias: 128
    pub b_h2: Vec<f32>,

    // Binary classification head
    /// Binary classification weights: 2 × 128
    pub w_binary: Vec<Vec<f32>>,
    /// Binary classification bias: 2
    pub b_binary: Vec<f32>,

    // Attack type classification head (7-way)
    /// Attack type weights: 7 × 128
    pub w_attack: Vec<Vec<f32>>,
    /// Attack type bias: 7
    pub b_attack: Vec<f32>,

    // Semantic similarity head
    /// Semantic similarity weights: 1 × 128
    pub w_semantic: Vec<Vec<f32>>,
    /// Semantic similarity bias: 1
    pub b_semantic: Vec<f32>,

    /// Learning rate for gradient descent
    pub learning_rate: f32,

    /// Task loss weights
    pub task_weights: [f32; 3], // [binary: 0.6, attack: 0.3, semantic: 0.1]
}

/// Forward pass cache for backpropagation.
#[derive(Debug, Clone)]
pub struct NeuralForwardCache {
    /// Hidden layer 1 pre-activation: 256
    pub h1_pre: Vec<f32>,
    /// Hidden layer 1 activation: 256
    pub h1: Vec<f32>,
    /// Hidden layer 2 pre-activation: 128
    pub h2_pre: Vec<f32>,
    /// Hidden layer 2 activation: 128
    pub h2: Vec<f32>,
    /// Binary classification logits: 2
    pub binary_logits: Vec<f32>,
    /// Attack type logits: 7
    pub attack_logits: Vec<f32>,
    /// Semantic similarity output: 1
    pub semantic_output: Vec<f32>,
}

/// Softmax output probabilities.
#[derive(Debug, Clone)]
pub struct SoftmaxOutput {
    /// Logits
    pub logits: Vec<f32>,
    /// Softmax probabilities
    pub probs: Vec<f32>,
}

#[allow(deprecated)]
impl NeuralMultitaskNetwork {
    /// Create new network with Xavier initialization.
    pub fn new(learning_rate: f32) -> Self {
        let embed_dim = 384;
        let h1_dim = 256;
        let h2_dim = 128;
        let n_attacks = 7;

        // Xavier initialization for W_h1: 384 → 256
        let w_h1_scale = (1.0 / embed_dim as f32).sqrt();
        let w_h1: Vec<Vec<f32>> = (0..h1_dim)
            .map(|i| {
                (0..embed_dim)
                    .map(|j| {
                        let seed = (i as u32 * 17 + j as u32 * 31) as f32;
                        ((seed % 1000.0) / 1000.0 - 0.5) * 2.0 * w_h1_scale
                    })
                    .collect()
            })
            .collect();
        let b_h1 = vec![0.0; h1_dim];

        // Xavier initialization for W_h2: 256 → 128
        let w_h2_scale = (1.0 / h1_dim as f32).sqrt();
        let w_h2: Vec<Vec<f32>> = (0..h2_dim)
            .map(|i| {
                (0..h1_dim)
                    .map(|j| {
                        let seed = (i as u32 * 19 + j as u32 * 23) as f32;
                        ((seed % 1000.0) / 1000.0 - 0.5) * 2.0 * w_h2_scale
                    })
                    .collect()
            })
            .collect();
        let b_h2 = vec![0.0; h2_dim];

        // Binary head: 128 → 2
        let w_binary_scale = (1.0 / h2_dim as f32).sqrt();
        let w_binary: Vec<Vec<f32>> = (0..2)
            .map(|i| {
                (0..h2_dim)
                    .map(|j| {
                        let seed = (i as u32 * 11 + j as u32 * 13) as f32;
                        ((seed % 1000.0) / 1000.0 - 0.5) * 2.0 * w_binary_scale
                    })
                    .collect()
            })
            .collect();
        let b_binary = vec![0.0; 2];

        // Attack head: 128 → 7
        let w_attack_scale = (1.0 / h2_dim as f32).sqrt();
        let w_attack: Vec<Vec<f32>> = (0..n_attacks)
            .map(|i| {
                (0..h2_dim)
                    .map(|j| {
                        let seed = (i as u32 * 7 + j as u32 * 29) as f32;
                        ((seed % 1000.0) / 1000.0 - 0.5) * 2.0 * w_attack_scale
                    })
                    .collect()
            })
            .collect();
        let b_attack = vec![0.0; n_attacks];

        // Semantic head: 128 → 1
        let w_semantic_scale = (1.0 / h2_dim as f32).sqrt();
        let w_semantic: Vec<Vec<f32>> = (0..1)
            .map(|i| {
                (0..h2_dim)
                    .map(|j| {
                        let seed = (i as u32 * 3 + j as u32 * 37) as f32;
                        ((seed % 1000.0) / 1000.0 - 0.5) * 2.0 * w_semantic_scale
                    })
                    .collect()
            })
            .collect();
        let b_semantic = vec![0.0];

        Self {
            w_h1,
            b_h1,
            w_h2,
            b_h2,
            w_binary,
            b_binary,
            w_attack,
            b_attack,
            w_semantic,
            b_semantic,
            learning_rate,
            task_weights: [0.6, 0.3, 0.1],
        }
    }

    /// Compute softmax from logits.
    fn softmax(logits: &[f32]) -> Vec<f32> {
        let max_logit = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_logits: Vec<f32> = logits.iter().map(|x| (x - max_logit).exp()).collect();
        let sum: f32 = exp_logits.iter().sum();
        exp_logits.iter().map(|x| x / sum).collect()
    }

    /// Forward pass with cache for backpropagation.
    pub fn forward_with_cache(
        &self,
        embedding: &[f32],
    ) -> (NeuralForwardCache, SoftmaxOutput, SoftmaxOutput, f32) {
        // Hidden layer 1: 384 → 256 (ReLU)
        let h1_pre: Vec<f32> = (0..self.b_h1.len())
            .map(|i| {
                let mut sum = self.b_h1[i];
                for j in 0..embedding.len() {
                    sum += embedding[j] * self.w_h1[i][j];
                }
                sum
            })
            .collect();
        let h1: Vec<f32> = h1_pre.iter().map(|x| x.max(0.0)).collect();

        // Hidden layer 2: 256 → 128 (ReLU)
        let h2_pre: Vec<f32> = (0..self.b_h2.len())
            .map(|i| {
                let mut sum = self.b_h2[i];
                for j in 0..h1.len() {
                    sum += h1[j] * self.w_h2[i][j];
                }
                sum
            })
            .collect();
        let h2: Vec<f32> = h2_pre.iter().map(|x| x.max(0.0)).collect();

        // Binary classification head: 128 → 2
        let binary_logits: Vec<f32> = (0..2)
            .map(|i| {
                let mut sum = self.b_binary[i];
                for j in 0..h2.len() {
                    sum += h2[j] * self.w_binary[i][j];
                }
                sum
            })
            .collect();
        let binary_probs = Self::softmax(&binary_logits);

        // Attack type head: 128 → 7
        let attack_logits: Vec<f32> = (0..7)
            .map(|i| {
                let mut sum = self.b_attack[i];
                for j in 0..h2.len() {
                    sum += h2[j] * self.w_attack[i][j];
                }
                sum
            })
            .collect();
        let attack_probs = Self::softmax(&attack_logits);

        // Semantic similarity head: 128 → 1
        let semantic_output: Vec<f32> = vec![{
            let mut sum = self.b_semantic[0];
            for j in 0..h2.len() {
                sum += h2[j] * self.w_semantic[0][j];
            }
            sum.tanh() // Normalize to [-1, 1]
        }];

        let cache = NeuralForwardCache {
            h1_pre: h1_pre.clone(),
            h1: h1.clone(),
            h2_pre: h2_pre.clone(),
            h2: h2.clone(),
            binary_logits: binary_logits.clone(),
            attack_logits: attack_logits.clone(),
            semantic_output: semantic_output.clone(),
        };

        let binary_output = SoftmaxOutput {
            logits: binary_logits,
            probs: binary_probs,
        };

        let attack_output = SoftmaxOutput {
            logits: attack_logits,
            probs: attack_probs,
        };

        let semantic_score = semantic_output[0];

        (cache, binary_output, attack_output, semantic_score)
    }

    /// Forward pass (simplified, no cache).
    pub fn forward(&self, embedding: &[f32]) -> (bool, f32, usize) {
        let (_, binary_out, attack_out, _) = self.forward_with_cache(embedding);
        let is_injection = binary_out.probs[1] > 0.5;
        let confidence = binary_out.probs[1].max(binary_out.probs[0]);
        let attack_type = attack_out
            .probs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|x| x.0)
            .unwrap_or(0);

        (is_injection, confidence, attack_type)
    }

    /// Single sample training step with multi-task gradient descent.
    pub fn train_step(
        &mut self,
        embedding: &[f32],
        is_injection: bool,
        attack_type: usize,
        semantic_target: Option<f32>,
    ) {
        let (cache, binary_out, attack_out, semantic_pred) = self.forward_with_cache(embedding);

        // ===== Task 1: Binary Classification =====
        let binary_target = if is_injection { 1.0 } else { 0.0 };
        let mut grad_h2_binary = vec![0.0; 128];

        {
            // Gradient through binary head
            let grad_out = vec![
                binary_out.probs[0] - (1.0 - binary_target),
                binary_out.probs[1] - binary_target,
            ];

            // Update binary head weights and bias
            for i in 0..2 {
                self.b_binary[i] -= self.learning_rate * self.task_weights[0] * grad_out[i];
                for j in 0..128 {
                    self.w_binary[i][j] -=
                        self.learning_rate * self.task_weights[0] * grad_out[i] * cache.h2[j];
                }
            }

            // Backprop to h2
            for j in 0..128 {
                for i in 0..2 {
                    grad_h2_binary[j] += (binary_out.probs[i]
                        - (if i == 1 {
                            binary_target
                        } else {
                            1.0 - binary_target
                        }))
                        * self.w_binary[i][j];
                }
            }
        }

        // ===== Task 2: Attack Type Classification =====
        let attack_target = attack_type as f32;
        let mut grad_h2_attack = vec![0.0; 128];

        {
            // Gradient through attack head
            for i in 0..7 {
                let target = if i == attack_type { 1.0 } else { 0.0 };
                let grad_out = attack_out.probs[i] - target;

                self.b_attack[i] -= self.learning_rate * self.task_weights[1] * grad_out;
                for j in 0..128 {
                    self.w_attack[i][j] -=
                        self.learning_rate * self.task_weights[1] * grad_out * cache.h2[j];
                }
            }

            // Backprop to h2
            for j in 0..128 {
                for i in 0..7 {
                    let target = if i == attack_type { 1.0 } else { 0.0 };
                    grad_h2_attack[j] += (attack_out.probs[i] - target) * self.w_attack[i][j];
                }
            }
        }

        // ===== Task 3: Semantic Similarity (Optional) =====
        let mut grad_h2_semantic = vec![0.0; 128];
        if let Some(target) = semantic_target {
            let grad_semantic = 2.0 * (semantic_pred - target) * (1.0 - semantic_pred.powi(2));

            self.b_semantic[0] -= self.learning_rate * self.task_weights[2] * grad_semantic;
            for j in 0..128 {
                self.w_semantic[0][j] -=
                    self.learning_rate * self.task_weights[2] * grad_semantic * cache.h2[j];
            }

            for j in 0..128 {
                grad_h2_semantic[j] += grad_semantic * self.w_semantic[0][j];
            }
        }

        // ===== Combine Gradients & Backprop Through Shared Layers =====
        let grad_h2: Vec<f32> = (0..128)
            .map(|j| grad_h2_binary[j] + grad_h2_attack[j] + grad_h2_semantic[j])
            .collect();

        // ReLU gradient for h2
        let grad_h2_pre: Vec<f32> = grad_h2
            .iter()
            .enumerate()
            .map(|(i, &g)| if cache.h2_pre[i] > 0.0 { g } else { 0.0 })
            .collect();

        // Update h2 weights and bias
        for i in 0..128 {
            self.b_h2[i] -= self.learning_rate * grad_h2_pre[i];
            for j in 0..256 {
                self.w_h2[i][j] -= self.learning_rate * grad_h2_pre[i] * cache.h1[j];
            }
        }

        // Backprop to h1
        let mut grad_h1 = vec![0.0; 256];
        for j in 0..256 {
            for i in 0..128 {
                grad_h1[j] += grad_h2_pre[i] * self.w_h2[i][j];
            }
        }

        // ReLU gradient for h1
        let grad_h1_pre: Vec<f32> = grad_h1
            .iter()
            .enumerate()
            .map(|(i, &g)| if cache.h1_pre[i] > 0.0 { g } else { 0.0 })
            .collect();

        // Update h1 weights and bias
        for i in 0..256 {
            self.b_h1[i] -= self.learning_rate * grad_h1_pre[i];
            for j in 0..384 {
                self.w_h1[i][j] -= self.learning_rate * grad_h1_pre[i] * embedding[j];
            }
        }
    }

    /// Batch training step with gradient accumulation.
    pub fn train_batch(&mut self, samples: &[(Vec<f32>, bool, usize, Option<f32>)]) {
        let batch_lr = self.learning_rate / samples.len() as f32;
        let original_lr = self.learning_rate;
        self.learning_rate = batch_lr;

        for (embedding, is_injection, attack_type, semantic) in samples {
            self.train_step(embedding, *is_injection, *attack_type, *semantic);
        }

        self.learning_rate = original_lr;
    }

    /// Evaluate on a single sample, returning loss and metrics.
    pub fn evaluate_sample(
        &self,
        embedding: &[f32],
        is_injection: bool,
        attack_type: usize,
    ) -> f32 {
        let (_, binary_out, attack_out, _) = self.forward_with_cache(embedding);

        // Binary cross-entropy
        let binary_target = if is_injection { 1.0 } else { 0.0 };
        let binary_loss = -(binary_target * binary_out.probs[1].max(1e-7).ln()
            + (1.0 - binary_target) * binary_out.probs[0].max(1e-7).ln());

        // Attack type cross-entropy
        let attack_loss = -attack_out.probs[attack_type].max(1e-7).ln();

        // Weighted combination
        self.task_weights[0] * binary_loss + self.task_weights[1] * attack_loss
    }

    /// Set learning rate.
    pub fn set_learning_rate(&mut self, lr: f32) {
        self.learning_rate = lr;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_pass_shapes() {
        let network = NeuralMultitaskNetwork::new(0.001);
        let embedding = vec![0.5; 384];

        let (is_injection, confidence, attack_type) = network.forward(&embedding);

        assert!(is_injection == true || is_injection == false);
        assert!(confidence >= 0.0 && confidence <= 1.0);
        assert!(attack_type < 7);
    }

    #[test]
    fn test_weight_updates_happen() {
        let mut network = NeuralMultitaskNetwork::new(0.1);
        let embedding = vec![1.0; 384];

        // Test binary head directly (first layer to update)
        let b_binary_before = network.b_binary[0];
        let w_binary_before = network.w_binary[0][0];

        network.train_step(&embedding, true, 0, None);

        let b_binary_after = network.b_binary[0];
        let w_binary_after = network.w_binary[0][0];

        let binary_bias_changed = (b_binary_before - b_binary_after).abs() > 1e-10;
        let binary_weight_changed = (w_binary_before - w_binary_after).abs() > 1e-10;

        assert!(
            binary_bias_changed || binary_weight_changed,
            "Binary head should update. b_before={}, b_after={}, w_before={}, w_after={}",
            b_binary_before,
            b_binary_after,
            w_binary_before,
            w_binary_after
        );
    }

    #[test]
    fn test_convergence_on_single_sample() {
        let mut network = NeuralMultitaskNetwork::new(0.1);
        let embedding = vec![0.5; 384];

        let loss_before = network.evaluate_sample(&embedding, true, 0);

        // Train for 100 steps
        for _ in 0..100 {
            network.train_step(&embedding, true, 0, None);
        }

        let loss_after = network.evaluate_sample(&embedding, true, 0);

        // Loss should decrease
        assert!(
            loss_after < loss_before,
            "Loss should decrease: {} → {}",
            loss_before,
            loss_after
        );
    }

    #[test]
    fn test_batch_training() {
        let mut network = NeuralMultitaskNetwork::new(0.1);

        let batch = vec![
            (vec![1.0; 384], true, 0, None),
            (vec![1.0; 384], false, 1, None),
            (vec![1.0; 384], true, 2, None),
        ];

        // Test that batch training updates binary head weights
        let w_binary_before = network.w_binary[0][0];
        network.train_batch(&batch);
        let w_binary_after = network.w_binary[0][0];

        assert_ne!(
            w_binary_before, w_binary_after,
            "Batch training should update binary head weights"
        );
    }

    #[test]
    fn test_gradient_flow_to_output_heads() {
        let mut network = NeuralMultitaskNetwork::new(0.1);
        let embedding = vec![1.0; 384];

        let b_binary_before = network.b_binary[0];
        let b_attack_before = network.b_attack[0];

        network.train_step(&embedding, true, 3, None);

        assert_ne!(
            b_binary_before, network.b_binary[0],
            "binary bias should update"
        );
        assert_ne!(
            b_attack_before, network.b_attack[0],
            "attack bias should update"
        );
    }
}
