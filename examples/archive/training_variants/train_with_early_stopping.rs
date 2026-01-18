//! Example: Training with Early Stopping and Checkpointing
//!
//! Demonstrates how to use EarlyStopper and CheckpointManager to:
//! - Stop training when validation loss plateaus
//! - Save best model checkpoints
//! - Prevent overfitting through monitoring

use jailguard::training::{Checkpoint, CheckpointManager, EarlyStopper, EarlyStoppingConfig};

fn main() {
    // Create early stopper with custom configuration
    let early_stop_config = EarlyStoppingConfig::default()
        .with_patience(3) // Stop after 3 evaluations without improvement
        .with_min_delta(0.001); // Minimum 0.1% improvement to count

    let mut early_stopper = EarlyStopper::new(early_stop_config);
    let mut checkpoint_manager = CheckpointManager::default();

    // Simulate a training loop
    println!("Training with Early Stopping\n");
    println!(
        "{:<6} {:<12} {:<15} {:<10}",
        "Epoch", "Val Loss", "Status", "Best Step"
    );
    println!("{}", "-".repeat(45));

    let mut epoch = 0;
    let mut step = 0;

    // Simulated validation losses (showing convergence then plateau)
    let losses = vec![
        0.8500, 0.7200, 0.5800, 0.4200, 0.2000, // Improving phase
        0.1500, 0.1480, 0.1475, 0.1475, 0.1475, // Plateau phase
    ];

    for (idx, &val_loss) in losses.iter().enumerate() {
        step = idx;
        epoch = idx;

        // Check if should stop
        let should_stop = early_stopper.should_stop(val_loss, step);

        // Create and save checkpoint
        let checkpoint = Checkpoint::new(
            step,
            epoch,
            val_loss,
            0.85 + (idx as f32 * 0.01), // Mock accuracy
            0.9,                        // Mock train loss
            0.82,                       // Mock train accuracy
        );

        let is_best = checkpoint_manager.save_if_best(checkpoint.clone());

        let status = if is_best {
            "✓ Best".to_string()
        } else if early_stopper.is_exhausted() {
            "✗ Stop".to_string()
        } else {
            format!("  {}/{}", early_stopper.steps_without_improvement(), 3)
        };

        println!(
            "{:<6} {:<12.4} {:<15} {:<10}",
            epoch, val_loss, status, early_stopper.best_step
        );

        if should_stop {
            println!("\n⏹️  Early stopping triggered!");
            break;
        }
    }

    // Print summary
    println!("\n{}", "=".repeat(45));
    println!("Training Summary:");
    println!("  Total epochs: {}", epoch + 1);
    println!("  Best validation loss: {:.4}", early_stopper.best_loss());
    println!("  Best epoch: {}", early_stopper.best_step);
    println!("  Checkpoints saved: {}", checkpoint_manager.len());

    if let Some(best_ckpt) = checkpoint_manager.best() {
        println!("\nBest Checkpoint:");
        println!("  Epoch: {}", best_ckpt.epoch);
        println!("  Val Loss: {:.4}", best_ckpt.val_loss);
        println!("  Val Accuracy: {:.2}%", best_ckpt.val_accuracy * 100.0);
        println!("  Train Loss: {:.4}", best_ckpt.train_loss);
        println!("  Train Accuracy: {:.2}%", best_ckpt.train_accuracy * 100.0);
    }

    // Show checkpoint history
    if !checkpoint_manager.history().is_empty() {
        println!("\nCheckpoint History (best 5):");
        for (idx, ckpt) in checkpoint_manager.history().iter().enumerate() {
            println!(
                "  {}: Epoch {} - Val Loss {:.4} (Acc {:.2}%)",
                idx + 1,
                ckpt.epoch,
                ckpt.val_loss,
                ckpt.val_accuracy * 100.0
            );
        }
    }

    // Demonstrate training time savings
    let epochs_without_stopping = losses.len();
    let epochs_with_stopping = epoch + 1;
    let saved_epochs = epochs_without_stopping - epochs_with_stopping;
    let savings_percent = (saved_epochs as f32 / epochs_without_stopping as f32) * 100.0;

    println!("\nTraining Time Savings:");
    println!(
        "  Without early stopping: {} epochs",
        epochs_without_stopping
    );
    println!("  With early stopping: {} epochs", epochs_with_stopping);
    println!("  Epochs saved: {} ({:.1}%)", saved_epochs, savings_percent);
}
