# Scripts

---

## Benchmarking

Three scripts form the benchmarking pipeline. Results land in `results/` as JSONL
(one prediction per line) compatible with each other and with `evaluation/external/comparison_runner.rs`.

### Prerequisites

```sh
pip install transformers torch numpy          # local model comparison
pip install "optimum[onnxruntime]"            # only needed for hlyn INT8 model
pip install boto3                             # only needed for Bedrock benchmark

# Point all scripts at the dataset repo
export JAILGUARD_BENCH_DATA_DIR=/path/to/jailguard_dataset/data
```

---

### 1. `compare_models.py` — local CPU model comparison

Runs JailGuard and up to 8 open HuggingFace classifiers on three datasets.

**Datasets used:**

| Key | File | Samples | Notes |
|-----|------|---------|-------|
| `pipeline` | `pipeline_embeddings.json` | 5,945 | JailGuard's own test split (last 10%, non-augmented) |
| `j1n2` | `external/j1n2.json` | 5,000 | Fully OOD — J1N2/mix-prompt-injection-dataset |
| `shalyhin` | `external/shalyhinpavel_eval.json` | 147 | Hard-negative holdout — tests over-defense |

**Run all open models, all datasets (full):**
```sh
JAILGUARD_BENCH_DATA_DIR=~/projects/jailguard_dataset/data \
  python3 scripts/compare_models.py
```

**Faster smoke-test (1,000 balanced samples per dataset):**
```sh
JAILGUARD_BENCH_DATA_DIR=... \
  python3 scripts/compare_models.py --limit 1000
```

**Specific models only:**
```sh
JAILGUARD_BENCH_DATA_DIR=... \
  python3 scripts/compare_models.py --models jailguard,protectai-base,deepset
```

**Gated Meta models (requires HuggingFace account + license acceptance):**
```sh
JAILGUARD_BENCH_DATA_DIR=... HF_TOKEN=hf_xxx \
  python3 scripts/compare_models.py --models pg2-22m,pg2-86m
```

**Available model keys:**

| Key | Model | Params | Gated |
|-----|-------|--------|-------|
| `jailguard` | This repo | 33M+130K | No |
| `protectai-base` | protectai/deberta-v3-base-prompt-injection-v2 | 184M | No |
| `deepset` | deepset/deberta-v3-base-injection | 184M | No |
| `madhur` | madhurjindal/Jailbreak-Detector-Large | ~280M | No |
| `protectai-small` | protectai/deberta-v3-small-prompt-injection-v2 | ~44M | Yes |
| `pg2-22m` | meta-llama/Llama-Prompt-Guard-2-22M | 22M | Yes |
| `pg2-86m` | meta-llama/Llama-Prompt-Guard-2-86M | 86M | Yes |
| `sentinel` | qualifire/prompt-injection-sentinel | 395M | Yes |
| `hlyn` | hlyn/prompt-injection-judge-deberta-70m | 70M INT8 | Yes |

**Output:** `results/<model>_<dataset>.jsonl` + `results/summary.md`

---

### 2. `benchmark_bedrock.py` — AWS Bedrock Guardrails

Calls the `ApplyGuardrail` API per sample. Produces JSONL in the same format.

**Cost:** ~$0.15 per 1,000 text units (1 text unit = up to 1,000 chars).
Full pipeline test set (5,945 samples) costs roughly **$0.89**.

**Step 1 — create the guardrail (one-time):**
```sh
python3 scripts/benchmark_bedrock.py --create-guardrail --region us-east-1
# Prints: Created guardrail: abc123xyz  (version DRAFT)
```

**Step 2 — dry-run to confirm cost before spending money:**
```sh
JAILGUARD_BENCH_DATA_DIR=... \
  python3 scripts/benchmark_bedrock.py \
    --guardrail-id abc123xyz --guardrail-version DRAFT \
    --datasets pipeline,j1n2,shalyhin --dry-run
```

