# Data Workspace

Scratch directory used during local development for cached datasets and
precomputed embeddings. **Nothing in this directory ships with the published
crate** — all files matching `*.json` / `*.csv` are gitignored, and the
detector at runtime needs only the embedded classifier weights
(`models/neural_binary_200k.json`) plus the auto-downloaded ONNX embedder.

If you are a user of the `jailguard` crate, you do not need anything in
this directory.

## What lives here during development

| Path | Origin |
|------|--------|
| `combined.json`, `combined_200k.json` | Cached, de-duplicated mix of the 17 public HuggingFace datasets listed in [`BENCHMARKS.md`](../BENCHMARKS.md#composition). |
| `combined_*_embeddings.json` | Precomputed 384-dim MiniLM embeddings for the cached datasets. ~1 GB. |
| `expansion/`, `training/`, `baseline/`, `deepset/`, `collected_samples/` | Per-source intermediates from the training pipeline. |

## Reproducing the training data

JailGuard does not redistribute these datasets — they belong to their
original authors. The full per-source list (HuggingFace dataset IDs,
license terms, and the role each plays in the mix) is in
[`BENCHMARKS.md`](../BENCHMARKS.md#composition).

To rebuild a comparable training mix, fetch each source from HuggingFace
and apply the deduplication + normalization documented in `BENCHMARKS.md`.
The end-to-end pipeline (data prep, embedding, training, evaluation) is
not part of the public crate; the model checked into `models/` is the
authoritative training artifact.

## Data format

Cached samples are stored as JSON objects:

```json
{
  "text": "The prompt text",
  "label": 0,
  "type": "benign",
  "embedding": [0.1, 0.2, ...]
}
```

Labels: `0` = benign, `1` = injection.
