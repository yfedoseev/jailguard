#!/usr/bin/env python3
"""Benchmark AWS Bedrock Guardrails prompt-attack detection against our test datasets.

Produces JSONL output compatible with compare_models.py and compute_metrics.py.

Usage
-----
  # One-time: create a guardrail and print its ID
  python3 scripts/benchmark_bedrock.py --create-guardrail

  # Run the benchmark (pipeline dataset by default)
  JAILGUARD_BENCH_DATA_DIR=~/projects/jailguard_dataset/data \\
    python3 scripts/benchmark_bedrock.py \\
      --guardrail-id <id> --guardrail-version DRAFT

  # All three datasets, capped to 1000 samples each
  JAILGUARD_BENCH_DATA_DIR=... \\
    python3 scripts/benchmark_bedrock.py \\
      --guardrail-id <id> --guardrail-version DRAFT \\
      --datasets pipeline,j1n2,shalyhin --limit 1000

  # Estimate cost without running (dry-run)
  JAILGUARD_BENCH_DATA_DIR=... \\
    python3 scripts/benchmark_bedrock.py --guardrail-id <id> --dry-run

Prerequisites
-------------
  pip install boto3
  aws configure   (or use IAM role / environment credentials)

Pricing (as of Dec 2024)
------------------------
  Prompt attack detection: $0.15 per 1,000 text units
  1 text unit = up to 1,000 characters of input text
"""

from __future__ import annotations

import argparse
import json
import math
import os
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Shared data loading — mirrors compare_models.py
# ---------------------------------------------------------------------------

def get_data_dir() -> Path:
    raw = os.environ.get("JAILGUARD_BENCH_DATA_DIR", "").strip()
    if not raw:
        print(
            "Error: JAILGUARD_BENCH_DATA_DIR is not set.\n"
            "  export JAILGUARD_BENCH_DATA_DIR=/path/to/jailguard_dataset/data",
            file=sys.stderr,
        )
        sys.exit(1)
    return Path(raw).expanduser().resolve()


@dataclass
class Sample:
    id: str
    text: str
    label: bool


def _deterministic_shuffle(items: list) -> list:
    result = list(items)
    n = len(result)
    for i in range(n):
        j = (i * 17 + 42) % n
        result[i], result[j] = result[j], result[i]
    return result


def _balanced_cap(samples: list[Sample], limit: int) -> list[Sample]:
    inj = [s for s in samples if s.label]
    ben = [s for s in samples if not s.label]
    k = limit // 2
    return inj[:k] + ben[:k]


def load_pipeline(data_dir: Path, limit: Optional[int]) -> list[Sample]:
    path = data_dir / "pipeline_embeddings.json"
    print(f"  loading {path.name} …", end=" ", flush=True)
    with open(path) as f:
        raw = json.load(f)
    originals = [r for r in raw if not str(r.get("source", "")).endswith("_aug")]
    originals = _deterministic_shuffle(originals)
    test_start = int(len(originals) * 0.9)
    samples = [
        Sample(id=f"pipeline-{i:05d}", text=r["text"], label=bool(r["is_injection"]))
        for i, r in enumerate(originals[test_start:])
    ]
    if limit:
        samples = _balanced_cap(samples, limit)
    inj = sum(s.label for s in samples)
    print(f"{len(samples):,} samples  ({inj} inj / {len(samples)-inj} benign)")
    return samples


def load_j1n2(data_dir: Path, limit: Optional[int]) -> list[Sample]:
    path = data_dir / "external" / "j1n2.json"
    if not path.exists():
        print(f"  [skip] j1n2.json not found at {path}")
        return []
    with open(path) as f:
        raw = json.load(f)
    samples = [
        Sample(id=f"j1n2-{i:05d}", text=r["prompt"], label=bool(r["label"]))
        for i, r in enumerate(raw)
    ]
    if limit:
        samples = _balanced_cap(samples, limit)
    inj = sum(s.label for s in samples)
    print(f"  j1n2: {len(samples):,} samples  ({inj} inj / {len(samples)-inj} benign)")
    return samples


def load_shalyhinpavel(data_dir: Path, limit: Optional[int]) -> list[Sample]:
    path = data_dir / "external" / "shalyhinpavel_eval.json"
    if not path.exists():
        print(f"  [skip] shalyhinpavel_eval.json not found at {path}")
        return []
    with open(path) as f:
        raw = json.load(f)
    samples = [
        Sample(id=f"shalyhin-{i:05d}", text=r["text"], label=bool(r["label"] == 1))
        for i, r in enumerate(raw)
    ]
    if limit:
        samples = _balanced_cap(samples, limit)
    inj = sum(s.label for s in samples)
    print(f"  shalyhin: {len(samples):,} samples  ({inj} inj / {len(samples)-inj} benign)")
    return samples


# ---------------------------------------------------------------------------
# Cost estimation
# ---------------------------------------------------------------------------

