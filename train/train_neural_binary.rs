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

use jailguard::training::{AdamState, NeuralBinaryNetwork, NeuralDataLoader};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("\n{}", "=".repeat(80));
    eprintln!("BINARY CLASSIFICATION NEURAL NETWORK TRAINING");
    eprintln!("Architecture: 384 -> 256 (ReLU) -> 128 (ReLU) -> 1 (Sigmoid)");
    eprintln!("{}\n", "=".repeat(80));

    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut data_path = String::from("data/combined_minilm_embeddings_with_types.json");
    let mut model_output = String::from("models/neural_binary.json");
    let mut learning_rate_arg: Option<f32> = None;
    let mut injection_weight_arg: Option<f32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--data" if i + 1 < args.len() => {
                data_path = args[i + 1].clone();
                i += 2;
            }
            "--output" if i + 1 < args.len() => {
                model_output = args[i + 1].clone();
                i += 2;
            }
            "--lr" if i + 1 < args.len() => {
                learning_rate_arg = args[i + 1].parse().ok();
                i += 2;
            }
            "--injection-weight" if i + 1 < args.len() => {
                injection_weight_arg = args[i + 1].parse().ok();
                i += 2;
            }
            _ => i += 1,
        }
    }

    // === STEP 1: Load Data ===
    eprintln!("LOADING DATA: {}", data_path);
    eprintln!("{}", "-".repeat(80));

    let load_start = Instant::now();

    let loader = match NeuralDataLoader::load_from_file(&data_path) {
        Ok(loader) => {
            eprintln!("Loaded embeddings from {}", data_path);
            loader
        }
        Err(e) => {
            eprintln!("Error loading embeddings: {}", e);
            return Err(e.into());
        }
    };

    let load_time = load_start.elapsed();
    loader.print_stats();
    eprintln!("Load time: {:.2}s\n", load_time.as_secs_f32());

    // === STEP 2: Training Configuration ===
    eprintln!("TRAINING CONFIGURATION");
    eprintln!("{}", "-".repeat(80));

    let learning_rate = learning_rate_arg.unwrap_or(0.001);
    let injection_weight = injection_weight_arg.unwrap_or(2.5);
    let num_epochs = 50;
    let batch_size = 64;

    eprintln!("Learning rate:    {:.4}", learning_rate);
    eprintln!("Batch size:       {}", batch_size);
    eprintln!("Epochs:           {}", num_epochs);
    eprintln!("Dropout:          0.2 (per hidden layer)");
    eprintln!("Optimizer:        Adam (β1=0.9, β2=0.999, ε=1e-8)");
    eprintln!(
        "Loss:             weighted BCE, injection_weight={:.1}\n",
        injection_weight
    );

    // === STEP 3: Create Network ===
    eprintln!("INITIALIZING NETWORK");
    eprintln!("{}", "-".repeat(80));

    let trainer_start = Instant::now();
    let mut network = NeuralBinaryNetwork::new(learning_rate);
    let init_time = trainer_start.elapsed();

    eprintln!("Network initialized in {:.2}s", init_time.as_secs_f32());
    eprintln!("Architecture: 384 -> 256 (ReLU) -> 128 (ReLU) -> 1 (Sigmoid)\n");

    // === STEP 4: Training ===
    eprintln!("TRAINING START");
    eprintln!("{}", "-".repeat(80));

    let train_start = Instant::now();
    let mut best_val_acc = 0.0;
    let mut best_epoch = 0;
    let mut epoch_metrics = Vec::new();
    let mut adam = AdamState::new();

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
                network.train_step_adam(embedding, *is_injection, &mut adam, injection_weight);
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
            eprintln!(
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
            eprintln!("Early stopping at epoch {}", epoch + 1);
            break;
        }
    }

    let total_train_time = train_start.elapsed();

    eprintln!("\nTRAINING COMPLETE");
    eprintln!("Total time: {:.2}s", total_train_time.as_secs_f32());
    eprintln!(
        "Best validation accuracy: epoch {}, {:.2}%\n",
        best_epoch + 1,
        best_val_acc * 100.0
    );

    // === STEP 5: Test Evaluation ===
    eprintln!("TEST SET EVALUATION");
    eprintln!("{}", "-".repeat(80));

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

    eprintln!("Test loss: {:.4}", test_loss);
    eprintln!("Test accuracy: {:.2}%", test_acc * 100.0);
    eprintln!("Precision: {:.2}%", precision * 100.0);
    eprintln!("Recall: {:.2}%", recall * 100.0);
    eprintln!("F1 score: {:.4}", f1);
    eprintln!();
    eprintln!("Confusion Matrix:");
    eprintln!(
        "  True Positive:  {} (injections correctly detected)",
        test_tp
    );
    eprintln!("  True Negative:  {} (benign correctly accepted)", test_tn);
    eprintln!("  False Positive: {} (benign incorrectly flagged)", test_fp);
    eprintln!("  False Negative: {} (injections missed)", test_fn);

    // === STEP 6: Save Model ===
    eprintln!("\nSAVING MODEL");
    eprintln!("{}", "-".repeat(80));

    match network.save(&model_output) {
        Ok(()) => {
            let model_size = std::fs::metadata(&model_output)
                .map(|m| m.len() as f64 / (1024.0 * 1024.0))
                .unwrap_or(0.0);
            eprintln!("Saved to {} ({:.1} MB)", model_output, model_size);
        }
        Err(e) => {
            eprintln!("Error saving model: {}", e);
        }
    }

    // === STEP 7: Summary ===
    eprintln!("\nTRAINING SUMMARY");
    eprintln!("{}", "-".repeat(80));
    let total_samples =
        loader.train_samples.len() + loader.val_samples.len() + loader.test_samples.len();
    eprintln!("Dataset: {} samples", total_samples);
    eprintln!("  Train: {} (80%)", loader.train_samples.len());
    eprintln!("  Val:   {} (10%)", loader.val_samples.len());
    eprintln!("  Test:  {} (10%)", loader.test_samples.len());
    eprintln!();
    eprintln!(
        "Training time: {:.2}s total",
        total_train_time.as_secs_f32()
    );
    eprintln!("Epochs completed: {}", epoch_metrics.len());
    if !epoch_metrics.is_empty() {
        eprintln!(
            "Avg time/epoch: {:.2}s",
            total_train_time.as_secs_f32() / epoch_metrics.len() as f32
        );
    }
    eprintln!();
    eprintln!("RESULTS");
    if let Some((train_loss, train_acc, val_loss, val_acc)) = epoch_metrics.last() {
        eprintln!(
            "  Train loss: {:.4}, acc: {:.2}%",
            train_loss,
            train_acc * 100.0
        );
        eprintln!(
            "  Val loss:   {:.4}, acc: {:.2}%",
            val_loss,
            val_acc * 100.0
        );
    }
    eprintln!(
        "  Test loss:  {:.4}, acc: {:.2}%",
        test_loss,
        test_acc * 100.0
    );
    eprintln!("  Precision:  {:.2}%", precision * 100.0);
    eprintln!("  Recall:     {:.2}%", recall * 100.0);
    eprintln!("  F1:         {:.4}", f1);

    eprintln!("\n{}\n", "=".repeat(80));

    Ok(())
}
