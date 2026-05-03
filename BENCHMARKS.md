# JailGuard Benchmark Results

Head-to-head comparison of JailGuard against locally-runnable, CPU-only
prompt-injection classifiers on three independent datasets.

---

## Test Environment

| | |
|---|---|
| Hardware | Apple M3, 16 GB RAM |
| OS | macOS Darwin 25.4.0 |
| Python | 3.14.4 |
| PyTorch | 2.11.0 |
| Transformers | 4.57.6 |
| Rust | 1.85 (release build) |
| JailGuard | v0.1.0 |
| Date | 2026-04-30 |

Benchmark script: [`scripts/compare_models.py`](scripts/compare_models.py)
Raw JSONL predictions: `results/` directory (one file per model × dataset).

---

## Datasets

All metrics are measured on **held-out test sets not seen during JailGuard training**.

| Dataset | Samples | Injection | Benign | Source |
|---------|---------|-----------|--------|--------|
| **Pipeline** | 5,945 | 1,303 (21.9%) | 4,642 (78.1%) | Last 10% of JailGuard's own normalized dataset (non-augmented originals only). Same deterministic split used by the Rust benchmark binary. |
| **J1N2** | 5,000 | 1,539 (30.8%) | 3,461 (69.2%) | [`J1N2/mix-prompt-injection-dataset`](https://huggingface.co/datasets/J1N2/mix-prompt-injection-dataset) — fully OOD, not in training data. |
| **shalyhinpavel holdout** | 147 | 41 (27.9%) | 106 (72.1%) | [`shalyhinpavel/Prompt-Injection-Hard-Positives`](https://huggingface.co/datasets/shalyhinpavel/Prompt-Injection-Hard-Positives) validation split only. Benign samples are deliberately crafted to look suspicious (security policy docs, red-team exercises). Tests over-defense specifically. |

> **Pipeline** is JailGuard's own test split — in-distribution. J1N2 and shalyhinpavel are
> fully independent; neither was used in training or hyperparameter selection.

---

## Models

| Model | Key | Params | Architecture | CPU inference |
|-------|-----|--------|--------------|---------------|
| **JailGuard** (this repo) | `jailguard` | 33M + 130K | MiniLM-L6-v2 ONNX + MLP | Native Rust, local |
| protectai/deberta-v3-base-prompt-injection-v2 | `protectai-base` | 184M | DeBERTa-v3-base | Python/transformers |
| deepset/deberta-v3-base-injection | `deepset` | 184M | DeBERTa-v3-base | Python/transformers |
| madhurjindal/Jailbreak-Detector-Large | `madhur` | ~280M | mDeBERTa-v3-base | Python/transformers |

**Gated / token-required models** (not run — require `HF_TOKEN` + license acceptance):

| Model | Params | PINT score | Notes |
|-------|--------|-----------|-------|
| protectai/deberta-v3-small-prompt-injection-v2 | ~44M | — | Access restricted |
| meta-llama/Llama-Prompt-Guard-2-22M | 22M | — | Meta gated |
| meta-llama/Llama-Prompt-Guard-2-86M | 86M | 78.76% | Meta gated |
| qualifire/prompt-injection-sentinel | 395M | — | Gated |
| hlyn/prompt-injection-judge-deberta-70m | 70M INT8 | — | Gated |

---

## Accuracy Results

Metrics: **Acc** = overall accuracy, **Prec** = precision, **Rec** = recall,
**F1** = harmonic mean, **FPR** = false-positive rate on benign samples.
For production use, FPR matters most — it is the rate at which legitimate
user requests are incorrectly blocked.

### Pipeline test set (in-distribution)

JailGuard run on the full 5,945-sample split (natural distribution 21.9% injection).
Competitors run on a 1,000-sample balanced subset (500 injection / 500 benign) due to CPU inference cost;
accuracy numbers are not directly comparable but provide a useful directional signal.

| Model | N | Acc | Prec | Recall | F1 | FPR | TP | FP | TN | FN |
|-------|---|-----|------|--------|----|----|----|----|----|----|
| **JailGuard** | **5,945** | **99.34%** | **97.52%** | **99.54%** | **0.985** | **0.71%** | **1297** | **33** | **4609** | **6** |
| protectai/deberta-v3-base-v2 | 1,000† | 91.00% | 99.28% | 82.60% | 0.902 | 0.60% | 413 | 3 | 497 | 87 |
| deepset/deberta-v3-base | 1,000† | 78.20% | 69.80% | 99.40% | 0.820 | 43.00% | 497 | 215 | 285 | 3 |
| madhurjindal/Jailbreak-Detector | 1,000† | 66.50% | 93.65% | 35.40% | 0.514 | 2.40% | 177 | 12 | 488 | 323 |

† 1,000-sample balanced subset (500 inj / 500 benign); not directly comparable to JailGuard full-set numbers.

### J1N2 OOD dataset (fully out-of-distribution)

JailGuard run on the full 5,000-sample set. Competitors on 1,000-sample balanced subset (500 inj / 500 benign).

| Model | N | Acc | Prec | Recall | F1 | FPR |
|-------|---|-----|------|--------|----|-----|
| **JailGuard** | **5,000** | **99.38%** | **98.09%** | **99.94%** | **0.990** | **0.87%** |
| protectai/deberta-v3-base-v2 | 1,000† | 92.10% | 100.00% | 84.20% | 0.914 | 0.00% |
| madhurjindal/Jailbreak-Detector | 1,000† | 84.80% | 97.54% | 71.40% | 0.824 | 1.80% |
| deepset/deberta-v3-base | 1,000† | 61.10% | 56.31% | 99.00% | 0.718 | 76.80% |

† 1,000-sample balanced subset (500 inj / 500 benign).

### shalyhinpavel hard-negative holdout (147 samples — over-defense test)

Benign samples in this split are deliberately crafted to resemble injections
(security architecture documents, red-team exercise descriptions). High FPR
here means the model is too aggressive and will block legitimate professional
content. All models run on the full 147-sample set.

| Model | Acc | Prec | Recall | F1 | FPR | TP | FP | TN | FN |
|-------|-----|------|--------|----|-----|----|----|----|-----|
| protectai/deberta-v3-base-v2 | 90.48% | 96.55% | 68.29% | 0.800 | 0.94% | 28 | 1 | 105 | 13 |
| **JailGuard** | **89.12%** | **76.60%** | **87.80%** | **0.818** | **10.38%** | **36** | **11** | **95** | **5** |
| madhurjindal/Jailbreak-Detector | 68.71% | 44.44% | 48.78% | 0.465 | 23.58% | 20 | 25 | 81 | 21 |
| deepset/deberta-v3-base | 47.62% | 34.75% | 100.00% | 0.516 | 72.64% | 41 | 77 | 29 | 0 |

---

## Latency Results

All latency measured on CPU (Apple M3), no GPU used.
JailGuard latency is per-sample steady-state (ONNX session already warm).
Python/transformers latency is batch-averaged (batch size 16–32).

### Pipeline dataset

JailGuard latency from full 5,945-sample run (ONNX session warm, per-sample steady-state).
Competitor latency from 1,000-sample balanced run; batch size 16–32, per-sample = batch_time / batch_size.
High p99 for Python models reflects first-batch warmup variation.

| Model | p50 ms | p90 ms | p99 ms | Samples/sec |
|-------|--------|--------|--------|-------------|
| **JailGuard** | **18.1** | **19.2** | **22.3** | **~55** |
| protectai/deberta-v3-base-v2 | 70.5 | 497.3 | 669.8 | ~14 |
| deepset/deberta-v3-base | 80.5 | 438.8 | 873.0 | ~12 |
| madhurjindal/Jailbreak-Detector | 86.1 | 550.9 | 794.0 | ~12 |

---

## Published PINT Leaderboard Reference

PINT (Lakera, CC-BY-NC-4.0) is an independent 4,314-sample benchmark with
20.9% hard negatives (benign prompts containing injection-like words).
JailGuard has not yet been evaluated on PINT; run is pending.

| Model | PINT score | Notes |
|-------|-----------|-------|
| Lakera Guard | 95.22% | Closed-source API |
| AWS Bedrock Guardrails | 89.24% | Closed-source API |
| Azure Prompt Shields | 89.12% | Closed-source API |
| protectai/deberta-v3-base-v2 | 79.14% | Open, local |
| Llama Prompt Guard 2 86M | 78.76% | Gated, local |
| Google Model Armor | 70.07% | Closed-source API |
| Llama Prompt Guard 1 | 61.82% | Gated, local |
| **JailGuard** | *pending* | Open, local, Rust |

Source: [PINT leaderboard](https://github.com/lakeraai/pint-benchmark), 2025-05-02.

---

## Over-Defense Analysis

The [InjecGuard paper (arXiv:2410.22770)](https://arxiv.org/abs/2410.22770) measured
false-positive rates on 339 benign prompts containing injection-like words.
The shalyhinpavel holdout above is our independent equivalent.

| Model | FPR on hard-negative benign prompts | Source |
|-------|--------------------------------------|--------|
| Llama Prompt Guard 1 | ~99.1% | InjecGuard paper |
| deepset/deberta-v3-base | **72.6%** | shalyhinpavel holdout (this benchmark) |
| protectai/deberta-v3-base-v2 | 43.4% (paper) / **0.9%** (this benchmark) ‡ | InjecGuard paper / shalyhinpavel holdout |
| madhurjindal/Jailbreak-Detector | **23.6%** | shalyhinpavel holdout (this benchmark) |
| Lakera Guard | 12.4% | InjecGuard paper |
| **JailGuard** | **10.4%** | shalyhinpavel holdout (this benchmark) |

‡ The large discrepancy for protectai likely reflects different hard-negative corpora: the InjecGuard paper used 339 benign prompts with injection-like *words*; the shalyhinpavel holdout contains security architecture documents and red-team exercise descriptions. protectai-base appears to have been trained to recognize these document types as benign.

---

## Reproducing These Results

```sh
# Prerequisites
pip install transformers torch numpy
pip install "optimum[onnxruntime]"   # for ONNX models

# Set dataset location (jailguard_dataset repo)
export JAILGUARD_BENCH_DATA_DIR=/path/to/jailguard_dataset/data

# Run all open models, full datasets
python3 scripts/compare_models.py

# Run specific models with sample cap
python3 scripts/compare_models.py --models jailguard,protectai-base,deepset --limit 1000

# Gated models (requires HF account + license acceptance)
HF_TOKEN=hf_xxx python3 scripts/compare_models.py --models pg2-22m,pg2-86m
```

Output: `results/<model>_<dataset>.jsonl` (one prediction per line) and
`results/summary.md` (markdown table). JSONL schema is compatible with
`evaluation/external/comparison_runner.rs`.
