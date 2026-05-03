# Inference

The production inference path is the embedded library API:

```rust
use jailguard::{detect, is_injection, score};

if is_injection("ignore previous instructions") {
    // ...
}

let result = detect("What is the capital of France?");
println!("score={}, injection={}", result.score, result.is_injection);
```

The MLP weights and tokenizer are compiled into the binary; the 90 MB ONNX
embedding model is auto-downloaded on first use to `~/.cache/jailguard/`.
Latency: p50 18 ms, p99 35 ms (Apple M3, single thread). See
[`BENCHMARKS.md`](../BENCHMARKS.md) for the full numbers and a head-to-head
comparison against other CPU detectors.

## Standalone tools

| File | Purpose |
|------|---------|
| `verify_json_model.rs` | Load a `models/*.json` MLP weight file outside the embedded path, run inference on a small fixture, and confirm the file deserialises and produces sensible scores. Useful when bringing in a freshly-trained checkpoint before swapping it into `models/neural_binary_200k.json`. Not registered as a `cargo` binary; build with `rustc` or wrap in `cargo run --example` if needed. |

For batch evaluation, use the benchmark binary in
[`jailguard_dataset`](https://github.com/yfedoseev/jailguard_dataset):

```bash
cd ~/projects/jailguard_dataset
cargo run --bin benchmark --release           # in-domain pipeline test
cargo run --bin benchmark --release -- --external  # adds J1N2 + shalyhinpavel OOD
```
