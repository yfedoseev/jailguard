# JailGuard Examples

Runnable end-to-end examples for every supported language. Each example
is self-contained — copy-paste, install the language's package, run.

## Per-language directories

```
examples/
├── *.rs                Rust (built-in cargo examples)
├── python/             Python — see python/README.md
├── go/                 Go — see go/README.md
├── javascript/         Node.js / JavaScript (.mjs)
└── typescript/         TypeScript (typed Node.js)
```

## Rust

Each `.rs` file at this level is a `cargo run --example` target:

| Example | Run | Notes |
|---------|-----|-------|
| `quick_start.rs` | `cargo run --release --example quick_start` | Minimal `is_injection` check |
| `score_test.rs` | `cargo run --release --example score_test --features full` | 7-prompt smoke test |
| `score_jsonl.rs` | `cat prompts.jsonl \| cargo run --release --example score_jsonl` | Stdin → stdout JSONL scorer |
| `cold_start_bench.rs` | `cargo run --release --example cold_start_bench` | First-call latency measurement |
| `ensemble_demo.rs` | `cargo run --release --example ensemble_demo --features full` | Ensemble detector showcase |
| `full_pipeline.rs` | `cargo run --release --example full_pipeline --features full` | All defense layers |
| `unified_api_ensemble_demo.rs` | `cargo run --release --example unified_api_ensemble_demo --features full` | Unified ensemble API |

## Python

```bash
pip install jailguard
cd examples/python/01-quick-start && python main.py
```

| Example | Demonstrates |
|---------|--------------|
| [`01-quick-start/`](python/01-quick-start/main.py) | `is_injection`, `detect`, `score`, `download_model` |
| [`02-batch/`](python/02-batch/main.py) | `detect_batch` for log-style scoring |
| [`03-async/`](python/03-async/main.py) | `detect_async`, `AsyncDetector`, `asyncio.gather` fan-out |

## Go

```bash
go get github.com/yfedoseev/jailguard/go
cd examples/go/01-quick-start
CGO_CFLAGS="-I$(pwd)/../../../include" \
CGO_LDFLAGS="-L$(pwd)/../../../target/release -ljailguard" \
DYLD_LIBRARY_PATH=$(pwd)/../../../target/release \
go run main.go
```

The cgo flags point at the Rust cdylib in `target/release/`. In production,
install `libjailguard.{so,dylib}` to a system library path so the linker
finds it without flags.

| Example | Demonstrates |
|---------|--------------|
| [`01-quick-start/`](go/01-quick-start/main.go) | `IsInjection`, `Detect`, `Score`, `DownloadModel` |
| [`02-batch/`](go/02-batch/main.go) | `DetectBatch` with mixed inputs and risk aggregation |

## JavaScript (Node.js)

```bash
npm install @jailguard/jailguard
node examples/javascript/01-quick-start/index.mjs
```

| Example | Demonstrates |
|---------|--------------|
| [`01-quick-start/`](javascript/01-quick-start/index.mjs) | `isInjection`, `detect`, `score`, `RiskLevel` enum |
| [`02-batch/`](javascript/02-batch/index.mjs) | `detectBatch`, formatted output table |

## TypeScript

```bash
npm install @jailguard/jailguard
npx tsx examples/typescript/01-quick-start/index.ts
```

These are the same demos as the JavaScript ones but with explicit
type annotations (`DetectionResult`, `RiskLevel`, `string`, `number`)
to show the full TS contract. They type-check under `tsc --strict`.

| Example | Demonstrates |
|---------|--------------|
| [`01-quick-start/`](typescript/01-quick-start/index.ts) | Typed `DetectionResult`, JSDoc-aware autocomplete |
| [`02-batch/`](typescript/02-batch/index.ts) | `Map<RiskLevel, number>`, type narrowing, `?? "fallback"` |

## What every example covers

- Pre-fetching the ONNX model (`download_model`) for predictable startup latency
- The 4-function public API: detect, is_injection, score, detect_batch
- Reading the `RiskLevel` enum (Safe / Low / Medium / High / Critical)
- The `DetectionResult` shape (`is_injection`, `score`, `confidence`, `risk`)

What's *not* yet covered (planned for later):

- Streaming / single-request HTTP server integration
- Sub-process / worker pool patterns beyond the Python `AsyncDetector`
- Browser / Deno / Cloudflare Workers (blocked on the WASM gap — see [js/README.md](../js/README.md#wasm-status))
- Scoring against a public benchmark dataset (PINT, AgentDojo) — see [`jailguard-datasets/BENCHMARKS.md`](https://github.com/yfedoseev/jailguard-datasets/blob/main/BENCHMARKS.md)
