#!/usr/bin/env python3
"""Compare JailGuard against all locally-runnable CPU prompt-injection classifiers.

Environment variables
---------------------
JAILGUARD_BENCH_DATA_DIR   Path to the data/ directory containing the benchmark
                           datasets (required).  Example:
                             export JAILGUARD_BENCH_DATA_DIR=~/projects/jailguard_dataset/data
HF_TOKEN                   HuggingFace token — required only for gated models
                           (meta-llama/Llama-Prompt-Guard-2-*).

Usage
-----
  # minimal — run all models, all three datasets
  JAILGUARD_BENCH_DATA_DIR=~/projects/jailguard_dataset/data python3 scripts/compare_models.py

  # fast smoke-test (500 samples per dataset)
  JAILGUARD_BENCH_DATA_DIR=... python3 scripts/compare_models.py --limit 500

  # specific models only
  JAILGUARD_BENCH_DATA_DIR=... python3 scripts/compare_models.py --models jailguard,protectai-small,pg2-22m

  # gated Meta models
  JAILGUARD_BENCH_DATA_DIR=... HF_TOKEN=hf_xxx python3 scripts/compare_models.py --models pg2-22m,pg2-86m

Datasets used (all from JAILGUARD_BENCH_DATA_DIR)
--------------------------------------------------
  pipeline_embeddings.json         — last 10% of non-augmented originals (5,945 samples)
                                     same deterministic split as the Rust benchmark binary
  external/j1n2.json               — 5,000-sample OOD mix (J1N2/mix-prompt-injection-dataset)
  external/shalyhinpavel_eval.json — 147-sample hard-negative holdout

Models
------
  jailguard        JailGuard (this repo) — built locally, called via subprocess
  protectai-base   protectai/deberta-v3-base-prompt-injection-v2   184 M
  protectai-small  protectai/deberta-v3-small-prompt-injection-v2   ~44 M
  deepset          deepset/deberta-v3-base-injection                184 M
  pg2-22m          meta-llama/Llama-Prompt-Guard-2-22M               22 M  *gated*
  pg2-86m          meta-llama/Llama-Prompt-Guard-2-86M               86 M  *gated*
  madhur           madhurjindal/Jailbreak-Detector-Large            280 M
  sentinel         qualifire/prompt-injection-sentinel              395 M  slow on CPU
  hlyn             hlyn/prompt-injection-judge-deberta-70m           70 M INT8 ONNX

Outputs
-------
  results/<model>_<dataset>.jsonl   per-sample predictions (comparison_runner.rs schema)
  results/summary.md                markdown table

Dependencies
------------
  pip install transformers torch numpy
  pip install "optimum[onnxruntime]"   # only needed for hlyn
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Configuration — datasets
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


# ---------------------------------------------------------------------------
# Sample type
# ---------------------------------------------------------------------------

@dataclass
class Sample:
    id: str
    text: str
    label: bool   # True = injection


# ---------------------------------------------------------------------------
# Dataset loaders — all paths derived from JAILGUARD_BENCH_DATA_DIR
# ---------------------------------------------------------------------------

def _deterministic_shuffle(items: list) -> list:
    """Exact port of the Rust shuffle in benchmark.rs: swap(i, (i*17+42)%n)."""
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
    if not path.exists():
        raise FileNotFoundError(
            f"pipeline_embeddings.json not found at {path}\n"
            "Run `cargo run --bin pipeline --release` in jailguard_dataset first."
        )
    print(f"  loading {path.name} …", end=" ", flush=True)
    with open(path) as f:
        raw = json.load(f)
    originals = [r for r in raw if not str(r.get("source", "")).endswith("_aug")]
    originals = _deterministic_shuffle(originals)
    test_start = int(len(originals) * 0.9)
    test_raw = originals[test_start:]
    samples = [
        Sample(id=f"pipeline-{i:05d}", text=r["text"], label=bool(r["is_injection"]))
        for i, r in enumerate(test_raw)
    ]
    if limit:
        samples = _balanced_cap(samples, limit)
    _report_counts("pipeline", samples)
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
    _report_counts("j1n2", samples)
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
    _report_counts("shalyhinpavel", samples)
    return samples


def _report_counts(name: str, samples: list[Sample]) -> None:
    inj = sum(s.label for s in samples)
    print(f"{name}: {len(samples):,} samples  ({inj} inj / {len(samples)-inj} benign)")


# ---------------------------------------------------------------------------
# Model configuration
# ---------------------------------------------------------------------------

@dataclass
class ModelConfig:
    key: str
    display: str
    params: str
    hf_id: Optional[str]              # None for jailguard (uses subprocess)
    injection_labels: list[str]        # label prefixes that map to positive class
    gated: bool = False
    onnx: bool = False                 # needs optimum[onnxruntime]
    slow_on_cpu: bool = False
    batch_size: int = 32


MODELS: list[ModelConfig] = [
    ModelConfig(
        key="jailguard",
        display="JailGuard (this repo)",
        params="33M+130K",
        hf_id=None,
        injection_labels=[],
    ),
    ModelConfig(
        key="protectai-base",
        display="protectai/deberta-v3-base-injection-v2",
        params="184M",
        hf_id="protectai/deberta-v3-base-prompt-injection-v2",
        injection_labels=["INJECTION"],
        batch_size=16,
    ),
    ModelConfig(
        key="protectai-small",
        display="protectai/deberta-v3-small-injection-v2",
        params="~44M",
        hf_id="protectai/deberta-v3-small-prompt-injection-v2",
        injection_labels=["INJECTION"],
        batch_size=32,
    ),
    ModelConfig(
        key="deepset",
        display="deepset/deberta-v3-base-injection",
        params="184M",
        hf_id="deepset/deberta-v3-base-injection",
        injection_labels=["INJECTION"],
        batch_size=16,
    ),
    ModelConfig(
        key="pg2-22m",
        display="Llama Prompt Guard 2 22M",
        params="22M",
        hf_id="meta-llama/Llama-Prompt-Guard-2-22M",
        injection_labels=["INJECTION", "JAILBREAK"],
        gated=True,
        batch_size=64,
    ),
    ModelConfig(
        key="pg2-86m",
        display="Llama Prompt Guard 2 86M",
        params="86M",
        hf_id="meta-llama/Llama-Prompt-Guard-2-86M",
        injection_labels=["INJECTION", "JAILBREAK"],
        gated=True,
        batch_size=32,
    ),
    ModelConfig(
        key="madhur",
        display="Jailbreak-Detector-Large",
        params="~280M",
        hf_id="madhurjindal/Jailbreak-Detector-Large",
        injection_labels=["JAILBREAK"],
        batch_size=16,
    ),
    ModelConfig(
        key="sentinel",
        display="qualifire/prompt-injection-sentinel",
        params="395M",
        hf_id="qualifire/prompt-injection-sentinel",
        injection_labels=["INJECTION", "JAILBREAK", "1"],
        slow_on_cpu=True,
        batch_size=8,
    ),
    ModelConfig(
        key="hlyn",
        display="hlyn deberta-70m INT8 ONNX",
        params="70M INT8",
        hf_id="hlyn/prompt-injection-judge-deberta-70m",
        injection_labels=["INJECTION", "1"],
        onnx=True,
        batch_size=32,
    ),
]

MODEL_BY_KEY: dict[str, ModelConfig] = {m.key: m for m in MODELS}


# ---------------------------------------------------------------------------
# Prediction type and metrics
# ---------------------------------------------------------------------------

@dataclass
class Prediction:
    sample_id: str
    label: bool
    pred: bool
    score: float
    latency_ms: float


@dataclass
class Metrics:
    accuracy: float
    precision: float
    recall: float
    f1: float
    fpr: float
    p50_ms: float
    p90_ms: float
    p99_ms: float
    n: int
    tp: int; fp: int; tn: int; fn_: int


def compute_metrics(preds: list[Prediction]) -> Metrics:
    tp = fp = tn = fn_ = 0
    lats: list[float] = []
    for p in preds:
        lats.append(p.latency_ms)
        if p.label and p.pred:       tp  += 1
        elif not p.label and p.pred: fp  += 1
        elif not p.label:            tn  += 1
        else:                        fn_ += 1
    total = tp + fp + tn + fn_
    acc   = (tp + tn) / total if total else 0.0
    prec  = tp / (tp + fp) if (tp + fp) else 0.0
    rec   = tp / (tp + fn_) if (tp + fn_) else 0.0
    f1    = 2 * prec * rec / (prec + rec) if (prec + rec) else 0.0
    fpr   = fp / (fp + tn) if (fp + tn) else 0.0
    lats.sort()
    def pct(p: float) -> float:
        if not lats: return 0.0
        return lats[min(int(len(lats) * p / 100), len(lats) - 1)]
    return Metrics(acc, prec, rec, f1, fpr, pct(50), pct(90), pct(99),
                   len(preds), tp, fp, tn, fn_)


# ---------------------------------------------------------------------------
# JailGuard runner — via compiled Rust binary
# ---------------------------------------------------------------------------

_REPO_ROOT = Path(__file__).resolve().parent.parent
_SCORER_BIN = _REPO_ROOT / "target" / "release" / "examples" / "score_jsonl"


def _build_scorer() -> bool:
    if _SCORER_BIN.exists():
        return True
    print("  Building JailGuard scorer (cargo build --release --example score_jsonl) …")
    result = subprocess.run(
        ["cargo", "build", "--release", "--example", "score_jsonl"],
        cwd=_REPO_ROOT,
        capture_output=False,
    )
    if result.returncode != 0:
        print("  [error] cargo build failed")
        return False
    print("  Build complete.")
    return True


def run_jailguard(samples: list[Sample]) -> list[Prediction]:
    if not _build_scorer():
        return []

    print(f"  Running {len(samples):,} samples through JailGuard …", end=" ", flush=True)
    input_lines = "\n".join(
        json.dumps({"id": s.id, "text": s.text, "label": int(s.label)})
        for s in samples
    )
    t0 = time.perf_counter()
    proc = subprocess.run(
        [str(_SCORER_BIN)],
        input=input_lines,
        capture_output=True,
        text=True,
    )
    elapsed = time.perf_counter() - t0
    if proc.returncode != 0:
        print(f"\n  [error] scorer exited {proc.returncode}: {proc.stderr[:200]}")
        return []

    preds: list[Prediction] = []
    for line in proc.stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        r = json.loads(line)
        preds.append(Prediction(
            sample_id=r["id"],
            label=r["label"] == 1,
            pred=r["pred"] == 1,
            score=float(r["score"]),
            latency_ms=float(r["latency_ms"]),
        ))
    print(f"{len(preds)} predictions in {elapsed:.1f}s")
    return preds


# ---------------------------------------------------------------------------
# Transformer runner
# ---------------------------------------------------------------------------

def _is_injection_label(label: str, injection_labels: list[str]) -> bool:
    upper = label.upper()
    return any(upper.startswith(il.upper()) for il in injection_labels)


def run_transformer_model(
    cfg: ModelConfig,
    samples: list[Sample],
    hf_token: Optional[str],
) -> list[Prediction]:
    try:
        from transformers import pipeline as hf_pipeline
    except ImportError:
        print("  [error] pip install transformers torch")
        return []

    token = hf_token if cfg.gated else None
    print(f"  Loading {cfg.hf_id} …", end=" ", flush=True)
    t_load = time.perf_counter()
    try:
        pipe = hf_pipeline(
            "text-classification",
            model=cfg.hf_id,
            tokenizer=cfg.hf_id,
            device=-1,        # CPU
            token=token,
            truncation=True,
            max_length=512,
        )
    except Exception as e:
        print(f"\n  [error] {e}")
        if cfg.gated:
            print(f"  Gated model — set HF_TOKEN and accept license at https://huggingface.co/{cfg.hf_id}")
        return []
    print(f"loaded in {time.perf_counter()-t_load:.1f}s")

    preds: list[Prediction] = []
    total = len(samples)
    for start in range(0, total, cfg.batch_size):
        batch = samples[start : start + cfg.batch_size]
        texts = [s.text for s in batch]
        t0 = time.perf_counter()
        try:
            results = pipe(texts, batch_size=cfg.batch_size)
        except Exception as e:
            print(f"\n  [error] batch {start}: {e}")
            for s in batch:
                preds.append(Prediction(s.id, s.label, False, 0.0, 0.0))
            continue
        per_ms = (time.perf_counter() - t0) * 1000 / len(batch)

        for s, r in zip(batch, results):
            is_inj = _is_injection_label(r["label"], cfg.injection_labels)
            raw_score = float(r["score"])
            score = raw_score if is_inj else 1.0 - raw_score
            preds.append(Prediction(s.id, s.label, score >= 0.5, score, per_ms))

        done = min(start + cfg.batch_size, total)
        print(f"  {done}/{total}  ({per_ms:.1f} ms/sample)    ", end="\r")

    print(f"  {len(preds)}/{total} done" + " " * 30)
    return preds


# ---------------------------------------------------------------------------
# ONNX runner (hlyn INT8 model)
# ---------------------------------------------------------------------------

def run_onnx_model(
    cfg: ModelConfig,
    samples: list[Sample],
    hf_token: Optional[str],
) -> list[Prediction]:
    try:
        from optimum.onnxruntime import ORTModelForSequenceClassification
        from transformers import AutoTokenizer
        import torch
    except ImportError:
        print("  [skip] pip install 'optimum[onnxruntime]'")
        return []

    print(f"  Loading {cfg.hf_id} (ONNX) …", end=" ", flush=True)
    t0 = time.perf_counter()
    try:
        tokenizer = AutoTokenizer.from_pretrained(cfg.hf_id, token=hf_token)
        model = ORTModelForSequenceClassification.from_pretrained(cfg.hf_id, token=hf_token)
    except Exception as e:
        print(f"\n  [error] {e}")
        return []
    id2label: dict = model.config.id2label
    print(f"loaded in {time.perf_counter()-t0:.1f}s  labels={list(id2label.values())}")

    preds: list[Prediction] = []
    for start in range(0, len(samples), cfg.batch_size):
        batch = samples[start : start + cfg.batch_size]
        enc = tokenizer(
            [s.text for s in batch],
            truncation=True, max_length=512, padding=True, return_tensors="pt",
        )
        t0 = time.perf_counter()
        with torch.no_grad():
            logits = model(**enc).logits
        probs = torch.softmax(logits, dim=-1).numpy()
        per_ms = (time.perf_counter() - t0) * 1000 / len(batch)

        inj_idx = next(
            (i for i, lbl in id2label.items() if _is_injection_label(str(lbl), cfg.injection_labels)),
            None,
        )
        for s, row in zip(batch, probs):
            score = float(row[inj_idx]) if inj_idx is not None else float(row.max())
            preds.append(Prediction(s.id, s.label, score >= 0.5, score, per_ms))

        done = min(start + cfg.batch_size, len(samples))
        print(f"  {done}/{len(samples)}", end="\r")

    print(f"  {len(preds)}/{len(samples)} done" + " " * 20)
    return preds


# ---------------------------------------------------------------------------
# Dispatch
# ---------------------------------------------------------------------------

def run_model(cfg: ModelConfig, samples: list[Sample], hf_token: Optional[str]) -> list[Prediction]:
    if not samples:
        return []
    if cfg.key == "jailguard":
        return run_jailguard(samples)
    if cfg.onnx:
        return run_onnx_model(cfg, samples, hf_token)
    return run_transformer_model(cfg, samples, hf_token)


# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------

def save_jsonl(preds: list[Prediction], model_key: str, dataset: str, out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    safe = dataset.replace(" ", "_").replace("/", "_")
    with open(out_dir / f"{model_key}_{safe}.jsonl", "w") as f:
        for p in preds:
            f.write(json.dumps({
                "id": p.sample_id,
                "label": int(p.label),
                "pred": int(p.pred),
                "score": round(p.score, 6),
                "latency_ms": round(p.latency_ms, 3),
                "model": model_key,
            }) + "\n")


@dataclass
class Row:
    model: str
    dataset: str
    m: Metrics


def format_table(rows: list[Row]) -> str:
    COL = [
        ("Model",   42, "l"),
        ("Dataset", 12, "l"),
        ("Acc",      7, "r"),
        ("Prec",     7, "r"),
        ("Recall",   7, "r"),
        ("F1",       6, "r"),
        ("FPR",      6, "r"),
        ("p50ms",    6, "r"),
        ("p99ms",    6, "r"),
        ("N",        6, "r"),
    ]

    def fmt_hdr(name, w, _): return f" {name:<{w}} " if _ == "l" else f" {name:>{w}} "
    def fmt_sep(_, w, __):   return "-" * (w + 2)

    header = "|" + "|".join(fmt_hdr(*c) for c in COL) + "|"
    sep    = "|" + "|".join(fmt_sep(*c) for c in COL) + "|"
    lines  = [header, sep]
    for r in rows:
        m = r.m
        vals = [
            r.model, r.dataset,
            f"{m.accuracy*100:.2f}%", f"{m.precision*100:.2f}%",
            f"{m.recall*100:.2f}%",   f"{m.f1:.3f}",
            f"{m.fpr*100:.1f}%",      f"{m.p50_ms:.1f}",
            f"{m.p99_ms:.1f}",        str(m.n),
        ]
        cells = []
        for (_, w, align), v in zip(COL, vals):
            cells.append(f" {v:<{w}} " if align == "l" else f" {v:>{w}} ")
        lines.append("|" + "|".join(cells) + "|")
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    p.add_argument("--models", default="all",
                   help=f"Comma-separated model keys or 'all'. Keys: {', '.join(MODEL_BY_KEY)}")
    p.add_argument("--datasets", default="pipeline,j1n2,shalyhin",
                   help="Comma-separated: pipeline, j1n2, shalyhin")
    p.add_argument("--limit", type=int, default=None,
                   help="Cap each dataset to N samples (balanced). Useful for slow models.")
    p.add_argument("--output-dir", default="results",
                   help="Output directory for JSONL + summary.md (default: results/)")
    p.add_argument("--rebuild", action="store_true",
                   help="Force rebuild of the JailGuard scorer binary.")
    return p.parse_args()


def main() -> None:
    args = parse_args()
    data_dir = get_data_dir()
    out_dir  = Path(args.output_dir)
    hf_token = os.environ.get("HF_TOKEN") or os.environ.get("HUGGINGFACE_TOKEN")

    # Resolve requested models
    if args.models.strip().lower() == "all":
        selected = MODELS
    else:
        keys = [k.strip() for k in args.models.split(",")]
        missing = [k for k in keys if k not in MODEL_BY_KEY]
        if missing:
            print(f"Unknown model keys: {missing}\nAvailable: {list(MODEL_BY_KEY)}", file=sys.stderr)
            sys.exit(1)
        selected = [MODEL_BY_KEY[k] for k in keys]

    # Optionally force rebuild
    if args.rebuild and _SCORER_BIN.exists():
        _SCORER_BIN.unlink()

    # Load datasets
    ds_keys = {k.strip() for k in args.datasets.split(",")}
    print(f"\nData directory: {data_dir}\n")
    datasets: dict[str, list[Sample]] = {}
    if "pipeline" in ds_keys:
        try:
            datasets["pipeline"] = load_pipeline(data_dir, args.limit)
        except FileNotFoundError as e:
            print(f"  [skip] {e}")
    if "j1n2" in ds_keys:
        s = load_j1n2(data_dir, args.limit)
        if s: datasets["j1n2"] = s
    if "shalyhin" in ds_keys:
        s = load_shalyhinpavel(data_dir, args.limit)
        if s: datasets["shalyhin"] = s

    if not datasets:
        print("No datasets loaded — check JAILGUARD_BENCH_DATA_DIR.", file=sys.stderr)
        sys.exit(1)

    # Run models
    all_rows: list[Row] = []
    for cfg in selected:
        print(f"\n{'='*60}")
        print(f"  {cfg.display}  ({cfg.params})")
        print(f"{'='*60}")

        if cfg.gated and not hf_token:
            print(f"  [skip] Gated — set HF_TOKEN=hf_xxx")
            print(f"  Accept license at: https://huggingface.co/{cfg.hf_id}")
            continue
        if cfg.slow_on_cpu and not args.limit:
            print(f"  [warn] {cfg.params} is slow on CPU. Consider --limit 200.")

        for ds_name, samples in datasets.items():
            print(f"\n  Dataset: {ds_name}  ({len(samples):,} samples)")
            preds = run_model(cfg, samples, hf_token)
            if not preds:
                continue
            m = compute_metrics(preds)
            save_jsonl(preds, cfg.key, ds_name, out_dir)
            all_rows.append(Row(cfg.display, ds_name, m))
            print(
                f"  acc={m.accuracy*100:.2f}%  prec={m.precision*100:.2f}%  "
                f"rec={m.recall*100:.2f}%  F1={m.f1:.3f}  "
                f"FPR={m.fpr*100:.1f}%  p50={m.p50_ms:.1f}ms  p99={m.p99_ms:.1f}ms"
            )

    # Print and save summary
    if not all_rows:
        print("\nNo results to display.")
        return

    table = format_table(all_rows)
    print(f"\n\n{'='*60}\n  RESULTS\n{'='*60}\n")
    print(table)

    out_dir.mkdir(parents=True, exist_ok=True)
    summary = out_dir / "summary.md"
    with open(summary, "w") as f:
        f.write("# Local CPU Model Comparison\n\n")
        f.write(table)
        f.write(
            "\n\n**FPR** = false-positive rate on benign samples (lower is better).\n"
            "**Pipeline** test set = last 10 % of non-augmented originals, "
            "same deterministic split as the Rust benchmark binary.\n"
        )
    print(f"\nSummary → {summary}")
    print(f"JSONL predictions → {out_dir}/")


if __name__ == "__main__":
    main()
