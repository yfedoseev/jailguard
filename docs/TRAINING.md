# JailGuard Training Guide

## Overview

This guide covers training and fine-tuning the JailGuard detection model using the Burn deep learning framework.

## Dataset Preparation

### Recommended Datasets

1. **Primary**: deepset/prompt-injections
   - 1,000+ labeled injection examples
   - Multiple attack types
   - High quality annotations

2. **Augmentation**: Custom dataset creation
   - Benign examples from your domain
   - Attack variations using adversarial generation
   - Multi-lingual examples

### Data Format

```rust
pub struct MultiTaskSample {
    pub text: String,
    pub is_injection: bool,
    pub attack_type: AttackType,  // 7-way classification
    pub expected_output: Option<String>,  // For semantic similarity
}

pub enum AttackType {
    RoleplayInjection,
    InstructionOverride,
    ContextManipulation,
    OutputManipulation,
    EncodingObfuscation,
    JailbreakPatterns,
    Benign,
}
```

### Creating Training Dataset

```python
import json
from datasets import load_dataset

# Load base dataset
ds = load_dataset("deepset/prompt-injections", split="train")

# Convert to JailGuard format
training_samples = []
for item in ds:
    sample = {
        "text": item["text"],
        "is_injection": item["label"] == "injection",
        "attack_type": classify_attack_type(item["text"]),
        "expected_output": None,
    }
    training_samples.append(sample)

# Save as JSON
with open("training_data.json", "w") as f:
    json.dump(training_samples, f)
```

## Adversarial Training

### Attack Synthesis

```rust
pub struct AdversarialGenerator {
    // 30% of training batch is adversarial
    adversarial_ratio: f32,

    // Three attack variants per sample:
    char_substitution: CharSubstitutionAttack,
    encoding_attack: EncodingAttack,
    paraphrase_attack: ParaphraseAttack,
}
```

### Attack Types

1. **Character Substitution** (40% of adversarial)
   - Homoglyph substitution (а → a, е → e)
   - Leetspeak (a → 4, e → 3, i → 1)
   - Case variation (InJeCtIoN)

2. **Encoding Attack** (30% of adversarial)
   - Base64 encoding
   - URL encoding
   - ROT13/Caesar cipher
   - Unicode escaping

3. **Paraphrasing** (30% of adversarial)
   - Synonym substitution
   - Grammatical variation
   - Semantic equivalence

### Implementation

```rust
let adversarial_gen = AdversarialGenerator::new();

// Generate 3 variants of each injection
for sample in &injection_samples {
    let variant1 = adversarial_gen.char_substitute(&sample.text);
    let variant2 = adversarial_gen.encode(&sample.text);
    let variant3 = adversarial_gen.paraphrase(&sample.text);

    // All variants keep the original label
    training_data.push(MultiTaskSample {
        text: variant1,
        is_injection: sample.is_injection,
        attack_type: sample.attack_type.clone(),
        ..sample.clone()
    });
    // ... variant2, variant3
}
```

## Model Training

### Configuration

```rust
pub struct TrainerConfig {
    pub learning_rate: f64,      // 1e-4
    pub batch_size: usize,       // 32
    pub num_epochs: usize,       // 20
    pub warmup_steps: usize,     // 1000
    pub weight_decay: f64,       // 1e-5
}

pub struct TransformerConfig {
    pub vocab_size: usize,       // 30,000
    pub embed_dim: usize,        // 256
    pub num_heads: usize,        // 4
    pub num_layers: usize,       // 3
    pub ff_dim: usize,           // 1024
    pub max_seq_len: usize,      // 512
    pub dropout: f64,            // 0.1
}
```

### Multi-Task Loss

```rust
pub struct MultiTaskLoss {
    alpha: f32,   // 0.6 - binary classification
    beta: f32,    // 0.3 - attack type classification
    gamma: f32,   // 0.1 - semantic similarity
}

// Total loss: 0.6*L_binary + 0.3*L_attack + 0.1*L_semantic
```

### Training Loop

