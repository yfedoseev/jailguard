# Local CPU Model Comparison

| Model                                      | Dataset      |     Acc |    Prec |  Recall |     F1 |    FPR |  p50ms |  p99ms |      N |
|--------------------------------------------|--------------|---------|---------|---------|--------|--------|--------|--------|--------|
| Jailbreak-Detector-Large                   | pipeline     |  66.50% |  93.65% |  35.40% |  0.514 |   2.4% |   86.1 |  794.0 |   1000 |
| Jailbreak-Detector-Large                   | j1n2         |  84.80% |  97.54% |  71.40% |  0.824 |   1.8% |  417.7 |  611.9 |   1000 |
| Jailbreak-Detector-Large                   | shalyhin     |  68.71% |  44.44% |  48.78% |  0.465 |  23.6% |  168.1 |  242.7 |    147 |

**FPR** = false-positive rate on benign samples (lower is better).
**Pipeline** test set = last 10 % of non-augmented originals, same deterministic split as the Rust benchmark binary.
