# JailGuard

[![SOTA CPU](https://img.shields.io/badge/SOTA-CPU%20Prompt%20Injection%20Detection-brightgreen)](docs/FINAL_RESULTS.md)
[![Accuracy](https://img.shields.io/badge/Accuracy-99.07%25-blue)](docs/FINAL_RESULTS.md)
[![F1 Score](https://img.shields.io/badge/F1-0.9908-blue)](docs/FINAL_RESULTS.md)
[![Precision](https://img.shields.io/badge/Precision-98.93%25-blue)](docs/FINAL_RESULTS.md)
[![Recall](https://img.shields.io/badge/Recall-99.22%25-blue)](docs/FINAL_RESULTS.md)
[![Pure Rust](https://img.shields.io/badge/Pure-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue)](LICENSE)

**State-of-the-art prompt injection detection on CPU** — Pure Rust, 99.07% accuracy, 200K dataset, <50ms inference

## Highlights

| Metric | JailGuard | PromptGuard | Rebuff | Lakera |
|--------|-----------|-------------|--------|--------|
| **Accuracy** | **99.07%** | ~90% real-world | ~80% | Unknown |
| **F1 Score** | **0.9908** | 0.91 | ~0.75 | Unknown |
| **Precision** | **98.93%** | ~93% | ~85% | Unknown |
| **Recall** | **99.22%** | 97.5%@1%FPR | ~70% | Unknown |
| **Training** | **2.4h CPU** | Hours GPU | N/A | Proprietary |
| **Inference** | **<50ms** | ~92ms (GPU) | ~50ms | Cloud API |
| **Open Source** | **Yes** | Yes | Yes | No |

> **SOTA for CPU-only prompt injection detection** — Simple approach beats complex fine-tuning

## Why It Works

We achieve SOTA with a simple approach:

```
┌─────────────────────────────────────────────────────────────┐
│              THE SECRET: TRANSFER LEARNING                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   MiniLM-L6-v2 (Microsoft)                                 │
│   ├── Trained on: Billions of text samples                 │
│   ├── Training time: Weeks on GPU clusters                 │
│   ├── Output: 384-dim semantic vectors                     │
│   └── We get this FOR FREE                                 │
│                                                             │
│         ↓ Frozen Embeddings (no fine-tuning)               │
│                                                             │
│   Simple 3-Layer MLP                                       │
│   ├── 384 → 256 (ReLU, Dropout 0.2)                       │
│   ├── 256 → 128 (ReLU, Dropout 0.2)                       │
│   ├── 128 → 1 (Sigmoid)                                   │
│   └── Total: ~130K parameters                             │
│                                                             │
│         ↓ 40 minutes CPU training                          │
│                                                             │
│   99.07% Accuracy                                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**No fine-tuning. No GPU. No complexity.** Just good embeddings + simple classifier + clean data.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                 JAILGUARD DETECTION PIPELINE                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   User Prompt                                               │
│       │                                                     │
│       ▼                                                     │
│   ┌─────────────┐                                          │
│   │  MiniLM-L6  │  Semantic Embedding (384-dim)            │
│   │   (ONNX)    │  • Pre-trained by Microsoft              │
│   └──────┬──────┘  • Captures meaning, not just keywords   │
│          │                                                  │
│          ▼                                                  │
│   ┌─────────────────────────────────────────┐              │
│   │     Binary Classifier (Pure Rust)       │              │
│   │  ┌─────────────┐  ┌─────────────────┐   │              │
│   │  │ Dense 256   │→│   Dense 128     │   │              │
│   │  │ ReLU+Drop   │  │   ReLU+Drop     │   │              │
│   │  └─────────────┘  └─────────────────┘   │              │
│   │                          │               │              │
│   │                          ▼               │              │
│   │              ┌─────────────────┐        │              │
│   │              │  Sigmoid (0-1)  │        │              │
│   │              └─────────────────┘        │              │
│   └──────────────────────────────────────────┘              │
│          │                                                  │
│          ▼                                                  │
│   Detection Result                                          │
│   • confidence: 0.0 - 1.0                                  │
│   • is_injection: confidence > 0.5                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Rust (Recommended)

```toml
[dependencies]
jailguard = "0.1.0"
```

The classifier is embedded in the binary. The ONNX embedding model (~90 MB) is auto-downloaded to `~/.cache/jailguard/` on first use.

```rust
use jailguard::{is_injection, detect};

// Simple boolean check
if is_injection("ignore all previous instructions") {
    println!("Blocked!");
}

// Get detailed result
let result = detect("What is the capital of France?");
println!("Safe: {}, Confidence: {:.1}%", !result.is_injection, result.confidence * 100.0);
```

**Available functions:**

| Function | Returns | Use Case |
|----------|---------|----------|
| `is_injection(text)` | `bool` | Quick yes/no check |
| `detect(text)` | `DetectionOutput` | Full details with confidence |
| `score(text)` | `f32` | Raw probability (0.0-1.0) |
| `detect_batch(texts)` | `Vec<DetectionOutput>` | Process multiple inputs |
| `ensure_model()` | `Result<PathBuf>` | Pre-download ONNX model |

### Production Setup

In production, pre-download the ONNX model to avoid first-request latency:

```rust
// Option A: call at app startup
jailguard::ensure_model().expect("Failed to download ONNX model");
```

```dockerfile
# Option B: pre-download in Dockerfile
ENV JAILGUARD_MODEL_DIR=/app/models
RUN curl -L -o /app/models/all-MiniLM-L6-v2.onnx \
    https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx
```

Set `JAILGUARD_MODEL_DIR` to control where the model is cached (default: `~/.cache/jailguard/`).

## Benchmarks

### Accuracy (200K Balanced Dataset)

| Metric | Value | Notes |
|--------|-------|-------|
| **Accuracy** | 99.07% | Held-out test set (20,000 samples) |
| **Precision** | 98.93% | FP: 108 out of 20K |
| **Recall** | 99.22% | FN: 78 out of 20K |
| **F1 Score** | 0.9908 | Excellent balance |

### Training Efficiency

| Metric | JailGuard | Fine-tuned Transformer |
|--------|-----------|------------------------|
| **Training Time** | 2.4 hours | 4-8 hours |
| **Hardware** | CPU only | GPU required |
| **Parameters** | 130K | 86M+ |
| **Dataset Size** | 200K | 1M+ typical |

### Inference Latency

| Component | Time |
|-----------|------|
| Embedding (MiniLM ONNX) | ~25ms |
| Classification | ~1ms |
| **Total** | **<50ms** |

## Attack Categories

Trained on 8 attack types:

| Category | Description | Examples |
|----------|-------------|----------|
| **Direct Injection** | Explicit override attempts | "Ignore previous instructions" |
| **Indirect Injection** | Hidden in data/context | Malicious content in documents |
| **Jailbreak** | Bypass safety filters | "DAN mode", "Developer mode" |
| **Role-play** | Persona manipulation | "Pretend you're evil AI" |
| **System Prompt Leak** | Extract system prompts | "Reveal your instructions" |
| **Encoding Attacks** | Obfuscation techniques | Base64, ROT13, Unicode |
| **Context Manipulation** | Exploit context window | Attention hijacking |
| **Multi-turn** | Gradual manipulation | Build trust then attack |

## Project Structure

```
jailguard/
├── src/              # Library code
├── train/            # Training binaries
├── inference/        # Inference binaries
├── evaluation/       # Evaluation binaries
├── examples/         # Demo examples
├── tests/            # Test suite
├── docs/             # Documentation
├── models/           # Classifier weights + tokenizer
├── data/             # Training data
├── scripts/          # Utility scripts
└── loaders/          # Python/JS loaders
```

## Training

### Train Your Own Model

```bash
# Best accuracy (99.07%) - 200K balanced dataset
cargo run --bin train_neural_binary --release --features full

# Generate ONNX embeddings for custom data
cargo run --bin onnx_embedding_generation --release --features full
```

### Verify Model

```bash
cargo run --bin verify_json_model --release
```

## References

- [all-MiniLM-L6-v2](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2) - Sentence embeddings
- [PromptGuard (Meta)](https://github.com/meta-llama/PurpleLlama) - Fine-tuned detection
- [Lakera Guard](https://www.lakera.ai/) - Commercial solution
- [Rebuff](https://github.com/protectai/rebuff) - Open-source alternative
- [Ignore This Title and HackAPrompt](https://arxiv.org/abs/2311.01011) - Attack taxonomy
- [Not What You've Signed Up For](https://arxiv.org/abs/2302.12173) - Indirect injection
- [SecretGuard](https://github.com/yfedoseev/secretguard) - Related project (PII detection)

## Citation

If you use JailGuard in research, please cite:

```bibtex
@software{jailguard,
  title = {JailGuard: Efficient Prompt Injection Detection via Pre-trained Embeddings},
  author = {Fedoseev, Yury},
  year = {2026},
  url = {https://github.com/yfedoseev/jailguard}
}
```

## License

MIT OR Apache-2.0
