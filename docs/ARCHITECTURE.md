# Architecture

JailGuard is a two-stage detector:

```
text ──► [ MiniLM-L6-v2 (ONNX) ] ──► 384-dim vector ──► [ 3-layer MLP ] ──► score (0..1)
```

Neither stage is trained at runtime. The MiniLM encoder is the pre-trained
public checkpoint from `sentence-transformers`. The classifier head is a small
MLP trained once on a 200K-sample mix of public prompt-injection datasets; its
weights ship inside the crate as JSON.

## Stage 1 — sentence embedding

**Model.** `sentence-transformers/all-MiniLM-L6-v2`, ONNX format, 384-dim
output. The file (~90 MB) is not embedded in the crate; it is downloaded on
first use to `$JAILGUARD_MODEL_DIR` or `$HOME/.cache/jailguard/` and memoised.

**Runtime.** The `ort` crate (`=2.0.0-rc.9`) loads the model via the ONNX
Runtime C API. Inference runs single-threaded on CPU with the default ORT
execution provider.

**Pipeline.**

1. `tokenizers::Tokenizer` tokenises the input with the embedded
   HuggingFace tokenizer (466 KB, `include_bytes!`-ed).
2. Input is truncated to `MAX_SEQ_LENGTH = 256` tokens.
3. ONNX session produces per-token hidden states, shape `[1, seq_len, 384]`.
4. Hidden states are **mean-pooled** weighted by the attention mask.
5. The pooled vector is **L2-normalised** to unit length.

The result is a 384-dim, unit-norm sentence embedding.

## Stage 2 — binary classifier

**Shape.** `384 → 256 → 128 → 1` MLP, ~130 K parameters.

- Linear(384, 256) → ReLU → Dropout(0.2)
- Linear(256, 128) → ReLU → Dropout(0.2)
- Linear(128, 1)   → Sigmoid

Dropout is a train-time-only component; `forward_eval` does not apply it.

**Storage.** Weights live in `models/neural_binary_200k.json` (1.5 MB) and are
compiled into the binary via `include_str!`. At startup, `serde_json`
deserialises them into a `NeuralBinaryNetwork` struct held inside a
`once_cell::Lazy<EmbeddedDetector>`.

**Training.** Binary cross-entropy, Adam-style gradient descent, batch size
64, learning rate 0.01, 50 epochs with early stopping (best val epoch: 44).
Training code lives under `src/training/` behind the `full`/`train` feature
flags; datasets and data-prep scripts are not distributed with the crate.

## Detector lifecycle

```
                    ┌───────────────────────────────┐
first detect()  ──► │ EmbeddedDetector::new()       │
                    │  • ensure_model()              │
                    │  • Session::from_file(...)     │
                    │  • Tokenizer::from_bytes(...)  │
                    │  • serde_json::from_str(...)   │
                    └──────────────┬────────────────┘
                                   │
                                   ▼
                         ┌─────────────────────┐
  detect(text) ────────► │  embed() ── forward │ ──► DetectionOutput
                         └─────────────────────┘
```

Initialisation is done once per process via `once_cell::sync::Lazy`.
Concurrent first-callers block on the same init future; subsequent calls are
lock-free reads.

## Crate layout (public surface)

```
src/
├── lib.rs           — re-exports, feature gating
├── embedded.rs      — detect / is_injection / score / detect_batch, RiskLevel
├── model_manager.rs — ensure_model(), cache resolution, download
├── network.rs       — NeuralBinaryNetwork forward_eval (inference only)
└── error.rs         — Error, Result
```

Everything under `src/training/`, `src/collection/`, `src/api/`, `src/agent/`,
`src/dataset/`, `src/detection/`, `src/ensemble/`, etc., is gated behind the
`full` feature and is not part of the stable 0.1.x API.

## Threading and concurrency

- The `once_cell` detector is `Sync`. `detect()` can be called from any thread.
- ONNX Runtime sessions are not thread-safe for concurrent `run()` calls in
  ORT 2.x rc.9. Serialise access yourself, or create per-worker sessions if
  you need parallelism. A future release may expose a parallel-friendly
  `Detector` builder — open an issue if you need it.

## What is deliberately **not** here

- **Agentic defense layers** (spotlighting, task tracking, privilege context,
  output validation, behaviour monitoring). These exist under the `full`
  feature as research code; they are not part of the library's primary
  detection contract.
- **Online learning / feedback loop.** The shipped classifier is static. The
  `feedback` and `online` modules under `full` are experimental.
- **Indirect injection routing.** The binary classifier treats tool outputs
  the same as user inputs. Detecting prompt-injection content in retrieved
  documents is a call site decision (run `detect()` on the retrieved text
  before concatenating it into the prompt).

## Dependencies at runtime (default features)

| Crate         | Purpose                             |
|---------------|-------------------------------------|
| `ort`         | ONNX Runtime bindings (rc.9 pinned) |
| `tokenizers`  | HuggingFace BPE tokeniser            |
| `ndarray`     | Tensor shaping for ONNX input        |
| `serde_json`  | Classifier weight deserialisation    |
| `once_cell`   | Lazy singleton detector              |
| `ureq`        | Blocking HTTP download of ONNX file  |
| `thiserror`   | Error derive                         |

The feature-gated stack (`full`) pulls in Burn, tracing, regex, reqwest,
tokio, and friends. Stay on default features for a lean dependency tree.
