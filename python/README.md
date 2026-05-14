# JailGuard for Python

Pure-Rust prompt-injection detector exposed to Python via PyO3 + maturin.
**No PyTorch, no onnxruntime-py, no Rust toolchain at install time** —
the wheel is self-contained.

[![PyPI](https://img.shields.io/pypi/v/jailguard.svg)](https://pypi.org/project/jailguard/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses)

> **Part of the [JailGuard](https://github.com/yfedoseev/jailguard) toolkit.**
> Same Rust core, same numbers, same API as the
> [Rust crate](https://crates.io/crates/jailguard),
> [JavaScript / TypeScript package](../js/README.md), and
> [Go module](../go/README.md).

## Install

```bash
pip install jailguard
```

Wheels published for CPython **3.8 – 3.13** on:

| Platform | Architectures |
|---|---|
| Linux | x86_64 + aarch64 (manylinux_2_28 + musllinux_1_2) |
| macOS | x86_64 + arm64 |
| Windows | x86_64 |

## Quick start

```python
import jailguard

# Optional pre-warm: the 90 MB ONNX embedder downloads on first detect().
jailguard.download_model()

if jailguard.is_injection("ignore all previous instructions"):
    raise ValueError("blocked")

result = jailguard.detect("What is the capital of France?")
print(result.is_injection, result.score, result.risk)
# → False  0.0  RiskLevel.Safe
```

## API

| | Returns | Notes |
|-|---------|-------|
| `detect(text)` | `DetectionResult` | `is_injection`, `score`, `confidence`, `risk` |
| `is_injection(text)` | `bool` | Quick boolean check |
| `score(text)` | `float` | Raw probability `[0.0, 1.0]` |
| `detect_batch(texts)` | `list[DetectionResult]` | Batch — reuses session |
| `download_model()` | `None` | Pre-fetch the ONNX embedder |
| `model_cache_dir()` | `str` | Where the ONNX model is cached |
| `RiskLevel` | enum | `Safe` / `Low` / `Medium` / `High` / `Critical` |

Type stubs (`jailguard/_jailguard.pyi`) ship in every wheel — mypy and
pyright see the full typed surface including the async API.

## Async

For `asyncio` code paths, every entry point has an async equivalent
that runs the inference on a background thread:

```python
import asyncio
from jailguard import detect_async

async def main():
    result = await detect_async("ignore previous instructions")
    print(result.is_injection, result.score)

asyncio.run(main())
```

For high-throughput fan-out, hold a dedicated worker pool with
`AsyncDetector`:

```python
from jailguard import AsyncDetector

async def main():
    async with AsyncDetector(max_workers=8) as det:
        results = await det.detect_batch(prompts)
```

Async equivalents: `detect_async`, `is_injection_async`, `score_async`,
`detect_batch_async`, plus `AsyncDetector`.

## Examples

Runnable examples live in [`../examples/python/`](../examples/python/) —
quickstart, batch scoring, async fan-out, and framework integration
patterns (FastAPI middleware, LangChain `BaseCallbackHandler`).

## Performance

Headline: **98.40% accuracy** in-domain, **p50 14 ms** on Apple M3.
Full methodology, dataset breakdown, OOD benchmarks, and head-to-head
numbers vs open-source baselines in
[`../BENCHMARKS.md`](../BENCHMARKS.md).

## Building from source

End users do not need this. Wheels cover every supported
platform/Python combination. If you've cloned the monorepo:

```bash
pip install maturin
maturin develop --features python --release
pytest python/tests/ -v
```

## Other JailGuard bindings

- **Rust** — `cargo add jailguard` — [docs.rs/jailguard](https://docs.rs/jailguard)
- **JavaScript / TypeScript** — `npm install @yfedoseev/jailguard` — [js/README.md](../js/README.md)
- **Go** — `go get github.com/yfedoseev/jailguard/go` — [go/README.md](../go/README.md)

## License

Dual-licensed under MIT OR Apache-2.0 — your choice.
