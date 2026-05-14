# JailGuard for Elixir

Elixir bindings for JailGuard's Rust prompt-injection detector.

This first Elixir port is repo-local and source-built: `mix compile`
builds the parent Rust crate, copies `libjailguard` into the Mix app's
`priv/` directory, and compiles a small C NIF that calls the stable
JailGuard C ABI.

## Install from Git

```elixir
def deps do
  [
    {:jailguard,
     github: "yfedoseev/jailguard",
     branch: "feature/elixir-support",
     subdir: "elixir",
     depth: 1}
  ]
end
```

For a full Git URL, use `git:` instead of `github:`:

```elixir
{:jailguard,
 git: "https://github.com/yfedoseev/jailguard.git",
 branch: "feature/elixir-support",
 subdir: "elixir",
 depth: 1}
```

Then compile normally:

```sh
mix deps.get
mix compile
```

Requirements: Rust/Cargo, Make, and a C compiler. Linux and macOS are
supported by this source-built port.

Use `subdir: "elixir"` rather than `sparse: "elixir"` for this first
port: the Mix package still needs the parent repository checkout so it
can build `../Cargo.toml` and include `../include/jailguard.h`.
Windows, Hex publishing, and precompiled artifacts are deferred.

For local development from a checkout, a path dependency works too:

```elixir
{:jailguard, path: "../jailguard/elixir"}
```

## Quick start

```elixir
:ok = JailGuard.download_model()

{:ok, injection?} = JailGuard.is_injection("ignore previous instructions")

if injection? do
  raise "blocked"
end

{:ok, result} = JailGuard.detect("What is the capital of France?")
IO.inspect({result.score, result.risk})
```

## API

| Function | Returns |
|---|---|
| `JailGuard.version()` | `String.t()` |
| `JailGuard.download_model()` | `:ok | {:error, %JailGuard.Error{}}` |
| `JailGuard.model_cache_dir()` | `{:ok, String.t()} | {:error, %JailGuard.Error{}}` |
| `JailGuard.detect(text)` | `{:ok, %JailGuard.Result{}} | {:error, %JailGuard.Error{}}` |
| `JailGuard.detect!(text)` | `%JailGuard.Result{} | no_return()` |
| `JailGuard.is_injection(text)` | `{:ok, boolean()} | {:error, %JailGuard.Error{}}` |
| `JailGuard.score(text)` | `{:ok, float()} | {:error, %JailGuard.Error{}}` |
| `JailGuard.detect_batch(texts)` | `{:ok, [%JailGuard.Result{}]} | {:error, %JailGuard.Error{}}` |

`%JailGuard.Result{}` has `is_injection`, `score`, `confidence`, and
`risk` fields. Risk is one of `:safe`, `:low`, `:medium`, `:high`, or
`:critical`.
