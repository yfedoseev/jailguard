# Integration guide

Patterns for using JailGuard in real applications. Each section is a recipe —
pick the one that matches your architecture.

## Pre-warm before serving traffic

The first detection call per process downloads the ONNX model (if missing) and
loads the session. Move that cost out of the hot path:

```rust
fn main() -> anyhow::Result<()> {
    jailguard::ensure_model()?;            // download if needed
    let _ = jailguard::detect("warm up");  // build the session now
    serve()?;
    Ok(())
}
```

In a Kubernetes readiness probe, fail the probe until a warm-up detection
succeeds.

## HTTP middleware (axum)

```rust
use axum::{
    extract::Json, http::StatusCode, middleware::Next,
    response::Response, body::Body, extract::Request,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Prompt { text: String }

pub async fn inject_guard(
    Json(body): Json<Prompt>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // `detect` is sync and fast (<50 ms); for very high QPS wrap in spawn_blocking.
    let out = jailguard::detect(&body.text);
    if out.is_injection {
        tracing::warn!(score = out.score, "blocked injection attempt");
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(next.run(req).await)
}
```

For frameworks without this middleware shape (actix-web, rocket, tonic), the
pattern is the same: call `jailguard::detect` synchronously before forwarding
the prompt to the LLM.

## Gate on score, not only `is_injection`

The binary flag uses a fixed 0.5 threshold. Production systems usually want
something richer:

```rust
use jailguard::{score, RiskLevel};

let s = jailguard::score(user_input);
match s {
    s if s >= 0.9 => block(),                      // Critical
    s if s >= 0.7 => block_and_alert(),            // High
    s if s >= 0.5 => require_stricter_llm_guard(), // Medium
    s if s >= 0.3 => log_for_review(),             // Low
    _            => allow(),                       // Safe
}
```

The `RiskLevel` enum encodes these same bands if you prefer a structured
match.

## Indirect injection in retrieved context

The classifier does not know whether text came from a user or from a
retrieval step. Run it on each retrieved chunk before concatenating:

```rust
fn safe_chunks(chunks: Vec<String>) -> Vec<String> {
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    jailguard::detect_batch(&refs)
        .into_iter()
        .zip(chunks)
        .filter_map(|(out, chunk)| (!out.is_injection).then_some(chunk))
        .collect()
}
```

For agent tool outputs, apply the same filter before feeding results back
into the LLM.

## Dockerfile — cache the ONNX model in the image

Avoid runtime downloads and registry rate limits by baking the model in:

```dockerfile
FROM rust:1.78-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin your-service

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*

ENV JAILGUARD_MODEL_DIR=/opt/jailguard
RUN mkdir -p $JAILGUARD_MODEL_DIR \
 && curl -L --fail -o $JAILGUARD_MODEL_DIR/all-MiniLM-L6-v2.onnx \
      https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx

COPY --from=builder /app/target/release/your-service /usr/local/bin/
ENTRYPOINT ["your-service"]
```

The resulting image carries ~90 MB of model weight. At start-up, `detect()`
loads from `/opt/jailguard` with no network dependency.

## CI: block PRs that ship injection-prone prompts

```yaml
# .github/workflows/prompt-lint.yml
name: prompt-lint
on: pull_request
jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Cache ONNX model
        uses: actions/cache@v4
        with:
          path: ~/.cache/jailguard
          key: jailguard-minilm-v1
      - run: cargo run --release --example prompt_lint -- prompts/
```

Write `examples/prompt_lint.rs` to walk a directory of prompt files and fail
when `jailguard::detect` flags any of them. Use the cache action so the
90 MB download happens at most once per cache key.

## Threading

`ort` 2.0-rc.9 sessions are not safe for concurrent `run()` from multiple
threads in the default configuration. The crate serialises access through a
`once_cell::Lazy` singleton — correct, but not parallel. For high QPS:

- Bound the detector behind a dedicated worker pool (`tokio::spawn_blocking`
  with a small semaphore).
- Or wait for a future release exposing a `Detector::builder()` that lets
  you create per-worker detectors.

If you hit a contention bottleneck, benchmark before layering complexity —
25 ms per call per core is often enough.

## Sizing

Per process, steady-state:

- **Classifier (MLP):** ~1.5 MB, resident in `.rodata`.
- **ONNX session:** ~120–150 MB resident after load.
- **Per-call latency:** ~25 ms (embedding) + ~1 ms (MLP) on a single modern
  x86 core.

Memory is dominated by the ONNX session. Plan for ~200 MB RSS above your
application baseline.

## Observability

`detect()` logs nothing by default. If you want structured events on blocks:

```rust
let out = jailguard::detect(prompt);
if out.is_injection {
    tracing::warn!(
        score = out.score,
        risk = ?out.risk,
        prompt_len = prompt.len(),
        "jailguard block",
    );
}
```

Do **not** log the raw prompt in production unless you have a clear retention
and redaction policy — prompts can contain user secrets.

## Related

- [Getting started](GETTING_STARTED.md)
- [API reference](API.md)
- [Architecture](ARCHITECTURE.md)
