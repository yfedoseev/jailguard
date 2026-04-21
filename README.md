# JailGuard

[![Accuracy](https://img.shields.io/badge/Accuracy-99.07%25-blue)](CHANGELOG.md)
[![F1 Score](https://img.shields.io/badge/F1-0.9908-blue)](CHANGELOG.md)
[![Pure Rust](https://img.shields.io/badge/Pure-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue)](LICENSE)

**Fast, lightweight prompt injection detection in pure Rust.** Embedded 130K-parameter classifier, CPU-only, sub-50 ms inference, zero-config API.

## Highlights

- **99.07% accuracy / 0.9908 F1** on a 20K held-out split of our 200K training mix.
- **CPU-only.** No GPU required for inference or training.
- **Embedded model.** The classifier ships inside the crate; the 90 MB ONNX embedding model is auto-downloaded and cached on first use.
- **Small surface area.** Three functions: `detect()`, `is_injection()`, `score()`.
- **Permissive.** MIT OR Apache-2.0.

> Numbers above are measured on a held-out split of the project's own 200K dataset mix. Independent benchmark validation on [Lakera PINT](https://github.com/lakeraai/pint-benchmark) and [AgentDojo](https://agentdojo.spylab.ai/) is planned for a future release.

## How it works

JailGuard pairs a frozen sentence-embedding model with a small classifier:

1. **MiniLM-L6-v2** (384-dim, ONNX) produces a semantic vector for the input.
2. A 3-layer MLP (384 → 256 → 128 → 1, ~130K parameters, ReLU + dropout 0.2 + sigmoid) scores it as injection vs. benign.

The embedding model is frozen — no fine-tuning — which keeps the training and inference cost on CPU modest.

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

## Measurements

All numbers measured on a held-out 20,000-sample split of the project's 200K
training mix — **not** an independent public benchmark. Independent PINT and
AgentDojo runs are planned for a later release.

| Metric    | Value   | Notes                              |
|-----------|---------|------------------------------------|
| Accuracy  | 99.07%  | 20,000-sample held-out split       |
| Precision | 98.93%  | 108 false positives out of 20K     |
| Recall    | 99.22%  | 78 false negatives out of 20K      |
| F1        | 0.9908  |                                    |

### Latency (single CPU thread)

| Component               | Time    |
|-------------------------|---------|
| Embedding (MiniLM ONNX) | ~25 ms  |
| Classification (MLP)    | ~1 ms   |
| **Total**               | **<50 ms** |

## Attack categories covered in training

The classifier is binary (injection / benign), but its training mix spans eight
attack families:

| Category              | Examples                                |
|-----------------------|-----------------------------------------|
| Direct injection      | "Ignore previous instructions"          |
| Jailbreak             | DAN, developer-mode prompts             |
| Role-play             | Persona-based overrides                 |
| System prompt leak    | "Reveal your instructions"              |
| Encoding attacks      | Base64, ROT13, Unicode obfuscation      |
| Context manipulation  | Framing and separator tricks            |
| Output manipulation   | Format coercion                         |
| Indirect injection    | Malicious content embedded in documents |

## References

- [all-MiniLM-L6-v2](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2) — sentence embeddings
- [PromptGuard (Meta)](https://github.com/meta-llama/PurpleLlama)
- [Lakera Guard](https://www.lakera.ai/) and the [PINT benchmark](https://github.com/lakeraai/pint-benchmark)
- [Rebuff](https://github.com/protectai/rebuff)
- [Sentinel: SOTA model to protect against prompt injections](https://arxiv.org/abs/2506.05446)
- [Not What You've Signed Up For — indirect injection](https://arxiv.org/abs/2302.12173)

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
