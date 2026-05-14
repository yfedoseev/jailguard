//! Phase 6 trainer orchestrating the complete training loop.
//!
//! Manages training epochs, validation, learning rate scheduling, and early stopping.
#![allow(missing_docs)]

use super::early_stopping::{EarlyStopper, EarlyStoppingConfig};
use super::neural_data_loader::{NeuralDataLoader, NeuralEmbeddingSample};
#[allow(deprecated)]
use super::neural_multitask_network::NeuralMultitaskNetwork;

/// Metrics from a training epoch.
#[derive(Debug, Clone)]
pub struct NeuralTrainingMetrics {
    /// Epoch number
    pub epoch: u32,
    /// Training loss
    pub train_loss: f32,
    /// Validation loss
    pub val_loss: f32,
    /// Training accuracy
    pub train_accuracy: f32,
    /// Validation accuracy
    pub val_accuracy: f32,
    /// Attack type classification F1
    pub attack_f1: f32,
    /// Learning rate used this epoch
    pub learning_rate: f32,
    /// Time elapsed in seconds
    pub elapsed_secs: f32,
}

/// Learning rate schedule.
#[derive(Debug, Clone)]
pub enum NeuralLRSchedule {
    /// Constant learning rate
    Constant { lr: f32 },
    /// Warmup then exponential decay
    ExponentialDecay {
        initial_lr: f32,
        warmup_steps: u32,
        decay_rate: f32,
    },
    /// Warmup then linear decay
    LinearDecay {
        initial_lr: f32,
        warmup_steps: u32,
        total_steps: u32,
    },
}

impl NeuralLRSchedule {
    /// Get learning rate for a given step.
    pub fn get_lr(&self, step: u32) -> f32 {
        match self {
            NeuralLRSchedule::Constant { lr } => *lr,
            NeuralLRSchedule::ExponentialDecay {
                initial_lr,
                warmup_steps,
                decay_rate,
            } => {
                if step < *warmup_steps {
                    // Linear warmup
                    initial_lr * step as f32 / *warmup_steps as f32
                } else {
                    // Exponential decay after warmup
                    let decay_steps = (step - warmup_steps) as f32;
                    initial_lr * decay_rate.powf(decay_steps / 1000.0)
                }
            }
            NeuralLRSchedule::LinearDecay {
                initial_lr,
                warmup_steps,
                total_steps,
            } => {
                if step < *warmup_steps {
                    // Linear warmup
                    initial_lr * step as f32 / *warmup_steps as f32
                } else {
                    // Linear decay after warmup
                    let remaining_steps = (*total_steps - step) as f32;
                    let total_decay_steps = (*total_steps - warmup_steps) as f32;
                    initial_lr * (remaining_steps / total_decay_steps)
                }
            }
        }
    }
}

/// Phase 6 trainer configuration.
#[derive(Debug, Clone)]
pub struct NeuralTrainerConfig {
    /// Initial learning rate
    pub learning_rate: f32,
    /// Learning rate schedule
    pub lr_schedule: NeuralLRSchedule,
    /// Batch size
    pub batch_size: usize,
    /// Number of epochs
    pub num_epochs: u32,
    /// Early stopping patience (number of epochs)
    pub patience: u32,
    /// Balance batches (50% inj, 50% benign)
    pub balance_batches: bool,
    /// Print interval (every N epochs)
    pub print_interval: u32,
}

impl Default for NeuralTrainerConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            lr_schedule: NeuralLRSchedule::ExponentialDecay {
                initial_lr: 0.001,
                warmup_steps: 100,
                decay_rate: 0.95,
            },
            batch_size: 32,
            num_epochs: 30,
            patience: 5,
            balance_batches: true,
            print_interval: 1,
        }
    }
}

/// Phase 6 trainer.
#[allow(deprecated)]
pub struct NeuralTrainer {
    network: NeuralMultitaskNetwork,
    config: NeuralTrainerConfig,
    early_stopper: EarlyStopper,
    total_steps: u32,
    metrics_history: Vec<NeuralTrainingMetrics>,
}

#[allow(deprecated)]
impl NeuralTrainer {
    /// Create new trainer.
    pub fn new(config: NeuralTrainerConfig) -> Self {
        let early_stopping_config =
            EarlyStoppingConfig::default().with_patience(config.patience as usize);
        Self {
            network: NeuralMultitaskNetwork::new(config.learning_rate),
            config,
            early_stopper: EarlyStopper::new(early_stopping_config),
            total_steps: 0,
            metrics_history: Vec::new(),
        }
    }

    /// Train for a single epoch.
    pub fn train_epoch(
        &mut self,
        loader: &NeuralDataLoader,
    ) -> Result<NeuralTrainingMetrics, String> {
        let start = std::time::Instant::now();

        // Create batches
        let batches = loader.create_batches(self.config.batch_size, self.config.balance_batches);

        let mut epoch_loss = 0.0;
        let mut epoch_correct = 0;
        let mut epoch_total = 0;

        // Training loop
        for batch in &batches {
            for (embedding, is_injection, attack_type) in batch {
                // Get learning rate for this step
                let lr = self.config.lr_schedule.get_lr(self.total_steps);
                self.network.set_learning_rate(lr);

                // Training step
                self.network
                    .train_step(embedding, *is_injection, *attack_type, None);

                // Evaluate for metrics
                let loss = self
                    .network
                    .evaluate_sample(embedding, *is_injection, *attack_type);
                let (pred_is_inj, _, _pred_attack) = self.network.forward(embedding);

                epoch_loss += loss;
                epoch_correct += if pred_is_inj == *is_injection { 1 } else { 0 };
                epoch_total += 1;

                self.total_steps += 1;
            }
        }

        let train_loss = epoch_loss / epoch_total as f32;
        let train_accuracy = epoch_correct as f32 / epoch_total as f32;

        // Validation
        let (val_loss, val_accuracy, attack_f1) = self.evaluate(&loader.val_samples)?;

        let elapsed = start.elapsed().as_secs_f32();

        let current_lr = self.config.lr_schedule.get_lr(self.total_steps);

        let metrics = NeuralTrainingMetrics {
            epoch: self.metrics_history.len() as u32,
            train_loss,
            val_loss,
            train_accuracy,
            val_accuracy,
            attack_f1,
            learning_rate: current_lr,
            elapsed_secs: elapsed,
        };

        Ok(metrics)
    }

