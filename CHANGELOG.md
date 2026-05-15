# Changelog

All notable changes to JailGuard are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.2] - 2026-05-14

> Elixir bindings + Windows support

### Added

- **Elixir binding** ([Hex package `jailguard`](https://hex.pm/packages/jailguard)).
  Implemented as a Rustler NIF in `elixir/native/jailguard_nif/` and shipped
  as precompiled artifacts via `rustler_precompiled` for
  `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
  `x86_64-apple-darwin`, `aarch64-apple-darwin`, and `x86_64-pc-windows-msvc`.
  Exposes `detect`, `detect!`, `is_injection`, `score`, `detect_batch`,
  `download_model`, `model_cache_dir`, and `version`. Set `JAILGUARD_BUILD=1`
  to compile from source on unsupported targets.
  Originally contributed as a C-NIF source-built port by
  [@elchemista](https://github.com/elchemista) in
  [PR #16](https://github.com/yfedoseev/jailguard/pull/16); reworked to
  Rustler + `rustler_precompiled` + Hex publishing for this release. See
  the Community contributors section below for the full credit.
- **Windows (x86_64) support** for the native `cdylib` / `staticlib`,
  Python wheels, the NodeJS napi addon, and the Elixir NIFs — all built
  for `x86_64-pc-windows-msvc` in the release matrix and exercised by
  the per-binding CI workflows. Resolves the long-standing
  ort-sys (`/MD`) vs esaxx-rs (`/MT`) LNK2038 mismatch by disabling
  `tokenizers`'s `esaxx_fast` default feature — esaxx-rs's `build.rs`
  hardcodes `cc::Build::new().static_crt(true)`, so the C++ blob can't
  be persuaded to use `/MD` via rustflags or env vars. We don't use
  Unigram-model suffix-array tokenization (only WordPiece for BERT-style
  models), so dropping `esaxx_fast` removes the C++ dep at no functional
  cost. `.cargo/config.toml` retains the `target-feature=-crt-static`
  pin as defense-in-depth for any future `cc-rs` consumers.

  Also fixes `model_manager::cache_dir()`, which previously errored with
  "HOME environment variable not set" on Windows. The fallback chain is
  now `JAILGUARD_MODEL_DIR` → `HOME` → `USERPROFILE` → `LOCALAPPDATA`.

  **Go on Windows is deferred to a follow-up.** Go's `cgo` on Windows
  uses mingw-gcc, which can't link MSVC-built libraries (jailguard.lib
  carries MSVC-mangled C++ symbols from the bundled onnxruntime). The
  proper fix is to ship a parallel `x86_64-pc-windows-gnu` artifact for
  Go consumers; tracked separately for 0.1.3.
- **Public Rust API**: `jailguard::model_cache_dir()` (re-export of
  `model_manager::cache_dir_string`). Non-breaking addition that lets the
  Elixir NIF — and any out-of-tree consumer — query the ONNX cache
  directory without going through the C ABI.
- **Hex publish pipeline** end-to-end: new `build-elixir-nifs` matrix
  (philss/rustler-precompiled-action across 5 targets × NIF versions
  2.16/2.17), new `smoke-test-nifs` job that loads the just-built NIF in
  a real Mix project and runs `JailGuard.detect/1` on injection + benign
  samples, new `publish-hex` job gated on the smoke test and the GitHub
  release (so `mix rustler_precompiled.download` can fetch the attached
  NIF tarballs to write the checksum file).
- **`mix hex.outdated` monthly issue automation** in `outdated.yml`,
  matching the existing `cargo outdated` job.

### Changed

- Dependency bumps via Dependabot: `once_cell` 1.21.3→1.21.4,
  `regex` 1.12.2→1.12.3, `reqwest` 0.12.28→0.13.3, `thiserror` 2.0.17→2.0.18,
  `unicode-segmentation` 1.12.0→1.13.2.
- Dependabot now tracks Hex (`/elixir`) and the NIF cargo crate
  (`/elixir/native/jailguard_nif`) in addition to the existing Rust /
  Python / Node / Go / actions ecosystems.

### Fixed

- **`publish-pypi` actually publishes to PyPI now.** The earlier 0.1.0
  and 0.1.1 release runs both failed silently at the publish step:
  `actions/download-artifact` was unfiltered, so it pulled every
  artifact (napi addons, `libjailguard.so/.a/.rlib`, `jailguard.h`, the
  `SHA256SUMS` manifest) into `dist/` alongside the wheels. `twine check
  dist/*` then rejected the upload with
  `InvalidDistribution: Unknown distribution format: 'SHA256SUMS'`
  before any wheel was actually transferred — PyPI never received the
  package on either release. The job now filters the download to
  `wheels-*` and `sdist` patterns only and adds a guard step that fails
  the job if anything other than `.whl` / `.tar.gz` slips into `dist/`.

### Community contributors

- **[@elchemista](https://github.com/elchemista)** (Yuriy Zhar) — contributed
  the initial Elixir binding in [PR #16](https://github.com/yfedoseev/jailguard/pull/16):
  a complete C-NIF port of the JailGuard C ABI, the full Mix package
  scaffold (mix.exs, `JailGuard` / `JailGuard.Result` / `JailGuard.Error`
  modules), an 11-case ExUnit suite covering version/cache-dir lookup,
  injection vs benign classification, `detect!`, batch ordering, invalid
  inputs, and concurrent detection, plus repo wiring (Makefile targets,
  CODEOWNERS, README, docs/API.md). Yuriy made the design choice to wrap
  the existing stable C ABI rather than reaching into the Rust crate
  directly, which gave the binding a clean three-layer split
  (`JailGuard` → `JailGuard.Native` → C NIF) and idiomatic Elixir
  `{:ok, _}/{:error, _}` ergonomics from day one. The Rustler +
  precompiled-NIF + Hex-publishing layer in this release builds on top
  of Yuriy's port — same public Elixir API surface, same test suite,
  same module structure. Hex publishing wouldn't have been on the 0.1.2
  scope without this initial contribution.

---

## [0.1.1] - 2026-05-14

Release-pipeline fixes only. No runtime/API changes — all detector
behavior, accuracy, and embedded model assets are identical to 0.1.0.

The v0.1.0 release pipeline shipped `cargo` + `npm` + the Go module
tag, but the PyPI publish, Go FFI binary tarballs, and GitHub Release
were silently skipped because the Linux wheel **smoke test** failed
and didn't fail-stop the publish jobs that should have depended on it.
0.1.1 fixes the smoke test itself, the Go FFI packaging step that
also failed, and the dependency wiring so a smoke failure now blocks
every downstream publish.

### Fixed (release pipeline)

- **`smoke-test-wheels`**: the previous step inlined Python source
  inside a YAML `run: |` heredoc and `python -c "..."`. The 12-space
  YAML indentation became part of the Python string, producing
  `IndentationError: unexpected indent` on `import jailguard` inside
  both `debian:bookworm-slim` and `amazonlinux:2023`. Extracted the
  smoke test to `.github/scripts/smoke-test.py` (real file, column-0
  source) and mounted into the container via `docker -v`.
- **`package-go-ffi` (verify checksums step)**: invalid bash
  substitution `${$(ls *.tar.gz | wc -l)}` → `count=$(ls *.tar.gz | wc -l)`.
  All 8 tarballs (4 static + 4 shared) verified fine; the failure
  was only in the final summary echo, but it failed the job and
  cascaded into the GitHub-release skip.
- **Publish-job gating**: `publish-crates-io`, `publish-npm`,
  `publish-go`, `package-go-ffi`, and `github-release` now all
  `needs: smoke-test-wheels` in addition to their build deps. v0.1.0
  shipped to `crates.io`, `npm`, and the Go module tag despite the
  smoke failure because none of those jobs depended on the smoke
  step. From v0.1.1 on, a smoke failure fail-stops every publish —
  no asymmetric releases.

### Changed (release pipeline only)

- `release.yml` build-wheels-linux matrix no longer includes the
  `musllinux_1_2` rows; `ort-sys 2.0.0-rc.9` does not ship prebuilt
  onnxruntime for `*-unknown-linux-musl`. Matches the parallel
  removal already done in `python.yml`. Re-enable when upstream
  publishes musl binaries or we vendor our own.

### Distribution status

The v0.1.0 partial release stays as-is on `crates.io` (cannot delete)
and `npm` (we are within the 72-hour window, but leaving in place
since the artifact itself is correct — only PyPI was missing). The
v0.1.1 tag will publish a full set across all five channels:
`crates.io`, `PyPI`, `npm`, the `go/v0.1.1` module tag, and the
GitHub Release with the Go FFI tarballs.

---

## [0.1.0] - 2026-04-21

Initial public release.

### Added

- **Embedded detector API.** Zero-config entry points at the crate root:
  - `detect(text) -> DetectionOutput`
  - `is_injection(text) -> bool`
  - `score(text) -> f32`
  - `detect_batch(texts) -> Vec<DetectionOutput>`
  - `download_model() -> Result<PathBuf>` for pre-warming the ONNX cache.
- **Embedded 200K classifier.** A ~130K-parameter MLP (`384 → 256 → 128 → 1`)
  trained on 200,000 balanced samples from 14 public datasets. The classifier
  weights (`models/neural_binary_200k.json`, 1.5 MB) ship inside the crate and
  load lazily via `once_cell`.
- **ONNX embedding backend.** Uses `sentence-transformers/all-MiniLM-L6-v2`
  (384-dim) via the `ort` crate. The 90 MB ONNX file is auto-downloaded to
  `~/.cache/jailguard/` on first use; override with the `JAILGUARD_MODEL_DIR`
  environment variable.
- **Feature flags.**
  - `default`: embedded detector only (minimal dependency set).
  - `python`, `napi`: language bindings (pyo3 / napi-rs).
  - `full`: training, evaluation, ensemble, and experimental modules.
  - `training`, `download`: narrower opt-ins for training-pipeline tooling.
- **Example.** `examples/quick_start.rs` demonstrates the three-function API.

### Measured

Production model: 17-source pipeline (79,626 samples; ALERT adversarial,
LMSYS Toxic Chat, JailbreakBench, BeaverTails, Alpaca, Dolly, shalyhinpavel
hard-negatives, etc.). Trained with Adam (lr=0.001, β₁=0.9, β₂=0.999) +
weighted BCE (injection_weight=2.5). Re-validated 2026-05-03.

| Test set | Samples | Accuracy | Precision | Recall | F1 |
|----------|---------|----------|-----------|--------|-----|
| Pipeline (in-distribution) | 7,049 | 98.40% | 98.56% | 97.98% | 0.983 |
| J1N2 mix (OOD) | 5,000 | 99.38% | 98.09% | 99.94% | 0.990 |
| shalyhinpavel hard-negatives (OOD) | 147 | 89.12% | 76.60% | 87.80% | 0.818 |

CPU latency: p50 14 ms, p99 19 ms (Apple M3, single thread).
Cold start ~140 ms; warm call ~20 ms.

On a 4-year-old low-power Chromebook (Intel i5-10210U @ 1.6 GHz, 4c/8t),
p50 ~37 ms, p99 ~43 ms; cold start ~350 ms. See [`BENCHMARKS.md`](./BENCHMARKS.md)
for the full two-machine comparison.

The pipeline test split is in-distribution. J1N2 and shalyhinpavel are
held outside the training data — see [`BENCHMARKS.md`](./BENCHMARKS.md).

### Known limitations

- First call to `detect()` without a cached ONNX model triggers a 90 MB
  download from HuggingFace. Call `download_model()` at startup to avoid this.
- Indirect injection (tool-output contamination) is not a dedicated category
  in the current binary classifier.

### Not shipped

Training datasets, dataset-preparation scripts, and training-time documentation
live outside the published crate.

---

[0.1.1]: https://github.com/yfedoseev/jailguard/releases/tag/v0.1.1
[0.1.0]: https://github.com/yfedoseev/jailguard/releases/tag/v0.1.0
