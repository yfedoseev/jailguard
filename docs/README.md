# JailGuard documentation

| Document                                 | What it covers                                      |
|------------------------------------------|-----------------------------------------------------|
| [Getting started](GETTING_STARTED.md)    | Install, first call, ONNX-model caching, env vars   |
| [API reference](API.md)                  | `detect`, `is_injection`, `score`, `detect_batch`   |
| [Architecture](ARCHITECTURE.md)          | How the embedding + MLP pipeline is wired together  |
| [Integration guide](INTEGRATION_GUIDE.md)| Embedding JailGuard into an app or service          |

The authoritative API documentation is the rustdoc generated from `src/`:

```bash
cargo doc --open
```

## Performance and caveats

See the [README](../README.md) for measured accuracy, latency, and the
held-out-split caveat. The [CHANGELOG](../CHANGELOG.md) records what shipped in
each release.

## Not in this release

Training code, dataset curation, and benchmark writeups are kept out of the
published crate. The `train` feature flag still compiles the in-tree training
modules (see `src/training/`) for users who want to reproduce or extend the
model, but the datasets and dataset-preparation scripts are not distributed
with the crate.