    /// Evaluate on a dataset.
    pub fn evaluate(&self, samples: &[NeuralEmbeddingSample]) -> Result<(f32, f32, f32), String> {
        let mut total_loss = 0.0;
        let mut correct = 0;
        let mut attack_correct = 0;

        for sample in samples {
            let loss = self.network.evaluate_sample(
                &sample.embedding,
                sample.is_injection,
                sample.attack_type,
            );
            let (pred_is_inj, _, pred_attack) = self.network.forward(&sample.embedding);

            total_loss += loss;
            correct += if pred_is_inj == sample.is_injection {
                1
            } else {
                0
            };
            attack_correct += if pred_attack == sample.attack_type {
                1
            } else {
                0
            };
        }

        let avg_loss = total_loss / samples.len() as f32;
        let accuracy = correct as f32 / samples.len() as f32;
        let attack_accuracy = attack_correct as f32 / samples.len() as f32;

        // Approximate F1 as accuracy for now
        Ok((avg_loss, accuracy, attack_accuracy))
    }

    /// Train for multiple epochs.
    pub fn train(
        &mut self,
        loader: &NeuralDataLoader,
    ) -> Result<Vec<NeuralTrainingMetrics>, String> {
        println!("=== Phase 6 Training ===");
        println!(
            "Samples: Train={}, Val={}, Test={}",
            loader.train_samples.len(),
            loader.val_samples.len(),
            loader.test_samples.len()
        );
        println!(
            "Epochs: {}, Batch Size: {}, LR: {:.6}",
            self.config.num_epochs, self.config.batch_size, self.config.learning_rate
        );
        println!();

        for epoch in 0..self.config.num_epochs {
            let metrics = self.train_epoch(loader)?;
            self.metrics_history.push(metrics.clone());

            // Print progress
            if epoch % self.config.print_interval == 0 {
                println!(
                    "Epoch {:3}/{}: train_loss={:.4}, train_acc={:.2}%, val_loss={:.4}, val_acc={:.2}%, lr={:.6}, {:.1}s",
                    epoch + 1,
                    self.config.num_epochs,
                    metrics.train_loss,
                    metrics.train_accuracy * 100.0,
                    metrics.val_loss,
                    metrics.val_accuracy * 100.0,
                    metrics.learning_rate,
                    metrics.elapsed_secs
                );
            }

            // Check early stopping
            if self
                .early_stopper
                .should_stop(metrics.val_loss, epoch as usize)
            {
                println!("\n✓ Early stopping triggered after epoch {}", epoch + 1);
                break;
            }
        }

        println!();
        Ok(self.metrics_history.clone())
    }

    /// Get best validation accuracy.
    pub fn best_val_accuracy(&self) -> f32 {
        self.metrics_history
            .iter()
            .map(|m| m.val_accuracy)
            .fold(0.0, f32::max)
    }

    /// Get metrics history.
    pub fn metrics(&self) -> &[NeuralTrainingMetrics] {
        &self.metrics_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lr_schedule_constant() {
        let schedule = NeuralLRSchedule::Constant { lr: 0.001 };
        assert_eq!(schedule.get_lr(0), 0.001);
        assert_eq!(schedule.get_lr(1000), 0.001);
    }

    #[test]
    fn test_lr_schedule_warmup() {
        let schedule = NeuralLRSchedule::ExponentialDecay {
            initial_lr: 0.001,
            warmup_steps: 100,
            decay_rate: 0.95,
        };

        let lr_0 = schedule.get_lr(0);
        let lr_50 = schedule.get_lr(50);
        let lr_100 = schedule.get_lr(100);

        assert!(lr_0 < lr_50);
        assert!(lr_50 < lr_100);
    }

    #[test]
    fn test_trainer_config() {
        let config = NeuralTrainerConfig::default();
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.num_epochs, 30);
    }

    #[test]
    fn test_trainer_creation() {
        let config = NeuralTrainerConfig::default();
        let trainer = NeuralTrainer::new(config);
        assert_eq!(trainer.metrics_history.len(), 0);
    }

    #[test]
    fn test_best_val_accuracy() {
        let config = NeuralTrainerConfig::default();
        let mut trainer = NeuralTrainer::new(config);

        trainer.metrics_history.push(NeuralTrainingMetrics {
            epoch: 0,
            train_loss: 0.5,
            val_loss: 0.4,
            train_accuracy: 0.8,
            val_accuracy: 0.75,
            attack_f1: 0.7,
            learning_rate: 0.001,
            elapsed_secs: 1.0,
        });

        trainer.metrics_history.push(NeuralTrainingMetrics {
            epoch: 1,
            train_loss: 0.3,
            val_loss: 0.35,
            train_accuracy: 0.9,
            val_accuracy: 0.85,
            attack_f1: 0.8,
            learning_rate: 0.0005,
            elapsed_secs: 1.0,
        });

        assert_eq!(trainer.best_val_accuracy(), 0.85);
    }
}
