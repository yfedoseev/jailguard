# Implementing Gradient-Based Training

This guide explains how to implement actual gradient descent training for JailGuard's multi-task detector.

## Current State

The detector is **inference-ready** but weights are **randomly initialized**:
- Semantic embeddings: Deterministic hash-based (no learning)
- Transformer encoder: Random weights (untrained)
- Classification heads: Random weights (untrained)
- Loss function: Implemented but not used for backprop

**Current Performance**: 50% accuracy (random baseline)
**Expected After Training**: 85-95% accuracy

## Step-by-Step Implementation Plan

### Step 1: Set Up Autodiff Module (2-3 hours)

Create a new trainable detector wrapper with autodiff support:

```rust
// src/training/gradient_descent_trainer.rs

use burn::module::Module;
use burn::nn::{Linear, LinearConfig};
use burn::optim::{Adam, AdamConfig};
use burn::tensor::Tensor;
use burn_ndarray::NdArray;

type B = NdArray;

#[derive(Module)]
pub struct GradientDescentDetector {
    // Trainable heads (encoder embeddings stay fixed initially)
    pub binary_head: Linear<B>,
    pub attack_head: Linear<B>,  // Will use AttackClassifier
    pub semantic_head: Linear<B>,
}

impl GradientDescentDetector {
    pub fn new() -> Self {
        let device = Default::default();

        let binary_head = LinearConfig::new(384, 2).init(&device);
        let attack_head = LinearConfig::new(384, 7).init(&device);
        let semantic_head = LinearConfig::new(384, 1).init(&device);

        Self {
            binary_head,
            attack_head,
            semantic_head,
        }
    }

    pub fn forward(&self, embeddings: Tensor<B, 3>) -> (Tensor<B, 2>, Tensor<B, 2>, Tensor<B, 2>) {
        // Pool embeddings: [batch, seq_len, 384] -> [batch, 384]
        let pooled = embeddings.mean_dim(1);

        let binary_out = self.binary_head.forward(pooled.clone());
        let attack_out = self.attack_head.forward(pooled.clone());
        let semantic_out = self.semantic_head.forward(pooled);

        (binary_out, attack_out, semantic_out)
    }
}
```

### Step 2: Implement Training Loop (3-4 hours)

```rust
pub struct GradientTrainingConfig {
    pub learning_rate: f64,
    pub num_epochs: usize,
    pub batch_size: usize,
}

pub fn train_epoch<B: Backend>(
    model: &GradientDescentDetector,
    optimizer: &mut Adam<B>,
    train_loader: &DataLoader,
    loss_fn: &MultiLabelLoss,
) -> Result<f32> {
    let mut total_loss = 0.0;
    let mut num_batches = 0;

    for (embeddings, binary_targets, attack_targets, semantic_targets) in train_loader {
        // Forward pass
        let (binary_logits, attack_logits, semantic_scores) = model.forward(embeddings);

        // Compute loss
        let loss = loss_fn.compute(
            binary_logits,
            attack_logits,
            semantic_scores,
            binary_targets,
            attack_targets,
            semantic_targets,
        );

        // Backward pass (autograd)
        let grads = loss.backward();

        // Update weights
        optimizer.step(model, grads);

        total_loss += loss.to_scalar();
        num_batches += 1;
    }

    Ok(total_loss / num_batches as f32)
}
```

### Step 3: Create Training Data Loader (2-3 hours)

