# Examples

Demo examples showing JailGuard's multi-layer detection capabilities.

## Available Examples

| Example | Description | Command |
|---------|-------------|---------|
| `ensemble_demo.rs` | Basic ensemble demonstration | `cargo run --example ensemble_demo --release` |
| `unified_api_ensemble_demo.rs` | Ensemble via unified API (96-98%) | `cargo run --example unified_api_ensemble_demo --release` |
| `full_pipeline.rs` | All 6 defense layers working together | `cargo run --example full_pipeline --release` |

## Quick Start

```bash
# Simple ensemble demo
cargo run --example ensemble_demo --release

# Full 6-layer pipeline
cargo run --example full_pipeline --release

# Unified API with ensemble
cargo run --example unified_api_ensemble_demo --release
```

## See Also

For production code, see:
- `train/` - Training scripts
- `inference/` - Inference & API server
- `evaluation/` - Benchmarking & metrics
