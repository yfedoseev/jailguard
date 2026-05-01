#!/usr/bin/env python3
"""Compute accuracy/precision/recall/F1/FPR + latency percentiles from JSONL result files.

Usage:
    python3 scripts/compute_metrics.py [results_dir]

Prints a markdown summary to stdout.
"""

import json
import os
import sys
from pathlib import Path
from statistics import median, quantiles

RESULTS_DIR = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("results")


def load_jsonl(path):
    rows = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


def metrics(rows):
    tp = fp = tn = fn = 0
    latencies = []
    for r in rows:
        pred = int(r["pred"])
        label = int(r["label"])
        if pred == 1 and label == 1:
            tp += 1
        elif pred == 1 and label == 0:
            fp += 1
        elif pred == 0 and label == 0:
            tn += 1
        else:
            fn += 1
        if "latency_ms" in r:
            latencies.append(float(r["latency_ms"]))

    n = tp + fp + tn + fn
    acc = (tp + tn) / n if n else 0
    prec = tp / (tp + fp) if (tp + fp) else 0
    rec = tp / (tp + fn) if (tp + fn) else 0
    f1 = 2 * prec * rec / (prec + rec) if (prec + rec) else 0
    fpr = fp / (fp + tn) if (fp + tn) else 0

    p50 = p90 = p99 = None
    if latencies:
        latencies.sort()
        p50 = latencies[int(len(latencies) * 0.50)]
        p90 = latencies[int(len(latencies) * 0.90)]
        p99 = latencies[min(int(len(latencies) * 0.99), len(latencies) - 1)]
        sps = 1000.0 / p50 if p50 else None

    return {
        "n": n, "tp": tp, "fp": fp, "tn": tn, "fn": fn,
        "acc": acc, "prec": prec, "rec": rec, "f1": f1, "fpr": fpr,
        "p50": p50, "p90": p90, "p99": p99,
        "sps": 1000.0 / p50 if p50 else None,
    }


def pct(v):
    return f"{v*100:.2f}%"


def ms(v):
    return f"{v:.1f}" if v is not None else "—"


def main():
    files = sorted(RESULTS_DIR.glob("*.jsonl"))
    if not files:
        print(f"No JSONL files in {RESULTS_DIR}")
        return

    # group by dataset
    by_dataset = {}
    for f in files:
        stem = f.stem  # e.g. "protectai-base_j1n2"
        parts = stem.rsplit("_", 1)
        if len(parts) == 2:
            model, dataset = parts
        else:
            model, dataset = stem, "unknown"
        by_dataset.setdefault(dataset, {})[model] = f

    for dataset, model_files in sorted(by_dataset.items()):
        print(f"\n### {dataset}\n")
        print("| Model | N | Acc | Prec | Recall | F1 | FPR | TP | FP | TN | FN | p50ms | p90ms | p99ms |")
        print("|-------|---|-----|------|--------|----|-----|----|----|----|----|-------|-------|-------|")
        for model, path in sorted(model_files.items()):
            rows = load_jsonl(path)
            m = metrics(rows)
            print(
                f"| {model} | {m['n']} "
                f"| {pct(m['acc'])} | {pct(m['prec'])} | {pct(m['rec'])} "
                f"| {m['f1']:.3f} | {pct(m['fpr'])} "
                f"| {m['tp']} | {m['fp']} | {m['tn']} | {m['fn']} "
                f"| {ms(m['p50'])} | {ms(m['p90'])} | {ms(m['p99'])} |"
            )


if __name__ == "__main__":
    main()