```rust
pub struct DataLoader {
    samples: Vec<MultiLabelTrainingSample>,
    batch_size: usize,
}

impl DataLoader {
    pub fn new(samples: Vec<MultiLabelTrainingSample>, batch_size: usize) -> Self {
        Self { samples, batch_size }
    }

    pub fn batches(&self) -> Vec<Batch> {
        self.samples
            .chunks(self.batch_size)
            .map(|chunk| Batch::from_samples(chunk.to_vec()))
            .collect()
    }
}

pub struct Batch {
    embeddings: Tensor<B, 3>,    // [batch_size, 1, 384]
    binary_targets: Tensor<B, 1>, // [batch_size]
    attack_targets: Tensor<B, 1>, // [batch_size]
    semantic_targets: Tensor<B, 2>, // [batch_size, 1]
}

impl Batch {
    pub fn from_samples(samples: Vec<MultiLabelTrainingSample>) -> Self {
        let batch_size = samples.len();
        let device = Default::default();

        // Generate embeddings from texts
        let embeddings: Vec<f32> = samples
            .iter()
            .flat_map(|s| hash_to_embedding(&s.text))
            .collect();

        let embeddings_tensor = Tensor::from_data(
            burn::tensor::Data::new(embeddings, [batch_size, 1, 384]),
            &device,
        );

        // Extract targets
        let binary_targets_vec: Vec<i64> = samples
            .iter()
            .map(|s| if s.is_injection { 1 } else { 0 })
            .collect();

        let binary_targets_tensor = Tensor::from_data(
            burn::tensor::Data::new(binary_targets_vec, [batch_size]),
            &device,
        );

        // Similar for attack and semantic targets...

        Self {
            embeddings: embeddings_tensor,
            binary_targets: binary_targets_tensor,
            attack_targets: attack_targets_tensor,
            semantic_targets: semantic_targets_tensor,
        }
    }
}
```

### Step 4: Integrate with Existing Trainer (2-3 hours)

Modify `src/training/multilabel_trainer.rs`:

```rust
pub struct MultiLabelTrainer {
    detector: MultiLabelDetector,
    gradient_detector: Option<GradientDescentDetector>,
    optimizer: Option<Adam<B>>,
    config: MultiLabelTrainingConfig,
}

impl MultiLabelTrainer {
    pub fn enable_gradient_training(&mut self) -> Result<()> {
        self.gradient_detector = Some(GradientDescentDetector::new());

        let adam_config = AdamConfig::new()
            .with_beta_1(0.9)
            .with_beta_2(0.999)
            .with_epsilon(1e-8);
        self.optimizer = Some(adam_config.init());

        Ok(())
    }

    pub fn train_epoch_with_gradients(
        &mut self,
        samples: &[MultiLabelTrainingSample],
    ) -> Result<TrainingMetrics> {
        if self.gradient_detector.is_none() {
            self.enable_gradient_training()?;
        }

        let detector = self.gradient_detector.as_ref().unwrap();
        let optimizer = self.optimizer.as_mut().unwrap();

        // Create data loader
        let loader = DataLoader::new(samples.to_vec(), self.config.batch_size);

        let mut total_loss = 0.0;
        for batch in loader.batches() {
            let (binary_logits, attack_logits, semantic_scores) =
                detector.forward(batch.embeddings);

            let loss = compute_multi_task_loss(
                binary_logits,
                attack_logits,
                semantic_scores,
                batch.binary_targets,
                batch.attack_targets,
                batch.semantic_targets,
                self.loss_config,
            );

            // Backward pass
            let grads = loss.backward();
            optimizer.step(detector, grads);

            total_loss += loss.to_scalar();
        }

        Ok(TrainingMetrics {
            loss: total_loss / loader.batches().len() as f32,
            // ... other metrics
        })
    }
}
```

### Step 5: Create Training Script (1-2 hours)

```rust
// examples/train_full_gradient.rs

fn main() -> Result<()> {
    let train_samples = load_samples("data/training/splits/train.json")?;
    let val_samples = load_samples("data/training/splits/val.json")?;
    let test_samples = load_samples("data/training/splits/test.json")?;

    let mut trainer = MultiLabelTrainer::new(lookup, config)?;
    trainer.enable_gradient_training()?;

    for epoch in 0..config.num_epochs {
        let train_metrics = trainer.train_epoch_with_gradients(&train_samples)?;
        let val_metrics = trainer.evaluate(&val_samples)?;

        println!(
            "Epoch {}: train_loss={:.4}, val_acc={:.1}%",
            epoch, train_metrics.loss, val_metrics.accuracy * 100.0
        );

        // Early stopping
        if val_metrics.accuracy > best_accuracy {
            best_accuracy = val_metrics.accuracy;
            trainer.save_checkpoint()?;
        }
    }

    // Evaluate on test set
    let test_metrics = trainer.evaluate(&test_samples)?;
    println!("Test Accuracy: {:.1}%", test_metrics.accuracy * 100.0);

    Ok(())
}
```

