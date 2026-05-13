# JailGuard for JavaScript / TypeScript

Fast prompt-injection detection for Node.js. Pure-Rust core, exposed via
a napi-rs N-API native addon. **Prebuilt binaries ship for every
supported platform — no Rust toolchain required to install.**

[![npm version](https://img.shields.io/npm/v/@yfedoseev/jailguard.svg)](https://www.npmjs.com/package/@yfedoseev/jailguard)

> **Part of the [JailGuard](https://github.com/yfedoseev/jailguard) toolkit.**
> Same Rust core as the [Rust crate](https://crates.io/crates/jailguard),
> [Python package](../python/README.md), and [Go module](../go/README.md).

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
|----------|---------|-------------|
| `detect(text)` | `DetectionResult` | Full detection output |
| `isInjection(text)` | `boolean` | Quick boolean check |
| `score(text)` | `number` | Raw probability `[0, 1]` |
| `detectBatch(texts)` | `DetectionResult[]` | Batch processing |
| `downloadModel()` | `void` | Pre-fetch ONNX model |
| `modelCacheDir()` | `string` | Cache path |
| `version()` | `string` | Library version |

`DetectionResult`: `{ isInjection: boolean; score: number; confidence: number; risk: RiskLevel }`

`RiskLevel`: `Safe = 0`, `Low = 1`, `Medium = 2`, `High = 3`, `Critical = 4`

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
napi addon for every target platform, then stages them into
`js/prebuilds/<triple>/jailguard.node` and runs `npm publish` once.

## License

Dual-licensed under MIT OR Apache-2.0 — your choice.
