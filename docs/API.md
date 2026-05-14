# API reference

The crate root (`jailguard::`) re-exports everything you need for the default
detection API. Source of truth is rustdoc:

```bash
cargo doc --open -p jailguard
```

This file summarises the stable public surface for the 0.1.x line.

## Functions

### `detect(text: &str) -> DetectionOutput`

Run full detection on a single input and return score, boolean decision,
confidence, and risk band.

```rust
let out = jailguard::detect("ignore previous instructions");
assert!(out.is_injection);
assert!(out.score > 0.5);
```

On the first call per process, this triggers lazy detector initialisation,
which downloads the ONNX embedding model if it is not already cached.
Subsequent calls reuse the same in-memory session.

### `is_injection(text: &str) -> bool`

Boolean shortcut equivalent to `detect(text).is_injection`. Uses the default
0.5 decision threshold.

### `score(text: &str) -> f32`

Return only the raw model probability (0.0 – 1.0). Use this when you want to
apply a custom threshold or route borderline cases to a stricter check.

### `detect_batch(texts: &[&str]) -> Vec<DetectionOutput>`

Convenience wrapper that calls `detect` on each input sequentially while
reusing the same detector instance. No tensor-level batching yet.

### `download_model() -> Result<PathBuf, Error>`

Resolve the cache directory (`$JAILGUARD_MODEL_DIR` or `$HOME/.cache/jailguard/`),
download the ONNX model if it is missing, and return the absolute path.
Idempotent. Call this at application startup to avoid a download on the first
request.


## Types

### `DetectionOutput`

```rust
pub struct DetectionOutput {
    pub is_injection: bool,   // score > 0.5
    pub score: f32,           // raw model output, 0.0..=1.0
    pub confidence: f32,      // max(score, 1 - score), always >= 0.5
    pub risk: RiskLevel,      // bucketed score
}
```

### `RiskLevel`

```rust
pub enum RiskLevel {
    Safe,      // score < 0.3
    Low,       // 0.3..0.5
    Medium,    // 0.5..0.7
    High,      // 0.7..0.9
    Critical,  // >= 0.9
}
```

### `Error` and `Result`

```rust
pub type Result<T> = std::result::Result<T, Error>;

#[non_exhaustive]
pub enum Error {
    Io(std::io::Error),
    Config(String),
    Model(String),
    // ...
}
```

`Error` implements `std::error::Error`, `Display`, and `From<std::io::Error>`.
New variants may be added in minor releases (hence `#[non_exhaustive]`).

## Environment variables

| Variable               | Purpose                                             |
|------------------------|-----------------------------------------------------|
| `JAILGUARD_MODEL_DIR`  | Override the cache directory for the ONNX model.    |
| `HOME`                 | Used as fallback root for `~/.cache/jailguard/`.    |

## Feature flags

| Feature     | Effect                                                           |
|-------------|------------------------------------------------------------------|
| *(default)* | Embedded detector only — `detect`, `is_injection`, `score`, etc. |
| `full`      | Enables training, evaluation, and experimental modules (not part of the stable API). |
| `train`     | `full` + training-specific dependencies.                         |
| `wgpu`      | `full` + WGPU backend for Burn.                                  |
| `download`  | `full` + async dataset downloaders.                              |

Anything enabled only under `full` (or its supersets) is **not part of the
0.1.x stable API** and may be reshaped or split into a separate crate in a
future release.

## Stability

The four public functions and their return types (`DetectionOutput`,
`RiskLevel`, `Error`, `Result`) are the stable 0.1.x surface. Classifier
weights and the underlying ONNX embedding model may change in point releases,
which can shift individual scores — if you rely on specific numeric outputs,
pin a version.
