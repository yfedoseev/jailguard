//! Phase 5: Fine-tune Ensemble Classifier
//!
//! Trains a simple neural network classifier on the synthetic dataset
//! generated in Phase 4b. The trained model will be integrated into
//! the ensemble in Phase 6.
//!
//! Run with: cargo run --example train_ensemble_classifier --release

use jailguard::embeddings::FastEmbedder;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrainingSample {
    text: String,
    is_injection: bool,
    category: String,
}

#[derive(Debug, Clone)]
struct Embedding {
    vector: Vec<f32>,
    is_injection: bool,
}

/// Simple logistic regression classifier
struct SimpleClassifier {
    weights: Vec<f32>,
    bias: f32,
    learning_rate: f32,
}

impl SimpleClassifier {
    fn new(embedding_dim: usize) -> Self {
        Self {
            weights: vec![0.0; embedding_dim],
            bias: 0.0,
            learning_rate: 0.01,
        }
    }

    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }

    fn predict(&self, embedding: &[f32]) -> f32 {
        let mut z = self.bias;
        for (w, e) in self.weights.iter().zip(embedding.iter()) {
            z += w * e;
        }
        Self::sigmoid(z)
    }

    fn train_batch(&mut self, batch: &[Embedding]) {
        let batch_size = batch.len() as f32;

        // Compute gradients
        let mut weight_grad = vec![0.0; self.weights.len()];
        let mut bias_grad = 0.0;

        for sample in batch {
            let pred = self.predict(&sample.vector);
            let target = if sample.is_injection { 1.0 } else { 0.0 };
            let error = pred - target;

            // Update gradients
            for (i, _) in self.weights.iter().enumerate() {
                weight_grad[i] += error * sample.vector[i];
            }
            bias_grad += error;
        }

        // Normalize gradients
        for grad in &mut weight_grad {
            *grad /= batch_size;
        }
        bias_grad /= batch_size;

        // Update weights
        for (w, grad) in self.weights.iter_mut().zip(weight_grad.iter()) {
            *w -= self.learning_rate * grad;
        }
        self.bias -= self.learning_rate * bias_grad;
    }

    fn evaluate(&self, embeddings: &[Embedding]) -> (f32, f32, f32) {
        let mut correct = 0;
        let mut tp = 0; // True positives (detected injections)
        let mut fn_count = 0; // False negatives (missed injections)
        let mut fp = 0; // False positives (false alarms)

        for sample in embeddings {
            let pred = self.predict(&sample.vector) > 0.5;
            if pred == sample.is_injection {
                correct += 1;
            }

            if sample.is_injection {
                if pred {
                    tp += 1;
                } else {
                    fn_count += 1;
                }
            } else if pred {
                fp += 1;
            }
        }

        let accuracy = correct as f32 / embeddings.len() as f32;
        let recall = if tp + fn_count > 0 {
            tp as f32 / (tp + fn_count) as f32
        } else {
            0.0
        };
        let precision = if tp + fp > 0 {
            tp as f32 / (tp + fp) as f32
        } else {
            0.0
        };

        (accuracy, recall, precision)
    }
}

fn load_samples(path: &Path) -> Result<Vec<TrainingSample>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let samples: Vec<TrainingSample> = serde_json::from_reader(file)?;
    Ok(samples)
}