PRICE_PER_1K_TEXT_UNITS = 0.15  # USD, prompt attack detection, Dec 2024


def estimate_cost(samples: list[Sample]) -> tuple[int, float]:
    """Return (total_text_units, estimated_usd)."""
    units = sum(math.ceil(len(s.text) / 1000) for s in samples)
    cost = (units / 1000) * PRICE_PER_1K_TEXT_UNITS
    return units, cost


# ---------------------------------------------------------------------------
# Guardrail creation
# ---------------------------------------------------------------------------

GUARDRAIL_NAME = "jailguard-benchmark"


def create_guardrail(client) -> str:
    """Create a guardrail with prompt attack detection enabled. Returns guardrail ID."""
    print("Creating Bedrock guardrail …")
    resp = client.create_guardrail(
        name=GUARDRAIL_NAME,
        description="Prompt injection detection for JailGuard benchmark",
        contentPolicyConfig={
            "filtersConfig": [
                {
                    "type": "PROMPT_ATTACK",
                    "inputStrength": "HIGH",
                    "outputStrength": "NONE",  # we only evaluate inputs
                }
            ]
        },
        blockedInputMessaging="BLOCKED",
        blockedOutputsMessaging="BLOCKED",
    )
    guardrail_id = resp["guardrailId"]
    print(f"  Created guardrail: {guardrail_id}  (version DRAFT)")
    print(f"  Re-use it with: --guardrail-id {guardrail_id} --guardrail-version DRAFT")
    return guardrail_id


# ---------------------------------------------------------------------------
# Single-sample inference
# ---------------------------------------------------------------------------

def call_guardrail(
    runtime_client,
    guardrail_id: str,
    guardrail_version: str,
    sample: Sample,
    max_retries: int = 5,
) -> tuple[bool, float, float]:
    """
    Returns (is_injection, score, latency_ms).
    score = 1.0 if GUARDRAIL_INTERVENED, else 0.0.
    Retries on ThrottlingException with exponential backoff.
    """
    delay = 1.0
    for attempt in range(max_retries):
        try:
            t0 = time.perf_counter()
            resp = runtime_client.apply_guardrail(
                guardrailIdentifier=guardrail_id,
                guardrailVersion=guardrail_version,
                source="INPUT",
                content=[
                    {
                        "text": {
                            "text": sample.text,
                            "qualifiers": ["query"],  # mark as user input for prompt injection check
                        }
                    }
                ],
            )
            latency_ms = (time.perf_counter() - t0) * 1000
            is_injection = resp.get("action") == "GUARDRAIL_INTERVENED"
            score = 1.0 if is_injection else 0.0
            return is_injection, score, latency_ms
        except Exception as e:
            name = type(e).__name__
            if "ThrottlingException" in name or "TooManyRequests" in name:
                if attempt < max_retries - 1:
                    time.sleep(delay)
                    delay = min(delay * 2, 30.0)
                    continue
            raise
    raise RuntimeError(f"Failed after {max_retries} retries for sample {sample.id}")


# ---------------------------------------------------------------------------
# Dataset runner
# ---------------------------------------------------------------------------

@dataclass
class Prediction:
    sample_id: str
    label: bool
    pred: bool
    score: float
    latency_ms: float


def run_dataset(
    runtime_client,
    guardrail_id: str,
    guardrail_version: str,
    samples: list[Sample],
    workers: int = 10,
) -> list[Prediction]:
    preds: list[Prediction] = [None] * len(samples)  # type: ignore
    done = 0

    def _infer(idx_sample):
        idx, s = idx_sample
        is_inj, score, lat = call_guardrail(runtime_client, guardrail_id, guardrail_version, s)
        return idx, Prediction(s.id, s.label, is_inj, score, lat)

    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = {pool.submit(_infer, (i, s)): i for i, s in enumerate(samples)}
        for fut in as_completed(futures):
            idx, pred = fut.result()
            preds[idx] = pred
            done += 1
            if done % 100 == 0 or done == len(samples):
                print(f"  {done}/{len(samples)}", end="\r", flush=True)

    print(f"  {len(samples)}/{len(samples)} done" + " " * 20)
    return preds  # type: ignore


# ---------------------------------------------------------------------------
# Metrics + output
# ---------------------------------------------------------------------------

def compute_and_print_metrics(preds: list[Prediction], dataset: str) -> None:
    tp = fp = tn = fn = 0
    lats = sorted(p.latency_ms for p in preds)
    for p in preds:
        if p.label and p.pred:       tp += 1
        elif not p.label and p.pred: fp += 1
        elif not p.label:            tn += 1
        else:                        fn += 1
    n = tp + fp + tn + fn
    acc  = (tp + tn) / n if n else 0
    prec = tp / (tp + fp) if (tp + fp) else 0
    rec  = tp / (tp + fn) if (tp + fn) else 0
    f1   = 2 * prec * rec / (prec + rec) if (prec + rec) else 0
    fpr  = fp / (fp + tn) if (fp + tn) else 0
    p50  = lats[int(len(lats) * 0.50)] if lats else 0
    p99  = lats[min(int(len(lats) * 0.99), len(lats) - 1)] if lats else 0
    print(
        f"  [{dataset}] acc={acc*100:.2f}%  prec={prec*100:.2f}%  "
        f"rec={rec*100:.2f}%  F1={f1:.3f}  FPR={fpr*100:.1f}%  "
        f"p50={p50:.1f}ms  p99={p99:.1f}ms"
    )
    print(f"  TP={tp}  FP={fp}  TN={tn}  FN={fn}")


