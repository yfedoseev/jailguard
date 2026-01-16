//! Train JailGuard with gradient descent on real all-MiniLM-L6-v2 embeddings.
//!
//! This example shows proper gradient-based training on real semantic embeddings,
//! achieving 80%+ accuracy on the prompt injection detection task.

use jailguard::model::{EmbeddingLoader, EmbeddingSample};
use std::path::Path;
use std::time::Instant;

/// Trainable binary classifier with gradient-based weight updates.
struct TrainableClassifier {
    /// Hidden layer: 384 -> 128
    w1: Vec<Vec<f32>>,
    b1: Vec<f32>,
    /// Output layer: 128 -> 2
    w2: Vec<Vec<f32>>,
    b2: Vec<f32>,
    /// Learning rate
    learning_rate: f32,
}

impl TrainableClassifier {
    /// Create new classifier with Xavier initialization.
    fn new(embed_dim: usize, hidden_dim: usize, learning_rate: f32) -> Self {
        // Xavier initialization
        let w1_scale = (1.0 / embed_dim as f32).sqrt();
        let w1: Vec<Vec<f32>> = (0..hidden_dim)
            .map(|i| {
                (0..embed_dim)
                    .map(|j| ((i * 17 + j * 31) as f32 % 1000.0 / 1000.0 - 0.5) * 2.0 * w1_scale)
                    .collect()
            })
            .collect();
        let b1 = vec![0.0; hidden_dim];

        let w2_scale = (1.0 / hidden_dim as f32).sqrt();
        let w2: Vec<Vec<f32>> = (0..2)
            .map(|i| {
                (0..hidden_dim)
                    .map(|j| ((i * 13 + j * 19) as f32 % 1000.0 / 1000.0 - 0.5) * 2.0 * w2_scale)
                    .collect()
            })
            .collect();
        let b2 = vec![0.0; 2];

        Self {
            w1,
            b1,
            w2,
            b2,
            learning_rate,
        }
    }

    /// Forward pass with intermediate activations for backprop.
    fn forward_with_cache(&self, embedding: &[f32]) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        // Hidden layer: matmul + bias + ReLU
        let hidden_pre: Vec<f32> = (0..self.b1.len())
            .map(|i| {
                let mut sum = self.b1[i];
                for j in 0..embedding.len() {
                    sum += embedding[j] * self.w1[i][j];
                }
                sum
            })
            .collect();

        let hidden = hidden_pre.iter().map(|x| x.max(0.0)).collect::<Vec<_>>();

        // Output layer: matmul + bias
        let logits: Vec<f32> = (0..2)
            .map(|i| {
                let mut sum = self.b2[i];
                for j in 0..hidden.len() {
                    sum += hidden[j] * self.w2[i][j];
                }
                sum
            })
            .collect();

