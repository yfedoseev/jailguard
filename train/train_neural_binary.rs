//! Neural Network v1.1 Binary Classification Training
//!
//! Trains the simplified NeuralBinaryNetwork on the complete 15,185 sample dataset.
//! This version removes multi-task confusion and adds regularization for better convergence.
//!
//! Usage:
//! ```bash
//! cargo run --example train_neural_binary --release
//! ```
//!
//! Expected improvements over multi-task version:
//! - More stable convergence
//! - Better regularization via dropout
//! - Single clear optimization target (binary classification)
//! - Should exceed Phase 5d baseline (84.62%)

use jailguard::training::{NeuralBinaryNetwork, NeuralDataLoader};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(80));
    println!("PHASE 6.3: BINARY CLASSIFICATION NEURAL NETWORK");
    println!("Simplified approach with dropout regularization for >95% accuracy");
    println!("{}\n", "=".repeat(80));

    // === STEP 1: Load Data ===
    println!("📊 LOADING DATA");
    println!("{}", "-".repeat(80));

    let load_start = Instant::now();
    let data_path = "data/combined_minilm_embeddings_with_types.json";

    let loader = match NeuralDataLoader::load_from_file(data_path) {
        Ok(loader) => {
            println!("✅ Loaded embeddings from {}", data_path);
            loader
        }
        Err(e) => {
            eprintln!("❌ Error loading embeddings: {}", e);
            return Err(e.into());
        }
    };

    let load_time = load_start.elapsed();
    loader.print_stats();
    println!("Load time: {:.2}s\n", load_time.as_secs_f32());

    // === STEP 2: Training Configuration ===
    println!("⚙️  TRAINING CONFIGURATION");
    println!("{}", "-".repeat(80));

    let learning_rate = 0.01;
    let num_epochs = 50;
    let batch_size = 64;

    println!("Learning rate: {:.4}", learning_rate);
    println!("Batch size: {}", batch_size);
    println!("Epochs: {}", num_epochs);
    println!("Dropout: 0.2 (per hidden layer)");
    println!("Loss: Binary cross-entropy\n");

    // === STEP 3: Create Network ===
    println!("🤖 INITIALIZING NETWORK");
    println!("{}", "-".repeat(80));

    let trainer_start = Instant::now();
    let mut network = NeuralBinaryNetwork::new(learning_rate);
    let init_time = trainer_start.elapsed();

    println!("✅ Network initialized in {:.2}s", init_time.as_secs_f32());
    println!("Architecture: 384 → 256 (ReLU) → 128 (ReLU) → 1 (Sigmoid)");
    println!("Parameters: ~200K weights\n");

    // === STEP 4: Training ===
    println!("🔥 TRAINING START");
    println!("{}", "-".repeat(80));

    let train_start = Instant::now();
    let mut best_val_acc = 0.0;
    let mut best_epoch = 0;
    let mut epoch_metrics = Vec::new();

    for epoch in 0..num_epochs {
        let epoch_start = Instant::now();

        // Create batches
        let batches = loader.create_batches(batch_size, false); // No balancing for stability

        // Training
        let mut train_loss = 0.0;
        let mut train_correct = 0;
        let mut train_total = 0;

        for batch in &batches {
            for (embedding, is_injection, _) in batch {
                network.train_step(embedding, *is_injection);
                let loss = network.evaluate_loss(embedding, *is_injection);
                train_loss += loss;

                let pred = network.forward_eval(embedding);
                let pred_binary = pred > 0.5;
                if pred_binary == *is_injection {
                    train_correct += 1;
                }
                train_total += 1;
            }
        }

        let train_loss = train_loss / train_total as f32;
        let train_acc = train_correct as f32 / train_total as f32;

        // Validation
        let mut val_loss = 0.0;
        let mut val_correct = 0;
        for sample in &loader.val_samples {
            let loss = network.evaluate_loss(&sample.embedding, sample.is_injection);
            val_loss += loss;

            let pred = network.forward_eval(&sample.embedding);
            let pred_binary = pred > 0.5;
            if pred_binary == sample.is_injection {
                val_correct += 1;
            }
        }

        let val_loss = val_loss / loader.val_samples.len() as f32;
        let val_acc = val_correct as f32 / loader.val_samples.len() as f32;

        // Track best
        if val_acc > best_val_acc {
            best_val_acc = val_acc;
            best_epoch = epoch;
        }

        let elapsed = epoch_start.elapsed();

        epoch_metrics.push((train_loss, train_acc, val_loss, val_acc));

        // Print progress
        if epoch % 5 == 0 || epoch == num_epochs - 1 {
            println!(
                "Epoch {:3}/{}: train_loss={:.4}, train_acc={:.2}%, val_loss={:.4}, val_acc={:.2}%, {:.1}s",
                epoch + 1,
                num_epochs,
                train_loss,
                train_acc * 100.0,
                val_loss,
                val_acc * 100.0,
                elapsed.as_secs_f32()
            );
        }

        // Early stopping
        if epoch - best_epoch > 10 && epoch > 20 {
            println!("✓ Early stopping at epoch {}", epoch + 1);
            break;
        }
    }

    let total_train_time = train_start.elapsed();

    println!("\n✅ TRAINING COMPLETE");
    println!("Total time: {:.2}s", total_train_time.as_secs_f32());
    println!(
        "Best validation accuracy: epoch {}, {:.2}%\n",
        best_epoch + 1,
        best_val_acc * 100.0
    );

    // === STEP 5: Test Evaluation ===
    println!("📈 TEST SET EVALUATION");
    println!("{}", "-".repeat(80));

    let mut test_loss = 0.0;
    let mut test_correct = 0;
    let mut test_tp = 0;
    let mut test_fp = 0;
    let mut test_fn = 0;
    let mut test_tn = 0;

    for sample in &loader.test_samples {
        let loss = network.evaluate_loss(&sample.embedding, sample.is_injection);
        test_loss += loss;

        let pred = network.forward_eval(&sample.embedding);
        let pred_binary = pred > 0.5;

        if pred_binary == sample.is_injection {
            test_correct += 1;
        }

        // Confusion matrix
        if sample.is_injection {
            if pred_binary {
                test_tp += 1;
            } else {
                test_fn += 1;
            }
        } else {
            if pred_binary {
                test_fp += 1;
            } else {
                test_tn += 1;
            }
        }
    }

    let test_loss = test_loss / loader.test_samples.len() as f32;
    let test_acc = test_correct as f32 / loader.test_samples.len() as f32;

    let precision = test_tp as f32 / (test_tp + test_fp) as f32;
    let recall = test_tp as f32 / (test_tp + test_fn) as f32;
    let f1 = 2.0 * (precision * recall) / (precision + recall);

    println!("Test loss: {:.4}", test_loss);
    println!("Test accuracy: {:.2}%", test_acc * 100.0);
    println!("Precision: {:.2}%", precision * 100.0);
    println!("Recall: {:.2}%", recall * 100.0);
    println!("F1 score: {:.4}", f1);
    println!();
    println!("Confusion Matrix:");
    println!(
        "  True Positive:  {} (injections correctly detected)",
        test_tp
    );
    println!("  True Negative:  {} (benign correctly accepted)", test_tn);
    println!("  False Positive: {} (benign incorrectly flagged)", test_fp);
    println!("  False Negative: {} (injections missed)", test_fn);

    // === STEP 6: Comparison with Phase 5d ===
    println!("\n🎯 PHASE 5d COMPARISON");
    println!("{}", "-".repeat(80));

    let phase5d_accuracy = 0.8462;
    let improvement = (test_acc - phase5d_accuracy) * 100.0;
    let improvement_pct = if phase5d_accuracy > 0.0 {
        (test_acc / phase5d_accuracy - 1.0) * 100.0
    } else {
        0.0
    };

    println!("Phase 5d accuracy: {:.2}%", phase5d_accuracy * 100.0);
    println!("Neural Network v1.1 accuracy: {:.2}%", test_acc * 100.0);

    if test_acc >= phase5d_accuracy {
        println!(
            "✅ Improvement: +{:.2}% ({:+.1}%)",
            improvement, improvement_pct
        );
    } else {
        println!(
            "⚠️  Regression: {:.2}% ({:+.1}%)",
            improvement, improvement_pct
        );
    }

    if test_acc >= 0.95 {
        println!("\n🎉 TARGET ACHIEVED: >95% accuracy!");
    } else if test_acc >= phase5d_accuracy {
        println!("\n✅ EXCEEDS BASELINE: Neural Network v1.1 is better than Phase 5d");
    } else {
        println!("\n⚠️  Below baseline. Recommendations:");
        println!("  - Increase learning rate (try 0.02-0.05)");
        println!("  - Increase epochs (try 100+)");
        println!("  - Reduce dropout (try 0.1)");
        println!("  - Tune batch size");
    }

    // === STEP 7: Summary ===
    println!("\n📋 TRAINING SUMMARY");
    println!("{}", "-".repeat(80));
    println!("Dataset: 15,185 samples");
    println!("  Train: {} (80%)", loader.train_samples.len());
    println!("  Val:   {} (10%)", loader.val_samples.len());
    println!("  Test:  {} (10%)", loader.test_samples.len());
    println!();
    println!(
        "Training time: {:.2}s total",
        total_train_time.as_secs_f32()
    );
    println!("Epochs completed: {}", epoch_metrics.len());
    if !epoch_metrics.is_empty() {
        println!(
            "Avg time/epoch: {:.2}s",
            total_train_time.as_secs_f32() / epoch_metrics.len() as f32
        );
    }
    println!();
    println!("FINAL METRICS");
    if let Some((train_loss, train_acc, val_loss, val_acc)) = epoch_metrics.last() {
        println!(
            "  Train loss: {:.4}, acc: {:.2}%",
            train_loss,
            train_acc * 100.0
        );
        println!(
            "  Val loss:   {:.4}, acc: {:.2}%",
            val_loss,
            val_acc * 100.0
        );
    }
    println!(
        "  Test loss:  {:.4}, acc: {:.2}%",
        test_loss,
        test_acc * 100.0
    );

    println!("\n{}\n", "=".repeat(80));

    Ok(())
}