def save_jsonl(preds: list[Prediction], dataset: str, out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / f"bedrock-guardrails_{dataset}.jsonl"
    with open(path, "w") as f:
        for p in preds:
            f.write(json.dumps({
                "id":         p.sample_id,
                "label":      int(p.label),
                "pred":       int(p.pred),
                "score":      round(p.score, 6),
                "latency_ms": round(p.latency_ms, 3),
                "model":      "bedrock-guardrails",
            }) + "\n")
    print(f"  Saved → {path}")


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    p.add_argument("--create-guardrail", action="store_true",
                   help="Create a new guardrail and print its ID, then exit.")
    p.add_argument("--guardrail-id", default=None,
                   help="Existing guardrail ID to use.")
    p.add_argument("--guardrail-version", default="DRAFT",
                   help="Guardrail version (default: DRAFT).")
    p.add_argument("--datasets", default="pipeline",
                   help="Comma-separated: pipeline, j1n2, shalyhin (default: pipeline)")
    p.add_argument("--limit", type=int, default=None,
                   help="Cap each dataset to N samples (balanced).")
    p.add_argument("--workers", type=int, default=10,
                   help="Parallel API calls (default: 10). Reduce if throttled.")
    p.add_argument("--region", default="us-east-1",
                   help="AWS region (default: us-east-1).")
    p.add_argument("--output-dir", default="results",
                   help="Output directory for JSONL files (default: results/).")
    p.add_argument("--dry-run", action="store_true",
                   help="Estimate cost only — do not call the API.")
    return p.parse_args()


def main() -> None:
    args = parse_args()

    try:
        import boto3
    except ImportError:
        print("Error: pip install boto3", file=sys.stderr)
        sys.exit(1)

    bedrock      = boto3.client("bedrock",         region_name=args.region)
    bedrock_rt   = boto3.client("bedrock-runtime",  region_name=args.region)

    # Create guardrail and exit
    if args.create_guardrail:
        gid = create_guardrail(bedrock)
        print(f"\nRun the benchmark with:\n"
              f"  python3 scripts/benchmark_bedrock.py "
              f"--guardrail-id {gid} --guardrail-version DRAFT")
        return

    if not args.guardrail_id:
        print("Error: --guardrail-id is required (or use --create-guardrail first).",
              file=sys.stderr)
        sys.exit(1)

    # Load datasets
    data_dir = get_data_dir()
    ds_keys = {k.strip() for k in args.datasets.split(",")}
    datasets: dict[str, list[Sample]] = {}
    print(f"\nData directory: {data_dir}\n")
    if "pipeline" in ds_keys:
        datasets["pipeline"] = load_pipeline(data_dir, args.limit)
    if "j1n2" in ds_keys:
        s = load_j1n2(data_dir, args.limit)
        if s: datasets["j1n2"] = s
    if "shalyhin" in ds_keys:
        s = load_shalyhinpavel(data_dir, args.limit)
        if s: datasets["shalyhin"] = s

    if not datasets:
        print("No datasets loaded.", file=sys.stderr)
        sys.exit(1)

    # Cost estimate
    print()
    total_units = 0
    total_cost = 0.0
    for name, samples in datasets.items():
        units, cost = estimate_cost(samples)
        total_units += units
        total_cost += cost
        print(f"  Cost estimate [{name}]: {units:,} text units → ${cost:.4f}")
    print(f"  Total estimated cost: ${total_cost:.4f}")

    if args.dry_run:
        print("\n  --dry-run: no API calls made.")
        return

    print()
    out_dir = Path(args.output_dir)

    for ds_name, samples in datasets.items():
        print(f"\n{'='*60}")
        print(f"  Bedrock Guardrails  —  {ds_name}  ({len(samples):,} samples)")
        print(f"  guardrail: {args.guardrail_id}  version: {args.guardrail_version}")
        print(f"  workers: {args.workers}")
        print(f"{'='*60}")

        preds = run_dataset(
            bedrock_rt, args.guardrail_id, args.guardrail_version,
            samples, workers=args.workers,
        )
        compute_and_print_metrics(preds, ds_name)
        save_jsonl(preds, ds_name, out_dir)

    print("\nDone. Run compute_metrics.py to compare with other models:")
    print("  python3 scripts/compute_metrics.py results/")


if __name__ == "__main__":
    main()
