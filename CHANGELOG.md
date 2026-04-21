# Changelog

All notable changes to JailGuard are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-04-21

Initial public release.

### Added

- **Embedded detector API.** Zero-config entry points at the crate root:
  - `detect(text) -> DetectionOutput`
  - `is_injection(text) -> bool`
  - `score(text) -> f32`
  - `detect_batch(texts) -> Vec<DetectionOutput>`
  - `ensure_model() -> Result<PathBuf>` for pre-warming the ONNX cache.
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
  - `full`: training, evaluation, ensemble, and experimental modules.
  - `train`, `wgpu`, `download`: narrower opt-ins for training workflows.
- **Example.** `examples/quick_start.rs` demonstrates the three-function API.

### Measured

On a held-out split of the 200K training mix (20,000 samples):

| Metric       | Value   |
|--------------|---------|
| Accuracy     | 99.07%  |
| Precision    | 98.93%  |
| Recall       | 99.22%  |
| F1           | 0.9908  |
| CPU latency  | <50 ms  |

Note: these figures are on the project's own dataset split, not an independent
public benchmark. Evaluation on PINT and AgentDojo is planned for a later
release.

### Known limitations

- Not yet evaluated on Lakera PINT, AgentDojo, or DataSentinel benchmarks.
- First call to `detect()` without a cached ONNX model triggers a 90 MB
  download from HuggingFace. Call `ensure_model()` at startup to avoid this.
- Indirect injection (tool-output contamination) is not a dedicated category
  in the current binary classifier.

### Not shipped

Training datasets, dataset-preparation scripts, and training-time documentation
live outside the published crate.

---

[0.1.0]: https://github.com/yfedoseev/jailguard/releases/tag/v0.1.0