```rust
for epoch in 0..config.num_epochs {
    let mut epoch_loss = 0.0;

    for (batch_idx, batch) in train_loader.iter().enumerate() {
        // Forward pass
        let logits_binary = model.binary_head(&encoder_output);
        let logits_attack = model.attack_head(&encoder_output);
        let scores_semantic = model.semantic_head(&encoder_output);

        // Compute multi-task loss
        let loss_binary = cross_entropy(&logits_binary, &batch.labels_binary);
        let loss_attack = cross_entropy(&logits_attack, &batch.labels_attack);
        let loss_semantic = mse_loss(&scores_semantic, &batch.labels_semantic);

        let loss = 0.6 * loss_binary + 0.3 * loss_attack + 0.1 * loss_semantic;

        // Backward pass
        loss.backward();
        optimizer.step();
        optimizer.zero_grad();

        epoch_loss += loss.value();
    }

    println!("Epoch {}: Loss = {:.4}", epoch, epoch_loss / batch_count);
}
```

## Validation and Evaluation

### Metrics

```rust
pub struct EvaluationMetrics {
    // Binary classification
    pub binary_accuracy: f32,
    pub precision: f32,
    pub recall: f32,
    pub f1_score: f32,

    // Attack type classification
    pub attack_accuracy: f32,
    pub per_class_f1: HashMap<AttackType, f32>,

    // Calibration
    pub ece: f32,  // Expected Calibration Error
    pub mce: f32,  // Maximum Calibration Error
}
```

### Evaluation Code

```rust
let metrics = evaluate(&model, &val_loader);

println!("Binary Accuracy: {:.2}%", metrics.binary_accuracy * 100.0);
println!("Attack Type Accuracy: {:.2}%", metrics.attack_accuracy * 100.0);
println!("F1 Score: {:.4}", metrics.f1_score);
println!("ECE: {:.4}", metrics.ece);

// Verify targets met
assert!(metrics.binary_accuracy > 0.95, "Binary accuracy < 95%");
assert!(metrics.attack_accuracy > 0.85, "Attack accuracy < 85%");
assert!(metrics.ece < 0.05, "ECE > 0.05");
```

## Confidence Calibration

### Temperature Scaling

```rust
pub struct TemperatureScaling {
    temperature: f32,  // Learnable parameter
}

impl TemperatureScaling {
    pub fn calibrate(&mut self, val_logits: &Tensor, val_targets: &Tensor) {
        // Optimize temperature to minimize NLL loss
        for _ in 0..100 {
            let logits_scaled = val_logits / self.temperature;
            let loss = nll_loss(&logits_scaled, &val_targets);
            // Gradient descent on temperature
            self.temperature -= 0.01 * loss_gradient;
        }
    }

    pub fn apply(&self, logits: &Tensor) -> Tensor {
        logits / self.temperature
    }
}
```

### ECE Calculation

```rust
pub fn compute_ece(probs: &[f32], targets: &[bool], num_bins: usize) -> f32 {
    let mut bin_correct = vec![0; num_bins];
    let mut bin_total = vec![0; num_bins];
    let mut bin_confidence = vec![0.0; num_bins];

    for (prob, &target) in probs.iter().zip(targets.iter()) {
        let bin_idx = (prob * num_bins as f32) as usize;
        bin_total[bin_idx] += 1;
        bin_confidence[bin_idx] += prob;
        if (prob > 0.5) == target {
            bin_correct[bin_idx] += 1;
        }
    }

    let mut ece = 0.0;
    let total = targets.len() as f32;

    for i in 0..num_bins {
        if bin_total[i] > 0 {
            let accuracy = bin_correct[i] as f32 / bin_total[i] as f32;
            let confidence = bin_confidence[i] / bin_total[i] as f32;
            let weight = bin_total[i] as f32 / total;

            ece += weight * (confidence - accuracy).abs();
        }
    }

    ece
}
```

## Online Learning / Fine-Tuning

### Feedback Collection

```rust
pub struct FeedbackCollector {
    buffer: VecDeque<FeedbackSample>,
    max_size: usize,
}

pub struct FeedbackSample {
    pub text: String,
    pub predicted: DetectionResult,
    pub actual: Option<bool>,  // User correction
    pub timestamp: SystemTime,
}

impl FeedbackCollector {
    pub fn add_feedback(&mut self, sample: FeedbackSample) {
        self.buffer.push_back(sample);
        if self.buffer.len() > self.max_size {
            self.buffer.pop_front();
        }
    }

    pub fn get_corrections(&self) -> Vec<MultiTaskSample> {
        self.buffer
            .iter()
            .filter(|s| s.predicted.is_injection != s.actual.unwrap_or(false))
            .map(|s| MultiTaskSample {
                text: s.text.clone(),
                is_injection: s.actual.unwrap_or(false),
                ..Default::default()
            })
            .collect()
    }
}
```

### Incremental Training

