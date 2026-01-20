# Utility Scripts

This directory contains Python and shell scripts for dataset management, embedding generation, and model conversion.

## Dataset Management

| Script | Description |
|--------|-------------|
| `download_and_combine_datasets.py` | Download datasets from multiple sources |
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

## Quick Start

### Download and prepare data
```bash
python scripts/download_and_combine_datasets.py
python scripts/prepare_datasets.py
```

### Generate embeddings
```bash
python scripts/embedding_pipeline.py
```

### Create balanced splits
```bash
python scripts/dataset_split.py \
  --input data/combined_injection_dataset.json \
  --output-dir data/training/ \
  --train-ratio 0.8 \
  --val-ratio 0.15 \
  --test-ratio 0.05
```

### Convert model to ONNX
```bash
python scripts/convert_to_onnx.py models/jailguard_injection_detector.json
```

## Requirements

Most scripts require:
```bash
pip install numpy pandas torch transformers sentence-transformers
```

For ONNX conversion:
```bash
pip install onnx onnxruntime
```
