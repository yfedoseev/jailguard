# JailGuard for JavaScript / TypeScript

Fast prompt-injection detection for Node.js. Pure-Rust core via napi-rs N-API
native module. WASM build is available as alpha (see [WASM status](#wasm-status)).

> **Part of the [JailGuard](https://github.com/yfedoseev/jailguard) toolkit.**
> Same Rust core as the [Rust crate](https://crates.io/crates/jailguard),
> [Python package](../python/README.md), and [Go module](../go/README.md).

## Install

```bash
npm install @jailguard/jailguard
```

Prebuilt native binaries ship for Node.js 18+ on Linux x64/arm64, macOS x64/arm64,
and Windows x64.

## Quick start

```typescript
import { detect, isInjection, downloadModel } from "@jailguard/jailguard";

// Pre-fetch the ONNX model (~90 MB, one-time, cached at ~/.cache/jailguard/).
// Optional — first detect() will download it on demand if you skip this.
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
| `detect(text)` | `DetectionResult` | Full output |
| `isInjection(text)` | `boolean` | Quick boolean check |
| `score(text)` | `number` | Raw probability `[0, 1]` |
| `detectBatch(texts)` | `DetectionResult[]` | Batch processing |
| `downloadModel()` | `void` | Pre-fetch ONNX model |
| `modelCacheDir()` | `string` | Cache path |
| `version()` | `string` | Library version |

`DetectionResult`: `{ isInjection: boolean; score: number; confidence: number; risk: RiskLevel }`

`RiskLevel`: `Safe = 0`, `Low = 1`, `Medium = 2`, `High = 3`, `Critical = 4`

## Build from source

```bash
cd js
npm install
npm run build       # invokes cargo build --release --features napi + tsc
npm test            # vitest, 16 tests
```

`scripts/build-native.mjs` runs `cargo build --release --features napi` from the
repo root, then copies the resulting `libjailguard.{so,dylib,dll}` to
`build/jailguard.node` for Node to require.

## WASM status

The package ships a WASM entry at `@jailguard/jailguard/wasm` for browsers, Deno, and Cloudflare Workers. **Status: alpha.** The Rust ONNX runtime we use (`ort`) doesn't yet compile to `wasm32-unknown-unknown`, so the WASM build exposes the public API surface but every detection call returns an error explaining the gap.

For production WASM deployments, the path forward is one of:
1. Switch to `tract` (pure-Rust ONNX runtime, slower but WASM-friendly)
2. Defer to ORT-Web via JS interop

Tracked in [issue #N](https://github.com/yfedoseev/jailguard/issues). For now, use the Node.js binding for browser-side workloads via a small server proxy.

## Thread safety

Detection calls are synchronous and serialize on a Mutex internally (matches
the Python and Go bindings). For high-concurrency workloads, fan out via
`worker_threads` — each worker gets its own copy of the addon.

## License

Dual-licensed under MIT OR Apache-2.0 — your choice.
