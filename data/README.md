# Training Data

This directory contains training data for the JailGuard prompt injection detection model.

## Overview

| File | Description | Size |
|------|-------------|------|
| `combined_injection_dataset.json` | Core training dataset | ~29 MB |
| `combined_minilm_embeddings.json` | Precomputed MiniLM embeddings | ~156 MB |
| `prompt_injections_real.json` | Real-world injection examples | ~152 KB |

## Dataset Composition

The core dataset contains 15,185 samples:
- **Benign prompts**: 13,567 samples (89.3%)
- **Injection attacks**: 1,618 samples (10.7%)

## Downloading Data

Data files are not included in the git repository due to size. To download:

```bash
# Option 1: Download pre-built datasets
python scripts/download_and_combine_datasets.py

# Option 2: Generate from scratch
python scripts/prepare_datasets.py
python scripts/embedding_pipeline.py
```

## Large Dataset (200K Balanced)

For training with the full 200K balanced dataset:

```bash
bash scripts/download_large_datasets.sh
```

This creates `splits_200k/` with:
- `train.json` - 70,000 training samples
- `val.json` - 27,500 validation samples
- `test.json` - 1,875 test samples

## Data Format

Each sample in JSON format:
```json
{
  "text": "The prompt text",
  "label": 0,
  "type": "benign",
  "embedding": [0.1, 0.2, ...]
}
```

Labels:
- `0` = Benign (safe prompt)
- `1` = Injection (attack attempt)

## Subdirectories

- `training/` - Train/val/test splits
- `baseline/` - Baseline evaluation metrics
- `expansion/` - Dataset expansion results
- `deepset/` - DeepSet prompt injection dataset
- `collected_samples/` - User-collected samples

## Citation

If you use this dataset, please cite:
```
@software{jailguard,
  title = {JailGuard: Prompt Injection Detection},
  url = {https://github.com/yourusername/jailguard}
}
```
