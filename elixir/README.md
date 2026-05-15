# JailGuard for Elixir

[![Hex.pm](https://img.shields.io/hexpm/v/jailguard.svg)](https://hex.pm/packages/jailguard)
[![HexDocs](https://img.shields.io/badge/hex-docs-blue.svg)](https://hexdocs.pm/jailguard)

Elixir bindings for [JailGuard](https://github.com/yfedoseev/jailguard) — a
pure-Rust prompt-injection detector with a 1.5 MB embedded MLP classifier.
98.40% accuracy, p50 14 ms on CPU. Loads via a Rustler NIF; precompiled
artifacts ship for Linux (x86_64, aarch64), macOS (x86_64, aarch64), and
Windows (x86_64) — no Rust toolchain required on install.

## Install

```elixir
def deps do
  [
    {:jailguard, "~> 0.1.2"}
  ]
end
```

```sh
mix deps.get
```

That's it. The matching precompiled NIF is fetched from the GitHub release
and checksum-verified against the bundled `checksum-Elixir.JailGuard.Native.exs`.

### Building from source

If you're on an unsupported target, clone the source repo and use a path
dependency. `JAILGUARD_BUILD=1` from the Hex package alone won't work — the
NIF crate path-depends on the parent Rust crate, which only ships with the
git repo (not the Hex tarball).

```sh
git clone https://github.com/yfedoseev/jailguard.git
```

```elixir
def deps do
  [{:jailguard, path: "/path/to/jailguard/elixir"}]
end
```

Then `JAILGUARD_BUILD=1 mix deps.compile jailguard --force`. Requires Rust
1.85+ and a working `cc`/MSVC toolchain.

## Quick start

```elixir
:ok = JailGuard.download_model()

{:ok, injection?} = JailGuard.is_injection("ignore previous instructions")
if injection?, do: raise("blocked")

{:ok, result} = JailGuard.detect("What is the capital of France?")
IO.inspect({result.score, result.risk})
```

The 90 MB MiniLM ONNX embedder is auto-downloaded to `~/.cache/jailguard/` on
first use. For production: call `JailGuard.download_model/0` at startup to
warm the cache before serving traffic.

## API

| Function | Returns |
|---|---|
| `JailGuard.version()` | `String.t()` |
| `JailGuard.download_model()` | `:ok \| {:error, %JailGuard.Error{}}` |
| `JailGuard.model_cache_dir()` | `{:ok, String.t()} \| {:error, %JailGuard.Error{}}` |
| `JailGuard.detect(text)` | `{:ok, %JailGuard.Result{}} \| {:error, %JailGuard.Error{}}` |
| `JailGuard.detect!(text)` | `%JailGuard.Result{} \| no_return()` |
| `JailGuard.is_injection(text)` | `{:ok, boolean()} \| {:error, %JailGuard.Error{}}` |
| `JailGuard.score(text)` | `{:ok, float()} \| {:error, %JailGuard.Error{}}` |
| `JailGuard.detect_batch(texts)` | `{:ok, [%JailGuard.Result{}]} \| {:error, %JailGuard.Error{}}` |

`%JailGuard.Result{}` has `is_injection`, `score`, `confidence`, and `risk`.
`risk` is one of `:safe | :low | :medium | :high | :critical`.

## Supported targets (precompiled)

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

Other targets fall back to `JAILGUARD_BUILD=1` source builds.

## License

Dual-licensed under MIT OR Apache-2.0.
