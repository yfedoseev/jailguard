# Changelog

All notable changes to JailGuard are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
