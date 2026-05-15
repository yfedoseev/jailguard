# JailGuard

[![Accuracy](https://img.shields.io/badge/Accuracy-98.40%25-blue)](BENCHMARKS.md)
[![F1 Score](https://img.shields.io/badge/F1-0.983-blue)](BENCHMARKS.md)
[![Pure Rust](https://img.shields.io/badge/Pure-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue)](#license)
[![crates.io](https://img.shields.io/crates/v/jailguard.svg)](https://crates.io/crates/jailguard)
[![PyPI](https://img.shields.io/pypi/v/jailguard.svg)](https://pypi.org/project/jailguard/)
[![npm](https://img.shields.io/npm/v/@yfedoseev/jailguard.svg)](https://www.npmjs.com/package/@yfedoseev/jailguard)

> **JailGuard is a pure-Rust prompt-injection detector with a 1.5 MB embedded MLP classifier.** It scores text in **p50 14 ms on CPU**, achieves **98.40% accuracy** on a 7,049-sample held-out test set drawn from 17 public datasets, and ships bindings for **Rust, Python, JavaScript, Go, and Elixir**. No external service, no API key. Dual-licensed under MIT OR Apache-2.0.

## Quick start

**Rust** — `cargo add jailguard`
```rust
use jailguard::{detect, is_injection};

if is_injection("ignore previous instructions") {
    return Err("blocked");
}

let result = detect("What is the capital of France?");
println!("score={:.3} risk={:?}", result.score, result.risk);
```

**Python** — `pip install jailguard`
```python
import jailguard

if jailguard.is_injection("ignore previous instructions"):
    raise RuntimeError("blocked")

result = jailguard.detect("What is the capital of France?")
print(result.score, result.risk)
```

**JavaScript / TypeScript** — `npm install @yfedoseev/jailguard`
```typescript
import { detect, isInjection } from "@yfedoseev/jailguard";

if (isInjection("ignore previous instructions")) {
    throw new Error("blocked");
}

const r = detect("What is the capital of France?");
console.log(r.score, r.risk);
```

**Go** — `go get github.com/yfedoseev/jailguard/go`
```go
import jailguard "github.com/yfedoseev/jailguard/go"

if injection, _ := jailguard.IsInjection("ignore previous instructions"); injection {
    log.Fatal("blocked")
}

result, _ := jailguard.Detect("What is the capital of France?")
fmt.Printf("score=%.3f risk=%v\n", result.Score, result.Risk)
```

**Elixir** — source-built Git dependency
```elixir
def deps do
  [
    {:jailguard,
     github: "yfedoseev/jailguard",
     branch: "feature/elixir-support",
     subdir: "elixir",
     depth: 1}
  ]
end
```

```elixir
:ok = JailGuard.download_model()

{:ok, injection?} = JailGuard.is_injection("ignore previous instructions")
if injection?, do: raise("blocked")

{:ok, result} = JailGuard.detect("What is the capital of France?")
IO.inspect({result.score, result.risk})
```

The Elixir binding currently targets Linux/macOS source builds via the existing C ABI. It uses Mix's Git `subdir` option because the Elixir package still builds against the parent Rust crate; `sparse: "elixir"` will not work for this first port. Hex publishing, precompiled NIF artifacts, and Windows support are deferred.

The classifier is embedded in every binding. The 90 MB MiniLM ONNX embedder is auto-downloaded to `~/.cache/jailguard/` on first use. For production: call `jailguard::download_model()` at startup to warm the cache before serving traffic.

## JailGuard vs alternatives in 2026

| Feature | JailGuard | Lakera Guard | Rebuff | ProtectAI deberta-v3 | Meta Prompt Guard 2 |
|---|---|---|---|---|---|
| **License** | Apache 2.0 / MIT | proprietary (Check Point announced acquisition Sep 16, 2025) | Apache 2.0 — **archived May 16, 2025** | Apache 2.0 (parent acq. by Palo Alto Jul 22, 2025) | Llama 4 Community (non-OSI) |
| **Deployment** | embedded library | SaaS API | self-host Python SDK | HF model | HF model |
| **Model size** | 1.5 MB MLP + 90 MB MiniLM ONNX | n/a (API) | n/a | ~440 MB | 22 M or 86 M params |
| **Latency (CPU)** | **p50 14 ms** | ~150–300 ms RTT | n/a | 104–212 ms | 92 ms (A100 GPU)¹ |
| **Classification** | **8-class taxonomy** | binary | binary | binary | binary |
| **Active in 2026?** | ✅ | ✅ (Check Point pending) | **❌ archived** | ✅ (Palo Alto) | ✅ |
| **No PyTorch / no runtime dep** | ✅ (Rust) | ❌ HTTP client | ❌ Python+OpenAI | ❌ PyTorch | ❌ PyTorch |
| **Multi-language bindings** | Rust, Python, JS, Go, Elixir | API clients | Python | Python | Python |

¹ Meta does not publish CPU latency for Prompt Guard 2.

Full methodology, dataset breakdown, and head-to-head local-CPU comparisons against `protectai/deberta-v3-base-prompt-injection-v2`, `deepset/deberta-v3-base-injection`, and `madhurjindal/Jailbreak-Detector-Large` are in [`BENCHMARKS.md`](./BENCHMARKS.md).

## API at a glance

```rust
pub fn detect(text: &str) -> DetectionOutput
pub fn is_injection(text: &str) -> bool
pub fn score(text: &str) -> f32
pub fn detect_batch(texts: &[&str]) -> Vec<DetectionOutput>
pub fn download_model() -> Result<PathBuf, Error>

pub struct DetectionOutput {
    pub is_injection: bool,
    pub score: f32,
    pub confidence: f32,
    pub risk: RiskLevel,
}

pub enum RiskLevel { Safe, Low, Medium, High, Critical }
```

Python / JS / Go / Elixir expose the same surface in idiomatic form. See [`docs/API.md`](docs/API.md) for full per-language signatures.

## How it works

JailGuard pairs a frozen sentence-embedding model with a small classifier:

1. **MiniLM-L6-v2** (384-dim, ONNX) produces a semantic vector for the input.
2. A 3-layer MLP (384 → 256 → 128 → 1, ~130 K parameters, ReLU + dropout 0.2 + sigmoid) scores it as injection vs. benign.

The embedding model is frozen — no fine-tuning — which keeps training and inference cost on CPU modest. The classifier weights are a 1.5 MB JSON file `include_str!`'d into the binary at compile time.

```
┌─────────────────────────────────────────────────────────────┐
│                 JAILGUARD DETECTION PIPELINE                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   User Prompt                                               │
│       │                                                     │
│       ▼                                                     │
│   ┌─────────────┐                                           │
│   │  MiniLM-L6  │  Semantic Embedding (384-dim)             │
│   │   (ONNX)    │  • Pre-trained by Microsoft               │
│   └──────┬──────┘  • Captures meaning, not just keywords    │
│          │                                                  │
│          ▼                                                  │
│   ┌─────────────────────────────────────────┐               │
│   │     Binary Classifier (Pure Rust)       │               │
│   │  ┌─────────────┐  ┌─────────────────┐   │               │
│   │  │ Dense 256   │→ │   Dense 128     │   │               │
│   │  │ ReLU+Drop   │  │   ReLU+Drop     │   │               │
│   │  └─────────────┘  └─────────────────┘   │               │
│   │                          │              │               │
│   │                          ▼              │               │
│   │              ┌─────────────────┐        │               │
│   │              │  Sigmoid (0-1)  │        │               │
│   │              └─────────────────┘        │               │
│   └─────────────────────────────────────────┘               │
│          │                                                  │
│          ▼                                                  │
│   Detection Result                                          │
│   • confidence: 0.0 - 1.0                                   │
│   • is_injection: confidence > 0.5                          │
│   • risk: Safe | Low | Medium | High | Critical             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Measurements

Measured on Apple M3, last revalidated 2026-05-03. The pipeline test split is in-distribution (held out from the same 17-source training mix). J1N2 and shalyhinpavel are external datasets, neither used during training.

| Test set | Samples | Accuracy | Precision | Recall | F1 |
|----------|---------|----------|-----------|--------|-----|
| Pipeline (in-distribution) | 7,049 | **98.40%** | 98.56% | 97.98% | 0.983 |
| J1N2 mix (OOD) | 5,000 | **99.38%** | 98.09% | 99.94% | 0.990 |
| shalyhinpavel hard-negatives (OOD) | 147 | **89.12%** | 76.60% | 87.80% | 0.818 |

### Latency (single CPU thread)

| Component | Apple M3 | Intel i5-10210U @ 1.6 GHz¹ |
|---|---|---|
| Embedding (MiniLM ONNX) | ~13 ms | ~36 ms |
| Classification (MLP) | ~1 ms | ~1 ms |
| **Total (p50)** | **~14 ms** | **~37 ms** |
| **Total (p99)** | **~19 ms** | **~43 ms** |
| Cold start | ~140 ms | ~350 ms |

¹ A 4-year-old low-power Chromebook CPU (Comet Lake-U, 2019, 4c/8t,
running ChromeOS Crostini Linux 6.6). Included to show JailGuard runs
well even on older / weaker hardware. Modern desktop or server CPUs
land closer to the M3 column. Full per-benchmark numbers in
[`BENCHMARKS.md`](./BENCHMARKS.md).

## Benchmarks

Reproducible latency and throughput numbers come from three harnesses:

- `benches/detect.rs` — Criterion bench covering single-shot `is_injection` / `detect` / `score` and batch throughput at `n = 1, 8, 32, 128`. Run with `cargo bench --bench detect`.
- `examples/cold_start_bench.rs` — process-startup cost (ONNX session init + first inference). Run with `cargo run --release --example cold_start_bench`.
- `scripts/bench.sh` — portable POSIX wrapper that captures machine metadata (CPU, arch, kernel, toolchain) and emits a single markdown report. Works on Linux x86_64, Linux aarch64, macOS Intel, macOS Apple Silicon, and Chromebook Crostini.

Full methodology and head-to-head local-CPU comparisons in [`BENCHMARKS.md`](./BENCHMARKS.md).

## Attack categories covered in training

The classifier output is binary at the public API (injection / benign), but its training mix spans eight attack families:

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
- [Rebuff](https://github.com/protectai/rebuff) (archived)
- [Sentinel: SOTA model to protect against prompt injections](https://arxiv.org/abs/2506.05446)
- [Not What You've Signed Up For — indirect injection](https://arxiv.org/abs/2302.12173)

## Citation

If you use JailGuard in research or production, please cite:

```bibtex
@software{jailguard,
  title = {JailGuard: Efficient Prompt Injection Detection via Pre-trained Embeddings},
  author = {Yury Fedoseev},
  year = {2026},
  url = {https://github.com/yfedoseev/jailguard}
}
```

A machine-readable [`CITATION.cff`](CITATION.cff) is also available.

## License

Dual-licensed under [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE) at your option.
