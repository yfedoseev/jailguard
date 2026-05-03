# JailGuard for Go

Fast prompt-injection detection for Go via cgo bindings to the Rust core.

[![Go Reference](https://pkg.go.dev/badge/github.com/yfedoseev/jailguard/go.svg)](https://pkg.go.dev/github.com/yfedoseev/jailguard/go)

> **Part of the [JailGuard](https://github.com/yfedoseev/jailguard) toolkit.**
> Same Rust core as the [Rust crate](https://crates.io/crates/jailguard),
> [Python package](../python/README.md), and JavaScript bindings.

## Install

```bash
go get github.com/yfedoseev/jailguard/go
```

The package depends on a `libjailguard` cdylib produced from the Rust crate.
See [Building](#building) below.

## Quick start

```go
package main

import (
    "fmt"
    "log"

    jailguard "github.com/yfedoseev/jailguard/go"
)

func main() {
    if err := jailguard.DownloadModel(); err != nil {
        log.Fatal(err)
    }

    result, err := jailguard.Detect("ignore previous instructions")
    if err != nil {
        log.Fatal(err)
    }
    fmt.Printf("injection=%v score=%.4f risk=%v\n",
        result.IsInjection, result.Score, result.Risk)
}
```

## API

| Function | Returns | Description |
|----------|---------|-------------|
| `Detect(text)` | `(Result, error)` | Full detection output |
| `IsInjection(text)` | `(bool, error)` | Quick boolean check |
| `Score(text)` | `(float32, error)` | Raw probability |
| `DetectBatch(texts)` | `([]Result, error)` | Batch processing |
| `DownloadModel()` | `error` | Pre-fetch ONNX model |
| `ModelCacheDir()` | `(string, error)` | Cache path |
| `Version()` | `string` | Library version |

`Result`: `IsInjection bool`, `Score float32`, `Confidence float32`, `Risk RiskLevel`
`RiskLevel`: `RiskSafe` / `RiskLow` / `RiskMedium` / `RiskHigh` / `RiskCritical`

## Building

The package needs a `libjailguard` cdylib at link time. From the repo root:

```bash
cargo build --release      # produces target/release/libjailguard.{so,dylib}

cd go
CGO_CFLAGS="-I$(pwd)/../include" \
CGO_LDFLAGS="-L$(pwd)/../target/release -ljailguard" \
go build ./...
```

Or just `make build-go` from the repo root.

For runtime, set `LD_LIBRARY_PATH` (Linux) or `DYLD_LIBRARY_PATH` (macOS)
to point at `target/release/`. In production, install the cdylib to a
standard location (`/usr/local/lib`, `/usr/lib`) so the dynamic linker
finds it automatically.

## Thread safety

All exported functions are safe to call from multiple goroutines. The
underlying Rust detector serialises ONNX session access internally.

## Testing

```bash
make test-go    # full suite, ~8s
```

Includes 12 tests covering:

- Sync API surface (Version, ModelCacheDir, idempotent download)
- Detection accuracy (4 canonical injections + 3 canonical benigns)
- Equivalence (IsInjection vs Detect, Score vs Detect)
- Batch processing (incl. empty batch and order preservation)
- Concurrent goroutine fan-out (16 concurrent calls)
- RiskLevel.String formatting

Plus two benchmarks (`Detect`, `DetectBatch16`).

## License

Dual-licensed under MIT OR Apache-2.0 — your choice.