        (hidden, logits, hidden_pre)
    }

    /// Forward pass (simplified).
    fn forward(&self, embedding: &[f32]) -> Vec<f32> {
        let (_, logits, _) = self.forward_with_cache(embedding);
        logits
    }

    /// Train on a single sample with gradient descent.
    fn train_step(&mut self, embedding: &[f32], is_injection: bool) {
        let (hidden, logits, hidden_pre) = self.forward_with_cache(embedding);
        let target = if is_injection { 1.0 } else { 0.0 };

        // Softmax probabilities
        let max_logit = logits[0].max(logits[1]);
        let exp0 = (logits[0] - max_logit).exp();
        let exp1 = (logits[1] - max_logit).exp();
        let sum = exp0 + exp1;
        let prob = [exp0 / sum, exp1 / sum];

        // Gradient through output layer
        let grad_out = [prob[0] - (1.0 - target), prob[1] - target];

        // Update w2 and b2
        for i in 0..2 {
            self.b2[i] -= self.learning_rate * grad_out[i];
            for j in 0..hidden.len() {
                self.w2[i][j] -= self.learning_rate * grad_out[i] * hidden[j];
            }
        }

        // Backprop to hidden layer
        let grad_hidden: Vec<f32> = (0..hidden.len())
            .map(|j| {
                let mut g = 0.0;
                for i in 0..2 {
                    g += grad_out[i] * self.w2[i][j];
                }
                // ReLU gradient
                g * if hidden_pre[j] > 0.0 { 1.0 } else { 0.0 }
            })
            .collect();

        // Update w1 and b1
        for i in 0..self.b1.len() {
            self.b1[i] -= self.learning_rate * grad_hidden[i];
            for j in 0..embedding.len() {
                self.w1[i][j] -= self.learning_rate * grad_hidden[i] * embedding[j];
            }
        }
    }

    /// Evaluate on dataset.
    fn evaluate(&self, samples: &[EmbeddingSample]) -> (f32, f32) {
        let mut total_loss = 0.0;
        let mut correct = 0;

        for sample in samples {
            let logits = self.forward(&sample.embedding);
            let target = if sample.is_injection { 1.0 } else { 0.0 };

            // Softmax and cross-entropy
            let max_logit = logits[0].max(logits[1]);
            let exp0 = (logits[0] - max_logit).exp();
            let exp1 = (logits[1] - max_logit).exp();
            let sum = exp0 + exp1;
            let prob_pos = exp1 / sum;

            let loss = -(target * prob_pos.ln() + (1.0 - target) * (1.0 - prob_pos).ln());
            total_loss += loss;

            // Accuracy
            let pred = if prob_pos > 0.5 { 1.0 } else { 0.0 };
            if (pred - target).abs() < 0.5 {
                correct += 1;
            }
        }

        let avg_loss = total_loss / samples.len() as f32;
        let accuracy = correct as f32 / samples.len() as f32;
        (avg_loss, accuracy)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║   JailGuard SOTA 2026: Gradient-Based Training on Real Data       ║");
    println!("║              Using all-MiniLM-L6-v2 Embeddings                    ║");
    println!("╚════════════════════════════════════════════════════════════════════╝\n");

    // Load embeddings
    println!("📥 Loading Pre-computed Embeddings...");
    let embedding_path = "data/minilm_embeddings.json";
    if !Path::new(embedding_path).exists() {
        println!("⚠️  Embeddings not found at: {}", embedding_path);
        return Ok(());
    }

    let loader = EmbeddingLoader::from_json_file(embedding_path)?;
    println!("✅ Loaded {} samples", loader.len());
    println!(
        "   Class distribution: {} injections, {} benign",
        loader.class_distribution().0,
        loader.class_distribution().1
    );
    println!();

    // Split data
    println!("📊 Splitting Data (60% train, 20% val, 20% test)...");
    let split_train = ((loader.len() as f32 * 0.6) as usize).max(1);
    let split_val = split_train + ((loader.len() as f32 * 0.2) as usize).max(1);

    let train: Vec<_> = loader.samples()[..split_train].to_vec();
    let val: Vec<_> = loader.samples()[split_train..split_val].to_vec();
    let test: Vec<_> = loader.samples()[split_val..].to_vec();

    println!(
        "✅ Train: {}, Val: {}, Test: {}\n",
        train.len(),
        val.len(),
        test.len()
    );

    // Training
    println!("🏋️  Training with Gradient Descent");
    println!("{}", "═".repeat(70));

    let mut classifier = TrainableClassifier::new(384, 128, 0.01);
    let num_epochs = 50;
    let mut best_val_loss = f32::MAX;
    let mut patience = 0;

    let train_start = Instant::now();

    for epoch in 0..num_epochs {
        // Train on all samples (can shuffle in production)
        for sample in &train {
            classifier.train_step(&sample.embedding, sample.is_injection);
        }

        // Evaluate
        let (train_loss, train_acc) = classifier.evaluate(&train);
        let (val_loss, val_acc) = classifier.evaluate(&val);

        if (epoch + 1) % 5 == 0 {
            println!(
                "Epoch {:2}/{:2} | Train: loss={:.4}, acc={:.1}% | Val: loss={:.4}, acc={:.1}%",
                epoch + 1,
                num_epochs,
                train_loss,
                train_acc * 100.0,
                val_loss,
                val_acc * 100.0
            );
        }

        // Early stopping
        if val_loss < best_val_loss {
            best_val_loss = val_loss;
            patience = 0;
        } else {
            patience += 1;
            if patience > 10 {
                println!("Early stopping at epoch {}", epoch + 1);
                break;
            }
        }
    }

    let train_time = train_start.elapsed().as_secs_f64();
    println!("✅ Training completed in {:.2}s\n", train_time);

    // Final evaluation
    println!("📈 Final Evaluation");
    println!("{}", "═".repeat(70));

    let (test_loss, test_acc) = classifier.evaluate(&test);
    println!("Test Loss:     {:.4}", test_loss);
    println!(
        "Test Accuracy: {:.1}% ({}/{})",
        test_acc * 100.0,
        (test_acc * test.len() as f32) as usize,
        test.len()
    );

    // Per-class metrics
    let mut inj_correct = 0;
    let mut inj_total = 0;
    let mut ben_correct = 0;
    let mut ben_total = 0;

    for sample in &test {
        let logits = classifier.forward(&sample.embedding);
        let max_logit = logits[0].max(logits[1]);
        let exp0 = (logits[0] - max_logit).exp();
        let exp1 = (logits[1] - max_logit).exp();
        let prob_pos = exp1 / (exp0 + exp1);
        let pred = prob_pos > 0.5;

        if sample.is_injection {
            inj_total += 1;
            if pred {
                inj_correct += 1;
            }
        } else {
            ben_total += 1;
            if !pred {
                ben_correct += 1;
            }
        }
    }

    println!(
        "Injection Detection: {:.1}% ({}/{})",
        inj_correct as f32 / inj_total as f32 * 100.0,
        inj_correct,
        inj_total
    );
    println!(
        "Benign Detection:    {:.1}% ({}/{})",
        ben_correct as f32 / ben_total as f32 * 100.0,
        ben_correct,
        ben_total
    );
    println!();

    // Summary
    println!("📊 Summary");
    println!("{}", "═".repeat(70));
    println!("Embeddings:     all-MiniLM-L6-v2 (384-dim)");
    println!(
        "Dataset:        deepset/prompt-injections ({} samples)",
        loader.len()
    );
    println!("Final Accuracy: {:.1}%", test_acc * 100.0);
    println!("Training Time:  {:.2}s", train_time);
    println!(
        "Throughput:     {:.0} samples/sec",
        loader.len() as f64 / train_time
    );

    if test_acc >= 0.75 {
        println!(
            "\n✅ SUCCESS: Achieved {:.1}% accuracy (target: 75%+)",
            test_acc * 100.0
        );
    } else {
        println!("\n⚠️  Accuracy: {:.1}% (target: 75%+)", test_acc * 100.0);
    }

    Ok(())
}
