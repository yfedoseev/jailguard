# Evaluation

Evaluation and benchmarking scripts for JailGuard models.

## Scripts

| Script | Description | Command |
|--------|-------------|---------|
| `evaluate_detector.rs` | Evaluate detector with detailed metrics | `cargo run --bin evaluate_detector --release` |
| `comprehensive_evaluation.rs` | Full evaluation framework | `cargo run --bin comprehensive_evaluation --release` |
| `phase_9_sota_validation.rs` | SOTA benchmark validation | `cargo run --bin phase_9_sota_validation --release` |
| `compare_embeddings.rs` | Compare hash vs semantic embeddings | `cargo run --bin compare_embeddings --release` |

## Quick Start

```bash
# Basic evaluation
cargo run --bin evaluate_detector --release

# Comprehensive metrics
cargo run --bin comprehensive_evaluation --release

# SOTA validation
cargo run --bin phase_9_sota_validation --release
```

## Metrics Computed

- **Accuracy**: Overall correctness
- **Precision**: True positives / (True positives + False positives)
- **Recall**: True positives / (True positives + False negatives)
- **F1 Score**: Harmonic mean of precision and recall
- **Specificity**: True negatives / (True negatives + False positives)
- **Confusion Matrix**: TP, FP, TN, FN breakdown

## Current Results

Authoritative benchmark numbers live in
[`jailguard_dataset/BENCHMARKS.md`](https://github.com/yfedoseev/jailguard_dataset/blob/main/BENCHMARKS.md).
Headline (re-validated 2026-05-03):

| Test set | Samples | Accuracy | Precision | Recall | F1 |
|----------|---------|----------|-----------|--------|-----|
| Pipeline (in-distribution) | 5,945 | 99.34% | 97.52% | 99.54% | 0.985 |
| J1N2 mix (OOD) | 5,000 | 99.38% | 98.09% | 99.94% | 0.990 |
| shalyhinpavel hard-negatives (OOD) | 147 | 89.12% | 76.60% | 87.80% | 0.818 |

## Requirements

- Pre-trained model in `models/`
- Test data in `data/` or `splits_200k/`