**Step 3 — run the benchmark:**
```sh
JAILGUARD_BENCH_DATA_DIR=... \
  python3 scripts/benchmark_bedrock.py \
    --guardrail-id abc123xyz --guardrail-version DRAFT \
    --datasets pipeline,j1n2,shalyhin
```

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--guardrail-id` | — | Required. Guardrail ID from step 1. |
| `--guardrail-version` | `DRAFT` | Guardrail version string. |
| `--datasets` | `pipeline` | Comma-separated: `pipeline`, `j1n2`, `shalyhin`. |
| `--limit N` | — | Cap each dataset to N balanced samples. |
| `--workers N` | `10` | Parallel API calls. Reduce to 3–5 if throttled. |
| `--region` | `us-east-1` | AWS region. |
| `--dry-run` | — | Print cost estimate only, no API calls. |

**AWS credentials** must be configured (`aws configure` or IAM role).

**Output:** `results/bedrock-guardrails_<dataset>.jsonl`

---

### 3. `compute_metrics.py` — compute metrics from any JSONL results

Reads all `*.jsonl` files in a directory and prints an accuracy / latency table.
Works on the output of both `compare_models.py` and `benchmark_bedrock.py`.

```sh
# All results in results/
python3 scripts/compute_metrics.py results/

# Specific directory
python3 scripts/compute_metrics.py /path/to/results/
```

**Output format:** Markdown table grouped by dataset, with Acc / Prec / Recall / F1 / FPR / p50 / p90 / p99 per model.

---

### Full workflow example

```sh
export JAILGUARD_BENCH_DATA_DIR=~/projects/jailguard_dataset/data

# 1. Run local models
python3 scripts/compare_models.py --datasets pipeline,j1n2,shalyhin

# 2. Run Bedrock (needs AWS creds + guardrail ID)
python3 scripts/benchmark_bedrock.py \
  --guardrail-id abc123xyz --guardrail-version DRAFT \
  --datasets pipeline,j1n2,shalyhin

# 3. Print full comparison table
python3 scripts/compute_metrics.py results/
```

Results and analysis: see [`jailguard-datasets/BENCHMARKS.md`](https://github.com/yfedoseev/jailguard-datasets/blob/main/BENCHMARKS.md).

---

## Dataset Management

| Script | Description |
|--------|-------------|
| `download_and_combine_datasets.py` | Moved to sibling repo: `~/projects/jailguard_dataset/scripts/download_and_combine_datasets.py`. Downloads + normalises 20 HF datasets across 50 languages for multilingual training. See `jailguard_dataset/MULTILINGUAL.md`. |
| `prepare_datasets.py` | Prepare and preprocess datasets |
| `dataset_split.py` | Create train/val/test splits |
| `balanced_augmentation.py` | Balance dataset with augmentation |
| `generate_synthetic_dataset.py` | Generate synthetic training data |
| `generate_training_data.py` | Generate training data pipeline |
| `download_training_data.py` | Download pre-built training data |

## Embeddings & Processing

| Script | Description |
|--------|-------------|
| `embedding_pipeline.py` | Full embedding generation pipeline |
| `precompute_embeddings_minilm.py` | Generate MiniLM embeddings |
| `taxonomy_integration.py` | Integrate attack taxonomy |
| `unified_schema.py` | Unify dataset schemas |

## Evaluation & Conversion

| Script | Description |
|--------|-------------|
| `baseline_evaluation.py` | Evaluate baseline metrics |
| `convert_to_onnx.py` | Convert model to ONNX format |

## Setup

| Script | Description |
|--------|-------------|
| `setup-hooks.sh` | Install git pre-commit hooks |
| `download_large_datasets.sh` | Download large 200K dataset |

---

## Requirements

```sh
# Core (all benchmark scripts)
pip install numpy

# Local model comparison
pip install transformers torch
pip install "optimum[onnxruntime]"   # hlyn INT8 model only

# Bedrock benchmark
pip install boto3

# Dataset scripts
pip install numpy pandas torch transformers sentence-transformers
pip install onnx onnxruntime          # ONNX conversion
```