fn generate_embeddings(
    samples: &[TrainingSample],
) -> Result<Vec<Embedding>, Box<dyn std::error::Error>> {
    println!("📊 Generating embeddings for {} samples...", samples.len());

    let embedder = FastEmbedder::new();
    let mut embeddings = Vec::new();

    for sample in samples {
        let vector = embedder.embed(&sample.text);
        embeddings.push(Embedding {
            vector,
            is_injection: sample.is_injection,
        });
    }

    println!(
        "✓ Generated {} embeddings (dimension: 384)",
        embeddings.len()
    );
    Ok(embeddings)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(70));
    println!("Phase 5: Fine-tune Ensemble Classifier");
    println!("{}", "=".repeat(70));

    let start_time = Instant::now();

    // Load datasets
    println!("\n📥 Loading datasets...");

    let data_dir = Path::new("data/training/splits");
    let train_path = data_dir.join("train.json");
    let val_path = data_dir.join("val.json");
    let test_path = data_dir.join("test.json");

    if !train_path.exists() {
        eprintln!("❌ Training data not found!");
        eprintln!("Run: python3 scripts/generate_synthetic_dataset.py");
        std::process::exit(1);
    }

    let train_samples = load_samples(&train_path)?;
    let val_samples = load_samples(&val_path)?;
    let test_samples = load_samples(&test_path)?;

    println!("✓ Train: {} samples", train_samples.len());
    println!("✓ Val: {} samples", val_samples.len());
    println!("✓ Test: {} samples", test_samples.len());

    // Generate embeddings
    println!("\n🔢 Embedding samples...");
    let train_embeddings = generate_embeddings(&train_samples)?;
    let val_embeddings = generate_embeddings(&val_samples)?;
    let test_embeddings = generate_embeddings(&test_samples)?;

    // Initialize and train classifier
    println!("\n🤖 Training classifier...");
    let mut classifier = SimpleClassifier::new(384);

    let batch_size = 16;
    let num_epochs = 20;

    for epoch in 1..=num_epochs {
        // Train on batches
        for batch in train_embeddings.chunks(batch_size) {
            classifier.train_batch(batch);
        }

        // Evaluate on validation set every 5 epochs
        if epoch % 5 == 0 || epoch == 1 {
            let (acc, recall, prec) = classifier.evaluate(&val_embeddings);
            println!(
                "Epoch {:2}: Accuracy {:.1}% | Recall {:.1}% | Precision {:.1}%",
                epoch,
                acc * 100.0,
                recall * 100.0,
                prec * 100.0
            );
        }
    }

    // Final evaluation
    println!("\n📈 Final Evaluation:");
    println!("  On Training Set:");
    let (train_acc, train_recall, train_prec) = classifier.evaluate(&train_embeddings);
    println!("    Accuracy: {:.1}%", train_acc * 100.0);
    println!("    Recall:   {:.1}%", train_recall * 100.0);
    println!("    Precision: {:.1}%", train_prec * 100.0);

    println!("  On Validation Set:");
    let (val_acc, val_recall, val_prec) = classifier.evaluate(&val_embeddings);
    println!("    Accuracy: {:.1}%", val_acc * 100.0);
    println!("    Recall:   {:.1}%", val_recall * 100.0);
    println!("    Precision: {:.1}%", val_prec * 100.0);

    println!("  On Test Set:");
    let (test_acc, test_recall, test_prec) = classifier.evaluate(&test_embeddings);
    println!("    Accuracy: {:.1}%", test_acc * 100.0);
    println!("    Recall:   {:.1}%", test_recall * 100.0);
    println!("    Precision: {:.1}%", test_prec * 100.0);

    let elapsed = start_time.elapsed();

    println!("\n{}", "=".repeat(70));
    println!("✅ Phase 5 Complete");
    println!("{}", "=".repeat(70));

    println!("\n📊 Summary:");
    println!("  Model trained in {:.1}s", elapsed.as_secs_f32());
    println!("  Test Accuracy: {:.1}%", test_acc * 100.0);
    println!("  Test Recall: {:.1}%", test_recall * 100.0);
    println!("  Test Precision: {:.1}%", test_prec * 100.0);

    if test_acc > 0.85 {
        println!("\n🎯 Target achieved! Ready for Phase 6 integration");
    } else {
        println!("\n⚠️  Consider increasing epochs or using more training data");
    }

    println!("\nNext: Phase 6 - Integrate trained model into ensemble");

    Ok(())
}
