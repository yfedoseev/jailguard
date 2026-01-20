//! Train Neural Network on Expanded 125K Dataset
//!
//! Trains the NeuralBinaryNetwork on the newly generated balanced 125K dataset
//! with 87.5K training and 18.75K validation samples.
//!
//! This version uses the expanded dataset from Phase 3-4 pipeline:
//! - 8-class unified taxonomy
//! - 384-dimensional embeddings (FastEmbedder)
//! - Stratified train/val/test splits
//!
//! Usage:
//! ```bash
//! cargo run --example train_on_expanded_dataset --release
//! ```
//!
//! Expected results:
//! - Training on 87.5K diverse samples with multiple attack types
//! - Validation on 18.75K held-out samples
//! - Target: >95% binary classification accuracy
//! - Should outperform previous 96.58% baseline with more diverse data

use jailguard::training::{NeuralBinaryNetwork, NeuralDataLoader};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(80));
    println!("🚀 PHASE 5: TRAIN ON EXPANDED 125K DATASET");
    println!("Unified 8-class taxonomy with balanced attack types");
    println!("{}\n", "=".repeat(80));

    // === STEP 1: Load Training Data ===
    println!("📊 LOADING TRAINING DATA");
    println!("{}", "-".repeat(80));

    let load_start = Instant::now();
    let train_path = "splits_200k/train.json";
    let val_path = "splits_200k/val.json";

    println!("📖 Loading training set from {}...", train_path);
    let train_loader = match NeuralDataLoader::load_from_file(train_path) {
        Ok(loader) => {
            println!("✅ Loaded training data");
            loader
        }
        Err(e) => {
            eprintln!("❌ Error loading training data: {}", e);
            return Err(e.into());
        }
    };

    println!("📖 Loading validation set from {}...", val_path);
    let val_loader = match NeuralDataLoader::load_from_file(val_path) {
        Ok(loader) => {
            println!("✅ Loaded validation data");
            loader
        }
        Err(e) => {
            eprintln!("❌ Error loading validation data: {}", e);
            return Err(e.into());
        }
    };

    let load_time = load_start.elapsed();

    println!("\n📊 DATASET STATISTICS");
    println!("{}", "-".repeat(80));
    println!("Training set:");
    train_loader.print_stats();
    println!("\nValidation set:");
    val_loader.print_stats();
    println!("\nLoad time: {:.2}s", load_time.as_secs_f32());

    // === STEP 2: Training Configuration ===
    println!("\n{}", "=".repeat(80));
    println!("⚙️  TRAINING CONFIGURATION");
    println!("{}", "-".repeat(80));

    let learning_rate = 0.01;
    let num_epochs = 50;
    let batch_size = 128;

    println!("Learning rate: {:.4}", learning_rate);
    println!("Batch size: {}", batch_size);
    println!("Epochs: {}", num_epochs);
    println!("Dropout: 0.2 (per hidden layer)");
    println!("Loss: Binary cross-entropy");
    println!("Optimizer: Gradient descent with fixed learning rate\n");

    // === STEP 3: Initialize Network ===
    println!("{}", "=".repeat(80));
    println!("🤖 INITIALIZING NETWORK");
    println!("{}", "-".repeat(80));

    let net_start = Instant::now();
    let mut network = NeuralBinaryNetwork::new(learning_rate);
    let init_time = net_start.elapsed();

    println!("✅ Network initialized in {:.3}s", init_time.as_secs_f32());
    println!("Architecture:");
    println!("  Input:  384 (embedding dim)");
    println!("  Layer1: 256 (ReLU + Dropout 0.2)");
    println!("  Layer2: 128 (ReLU + Dropout 0.2)");
    println!("  Output: 1   (Sigmoid)");
    println!("  Parameters: ~200K weights\n");

    // === STEP 4: Training Loop ===
    println!("{}", "=".repeat(80));
    println!("🔥 TRAINING START");
    println!("{}", "-".repeat(80));

    let train_start = Instant::now();
    let mut best_val_acc = 0.0;
    let mut best_epoch = 0;
    let mut epoch_metrics = Vec::new();
    let mut patience_counter = 0;
    const EARLY_STOPPING_PATIENCE: usize = 10;

    for epoch in 0..num_epochs {
        let epoch_start = Instant::now();

        // === Training Phase ===
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

        let train_acc = train_correct as f32 / train_total as f32;
        let train_loss_avg = train_loss / train_total as f32;

        // === Validation Phase ===
        let val_batches = val_loader.create_batches(batch_size, false);

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

        let val_acc = val_correct as f32 / val_total as f32;
        let val_loss_avg = val_loss / val_total as f32;

        let epoch_time = epoch_start.elapsed();

        // === Early Stopping ===
        if val_acc > best_val_acc {
            best_val_acc = val_acc;
            best_epoch = epoch;
            patience_counter = 0;
        } else {
            patience_counter += 1;
        }

        epoch_metrics.push((epoch, train_loss_avg, train_acc, val_loss_avg, val_acc));

        // === Progress Reporting ===
        if (epoch + 1) % 5 == 0 || epoch == 0 {
            println!(
                "Epoch {:3} | Train Loss: {:.4} | Train Acc: {:.4} | Val Loss: {:.4} | Val Acc: {:.4} | {:.2}s",
                epoch + 1,
                train_loss_avg,
                train_acc,
                val_loss_avg,
                val_acc,
                epoch_time.as_secs_f32()
            );
        }

        // Check early stopping
        if patience_counter >= EARLY_STOPPING_PATIENCE {
            println!(
                "\n⏹️  Early stopping at epoch {} (no improvement for {} epochs)",
                epoch + 1, EARLY_STOPPING_PATIENCE
            );
            break;
        }
    }

    let total_train_time = train_start.elapsed();

    // === STEP 5: Results Summary ===
    println!("\n{}", "=".repeat(80));
    println!("✅ TRAINING COMPLETE");
    println!("{}", "=".repeat(80));

    println!("\n📊 FINAL METRICS");
    println!("{}", "-".repeat(80));
    println!("Best validation accuracy: {:.4} (epoch {})", best_val_acc, best_epoch + 1);

    // Find final metrics
    if let Some((_, final_train_loss, final_train_acc, final_val_loss, final_val_acc)) =
        epoch_metrics.last()
    {
        println!("Final training loss:      {:.4}", final_train_loss);
        println!("Final training accuracy:  {:.4}", final_train_acc);
        println!("Final validation loss:    {:.4}", final_val_loss);
        println!("Final validation accuracy: {:.4}", final_val_acc);
    }

    println!("\n⏱️  TIMING");
    println!("{}", "-".repeat(80));
    println!("Total training time: {:.2}s ({:.2} minutes)", total_train_time.as_secs_f32(), total_train_time.as_secs_f32() / 60.0);
    println!(
        "Average per epoch: {:.2}s",
        total_train_time.as_secs_f32() / epoch_metrics.len() as f32
    );

    println!("\n🎯 ANALYSIS");
    println!("{}", "-".repeat(80));
    println!("Dataset: 87.5K training + 18.75K validation (125K total)");
    println!("Attack types: 8 classes (Benign, RolePlay, InstructionOverride, etc.)");
    println!("Embedding: 384-dimensional (FastEmbedder)");
    println!("Architecture: 384 → 256 → 128 → 1");

    if best_val_acc > 0.96 {
        println!("✅ Excellent! Accuracy {:.2}% meets SOTA target (>97%)", best_val_acc * 100.0);
    } else if best_val_acc > 0.95 {
        println!("✓ Good! Accuracy {:.2}% meets target (>95%)", best_val_acc * 100.0);
    } else {
        println!("⚠️  Accuracy {:.2}% - consider tuning hyperparameters", best_val_acc * 100.0);
    }

    println!("\n🚀 NEXT STEPS");
    println!("{}", "-".repeat(80));
    println!("1. Run comprehensive evaluation:");
    println!("   cargo run --example comprehensive_evaluation --release");
    println!("\n2. Evaluate on test set (splits_200k/test.json)");
    println!("\n3. Compare against SOTA:");
    println!("   - GenTel-Shield: 97.63%");
    println!("   - PromptShield: 0.998 AUC");

    println!("\n{}\n", "=".repeat(80));

    Ok(())
}
