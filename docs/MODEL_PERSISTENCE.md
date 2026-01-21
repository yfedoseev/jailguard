# Model Persistence & Inference Guide

## Overview

The JailGuard neural network can now be trained, saved, and loaded for production inference. This document explains the complete workflow for model serialization and usage.

---

## Architecture

### Model Structure

```
Input (384-dim embedding)
    ↓
Dense Layer 1: 384 → 256
    ↓
ReLU Activation
    ↓
Dropout (p=0.2)
    ↓
Dense Layer 2: 256 → 128
    ↓
ReLU Activation
    ↓
Dropout (p=0.2)
    ↓
Output Layer: 128 → 1
    ↓
Sigmoid Activation
    ↓
Output (0.0 to 1.0)
```

**Total Parameters**: ~200,000 float32 values

### Performance Characteristics

- **Model Size**: ~1 MB (JSON format)
- **Inference Latency**: <1ms per sample (CPU)
- **Throughput**: >1000 samples/second
- **Memory Footprint**: ~1 MB loaded

---

## Training and Saving

### Step 1: Train the Model

```bash
cargo run --example evaluate_on_test_set --release
```

This will:
1. Load the 125K balanced training dataset
2. Train for up to 50 epochs with early stopping (patience=10)
3. Evaluate on the held-out test set
4. **Automatically save the trained model** to `models/jailguard_injection_detector.json`

### Step 2: Verify Model File

```bash
ls -lh models/jailguard_injection_detector.json
```

Expected output:
```
-rw-r--r-- 1 user user 1.1M Jan 19 18:45 models/jailguard_injection_detector.json
```

### Model File Format

The model is saved as JSON with the following structure:

```json
{
  "w_h1": [[...], [...], ...],  // 256 × 384 weight matrix
  "b_h1": [...],                 // 256 biases
  "w_h2": [[...], [...], ...],   // 128 × 256 weight matrix
  "b_h2": [...],                 // 128 biases
  "w_out": [[...]],              // 1 × 128 weight matrix
  "b_out": [0.123],              // 1 bias
  "learning_rate": 0.01,         // Training learning rate (informational)
  "dropout_rate": 0.2            // Dropout rate during training (informational)
}
```

---

## Loading and Inference

### Basic Usage

```rust
use jailguard::training::NeuralBinaryNetwork;
use jailguard::embeddings::FastEmbedder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the trained model
    let network = NeuralBinaryNetwork::load("models/jailguard_injection_detector.json")?;

    // Initialize embedder
    let embedder = FastEmbedder::new();

    // Get embedding for text
    let text = "What is the capital of France?";
    let embedding = embedder.embed(text);

    // Run inference
    let prediction = network.forward_eval(&embedding);
    let is_injection = prediction > 0.5;
    let confidence = if is_injection { prediction } else { 1.0 - prediction };

    println!("Text: {}", text);
    println!("Is Injection: {}", is_injection);
    println!("Confidence: {:.4}", confidence);

    Ok(())
}
```

### Run the Complete Inference Example

```bash
cargo run --example load_and_inference --release
```

This demonstrates:
- Loading the saved model
- Generating embeddings for test texts
- Running inference on individual samples
- Batch inference on multiple samples
- Model information and statistics

---

## Production Integration

### Saving Models to Production

```rust
// After training completes
network.save("models/jailguard_v1.0.json")?;
```

### Loading for Inference

```rust
// In production code
let model = NeuralBinaryNetwork::load("path/to/model.json")?;

// Reuse for multiple predictions
for text in texts {
    let embedding = embedder.embed(&text);
    let prediction = model.forward_eval(&embedding);
    process_result(prediction);
}
```

### Batch Processing

```rust
let embedder = FastEmbedder::new();
let model = NeuralBinaryNetwork::load("models/jailguard_injection_detector.json")?;

let texts = vec![
    "Normal query",
    "Malicious prompt injection attempt",
    // ...
];

let predictions: Vec<f32> = texts
    .iter()
    .map(|text| {
        let embedding = embedder.embed(text);
        model.forward_eval(&embedding)
    })
    .collect();

for (text, pred) in texts.iter().zip(predictions.iter()) {
    let is_injection = pred > &0.5;
    println!("{}: {}", text, if is_injection { "INJECTION" } else { "BENIGN" });
}
```

---

## Model Versioning

### Recommended Naming Scheme

```
models/
├── jailguard_v1.0.json          # First production release
├── jailguard_v1.1.json          # Improved with 200K samples
├── jailguard_v2.0.json          # Major version with 8-class taxonomy
└── jailguard_latest.json        # Symlink to current best
```

### Version Metadata

Create a `models/metadata.json` file:

