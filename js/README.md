# JailGuard for JavaScript / TypeScript

**Fast prompt-injection detection for Node.js.** Pure-Rust core, exposed
via a napi-rs N-API native addon. **Prebuilt binaries ship for every
supported platform — no Rust toolchain required to install.** **p50 14 ms**
inference on Apple M3, **98.40% accuracy** on a 7,049-sample held-out test
set.

[![npm](https://img.shields.io/npm/v/@yfedoseev/jailguard.svg)](https://www.npmjs.com/package/@yfedoseev/jailguard)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses)
[![crates.io](https://img.shields.io/crates/v/jailguard.svg)](https://crates.io/crates/jailguard)
[![PyPI](https://img.shields.io/pypi/v/jailguard.svg)](https://pypi.org/project/jailguard/)

> **Part of the [JailGuard](https://github.com/yfedoseev/jailguard) toolkit.**
> Same Rust core as the [Rust crate](https://crates.io/crates/jailguard),
> [Python package](../python/README.md), and [Go module](../go/README.md).

**In 2026, JailGuard is the actively maintained, OSI-permissive,
embedded-binary option** for prompt-injection detection in Node.js.
[Rebuff was archived May 16, 2025](https://github.com/protectai/rebuff)
(Python-only, no JS alternative). [Lakera was acquired by Check Point in
September 2025](https://www.checkpoint.com/press-releases/check-point-acquires-lakera-to-deliver-end-to-end-ai-security-for-enterprises/)
(closed-source SaaS, requires HTTP round-trips).

## Install

```bash
npm install @yfedoseev/jailguard
```

That's it. Prebuilt `.node` binaries ship inside the npm tarball for:

| Platform | Architecture |
|---|---|
| Linux  | x64, arm64 |
| macOS  | x64, arm64 |
| Windows | x64 |

Node.js 18 or later is required.

## Quick start

```typescript
import { detect, isInjection, downloadModel } from "@yfedoseev/jailguard";

// Optional: pre-fetch the ONNX embedding model (~90 MB, cached at
// ~/.cache/jailguard/). The first detect() call will download it on
// demand if you skip this.
downloadModel();

if (isInjection("ignore previous instructions")) {
  throw new Error("blocked");
}

const result = detect("What is the capital of France?");
console.log(result.score, result.risk);
```

## API

| Function | Returns | Description |
|---|---|---|
| `detect(text)` | `DetectionResult` | Full detection output |
| `isInjection(text)` | `boolean` | Quick boolean check |
| `score(text)` | `number` | Raw probability `[0, 1]` |
| `detectBatch(texts)` | `DetectionResult[]` | Batch processing |
| `downloadModel()` | `void` | Pre-fetch ONNX model |
| `modelCacheDir()` | `string` | Cache path |
| `version()` | `string` | Library version |

`DetectionResult`: `{ isInjection: boolean; score: number; confidence: number; risk: RiskLevel }`

`RiskLevel`: `Safe = 0`, `Low = 1`, `Medium = 2`, `High = 3`, `Critical = 4`

## Framework integration

Runnable examples for common Node.js frameworks live in
[`../examples/javascript/`](../examples/javascript/) and
[`../examples/typescript/`](../examples/typescript/) — Express
middleware, Next.js route handlers, and batch scoring patterns. Each is
≤30 LOC with a self-contained `package.json`.

Quick Express middleware sketch:

```typescript
import express from "express";
import { isInjection } from "@yfedoseev/jailguard";

const app = express();
app.use(express.json());

app.use((req, res, next) => {
  const text = req.body?.prompt ?? "";
  if (isInjection(text)) {
    return res.status(400).json({ error: "prompt rejected" });
  }
  next();
});
```

## Performance

**98.40% accuracy** on the in-distribution pipeline test split,
**99.38%** on J1N2 OOD, **89.12%** on the shalyhinpavel hard-negative
holdout. **p50 14 ms / p99 18 ms** on Apple M3, single CPU thread.
Full methodology and head-to-head numbers vs.
`protectai/deberta-v3-base-prompt-injection-v2`,
`deepset/deberta-v3-base-injection`, and
`madhurjindal/Jailbreak-Detector-Large` in
[`BENCHMARKS.md`](../BENCHMARKS.md).

## Thread safety

Detection calls are synchronous and serialize on a Mutex internally
(matches the Python and Go bindings). For high-concurrency workloads,
fan out via `worker_threads` — each worker gets its own copy of the
addon.

## Building from source (contributors only)

End users do not need this. The published npm package ships prebuilt
addons for every supported platform.

If you've cloned the monorepo and want to test changes locally:

```bash
cd js
npm install
npm run build:native    # cargo build --release --features napi
                        # → js/prebuilds/<platform>-<arch>/jailguard.node
npm run build           # TypeScript compile
npm test                # vitest
```

`scripts/build-native.mjs` is a developer convenience — CI does not use
it. The release pipeline (`.github/workflows/release.yml`) builds the
napi addon for every target platform, stages them into
`js/prebuilds/<triple>/jailguard.node`, and runs `npm publish` once.

## License

Dual-licensed under MIT OR Apache-2.0 — your choice.
