# Training

Training scripts for JailGuard prompt injection detection models.

## Scripts

| Script | Description | Command |
|--------|-------------|---------|
| `train_neural_binary.rs` | Train on 15K dataset (99.62% accuracy) | `cargo run --bin train_neural_binary --release` |
| `train_on_expanded_dataset.rs` | Train on 125K balanced dataset (99.62% accuracy) | `cargo run --bin train_on_expanded_dataset --release` |
| `train_minilm_with_gradients.rs` | Baseline gradient descent training | `cargo run --bin train_minilm_with_gradients --release` |
| `evaluate_on_test_set.rs` | Train & evaluate with model export | `cargo run --bin evaluate_on_test_set --release` |
| `fast_embedding_generation.rs` | Generate embeddings (100-200x faster than Python) | `cargo run --bin fast_embedding_generation --release` |

## Quick Start

```bash
# Train best model (99.62% accuracy)
cargo run --bin train_on_expanded_dataset --release

# Or train smaller model (faster, 99.62% accuracy)
cargo run --bin train_neural_binary --release
```

## Output

Training produces model files in `models/`:
- `jailguard_injection_detector.json` - Human-readable format
- `jailguard_injection_detector.safetensors` - Hugging Face format
- `jailguard_injection_detector.onnx.metadata.json` - ONNX conversion metadata

## Requirements

- Training data in `data/` or `splits_200k/`
- Run `bash scripts/download_large_datasets.sh` if data not present