## Key Considerations

### 1. Tensor Operations
- Burn's tensor API requires careful shape management
- Use `.reshape()`, `.unsqueeze()`, `.mean_dim()` carefully
- Test shapes at each step with assertions

### 2. Loss Functions
- Softmax is numerically stable in burn
- Use `.log_softmax()` to avoid numerical issues
- Ensure targets are properly one-hot encoded

### 3. Optimization
- Adam learning rate: 1e-4 is good starting point
- Batch size: 32 is standard
- Epochs: 5-10 for convergence
- Early stopping on validation loss

### 4. Embedding Handling
- Keep pre-trained embeddings fixed initially
- Only train the 3 classification heads first
- After heads converge, fine-tune embeddings

### 5. Memory Management
- Use `.detach()` to prevent gradient accumulation
- Clear optimizer state between epochs
- Use `.to_data()` sparingly (converts to CPU)

## Testing Strategy

```bash
# 1. Test data loading
cargo test training::data_loader

# 2. Test forward pass
cargo test training::forward_pass

# 3. Test backward pass
cargo test training::backward_pass

# 4. Test full epoch
cargo test training::full_training_epoch

# 5. Test convergence
cargo test training::training_convergence
```

## Expected Improvement Curve

```
Epoch 1:  Loss: 0.8500, Accuracy: 52%
Epoch 2:  Loss: 0.7200, Accuracy: 65%
Epoch 3:  Loss: 0.5800, Accuracy: 74%
Epoch 4:  Loss: 0.4100, Accuracy: 82%
Epoch 5:  Loss: 0.2700, Accuracy: 87%
Epoch 10: Loss: 0.1200, Accuracy: 92%
```

## Debugging Tips

### Gradients Not Flowing?
1. Check that backward() is called on loss
2. Verify module parameters are trainable
3. Use `loss.backward()` return value

### Loss Not Decreasing?
1. Learning rate might be too high/low
2. Try 1e-4, 1e-3, 5e-4
3. Check target tensor shapes
4. Verify loss computation formula

### Out of Memory?
1. Reduce batch size
2. Use gradient accumulation
3. Clear cache periodically
4. Use f32 instead of f64

### Accuracy Not Improving?
1. Check data loading (are targets correct?)
2. Verify embeddings are meaningful
3. Test loss function independently
4. Check for bugs in forward pass

## Timeline Estimate

- **Setup**: 2-3 hours
- **Core training loop**: 3-4 hours
- **Data loader**: 2-3 hours
- **Integration**: 2-3 hours
- **Testing & debugging**: 4-5 hours
- **Optimization**: 2-3 hours

**Total**: ~18-21 hours of focused development

## Success Milestones

- [ ] Gradient computation working (loss decreases)
- [ ] Single epoch trains successfully
- [ ] Validation metrics tracked correctly
- [ ] Early stopping implemented
- [ ] Test accuracy >80%
- [ ] Attack type accuracy >40%
- [ ] Training converges in <10 epochs
- [ ] Latency acceptable (<100ms for 100 samples)

## Next Phases After Gradient Training

1. **ONNX Embeddings**: Replace hash-based with real semantic embeddings
2. **Adversarial Training**: Add attack variations to training data
3. **Quantization**: Reduce model size for deployment
4. **Ensemble**: Combine multiple detectors
5. **Production API**: Serve with FastAPI/Actix-web

---

**Difficulty Level**: Intermediate (burn tensor operations)
**Time Commitment**: 18-21 focused hours
**Priority**: High - blocks accuracy improvements
**Blocker**: None - all infrastructure ready
