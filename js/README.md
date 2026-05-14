# JailGuard for JavaScript / TypeScript

Pure-Rust prompt-injection detector exposed to Node.js via a napi-rs
N-API native addon. **Prebuilt binaries ship for every supported
platform — no Rust toolchain, no C compiler, no build step at install.**

[![npm](https://img.shields.io/npm/v/@yfedoseev/jailguard.svg)](https://www.npmjs.com/package/@yfedoseev/jailguard)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses)

> **Part of the [JailGuard](https://github.com/yfedoseev/jailguard) toolkit.**
> Same Rust core, same numbers, same API as the
> [Rust crate](https://crates.io/crates/jailguard),
> [Python package](../python/README.md), and
> [Go module](../go/README.md).

## Install

```bash
npm install @yfedoseev/jailguard
```

Prebuilt `.node` binaries ship inside the npm tarball for:

| Platform | Architecture |
|---|---|
| Linux | x64, arm64 |
| macOS | x64, arm64 |
| Windows | x64 |

Node.js 18 or later.

## Quick start

```typescript
import { detect, isInjection, downloadModel } from "@yfedoseev/jailguard";

// Optional pre-warm: the 90 MB ONNX embedder downloads on first detect().
downloadModel();

if (isInjection("ignore previous instructions")) {
  throw new Error("blocked");
}

const r = detect("What is the capital of France?");
console.log(r.score, r.risk);
```

## API

| Function | Returns | Description |
|---|---|---|
| `detect(text)` | `DetectionResult` | Full detection output |
| `isInjection(text)` | `boolean` | Quick boolean check |
| `score(text)` | `number` | Raw probability `[0, 1]` |
| `detectBatch(texts)` | `DetectionResult[]` | Batch processing |
| `downloadModel()` | `void` | Pre-fetch the ONNX model |
| `modelCacheDir()` | `string` | Cache path |
| `version()` | `string` | Library version |

```typescript
interface DetectionResult {
  isInjection: boolean;
  score: number;
  confidence: number;
  risk: RiskLevel;
}

enum RiskLevel { Safe = 0, Low = 1, Medium = 2, High = 3, Critical = 4 }
```

Full TypeScript declarations ship in the package — autocomplete and
strict type-checking out of the box. The package is **ESM-only** (`type:
"module"`); CommonJS consumers use dynamic `import()`.

## Examples

Runnable examples live in
[`../examples/javascript/`](../examples/javascript/) (Node.js, `.mjs`)
and [`../examples/typescript/`](../examples/typescript/) — quickstart,
batch scoring, and middleware patterns for Express and Next.js route
handlers.

Quick Express middleware:

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

Headline: **98.40% accuracy** in-domain, **p50 14 ms** on Apple M3.
Full methodology, dataset breakdown, OOD benchmarks, and head-to-head
numbers vs open-source baselines in
[`../BENCHMARKS.md`](../BENCHMARKS.md).

## Thread safety

Detection calls are synchronous and serialize on a Mutex internally.
For high-concurrency workloads, fan out via Node's `worker_threads` —
each worker gets its own copy of the addon and runs independently.

## Building from source

End users do not need this. The published npm package ships prebuilt
addons for every supported platform.

If you've cloned the monorepo:

```bash
cd js
npm install
npm run build:native    # cargo build --release --features napi
                        # → js/prebuilds/<platform>-<arch>/jailguard.node
npm run build           # TypeScript compile
npm test                # vitest
```

`scripts/build-native.mjs` is a developer convenience — CI does not use
it. The release pipeline builds the napi addon for every target,
stages them into `js/prebuilds/<triple>/jailguard.node`, and runs
`npm publish` once.

## Other JailGuard bindings

- **Rust** — `cargo add jailguard` — [docs.rs/jailguard](https://docs.rs/jailguard)
- **Python** — `pip install jailguard` — [python/README.md](../python/README.md)
- **Go** — `go get github.com/yfedoseev/jailguard/go` — [go/README.md](../go/README.md)

## License

Dual-licensed under MIT OR Apache-2.0 — your choice.