```rust
pub struct IncrementalTrainer {
    detector: TransformerDetector,
    optimizer: Adam,
    feedback_collector: FeedbackCollector,
}

impl IncrementalTrainer {
    pub fn update_from_feedback(&mut self) {
        let corrections = self.feedback_collector.get_corrections();

        if corrections.is_empty() {
            return;  // No corrections to learn from
        }

        // Use very conservative learning rate (1e-5)
        for correction in corrections {
            let output = self.detector.forward(&correction.text);
            let loss = compute_loss(&output, &correction.is_injection);

            loss.backward();
            self.optimizer.step();
            self.optimizer.zero_grad();
        }
    }
}
```

## Deployment Checklist

- [ ] Validation accuracy >95% on held-out test set
- [ ] Attack type accuracy >85%
- [ ] ECE < 0.05 (confidence calibration)
- [ ] CPU latency <30ms on target hardware
- [ ] GPU latency <5ms if applicable
- [ ] Model size <20MB
- [ ] All 150+ tests passing
- [ ] Benchmark tests meeting targets
- [ ] No catastrophic overfitting on training set
- [ ] Robustness to adversarial examples >50% improvement

## Performance Optimization

### Model Pruning

```rust
// Remove low-magnitude weights
pub fn prune_model(model: &mut TransformerDetector, threshold: f32) {
    for param in model.parameters_mut() {
        param[param.abs() < threshold] = 0.0;
    }
}
```

### Quantization

```rust
// FP32 → FP16 for inference
pub fn quantize_fp16(weights: &[f32]) -> Vec<f16> {
    weights.iter().map(|&w| f16::from_f32(w)).collect()
}
```

### Attention Optimization

```rust
// Kernel fusion for attention + FFN
pub fn fused_attention_ffn(
    query: &Tensor,
    key: &Tensor,
    value: &Tensor,
    w1: &Tensor,
    w2: &Tensor,
) -> Tensor {
    // Single kernel instead of 3 separate operations
    // Reduces memory bandwidth, improves cache locality
}
```

## Troubleshooting

### Training Loss Not Decreasing

```
Possible causes:
1. Learning rate too low (1e-5 → 1e-4)
2. Batch size too small (16 → 32)
3. Data quality issues (check labels)
4. Model architecture mismatch

Solution:
- Start with learning rate 1e-3, decay to 1e-5
- Verify data on sample
- Use smaller model first (test architecture)
```

### High False Positive Rate

```
Possible causes:
1. Block threshold too low (0.5 → 0.7)
2. Model underfitted (more epochs, data)
3. Domain mismatch (different text type)

Solution:
- Increase threshold progressively
- Train on domain-specific data
- Add domain-specific patterns to spotlighting
```

### Low Attack Type Accuracy

```
Possible causes:
1. Insufficient adversarial training
2. Attack type labels incorrect
3. Model capacity too low

Solution:
- Increase adversarial ratio (30% → 40%)
- Use multi-task learning (0.3 weight on attack)
- Increase model size (256→512 embedding dim)
```

## Advanced Topics

### Continual Learning Without Catastrophic Forgetting

```rust
// Use rehearsal: mix old data with new feedback
pub fn continual_update(
    feedback: &[FeedbackSample],
    old_data: &[MultiTaskSample],
    ratio: f32,  // 0.3 = 30% old, 70% new
) {
    let old_batch_size = (batch_size as f32 * ratio) as usize;
    let new_batch_size = batch_size - old_batch_size;

    for epoch in 0..5 {
        // Mix old and new data
        let mut mixed_batch = sample_batch(&old_data, old_batch_size);
        mixed_batch.extend(sample_batch(&feedback, new_batch_size));

        // Train on mixed batch
        train_batch(&mixed_batch);
    }
}
```

### Transfer Learning from Other Models

```rust
// Load pre-trained transformer
let pretrained = load_pretrained_bert();

// Replace detection head only
let mut model = TransformerDetector {
    encoder: pretrained.encoder,  // Reuse
    binary_head: initialize_new_head(),
    attack_head: initialize_new_head(),
    semantic_head: initialize_new_head(),
};

// Train only new heads first (frozen encoder)
train_with_frozen_backbone(&mut model, &training_data);

// Fine-tune all parameters
unfreeze_backbone(&mut model);
train_full_model(&mut model, &training_data);
```

## See Also

- [API Documentation](./API.md)
- [Architecture Documentation](./ARCHITECTURE.md)
- [Burn Framework Docs](https://burn.dev)
- Example: `examples/train_multitask.rs`
