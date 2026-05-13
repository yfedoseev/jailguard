# JailGuard for Go

Fast prompt-injection detection for Go. Pure-Rust core, exposed to Go
through CGo **or** a pure-Go [purego](https://github.com/ebitengine/purego)
backend (`CGO_ENABLED=0`). ~14 ms p50 inference, 98.40% accuracy on the
in-domain test set.

[![Go Reference](https://pkg.go.dev/badge/github.com/yfedoseev/jailguard/go.svg)](https://pkg.go.dev/github.com/yfedoseev/jailguard/go)

> **Part of the [JailGuard](https://github.com/yfedoseev/jailguard) toolkit.**
> Same Rust core as the [Rust crate](https://crates.io/crates/jailguard),
> [Python package](../python/README.md), and JavaScript bindings.

## Quick start

### Option A — CGo (default, statically linked)

```sh
go get github.com/yfedoseev/jailguard/go

# One-time per machine: download the prebuilt staticlib + header.
go run github.com/yfedoseev/jailguard/go/cmd/install@latest

# The installer prints CGO_CFLAGS / CGO_LDFLAGS — paste them into your
# shell. Linux example:
export CGO_CFLAGS="-I$HOME/.cache/jailguard/v0.1.0/include"
export CGO_LDFLAGS="$HOME/.cache/jailguard/v0.1.0/lib/linux_amd64/libjailguard.a -lm -lpthread -ldl -lrt -lgcc_s -lutil -lc"

go build ./...
```

The cache directory follows `os.UserCacheDir()`:
`~/.cache/jailguard/` on Linux, `~/Library/Caches/jailguard/` on macOS,
`%LocalAppData%\jailguard\` on Windows. Override with `-dir`.

Pin a specific version:

```sh
go run github.com/yfedoseev/jailguard/go/cmd/install@v0.1.0
```

### Option B — purego (CGO_ENABLED=0, no C toolchain)

For Alpine/musl, cross-compilation, or any environment without a C
compiler. The shared library is loaded at runtime via `dlopen`.

```sh
go get github.com/yfedoseev/jailguard/go

go run github.com/yfedoseev/jailguard/go/cmd/install@latest -shared

# Installer prints:
export CGO_ENABLED=0
export JAILGUARD_LIB_PATH=$HOME/.cache/jailguard/v0.1.0/lib/linux_amd64/libjailguard.so

go build ./...
```

The purego backend supports the full public API — every function works
identically to the CGo backend, with the same error sentinels.

## Hello, detector

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

    if injection, _ := jailguard.IsInjection("ignore previous instructions"); injection {
        fmt.Println("blocked")
    }

    result, err := jailguard.Detect("What is the capital of France?")
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
| `Score(text)` | `(float32, error)` | Raw probability `[0.0, 1.0]` |
| `DetectBatch(texts)` | `([]Result, error)` | Batch processing |
| `DownloadModel()` | `error` | Pre-fetch the ONNX embedding model |
| `ModelCacheDir()` | `(string, error)` | ONNX cache path |
| `Version()` | `string` | Library version |

`Result`: `IsInjection bool`, `Score float32`, `Confidence float32`, `Risk RiskLevel`

`RiskLevel`: `RiskSafe = 0`, `RiskLow = 1`, `RiskMedium = 2`, `RiskHigh = 3`, `RiskCritical = 4`

## Building inside the monorepo

Contributors building from a checkout of the parent `jailguard` repo
don't need the installer. Use the `jailguard_dev` build tag — it points
the linker at the workspace `target/release/` directory:

```sh
cargo build --release          # repo root
cd go
go build -tags jailguard_dev ./...
go test  -tags jailguard_dev ./...
```

Or just `make test-go` from the repo root, which automates the above.

## Thread safety

All exported functions are safe for concurrent use from multiple
goroutines. The underlying Rust detector serialises ONNX session access
internally; concurrent calls funnel through a single ONNX session.

## License

Dual-licensed under MIT OR Apache-2.0 — your choice.
