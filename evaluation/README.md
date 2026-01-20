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

| Metric | Value |
|--------|-------|
| Test Accuracy | 99.62% |
| Precision | 99.97% |
| Recall | 98.12% |
| F1 Score | 99.04% |
| Specificity | 99.99% |

## Requirements

- Pre-trained model in `models/`
- Test data in `data/` or `splits_200k/`
