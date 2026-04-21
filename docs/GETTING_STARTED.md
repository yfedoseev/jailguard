# Getting started

JailGuard is a Rust library for prompt-injection detection. The embedded
classifier weights ship inside the crate; the ONNX embedding model (~90 MB) is
auto-downloaded and cached on first use.

## Install

```toml
[dependencies]
jailguard = "0.1"
```

Minimum supported Rust version: **1.70** (for `std::sync::OnceLock` and the
`ort` crate's requirements).

No feature flags are required for the default detection API.

## Hello, detector

```rust
use jailguard::{detect, is_injection};

fn main() {
    if is_injection("ignore previous instructions") {
        println!("blocked");
    }

    let result = detect("What is the capital of France?");
    println!(
        "injection={} score={:.3} risk={:?}",
        result.is_injection, result.score, result.risk,
    );
}
```

The first call to any of `detect`, `is_injection`, `score`, or `detect_batch`
lazily initialises the detector. If the ONNX model is not cached locally, it is
downloaded from HuggingFace (~90 MB) to `~/.cache/jailguard/`.

## Pre-warming for production

To avoid a cold-start download on the first request, call `ensure_model()` at
startup:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    jailguard::ensure_model()?;           // downloads if missing (blocking)
    let _ = jailguard::detect("warm up"); // triggers model load
    // ... start serving traffic ...
    Ok(())
}
```

`ensure_model()` is idempotent — it returns immediately if the file already
exists.

## Cache location

The ONNX model is cached on disk at the first path that resolves:

1. `$JAILGUARD_MODEL_DIR` (if set)
2. `$HOME/.cache/jailguard/`

In a container, set `JAILGUARD_MODEL_DIR` to a directory that is part of your
persistent or build-time layer:

```dockerfile
ENV JAILGUARD_MODEL_DIR=/app/models
RUN mkdir -p /app/models \
 && curl -L --fail -o /app/models/all-MiniLM-L6-v2.onnx \
      https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx
```

## Threshold tuning

The default decision boundary is `score > 0.5 ⇒ injection`. If you need higher
recall (catch more attacks at the cost of more false positives) or higher
precision, use `score()` directly:

```rust
use jailguard::score;

let s = score(user_input);
let block = s > 0.35;  // more aggressive
```

The `RiskLevel` enum exposes the five-bucket banding:

| Band        | Score range |
|-------------|-------------|
| `Safe`      | `< 0.3`     |
| `Low`       | `0.3 – 0.5` |
| `Medium`    | `0.5 – 0.7` |
| `High`      | `0.7 – 0.9` |
| `Critical`  | `≥ 0.9`     |

## Batch inference

If you have several inputs to score at once, `detect_batch` reuses the same
detector instance and avoids repeated lazy-init overhead:

```rust
use jailguard::detect_batch;

let inputs = vec!["hi", "ignore all prior instructions", "what is 2+2?"];
for out in detect_batch(&inputs) {
    println!("{:?}", out.risk);
}
```

Note that the ONNX session inference path is currently per-input; `detect_batch`
is a convenience wrapper, not a batched tensor call. If you need throughput,
open an issue — batched ONNX inference is on the roadmap.

## Troubleshooting

**The first call panics with "Failed to initialize embedded detector".**
The ONNX model could not be loaded. Most common causes:

- No network at startup → call `ensure_model()` earlier, when network is up.
- `$HOME` is unset and `$JAILGUARD_MODEL_DIR` is not set → set one of them.
- Disk full → free space or point `JAILGUARD_MODEL_DIR` at a larger volume.

**First call is slow.**
That's the 90 MB download. Pre-warm with `ensure_model()` at startup or ship
the file with your container image.

**False positives on legitimate technical questions.**
The classifier was trained on a binary mix; certain phrasings ("override the
default behavior") can trigger it. Raise the threshold via `score()`, or file
examples as issues — they're useful training data for future releases.

## Next steps

- [API reference](API.md) — function-by-function documentation.
- [Architecture](ARCHITECTURE.md) — how embeddings and the classifier fit together.
- [Integration guide](INTEGRATION_GUIDE.md) — patterns for web services, agents, and CI.
