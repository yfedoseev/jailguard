# Training

Standalone training binaries for the JailGuard MLP classifier. The
**production** training pipeline lives in the sibling
[`jailguard_dataset`](https://github.com/yfedoseev/jailguard_dataset) repo
(`cargo run --bin pipeline -- --download/--normalize/--train`); the binaries
here are thin wrappers for ad-hoc experiments on pre-computed embeddings.

All examples require `--features full`.

## Examples

| Example | Description |
|---------|-------------|
| `train_neural_binary.rs` | Train the production-equivalent 384→256→128→1 MLP on a JSON file of pre-computed embeddings. Adam (lr=0.001), weighted BCE (injection_weight=2.5), 50 epochs with patience 10. |
| `onnx_embedding_generation.rs` | Generate `paraphrase-multilingual-MiniLM-L12-v2` or `all-MiniLM-L6-v2` embeddings from a JSON file of `{text, is_injection, attack_type, attack_type_idx, source}` records. |
| `evaluate_on_test_set.rs` | Load a trained MLP and report accuracy/precision/recall/F1 over a test split. |

## Quick experiment

```bash
# 1. Generate embeddings (uses the same ONNX session the production library does).
cargo run --example onnx_embedding_generation --features full --release -- \
    --input  path/to/dataset.json \
    --output path/to/embeddings.json \
    --model-dir ~/.cache/jailguard/

# 2. Train.
cargo run --example train_neural_binary --features full --release -- \
    --data   path/to/embeddings.json \
    --output models/my_model.json
```

`--lr` and `--injection-weight` flags are available on `train_neural_binary`
if you want to override the defaults.

## Production training

To rebuild the deployed v3 weights from scratch, use the Rust pipeline in
`jailguard_dataset`:

```bash
cd ~/projects/jailguard_dataset
HUGGINGFACE_TOKEN=hf_xxx cargo run --bin pipeline --release -- --download
cargo run --bin pipeline --release -- --normalize --force
cargo run --bin pipeline --release -- --train --force \
    --output models/neural_binary_new.json

# Drop the result into this repo and rebuild:
cp models/neural_binary_new.json ~/Projects/jailguard/models/neural_binary_200k.json
cd ~/Projects/jailguard
cargo build --release
cargo test --release    # exercises the embedded.rs integration tests
```

Expect ~25 min to embed the 79,626-sample pipeline dataset on Apple Silicon
+ ~25 min to train. Reproducibility caveat is in
[`jailguard_dataset/BENCHMARKS.md`](https://github.com/yfedoseev/jailguard_dataset/blob/main/BENCHMARKS.md):
the procedure is functionally reproducible but not byte-identical because of
non-deterministic CPU ONNX multi-threading.
