# AGENTS.md — JailGuard

Guidance for AI coding agents (Cursor, Claude Code, Devin, Aider,
Cline, Copilot Workspace, etc.) working in this repo. Optimized for
agent comprehension: commands first, prose minimal, examples concrete.

## What this project is

JailGuard is a pure-Rust prompt-injection detector with a 1.5 MB
embedded ONNX classifier. The Rust core ships bindings for Python
(pyo3 + maturin), JavaScript (napi-rs), and Go (CGo + purego). 98.40%
accuracy on the in-domain test set, p50 14 ms CPU inference on Apple M3.

## Build

```sh
cargo build --release                    # default features (embedded detector)
cargo build --release --all-features     # full surface (training, examples)
make build-py                            # Python wheel via maturin
make build-js                            # napi addon + tsc
make build-go                            # cgo binding (needs cargo build --release first)
```

## Test

```sh
cargo test --release --lib               # unit tests, fastest
cargo test --release                     # +integration tests
cargo test --release --doc               # doc-tests in src/lib.rs
make check                               # fmt-check + clippy + lib tests (CI gate)
make test-py                             # Python pytest (after build-py)
make test-js                             # vitest (after build-js)
make test-go                             # Go tests (uses -tags jailguard_dev)
```

## Lint / format

```sh
make lint                                # cargo clippy --release --all-targets -- -D warnings
make fmt                                 # cargo fmt + ruff format + taplo fmt
make fmt-check                           # all formatters in check mode
cargo clippy --features full             # informational only (research surface has known warnings)
```

## Project structure

| Path | Contents |
|---|---|
| `src/lib.rs` | Crate root + feature-gated module declarations |
| `src/embedded.rs` | Zero-config public API: `detect()`, `is_injection()`, `score()`, `detect_batch()` |
| `src/network.rs` | The 130K-parameter MLP (forward pass only at runtime) |
| `src/model_manager.rs` | ONNX model auto-download to `~/.cache/jailguard/` |
| `src/c_api.rs` | C ABI exported via `cdylib`/`staticlib` (consumed by Go + napi) |
| `src/python.rs` | pyo3 bindings (feature `python`) |
| `src/napi.rs` | napi-rs bindings (feature `napi`) |
| `src/heuristics.rs` | Regex-based first-pass detection (7 rule categories) |
| `src/detection/` | 8-class `AttackType` enum + result types |
| `src/training/` | Training surface — feature-gated `full` / `training`. Not on public API. |
| `python/` | Python package source + tests |
| `js/` | TypeScript wrapper + prebuilds loader (`src/index.ts`, `cmd/install`-style flow not present yet) |
| `go/` | Go bindings: `jailguard.go` (CGo), `jailguard_purego.go` (purego), `cmd/install/` (installer) |
| `models/` | Embedded classifier weights (`neural_binary_200k.json`, 1.5 MB) + tokenizer |
| `examples/` | One file per supported example, gated where they need `full` |
| `tests/` | Integration tests + scenario suites |
| `benches/` | Criterion benches (`detect.rs`) |
| `BENCHMARKS.md` | Authoritative numbers — methodology, dataset breakdown, latencies |
| `docs/` | API.md, ARCHITECTURE.md, GETTING_STARTED.md, INTEGRATION_GUIDE.md |

## Code style

- **Clippy lints** (strict, enforced by CI): `dbg_macro = "deny"`,
  `todo = "deny"`, `unimplemented = "deny"`. See `Cargo.toml`
  `[lints.clippy]` block for the full pedantic list.
- **Use `eprintln!` not `println!` in binaries** — `clippy::print_stdout`
  is warned on the library.
- **Regex word boundaries:** always use `\b` for short acronyms (DAN,
  STAN) to avoid matching inside unrelated words.
- **Public API surface is deliberately minimal:** three functions
  (`detect`, `is_injection`, `score`) plus `detect_batch` and
  `download_model`. Resist adding more to the public surface.
- **MSRV is 1.85** (`rust-version` in `Cargo.toml`).

## Feature flags

