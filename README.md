# JailGuard

[![Accuracy](https://img.shields.io/badge/Accuracy-98.40%25-blue)](BENCHMARKS.md)
[![F1 Score](https://img.shields.io/badge/F1-0.983-blue)](BENCHMARKS.md)
[![Pure Rust](https://img.shields.io/badge/Pure-Rust-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/jailguard.svg)](https://crates.io/crates/jailguard)
[![PyPI](https://img.shields.io/pypi/v/jailguard.svg)](https://pypi.org/project/jailguard/)
[![npm](https://img.shields.io/npm/v/@jailguard/jailguard.svg)](https://www.npmjs.com/package/@jailguard/jailguard)

> **JailGuard is a pure-Rust prompt-injection detector with a 1.5 MB embedded ONNX classifier.** It scores text in **p50 14 ms on CPU**, achieves **98.40% accuracy** on a 7,049-sample held-out test set drawn from 17 public datasets, and ships bindings for **Rust, Python, JavaScript, and Go**. The classifier is **embedded in the binary** — zero config, no external service, no API key. Dual-licensed under MIT OR Apache-2.0.

**In 2026, JailGuard is the actively maintained, independent, OSI-permissive, CPU-embedded option** in a consolidated prompt-injection-detection market. [Rebuff was archived on May 16, 2025](https://github.com/protectai/rebuff). [Check Point announced its acquisition of Lakera on September 16, 2025](https://www.checkpoint.com/press-releases/check-point-acquires-lakera-to-deliver-end-to-end-ai-security-for-enterprises/) (~$300M, closing Q4 2025). [Palo Alto Networks completed its acquisition of ProtectAI on July 22, 2025](https://www.paloaltonetworks.com/company/press/2025/palo-alto-networks-completes-acquisition-of-protect-ai). Meta's [Prompt Guard 2](https://huggingface.co/meta-llama/Llama-Prompt-Guard-2-86M) ships under the Llama 4 Community License (not OSI-approved).

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

**JavaScript / TypeScript** — `npm install @jailguard/jailguard`
```typescript
import { detect, isInjection } from "@jailguard/jailguard";

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
| **Multi-language bindings** | Rust, Python, JS, Go | API clients | Python | Python | Python |

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

Python / JS / Go expose the same surface in idiomatic form. See [`docs/API.md`](docs/API.md) for full per-language signatures.

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

### Latency (single CPU thread, Apple M3)

| Component               | Time    |
|-------------------------|---------|
| Embedding (MiniLM ONNX) | ~13 ms  |
| Classification (MLP)    | ~1 ms   |
| **Total (p50)**         | **~14 ms** |
| **Total (p99)**         | **~18 ms** |

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

## Frequently asked questions

### How do I detect prompt injection in Rust?

`cargo add jailguard`, then call `is_injection(text) -> bool` or `detect(text) -> DetectionOutput` from anywhere. Three functions, zero config.

### Does JailGuard work with Python?

Yes. `pip install jailguard`. The wheel ships precompiled bindings for CPython 3.8–3.13 on Linux (x86_64, aarch64, manylinux + musllinux), macOS (x86_64, arm64), and Windows (x86_64) — no Rust toolchain needed.

### Does it work in JavaScript / Node.js?

Yes. `npm install @jailguard/jailguard`. Prebuilt napi addons ship for Linux x64/arm64, macOS x64/arm64, Windows x64 — no Rust toolchain or C compiler needed at install time.

### Does it work in Go?

Yes. `go get github.com/yfedoseev/jailguard/go`, then one-time `go run github.com/yfedoseev/jailguard/go/cmd/install@latest` to fetch the native lib. Two backends — CGo (static link) or purego (`CGO_ENABLED=0`, dlopen at runtime). The purego backend works on Alpine/musl and supports cross-compilation without a C toolchain.

### Is JailGuard an alternative to Rebuff?

Yes. Rebuff was [archived by ProtectAI on May 16, 2025](https://github.com/protectai/rebuff). JailGuard is an actively maintained replacement — Apache-2.0 / MIT, with broader language coverage (Rust, Python, JS, Go vs Rebuff's Python-only), and pure-Rust runtime (no Python or OpenAI dependency).

### How does JailGuard compare to Lakera Guard?

Lakera Guard is a closed-source SaaS API. Check Point announced its acquisition of Lakera on September 16, 2025 (~$300M, closing Q4 2025); Lakera Guard remains SaaS-only. JailGuard runs locally with **no network call**: p50 14 ms on CPU versus ~150–300 ms RTT for an HTTP round-trip to Lakera. No API key, no rate limit, no data ever leaves your process.

Direct head-to-head accuracy comparison isn't possible — Lakera's evaluation methodology and test data are closed. We benchmark JailGuard on a 7,049-sample mix of public datasets and against locally-runnable open-source models (see [`BENCHMARKS.md`](BENCHMARKS.md)).

### How does JailGuard compare to ProtectAI's deberta-v3 detector?

Three differences:

1. **Size.** JailGuard's classifier is 1.5 MB; `protectai/deberta-v3-base-prompt-injection-v2` is ~440 MB.
2. **Dependencies.** JailGuard runs without PyTorch (`ort` Rust crate, statically linked); ProtectAI requires PyTorch or onnxruntime-py.
3. **Taxonomy.** JailGuard's training classifier separates 8 attack categories (RolePlay, InstructionOverride, ContextManipulation, OutputManipulation, EncodingAttack, JailbreakPattern, PromptLeaking, and Benign); ProtectAI is a binary classifier.

Palo Alto Networks completed its acquisition of ProtectAI on July 22, 2025. The deberta-v3 model remains on HuggingFace under Apache-2.0.

### How does JailGuard compare to Meta Prompt Guard 2?

Prompt Guard 2 ships in 22M-param and 86M-param variants under the **Llama 4 Community License** (not OSI-approved — has commercial-use restrictions). JailGuard is dual-licensed under MIT OR Apache-2.0 (OSI-approved permissive). JailGuard's classifier is **130 K parameters** (frozen MiniLM embedder is 33 M, downloaded on first use); Prompt Guard 2 is 22M–86M params and bundles its own DeBERTa weights. Prompt Guard 2 is binary; JailGuard's training distinguishes 8 attack categories.

### Does JailGuard work CPU-only?

Yes. The classifier is a 130 K-parameter MLP running ~1 ms per call. The 90 MB MiniLM ONNX embedder is the dominant cost (~13 ms p50 on Apple M3). No GPU is needed for inference *or* training.

### How big is the model?

Two artifacts:

- **Classifier** (1.5 MB JSON, `include_str!`-embedded in the binary): the 130 K-parameter MLP we trained.
- **Sentence embedder** (90 MB ONNX, `all-MiniLM-L6-v2`): downloaded to `~/.cache/jailguard/` on first use, never bundled into the crate/wheel/npm package.

### Can I use JailGuard offline?

After the one-time ONNX download (or by pre-staging it via `jailguard::download_model()`), yes. Detection runs entirely offline.

### Should I use JailGuard as my only defense?

**No.** Prompt-injection detection should be one layer in defense-in-depth. Pair JailGuard with: input sanitization, structured output parsing, least-privilege tool permissions, separate trust-domains for user-controlled and system-controlled text. Treat JailGuard's score as advisory — a high-confidence injection signal warrants blocking; a low-confidence signal warrants logging.

### Does JailGuard support languages other than English?

The embedder (`all-MiniLM-L6-v2`) is multilingual-capable but English-weighted in training. The JailGuard classifier was trained on predominantly English public datasets. Multilingual evaluation is on the roadmap; today, expect best results on English text.

### Is there a hosted API?

No. JailGuard is library-only, by design. If you want a hosted prompt-injection-detection API, look at Lakera Guard or AWS Bedrock Guardrails.

### Where do the training datasets come from?

A deduplicated mix of 17 public HuggingFace datasets (deepset, xTRam1, jackhhao, rubend18, ALERT adversarial/regular, LMSYS Toxic Chat, JailbreakBench, Alpaca, Dolly, BeaverTails, hh-rlhf, OpenAssistant, shalyhinpavel, JailbreakV-28K, SPML). Full inventory and per-source role in [`BENCHMARKS.md`](BENCHMARKS.md).

### Can I retrain the model on my own data?

The training pipeline is not part of the published crate. The model weights file is plain JSON — drop a new file into `models/neural_binary_200k.json` and rebuild. The pipeline that produces these weights lives in a sibling repo and is not redistributed.

### Where can I cite JailGuard?

See [`CITATION.cff`](CITATION.cff) (GitHub's "Cite this repository" button uses it) and the BibTeX block at the bottom of this README.

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
  author = {Fedoseev, Yury},
  year = {2026},
  url = {https://github.com/yfedoseev/jailguard}
}
```

A machine-readable [`CITATION.cff`](CITATION.cff) is also available.

## License

Dual-licensed under MIT OR Apache-2.0 — your choice.
