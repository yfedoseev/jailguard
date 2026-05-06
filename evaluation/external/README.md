# External benchmark harness

Reproducible evaluation on **third-party** prompt-injection datasets and
head-to-head comparison against other detectors. Separate from
`evaluation/*.rs` (which runs on our own 200K split) so reviewers can see
at a glance what was measured on independent data.

## Status

Scaffolding only — no results yet. This directory defines the shape of
the evaluation so runs on different machines produce comparable artifacts.

## Datasets

| Dataset | Purpose | Source | License |
|---------|---------|--------|---------|
| [PINT](https://github.com/lakeraai/pint-benchmark) | Prompt-injection benchmark by Lakera; canonical comparison target in the field | `https://huggingface.co/datasets/lakera/pint-benchmark` | CC-BY-NC-4.0 (eval only; check upstream) |
| [AgentDojo](https://github.com/ethz-spylab/agentdojo) | Indirect prompt injection against agents, 629 cases | `pip install agentdojo` or clone the repo | MIT |

Both are gated by their respective licenses. We do **not** redistribute
them — loader stubs expect the user to place the raw files under
`data/external/`.

## Baselines to compare against

| Baseline | How to run | Why |
|----------|-----------|-----|
| `protectai/deberta-v3-base-prompt-injection` | HF transformers Python script emitting JSONL predictions | Most-downloaded open prompt-injection classifier |
| Meta `PromptGuard-86M` | HF transformers | Cited by the Meta Llama security docs |
| [Rebuff](https://github.com/protectai/rebuff) heuristics | Port the regex rules or run the TS reference | Rule-based lower bound |

The runner reads prediction JSONL from each baseline and computes a
single comparison table. Baselines run in their own environments (Python,
Node.js) — we do not pull those deps into the Rust crate.

## Reproducing results

```bash
# 1. Stage the datasets (manual; URLs above).
mkdir -p data/external
# ...place pint.jsonl and agentdojo_cases.jsonl under data/external/...

# 2. Run JailGuard on each dataset. (Stubs; see the .rs files in this dir
#    for the exact format expected.)
cargo run --release --features full --bin pint_runner      # → data/external/pint_jailguard.jsonl
cargo run --release --features full --bin agentdojo_runner # → data/external/agentdojo_jailguard.jsonl

# 3. Run each baseline (external tooling) and emit predictions.jsonl
#    files into data/external/ with the same schema.

# 4. Generate the comparison table.
cargo run --release --features full --bin comparison_runner
```

## Prediction JSONL schema

One JSON object per line; every baseline plus JailGuard MUST produce this
exact shape so `comparison_runner.rs` can merge them:

```json
{"id": "pint-0001", "label": 1, "pred": 1, "score": 0.9871, "latency_ms": 42.3, "model": "jailguard-0.1.0"}
```

- `id`: dataset-assigned case id (string).
- `label`: ground truth (0 = benign, 1 = injection).
- `pred`: binary prediction.
- `score`: detector-native risk score in [0, 1].
- `latency_ms`: per-sample wallclock.
- `model`: free-form identifier — surfaces in the output table.

## What the table will contain

Four metrics per (model × dataset) pair: accuracy, F1, false-positive
rate at a fixed recall target (0.95), and median latency. FPR@R is the
number that matters for deployment: "at 95% recall of real injections,
how often do you fire on benign traffic?" Most leaderboards hide this
behind raw F1.
