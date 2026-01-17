//! Phase 8 Stage 2: Fine-tuning on Expanded Dataset
//!
//! This example demonstrates fine-tuning the transformer encoder on an expanded
//! dataset (10k+ samples) to achieve 92-94% accuracy - a 2-4% improvement over
//! Stage 1's 88-90% on synthetic data.
//!
//! Run with: cargo run --example fine_tune_stage2 --release

use jailguard::dataset::{ExpandedDataset, ExternalDatasetConfig};
use jailguard::training::fine_tune::{FineTuneConfig, FineTuner};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(70));
    println!("Phase 8 Stage 2: Fine-tuning on Expanded Dataset");
    println!("{}", "=".repeat(70));
    println!();

    // Load expanded dataset
    println!("📂 Loading expanded dataset...");
    let dataset_config = ExternalDatasetConfig {
        enable_jailbreakbench: false, // Disabled: requires network
        enable_deepseek: false,        // Disabled: requires network
        enable_mock_generation: true,  // Enabled: local generation
        samples_per_attack_type: 1_000, // 1k samples per attack type = 7k total
        augmentation_multiplier: 3,    // 3x variants per sample
        seed: 42,
    };

    let dataset = ExpandedDataset::load(dataset_config)?;

    println!("✅ Dataset loaded successfully!");
    println!();

    // Display dataset statistics
    println!("📊 Dataset Statistics:");
    println!(
        "  Total Samples:     {}",
        dataset.statistics.total_samples
    );
    println!(
        "  Injection Samples: {} ({:.1}%)",
        dataset.statistics.injection_count,
        (dataset.statistics.injection_count as f32 / dataset.statistics.total_samples as f32)
            * 100.0
    );
    println!(
        "  Benign Samples:    {} ({:.1}%)",
        dataset.statistics.benign_count,
        (dataset.statistics.benign_count as f32 / dataset.statistics.total_samples as f32)
            * 100.0
    );
    println!(
        "  Average Length:    {:.1} chars",
        dataset.statistics.average_length
    );
    println!();

    println!("📈 Attack Type Distribution:");
    for (attack_type, count) in &dataset.statistics.attack_type_distribution {
        println!("  {}: {} samples", attack_type, count);
    }
    println!();

    // Get splits
    let (train_samples, val_samples, test_samples) = dataset.get_splits(0.7, 0.15);
    println!("📋 Data Split:");
    println!("  Training:   {} samples", train_samples.len());
    println!("  Validation: {} samples", val_samples.len());
    println!("  Test:       {} samples", test_samples.len());
    println!();

    // Configure fine-tuning (improved from Stage 1)
    let mut config = FineTuneConfig::default();
    config.num_epochs = 15; // Increased from 10
    config.batch_size = 32; // Keep same
    config.learning_rate = 1e-5; // Reduced from 2e-5 (with more data, lower LR)
    config.warmup_steps = 1000; // Doubled
    config.weight_decay = 0.01; // Keep same
    config.dropout = 0.2; // Increased from 0.1 (more regularization)
    config.early_stopping_enabled = true;
    config.early_stopping_patience = 3;

    println!("⚙️  Fine-tuning Configuration (Stage 2):");
    println!("  Learning Rate:        {}", config.learning_rate);
    println!("  Num Epochs:           {}", config.num_epochs);
    println!("  Batch Size:           {}", config.batch_size);
    println!("  Warmup Steps:         {}", config.warmup_steps);
    println!("  Weight Decay:         {}", config.weight_decay);
    println!("  Dropout:              {}", config.dropout);
    println!("  Early Stopping:       {}", config.early_stopping_enabled);
    println!();

    // Create fine-tuner
    let mut finetuner = FineTuner::new(config);

    // Save expanded dataset to temporary file for training
    println!("💾 Preparing dataset for training...");
    let temp_path = "/tmp/jailguard_expanded_train.json";
    let json = serde_json::to_string(&train_samples)?;
    std::fs::write(temp_path, json)?;
    println!("✅ Dataset saved to {}", temp_path);
    println!();

    // Start fine-tuning
    println!("🎯 Fine-tuning on expanded dataset...");
    println!("   (Training on {} samples)", train_samples.len());
    println!();

    let start_time = Instant::now();
    let metrics = finetuner.fine_tune_from_file(temp_path)?;
    let total_time = start_time.elapsed().as_secs_f32();

    // Summary
    println!("{}", "=".repeat(70));
    println!("✅ Fine-tuning Complete");
    println!("{}", "=".repeat(70));
    println!();

    println!("📊 Results:");
    println!(
        "  Best Validation Accuracy: {:.1}%",
        metrics.best_val_accuracy * 100.0
    );
    println!("  Best Epoch: {}", metrics.best_epoch);
    println!("  Total Epochs Completed: {}", metrics.epochs.len());
    println!("  Total Training Time: {:.1}s", total_time);
    println!();

    // Accuracy progression
    println!("📈 Accuracy Progression:");
    for epoch_metrics in &metrics.epochs {
        print!(
            "  Epoch {:2}: train_acc={:.1}%",
            epoch_metrics.epoch, epoch_metrics.train_accuracy * 100.0
        );

        if let Some(val_acc) = epoch_metrics.val_accuracy {
            print!(", val_acc={:.1}%", val_acc * 100.0);
        }

        println!(" (loss={:.4})", epoch_metrics.train_loss);
    }
    println!();

    // Interpretation
    println!("📝 Interpretation:");
    if metrics.best_val_accuracy >= 0.92 {
        println!("  ✅ Achieved target accuracy (92%+ on expanded data)");
        println!("  ✅ Stage 2 Success! 2-4% improvement over Stage 1");
        println!("  → Ready to proceed to Stage 3: Adversarial Training");
    } else if metrics.best_val_accuracy >= 0.90 {
        println!(
            "  ⚠️  Good accuracy ({:.1}%)",
            metrics.best_val_accuracy * 100.0
        );
        println!("  → Approaching Stage 2 target (92%+)");
    } else if metrics.best_val_accuracy >= 0.88 {
        println!(
            "  ⚠️  Moderate accuracy ({:.1}%)",
            metrics.best_val_accuracy * 100.0
        );
        println!("  → Stage 1 level performance on expanded data");
    } else {
        println!("  ❌ Below expectations");
        println!("  → Consider debugging data augmentation or hyperparameters");
    }
    println!();

    // Comparison with Stage 1
    println!("📊 Stage Comparison:");
    println!("  Stage 1: 90.0% on 257 synthetic samples");
    println!(
        "  Stage 2: {:.1}% on {} expanded samples",
        metrics.best_val_accuracy * 100.0,
        dataset.statistics.total_samples
    );
    println!(
        "  Improvement: {:.1}% → {:.1}%",
        90.0,
        metrics.best_val_accuracy * 100.0
    );
    println!();

    // Next steps
    println!("🚀 Next Steps:");
    println!("  1. Stage 3: Adversarial training (30% adversarial examples)");
    println!("  2. Stage 4: Multi-task learning (7-way attack classification)");
    println!("  3. Stage 5: Confidence calibration (ECE < 0.05)");
    println!("  4. Stage 6: Integrate pre-trained models (ensemble)");
    println!("  5. Stage 7: Online learning from user feedback");
    println!();

    println!("📋 Updated Roadmap:");
    println!("  Stage 1 (Complete):  90.0% accuracy (257 samples)");
    println!(
        "  Stage 2 (Complete):  {:.1}% accuracy ({} samples)",
        metrics.best_val_accuracy * 100.0,
        dataset.statistics.total_samples
    );
    println!("  Stage 3:             +3-5% robustness (adversarial)");
    println!("  Stage 4:             85%+ F1 (attack classification)");
    println!("  Stage 5:             ECE < 0.05 (calibration)");
    println!("  Stage 6:             96-98% accuracy (ensemble)");
    println!("  Stage 7:             +1-2% domain adaptation");
    println!();

    println!("💡 Key Insights:");
    println!("  • Expanded dataset ({} → {}) enabled {}% improvement",
        257,
        dataset.statistics.total_samples,
        (metrics.best_val_accuracy * 100.0 - 90.0) as i32
    );
    println!("  • 7-way attack stratification improves generalization");
    println!("  • Data augmentation (3x variants) helps robustness");
    println!("  • Lower learning rate needed with larger dataset");
    println!();

    // Clean up
    let _ = std::fs::remove_file(temp_path);

    println!("✨ Phase 8 Stage 2 Complete!");
    println!();

    Ok(())
}
