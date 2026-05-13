# JailGuard for Go

Pure-Rust prompt-injection detector exposed to Go through **CGo** (default,
statically linked) or **purego** (`CGO_ENABLED=0`, dlopen at runtime).
The purego backend supports Alpine/musl and cross-compilation without a
C toolchain.

[![Go Reference](https://pkg.go.dev/badge/github.com/yfedoseev/jailguard/go.svg)](https://pkg.go.dev/github.com/yfedoseev/jailguard/go)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses)

> **Part of the [JailGuard](https://github.com/yfedoseev/jailguard) toolkit.**
> Same Rust core, same numbers, same API as the
> [Rust crate](https://crates.io/crates/jailguard),
> [Python package](../python/README.md), and
> [JavaScript / TypeScript package](../js/README.md).

## Install

```sh
go get github.com/yfedoseev/jailguard/go
```

Then download the native library once per machine:

### Option A — CGo (default, statically linked)

```sh
go run github.com/yfedoseev/jailguard/go/cmd/install@latest

# Installer prints CGO_CFLAGS / CGO_LDFLAGS. Linux example:
export CGO_CFLAGS="-I$HOME/.cache/jailguard/v0.1.0/include"
export CGO_LDFLAGS="$HOME/.cache/jailguard/v0.1.0/lib/linux_amd64/libjailguard.a -lm -lpthread -ldl -lrt -lgcc_s -lutil -lc"

go build ./...
```

The cache directory follows `os.UserCacheDir()`:
`~/.cache/jailguard/` on Linux, `~/Library/Caches/jailguard/` on macOS,
`%LocalAppData%\jailguard\` on Windows. Override with `-dir`. Pin a
specific version with `@v0.1.0`.

### Option B — purego (CGO_ENABLED=0)

For Alpine/musl, cross-compilation, or any environment without a C
compiler. The shared library is `dlopen`'d at runtime.

```sh
go run github.com/yfedoseev/jailguard/go/cmd/install@latest -shared

export CGO_ENABLED=0
export JAILGUARD_LIB_PATH=$HOME/.cache/jailguard/v0.1.0/lib/linux_amd64/libjailguard.so

go build ./...
```

The purego backend supports the full public API — every function works
identically to the CGo backend, with the same error sentinels.

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
|---|---|---|
| `Detect(text)` | `(Result, error)` | Full detection output |
| `IsInjection(text)` | `(bool, error)` | Quick boolean check |
| `Score(text)` | `(float32, error)` | Raw probability `[0.0, 1.0]` |
| `DetectBatch(texts)` | `([]Result, error)` | Batch processing |
| `DownloadModel()` | `error` | Pre-fetch the ONNX embedding model |
| `ModelCacheDir()` | `(string, error)` | ONNX cache path |
| `Version()` | `string` | Library version |

`Result`: `IsInjection bool`, `Score float32`, `Confidence float32`, `Risk RiskLevel`

`RiskLevel`: `RiskSafe = 0`, `RiskLow = 1`, `RiskMedium = 2`, `RiskHigh = 3`, `RiskCritical = 4`

Errors are wrapped sentinels — use `errors.Is` to match:

```go
if errors.Is(err, jailguard.ErrModelNotDownloaded) {
    _ = jailguard.DownloadModel()
}
```

## Examples

Runnable examples live in [`../examples/go/`](../examples/go/) —
quickstart, batch scoring, and middleware patterns for `net/http`,
`gin`, and `chi`.

Quick `net/http` middleware:

```go
import jailguard "github.com/yfedoseev/jailguard/go"

func guardMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        body, _ := io.ReadAll(r.Body)
        r.Body = io.NopCloser(bytes.NewReader(body))
        if injection, _ := jailguard.IsInjection(string(body)); injection {
            http.Error(w, "prompt rejected", http.StatusBadRequest)
            return
        }
        next.ServeHTTP(w, r)
    })
}
```

## Performance

Headline: **98.40% accuracy** in-domain, **p50 14 ms** on Apple M3.
Full methodology, dataset breakdown, OOD benchmarks, and head-to-head
numbers vs open-source baselines in
[`../BENCHMARKS.md`](../BENCHMARKS.md).

## Thread safety

All exported functions are safe for concurrent use from multiple
goroutines. The underlying Rust detector serializes ONNX session access
internally; concurrent calls funnel through a single ONNX session.

## Building from source

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

## Other JailGuard bindings

- **Rust** — `cargo add jailguard` — [docs.rs/jailguard](https://docs.rs/jailguard)
- **Python** — `pip install jailguard` — [python/README.md](../python/README.md)
- **JavaScript / TypeScript** — `npm install @yfedoseev/jailguard` — [js/README.md](../js/README.md)

## License

Dual-licensed under MIT OR Apache-2.0 — your choice.