| Feature | Purpose |
|---|---|
| `default` | Embedded detector only — minimal dep set |
| `python` | pyo3 + maturin bindings |
| `napi` | napi-rs + napi-build for Node.js |
| `full` | Training, evaluation, ensemble, experimental modules |
| `training` | Subset of `full` exposing `NeuralBinaryNetwork` / `NeuralDataLoader` to external training pipelines |
| `download` | Reqwest + tokio for dataset download (training only) |

Default is **deliberately small**. Don't enable `full` unless touching
the training surface.

## Attack taxonomy (8-class)

The training classifier knows 8 attack categories. The public API
exposes only `is_injection: bool` + a numeric `score: f32` + a
`RiskLevel` enum (`Safe`/`Low`/`Medium`/`High`/`Critical`). The 8-class
output is not on the public API.

| Index | Name | Description |
|---|---|---|
| 0 | Benign | No attack |
| 1 | RolePlay | Persona injection |
| 2 | InstructionOverride | "Ignore previous instructions" |
| 3 | ContextManipulation | Separators, framing |
| 4 | OutputManipulation | Format changes |
| 5 | EncodingAttack | Base64, ROT13, hex |
| 6 | JailbreakPattern | DAN, STAN, multi-technique |
| 7 | PromptLeaking | "Reveal system prompt" |

## ort crate (ONNX runtime) pinning

`ort` is pinned to **`=2.0.0-rc.9`**. `rc.11+` requires glibc 2.38+,
and our minimum target (Chromebook Crostini, manylinux_2_28) has glibc
2.36. Bumping `ort` requires verifying the target deployment glibc on
every platform in the release matrix.

## Boundaries

### Always do

- Run `cargo fmt --all` and `cargo clippy --all-targets` before committing.
- Add a `CHANGELOG.md` entry for user-visible changes.
- Update `BENCHMARKS.md` whenever the model is retrained or the test
  set changes.

### Ask first

- Renaming any public function/type in `src/embedded.rs`,
  `src/c_api.rs`, `src/python.rs`, `src/napi.rs`, or any language
  binding's user-facing surface.
- Changes to the `Cargo.toml` `include = […]` list (controls what
  ships to crates.io).
- Bumping the `ort` version (see pinning note above).
- Changes to the model weights file (`models/neural_binary_200k.json`).

### Never do

- Commit raw model files larger than 50 MB to git. The 90 MB ONNX
  embedder is auto-downloaded, not committed.
- Push to remote without explicit authorization. The user prefers a
  reversible-changes-only workflow.
- Skip git hooks (`--no-verify`) or bypass signing (`--no-gpg-sign`).
- Disable clippy lints with `#[allow(...)]` without a comment
  explaining why.

## Release flow

```sh
# 1. Verify version alignment first
grep -E '^version' Cargo.toml pyproject.toml
node -p "require('./js/package.json').version"

# 2. Tag and push
git tag v0.X.Y
git push origin v0.X.Y
# .github/workflows/release.yml fires automatically
```

The release workflow handles: Rust crate (`cargo publish`), Python
wheels (multi-platform via maturin → PyPI), npm package (prebuilt
napi addons), Go module subpath tag (`go/v0.X.Y`), Go FFI tarballs
attached to GitHub Release, and the GitHub Release itself. PyPI uses
an API token (`secrets.PYPI_API_TOKEN`), not OIDC.

## Common pitfalls

- **Bare `go build` fails:** the Go bindings need either the
  `jailguard_dev` build tag (monorepo) or `CGO_CFLAGS`/`CGO_LDFLAGS`
  from the installer. Monorepo shortcut: `go build -tags jailguard_dev ./...`.
- **`npm install` no longer rebuilds anything.** The new prebuilds
  pattern (see `js/prebuilds/` populated by CI) ships native addons in
  the tarball. For local dev: `cd js && npm run build:native`.
- **Python wheel needs `maturin` + recent Rust toolchain.**
- **Bare `cargo build` from inside `go/`:** doesn't work without env
  vars set, by design — see `go/README.md` Option A (CGo) or Option B
  (purego).
