# Pre-trained Models

This directory ships with the production JailGuard prompt-injection detector
weights and tokenizer that get compiled into the published library.

## Files

| File | Size | Description |
|------|------|-------------|
| `neural_binary_200k.json` | ~1.4 MB | Production MLP weights (v3, English-only, trained on 79,626-sample pipeline). Compiled into the binary via `include_str!`. |
| `neural_binary_200k_backup_v3.json` | ~1.4 MB | Byte-identical safety copy of the v3 weights. Restore via `cp` if `neural_binary_200k.json` is corrupted or replaced. |
| `tokenizer.json` | ~466 KB | `all-MiniLM-L6-v2` WordPiece tokenizer (vocab=30,522). Compiled into the binary via `include_bytes!`. |
| `jailguard_injection_detector.safetensors` | 795 B | SafeTensors index for HuggingFace Hub integration. |
| `jailguard_injection_detector.onnx.metadata.json` | 1.4 KB | ONNX conversion metadata. |

The 90 MB ONNX embedding model (`all-MiniLM-L6-v2.onnx`) is **not** bundled —
it is auto-downloaded to `~/.cache/jailguard/` on first use by
`jailguard::download_model()`.

## Performance

Authoritative benchmark numbers and methodology live in
[`jailguard_dataset/BENCHMARKS.md`](https://github.com/yfedoseev/jailguard_dataset/blob/main/BENCHMARKS.md).
Headline numbers (re-validated 2026-05-03):

| Test set | Samples | Accuracy | Precision | Recall | F1 | p50 latency |
|----------|---------|----------|-----------|--------|-----|-------------|
| Pipeline (in-distribution) | 5,945 | **99.34%** | 97.52% | 99.54% | 0.985 | 18 ms |
| J1N2 mix (OOD) | 5,000 | **99.38%** | 98.09% | 99.94% | 0.990 | 18 ms |
| shalyhinpavel hard-negatives (OOD) | 147 | **89.12%** | 76.60% | 87.80% | 0.818 | 18 ms |

## Architecture

```
Input: 384-dim embedding (all-MiniLM-L6-v2, ONNX, mean-pooled, L2-normalized)
  ↓
Dense Layer: 384 → 256 (ReLU, Dropout 0.2)
  ↓
Dense Layer: 256 → 128 (ReLU, Dropout 0.2)
  ↓
Output Layer: 128 → 1 (Sigmoid)
  ↓
Output: Injection probability [0.0, 1.0]   (threshold 0.5)
```

131,585 trainable parameters. Trained with Adam (β₁=0.9, β₂=0.999, ε=1e-8,
lr=0.001), weighted BCE (injection_weight=2.5), batch size 64, 50 epochs
with patience 10.

## Using the model

The compiled jailguard library exposes the embedded model directly:

```rust
use jailguard::{detect, is_injection};

if is_injection("ignore previous instructions") {
    println!("Blocked!");
}

let result = detect("What is the capital of France?");
println!("score={}, is_injection={}", result.score, result.is_injection);
```

No file paths, no config — the weights and tokenizer are embedded in the
binary. The ONNX embedding model is auto-downloaded on first use.

## Training a new model

The full pipeline (download → normalize → embed → train) lives in the
sibling [`jailguard_dataset`](https://github.com/yfedoseev/jailguard_dataset)
repository:

```sh
cd ../jailguard_dataset
HUGGINGFACE_TOKEN=hf_xxx cargo run --bin pipeline --release -- --download
cargo run --bin pipeline --release -- --normalize --force
cargo run --bin pipeline --release -- --train --force --output models/neural_binary_new.json
```

Then drop the result into this directory:

```sh
cp ../jailguard_dataset/models/neural_binary_new.json models/neural_binary_200k.json
cargo build --release           # rebuilds the library with the new weights
cargo test  --release           # exercises the end-to-end integration tests
```

For local quick experiments without the full pipeline, the `train/` directory
provides a thin wrapper as a `cargo` example (requires `--features full`):

```sh
cargo run --example train_neural_binary --features full --release -- \
    --data path/to/embeddings.json \
    --output models/my_model.json
```

The reproducibility caveat documented in `jailguard-datasets/BENCHMARKS.md` applies: re-running
training on the same data produces a functionally equivalent model (same
accuracy, same architecture, same weight statistics) but not byte-identical
weights, due to non-deterministic CPU ONNX multi-threading.
