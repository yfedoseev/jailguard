#!/usr/bin/env python3
"""Split training data into train (20) and test (5) sets."""

import json
from pathlib import Path
from random import shuffle

def split_datasets():
    """Split 500 samples into train (20) and test (5)."""
    print("Loading generated training data...\n")

    data_file = Path("data/training_data.json")
    if not data_file.exists():
        print("❌ Training data not found!")
        print("Run: python3 scripts/generate_training_data.py")
        return

    with open(data_file) as f:
        all_samples = json.load(f)

    print(f"Loaded {len(all_samples)} total samples")

    # Separate injections and benign
    injections = [s for s in all_samples if s["is_injection"]]
    benign = [s for s in all_samples if not s["is_injection"]]

    print(f"  Injections: {len(injections)}")
    print(f"  Benign: {len(benign)}\n")

    # Create training set (20 samples: ~8 injections, ~12 benign)
    num_train_injections = 8
    num_train_benign = 12

    train_injections = injections[:num_train_injections]
    train_benign = benign[:num_train_benign]
    train_samples = train_injections + train_benign
    shuffle(train_samples)

    # Create test set (5 samples: ~2 injections, ~3 benign from different data)
    num_test_injections = 2
    num_test_benign = 3

    test_injections = injections[num_train_injections:num_train_injections + num_test_injections]
    test_benign = benign[num_train_benign:num_train_benign + num_test_benign]
    test_samples = test_injections + test_benign
    shuffle(test_samples)

    # Save train set
    data_dir = Path("data")
    data_dir.mkdir(exist_ok=True)

    train_file = data_dir / "train_20.json"
    with open(train_file, "w") as f:
        json.dump(train_samples, f, indent=2)

    # Save test set
    test_file = data_dir / "test_5.json"
    with open(test_file, "w") as f:
        json.dump(test_samples, f, indent=2)

    print("=" * 60)
    print("TRAINING SET (20 samples)")
    print("=" * 60)
    print(f"Injections: {len(train_injections)}")
    print(f"Benign: {len(train_benign)}\n")
    for i, sample in enumerate(train_samples, 1):
        label = "INJ" if sample["is_injection"] else "BEN"
        print(f"  {i:2d}. [{label}] {sample['text'][:55]}")

    print("\n" + "=" * 60)
    print("TEST SET (5 samples - unseen, different from training)")
    print("=" * 60)
    print(f"Injections: {len(test_injections)}")
    print(f"Benign: {len(test_benign)}\n")
    for i, sample in enumerate(test_samples, 1):
        label = "INJ" if sample["is_injection"] else "BEN"
        print(f"  {i}. [{label}] {sample['text'][:55]}")

    print("\n" + "=" * 60)
    print("✅ Datasets created:")
    print(f"  Training: {train_file} ({len(train_samples)} samples)")
    print(f"  Testing:  {test_file} ({len(test_samples)} samples)")
    print("=" * 60)

if __name__ == "__main__":
    split_datasets()
