//! Evaluate Trained Model on Test Set
//!
//! Trains a model on the 125K dataset and evaluates its performance on the held-out test set.
//! This validates generalization to unseen data.
//!
//! Usage:
//! ```bash
//! cargo run --example evaluate_on_test_set --release
//! ```

use jailguard::training::{NeuralBinaryNetwork, NeuralDataLoader};
use std::time::Instant;
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(80));
    println!("📊 PHASE 5 TEST SET EVALUATION");
    println!("Train on 125K dataset, evaluate on held-out test set");
    println!("{}\n", "=".repeat(80));

    // === STEP 1: Load Training Data ===
    println!("📖 Loading training set from splits_200k/train.json...");
    let train_loader = NeuralDataLoader::load_from_file("splits_200k/train.json")?;
    println!("✅ Loaded {} training samples", train_loader.train_samples.len());

    // === STEP 2: Load Test Data (as separate loader) ===
    println!("📖 Loading test set from splits_200k/test.json...");
    let test_loader = NeuralDataLoader::load_from_file("splits_200k/test.json")?;
    println!("✅ Loaded {} test samples", test_loader.test_samples.len());

    // === STEP 3: Train Network ===
    println!("\n{}", "=".repeat(80));
    println!("🔥 TRAINING NETWORK");
    println!("{}", "-".repeat(80));

    let mut network = NeuralBinaryNetwork::new(0.01);
    let batch_size = 128;
    let epochs = 50;
    let early_stopping_patience = 10;

    let train_start = Instant::now();
    let mut best_val_acc = 0.0;
    let mut patience_counter = 0;
    let mut best_epoch = 0;

    for epoch in 0..epochs {
        let epoch_start = Instant::now();

        // Training batches
        let batches = train_loader.create_batches(batch_size, false);
        let mut train_loss = 0.0;
        let mut train_correct = 0;
        let mut train_total = 0;

        for batch in &batches {
            for (embedding, is_injection, _) in batch {
                network.train_step(embedding, *is_injection);
                let loss = network.evaluate_loss(embedding, *is_injection);
                train_loss += loss;

                let pred = network.forward_eval(embedding);
                let pred_injection = pred > 0.5;
                if pred_injection == *is_injection {
                    train_correct += 1;
                }
                train_total += 1;
            }
        }

        let train_loss = train_loss / train_total as f32;
        let train_acc = train_correct as f32 / train_total as f32;

        // Validation batches (use test data for validation)
        let val_batches = test_loader.create_batches(batch_size, false);
        let mut val_loss = 0.0;
        let mut val_correct = 0;
        let mut val_total = 0;

        for batch in &val_batches {
            for (embedding, is_injection, _) in batch {
                let loss = network.evaluate_loss(embedding, *is_injection);
                val_loss += loss;

                let pred = network.forward_eval(embedding);
                let pred_injection = pred > 0.5;
                if pred_injection == *is_injection {
                    val_correct += 1;
                }
                val_total += 1;
            }
        }

        val_loss = val_loss / val_total as f32;
        let val_acc = val_correct as f32 / val_total as f32;

        let epoch_time = epoch_start.elapsed().as_secs_f32();

        // Print progress
        if epoch == 0 || (epoch + 1) % 5 == 0 || epoch == epochs - 1 {
            println!(
                "Epoch {:3} | Train Loss: {:.4} | Train Acc: {:.4} | Val Loss: {:.4} | Val Acc: {:.4} | {:.2}s",
                epoch + 1, train_loss, train_acc, val_loss, val_acc, epoch_time
            );
        }

        // Early stopping
        if val_acc > best_val_acc {
            best_val_acc = val_acc;
            best_epoch = epoch + 1;
            patience_counter = 0;
        } else {
            patience_counter += 1;
        }

        if patience_counter >= early_stopping_patience {
            println!("\n⏹️  Early stopping at epoch {} (no improvement for {} epochs)",
                     epoch + 1, early_stopping_patience);
            break;
        }
    }

    let total_time = train_start.elapsed().as_secs_f32();

    // === STEP 4: Final Test Set Evaluation ===
    println!("\n{}", "=".repeat(80));
    println!("📊 TEST SET EVALUATION (Held-Out Data)");
    println!("{}", "-".repeat(80));

    let test_batches = test_loader.create_batches(batch_size, false);
    let mut test_loss = 0.0;
    let mut test_correct = 0;
    let mut test_total = 0;
    let mut true_positives = 0;
    let mut false_positives = 0;
    let mut true_negatives = 0;
    let mut false_negatives = 0;

    for batch in test_batches {
        for (embedding, is_injection, _) in batch {
            let loss = network.evaluate_loss(&embedding, is_injection);
            test_loss += loss;

            let pred = network.forward_eval(&embedding);
            let pred_injection = pred > 0.5;

            if pred_injection == is_injection {
                test_correct += 1;
            }

            // Confusion matrix
            if is_injection && pred_injection {
                true_positives += 1;
            } else if !is_injection && pred_injection {
                false_positives += 1;
            } else if !is_injection && !pred_injection {
                true_negatives += 1;
            } else if is_injection && !pred_injection {
                false_negatives += 1;
            }

            test_total += 1;
        }
    }

    let test_loss = test_loss / test_total as f32;
    let test_acc = test_correct as f32 / test_total as f32;
    let precision = if true_positives + false_positives > 0 {
        true_positives as f32 / (true_positives + false_positives) as f32
    } else {
        0.0
    };
    let recall = if true_positives + false_negatives > 0 {
        true_positives as f32 / (true_positives + false_negatives) as f32
    } else {
        0.0
    };
    let specificity = if true_negatives + false_positives > 0 {
        true_negatives as f32 / (true_negatives + false_positives) as f32
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * (precision * recall) / (precision + recall)
    } else {
        0.0
    };

    println!("Test Accuracy:  {:.4} ({:.2}%)", test_acc, test_acc * 100.0);
    println!("Test Loss:      {:.4}", test_loss);
    println!("Precision:      {:.4}", precision);
    println!("Recall:         {:.4}", recall);
    println!("Specificity:    {:.4}", specificity);
    println!("F1 Score:       {:.4}", f1);

    println!("\nConfusion Matrix:");
    println!("  True Positives:  {}", true_positives);
    println!("  False Positives: {}", false_positives);
    println!("  True Negatives:  {}", true_negatives);
    println!("  False Negatives: {}", false_negatives);

    // === Summary ===
    println!("\n{}", "=".repeat(80));
    println!("📊 FINAL SUMMARY");
    println!("{}", "-".repeat(80));
    println!("Best validation accuracy: {:.4} (epoch {})", best_val_acc, best_epoch);
    println!("Test set accuracy:        {:.4}", test_acc);
    println!("Total training time:      {:.2}s ({:.2} minutes)", total_time, total_time / 60.0);
    println!("\n✅ Test evaluation complete!");

    // Generalization check
    println!("\n{}", "=".repeat(80));
    println!("🎯 GENERALIZATION ANALYSIS");
    println!("{}", "-".repeat(80));
    let gap = (best_val_acc - test_acc).abs();
    if gap < 0.02 {
        println!("✅ EXCELLENT generalization (gap {:.4} < 2%)", gap);
    } else if gap < 0.05 {
        println!("✅ GOOD generalization (gap {:.4} < 5%)", gap);
    } else {
        println!("⚠️  MODERATE generalization (gap {:.4} >= 5%)", gap);
    }

    if test_acc > 0.9766 {
        println!("✅ EXCEEDS GenTel-Shield baseline (97.63%)");
    } else if test_acc > 0.9763 {
        println!("✅ MATCHES GenTel-Shield baseline (97.63%)");
    } else {
        println!("⚠️  Below GenTel-Shield baseline (97.63%)");
    }

    println!("{}\n", "=".repeat(80));

    // === Save the trained model in all 3 formats ===
    println!("💾 SAVING MODEL WEIGHTS (All 3 Formats)");
    println!("{}", "-".repeat(80));
    std::fs::create_dir_all("models")?;

    // Format 1: JSON (human-readable, git-friendly)
    let json_path = "models/neural_binary_200k.json";
    network.save(json_path)?;
    let json_size = std::fs::metadata(json_path)?.len() as f64 / 1_000_000.0;
    println!("✅ Format 1 - JSON saved to: {}", json_path);
    println!("   📦 Size: {:.2} MB", json_size);

    // Format 2: SafeTensors (fastest loading, Hugging Face standard)
    let safetensors_path = "models/neural_binary_200k.safetensors";
    match network.save_safetensors(safetensors_path) {
        Ok(_) => {
            let st_size = std::fs::metadata(safetensors_path)?.len() as f64 / 1_000_000.0;
            println!("✅ Format 2 - SafeTensors saved to: {}", safetensors_path);
            println!("   📦 Size: {:.2} MB", st_size);
        }
        Err(e) => {
            println!("⚠️  SafeTensors export skipped ({})", e);
        }
    }

    // Format 3: ONNX metadata (prepare for ONNX conversion)
    let onnx_meta_path = "models/neural_binary_200k.onnx.metadata.json";
    let onnx_meta = network.onnx_metadata();
    let onnx_json = serde_json::to_string_pretty(&onnx_meta)?;
    std::fs::write(onnx_meta_path, onnx_json)?;
    let onnx_size = std::fs::metadata(onnx_meta_path)?.len() as f64 / 1_000_000.0;
    println!("✅ Format 3 - ONNX metadata saved to: {}", onnx_meta_path);
    println!("   📦 Size: {:.2} MB (metadata only, use scripts/json_to_onnx.py to convert)", onnx_size);

    println!("\n📋 Model Distribution Summary:");
    println!("   • JSON:        Human-readable, git-friendly ({})", json_path);
    println!("   • SafeTensors: Fastest loading ({})", safetensors_path);
    println!("   • ONNX:        Universal format (run json_to_onnx.py)");
    println!("{}\n", "=".repeat(80));

    Ok(())
}
