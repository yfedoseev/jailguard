# Pre-trained Models

This directory ships the JailGuard prompt-injection detector weights and
tokenizer that get compiled into the published library.

## Files

| File | Size | Description |
|------|------|-------------|
| `neural_binary_200k.json` | ~1.4 MB | Iter-9 MLP weights, trained on the 17-source public pipeline (see [`BENCHMARKS.md`](../BENCHMARKS.md#training-dataset-stats)). Compiled into the binary via `include_str!`. |
| `tokenizer.json` | ~466 KB | `all-MiniLM-L6-v2` WordPiece tokenizer (vocab=30,522). Compiled into the binary via `include_bytes!`. |

The 90 MB ONNX embedding model (`all-MiniLM-L6-v2.onnx`) is **not** bundled —
it is auto-downloaded to `~/.cache/jailguard/` on first use by
`jailguard::download_model()`.

## Performance

Authoritative benchmark numbers and methodology live in
[`BENCHMARKS.md`](../BENCHMARKS.md). Headline numbers (iter-9):

| Test set | Samples | Accuracy | Precision | Recall | F1 |
|----------|---------|----------|-----------|--------|-----|
| Pipeline (in-distribution) | 7,049 | **98.40%** | 98.56% | 97.98% | 0.983 |
| J1N2 mix (OOD) | 5,000 | **99.38%** | 98.09% | 99.94% | 0.990 |
| shalyhinpavel hard-negatives (OOD) | 147 | **89.12%** | 76.60% | 87.80% | 0.818 |

Latency on Apple M3, single CPU thread: p50 ≈ 14 ms, p99 ≈ 18 ms.

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

131,585 trainable parameters. Trained with SGD (lr=0.01) + binary
cross-entropy, batch size 64, 50 epochs with patience-10 early stopping.

## Using the model

The published crate exposes the embedded model directly — no file paths,
no config:

```rust
use jailguard::{detect, is_injection};

if is_injection("ignore previous instructions") {
    println!("Blocked!");
}

let result = detect("What is the capital of France?");
println!("score={}, is_injection={}", result.score, result.is_injection);
```

## Replacing the weights

The model file is a plain JSON blob of named tensors. Drop a new file in
place and rebuild:

```sh
cp /path/to/new_weights.json models/neural_binary_200k.json
cargo build --release
cargo test  --release
```

The training pipeline that produces these weights is not part of the
public crate. The per-source dataset mix, deduplication strategy, and
training hyperparameters are documented in [`BENCHMARKS.md`](../BENCHMARKS.md).

> **Reproducibility caveat:** re-running training on the same data
> produces a functionally equivalent model (same accuracy, same
> architecture, same weight statistics) but not byte-identical weights,
> due to non-deterministic CPU ONNX multi-threading.