```json
{
  "version": "1.0",
  "date": "2026-01-19",
  "accuracy_test": 0.9962,
  "precision": 0.9990,
  "recall": 0.9793,
  "f1_score": 0.9890,
  "dataset_size": 125000,
  "epochs_trained": 6,
  "training_time_seconds": 2349,
  "architecture": "384-256-128-1",
  "embedding_dim": 384,
  "embedding_model": "FastEmbedder (Rust)",
  "notes": "Phase 5 final model, 99.62% test accuracy, zero overfitting"
}
```

---

## API Integration Examples

### REST API Endpoint

```rust
use actix_web::{web, HttpResponse, post};

#[post("/predict")]
async fn predict(
    text: web::Json<String>,
    model: web::Data<NeuralBinaryNetwork>,
    embedder: web::Data<FastEmbedder>,
) -> HttpResponse {
    let embedding = embedder.embed(&text);
    let prediction = model.forward_eval(&embedding);
    let confidence = (prediction * 100.0) as u32;

    HttpResponse::Ok().json(serde_json::json!({
        "text": &text,
        "is_injection": prediction > 0.5,
        "confidence": confidence,
        "score": prediction,
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let model = NeuralBinaryNetwork::load("models/jailguard_injection_detector.json")
        .expect("Failed to load model");
    let embedder = FastEmbedder::new();

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(web::Data::new(model.clone()))
            .app_data(web::Data::new(embedder.clone()))
            .service(predict)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
```

---

## Performance Benchmarks

### Single Sample Inference

```
Embedding generation:  ~0.1ms (FastEmbedder)
Model inference:       <1ms (384→256→128→1)
Total latency:         <1.1ms
```

### Batch Processing (1000 samples)

```
Total time:            ~1 second
Throughput:            1000+ samples/second
Per-sample average:    <1ms
```

### Memory Usage

```
Model in memory:       ~1 MB
Embedder:              ~0.5 MB
Working memory:        ~0.5 MB
Total per process:     ~2 MB
```

---

## Migration from Previous Versions

### From In-Memory Training

**Before** (weights lost on exit):
```rust
let mut network = NeuralBinaryNetwork::new(0.01);
// ... train ...
// Weights are lost when program exits
```

**After** (persistent model):
```rust
let mut network = NeuralBinaryNetwork::new(0.01);
// ... train ...
network.save("models/my_model.json")?;  // ✅ Save weights

// Later:
let network = NeuralBinaryNetwork::load("models/my_model.json")?;
```

---

## Troubleshooting

### Model File Not Found

```
Error: "Model file not found at models/jailguard_injection_detector.json"
```

**Solution**: First train and save the model:
```bash
cargo run --example evaluate_on_test_set --release
```

### JSON Parse Error

```
Error: "Failed to deserialize model: ..."
```

**Solution**: Ensure the JSON file is not corrupted:
```bash
cat models/jailguard_injection_detector.json | jq . > /dev/null
```

### Different Results on Different Runs

⚠️ **Important**: The FastEmbedder is deterministic, so the same text will always produce the same embedding. However, model predictions are **deterministic** but may appear different due to floating-point precision differences across systems.

---

## Model Updates

### When to Retrain

- Dataset expanded significantly (>20% more samples)
- New attack categories discovered
- Performance drops below baseline (95%)
- Major architectural changes needed

### Retraining Procedure

1. Update training data
2. Run training example: `cargo run --example evaluate_on_test_set --release`
3. Evaluate metrics and verify improvement
4. Save new model with version number: `models/jailguard_v2.0.json`
5. Update metadata file
6. Deploy with gradual rollout

---

## Future Enhancements

- [ ] Multi-class classification (8-class attack taxonomy)
- [ ] Quantization for even faster inference
- [ ] Model compression (distillation)
- [ ] Ensemble methods combining multiple models
- [ ] Online learning for continuous improvement
- [ ] A/B testing framework for model updates

---

## Quick Reference

| Task | Command |
|------|---------|
| Train & Save | `cargo run --example evaluate_on_test_set --release` |
| Test Inference | `cargo run --example load_and_inference --release` |
| Check Model | `ls -lh models/jailguard_injection_detector.json` |
| Validate JSON | `cat models/jailguard_injection_detector.json \| jq .` |

---

## File Structure

```
jailguard/
├── models/
│   ├── jailguard_injection_detector.json  # Trained model weights
│   └── metadata.json                      # Model information
├── examples/
│   ├── evaluate_on_test_set.rs           # Training & saving
│   └── load_and_inference.rs             # Loading & inference
└── src/training/
    └── neural_binary_network.rs          # Model implementation
```

---

## Related Documentation

- [FINAL_RESULTS.md](FINAL_RESULTS.md) - Complete training results
- [DATASET_GENERATION_GUIDE.md](DATASET_GENERATION_GUIDE.md) - Data preparation
- [PHASE_6_IMPLEMENTATION.md](PHASE_6_IMPLEMENTATION.md) - Evaluation framework
