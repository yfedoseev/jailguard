#!/usr/bin/env python3
"""
Phase 4: Dataset Preparation for JailGuard Accuracy Boost

Downloads and prepares datasets for fine-tuning:
- TrustAIRLab (15K samples) - recommended for quick training
- JailbreakBench (4.3K samples) - evaluation set
- LLMail-Inject (208K samples) - optional for maximum accuracy

Usage:
    python3 scripts/prepare_datasets.py --datasets trustairlab jailbreakbench
    python3 scripts/prepare_datasets.py --datasets all  # Download everything
"""

import json
import os
import random
import sys
from pathlib import Path
from typing import Dict, List, Tuple
from collections import defaultdict

try:
    import requests
except ImportError:
    print("Installing requests...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "requests"])
    import requests

# Configuration
DATA_DIR = Path(__file__).parent.parent / "data" / "training"
DATASETS = {
    "trustairlab": {
        "url": "https://huggingface.co/datasets/TrustAIRLab/in-the-wild-jailbreak-prompts/raw/main/data.json",
        "local_path": DATA_DIR / "trustairlab.json",
        "expected_size": 15000,
    },
    "jailbreakbench": {
        "url": "https://huggingface.co/datasets/JailbreakBench/JBB-Behaviors/raw/main/data.json",
        "local_path": DATA_DIR / "jailbreakbench.json",
        "expected_size": 4300,
    },
}

TRAIN_SPLIT = 0.6
VAL_SPLIT = 0.2
TEST_SPLIT = 0.2


def ensure_data_dir():
    """Create data directory if it doesn't exist."""
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    print(f"✓ Data directory: {DATA_DIR}")


def download_dataset(name: str, url: str, local_path: Path) -> bool:
    """Download dataset from HuggingFace."""
    if local_path.exists():
        print(f"✓ {name} already downloaded ({local_path.stat().st_size / 1024 / 1024:.1f}MB)")
        return True

    print(f"⬇️  Downloading {name}...")
    try:
        response = requests.get(url, timeout=30)
        response.raise_for_status()
        with open(local_path, "w") as f:
            f.write(response.text)
        print(f"✓ Downloaded {name} ({local_path.stat().st_size / 1024 / 1024:.1f}MB)")
        return True
    except Exception as e:
        print(f"✗ Failed to download {name}: {e}")
        return False


def load_trustairlab(path: Path) -> List[Dict]:
    """Load TrustAIRLab dataset."""
    print("Loading TrustAIRLab...")
    try:
        with open(path) as f:
            data = json.load(f)

        # TrustAIRLab format: list of jailbreak prompts
        samples = []
        for item in data:
            if isinstance(item, dict):
                text = item.get("text", item.get("prompt", ""))
                if text:
                    samples.append({
                        "text": text,
                        "is_injection": True,  # All are jailbreaks
                        "source": "trustairlab",
                    })
            elif isinstance(item, str):
                samples.append({
                    "text": item,
                    "is_injection": True,
                    "source": "trustairlab",
                })

        print(f"✓ Loaded {len(samples)} samples from TrustAIRLab")
        return samples
    except Exception as e:
        print(f"✗ Error loading TrustAIRLab: {e}")
        return []


def load_jailbreakbench(path: Path) -> List[Dict]:
    """Load JailbreakBench dataset."""
    print("Loading JailbreakBench...")
    try:
        with open(path) as f:
            data = json.load(f)

        samples = []
        if isinstance(data, list):
            for item in data:
                if isinstance(item, dict):
                    text = item.get("behavior", item.get("prompt", item.get("text", "")))
                    if text:
                        samples.append({
                            "text": text,
                            "is_injection": True,  # All are jailbreak behaviors
                            "source": "jailbreakbench",
                        })
        elif isinstance(data, dict):
            # Handle nested structure
            for key in ["behaviors", "prompts", "data"]:
                if key in data:
                    items = data[key]
                    if isinstance(items, list):
                        for item in items:
                            if isinstance(item, dict):
                                text = item.get("prompt", item.get("text", ""))
                            else:
                                text = str(item)
                            if text:
                                samples.append({
                                    "text": text,
                                    "is_injection": True,
                                    "source": "jailbreakbench",
                                })

        print(f"✓ Loaded {len(samples)} samples from JailbreakBench")
        return samples
    except Exception as e:
        print(f"✗ Error loading JailbreakBench: {e}")
        return []


def load_benign_examples() -> List[Dict]:
    """Create benign examples for balance."""
    benign_examples = [
        "What is the capital of France?",
        "How do I bake a chocolate cake?",
        "Explain the theory of relativity",
        "What is the best programming language?",
        "How do I learn machine learning?",
        "What is the weather like today?",
        "Can you help me with Python?",
        "What is the history of AI?",
        "How do I make pasta?",
        "Explain quantum computing",
        "What are the best books to read?",
        "How do I learn Spanish?",
        "What is the COVID-19 vaccine?",
        "How do I lose weight?",
        "What is blockchain?",
        "How do I invest in stocks?",
        "What is the meaning of life?",
        "How do I meditate?",
        "What is the best way to study?",
        "How do I travel to Japan?",
    ]

    return [
        {
            "text": text,
            "is_injection": False,
            "source": "benign_examples",
        }
        for text in benign_examples
    ]


def combine_datasets(injection_samples: List[Dict], benign_samples: List[Dict]) -> List[Dict]:
    """Combine injection and benign samples."""
    all_samples = injection_samples + benign_samples

    # Remove duplicates (case-insensitive)
    seen = set()
    unique_samples = []
    for sample in all_samples:
        key = sample["text"].lower().strip()
        if key not in seen:
            seen.add(key)
            unique_samples.append(sample)

    print(f"\n📊 Combined Dataset Statistics:")
    print(f"   Total samples: {len(unique_samples)}")

    # Count by type
    injections = sum(1 for s in unique_samples if s["is_injection"])
    benign = len(unique_samples) - injections
    print(f"   Injections: {injections} ({injections / len(unique_samples) * 100:.1f}%)")
    print(f"   Benign: {benign} ({benign / len(unique_samples) * 100:.1f}%)")

    return unique_samples


def stratified_split(
    samples: List[Dict], train_ratio: float = 0.6, val_ratio: float = 0.2
) -> Tuple[List[Dict], List[Dict], List[Dict]]:
    """
    Create stratified train/val/test splits preserving class distribution.
    """
    # Separate by class
    injections = [s for s in samples if s["is_injection"]]
    benign = [s for s in samples if not s["is_injection"]]

    # Shuffle independently
    random.shuffle(injections)
    random.shuffle(benign)

    # Split each class
    inj_train_size = int(len(injections) * train_ratio)
    inj_val_size = int(len(injections) * val_ratio)

    ben_train_size = int(len(benign) * train_ratio)
    ben_val_size = int(len(benign) * val_ratio)

    train = (
        injections[:inj_train_size] +
        benign[:ben_train_size]
    )
    val = (
        injections[inj_train_size:inj_train_size + inj_val_size] +
        benign[ben_train_size:ben_train_size + ben_val_size]
    )
    test = (
        injections[inj_train_size + inj_val_size:] +
        benign[ben_train_size + ben_val_size:]
    )

    # Shuffle final splits
    random.shuffle(train)
    random.shuffle(val)
    random.shuffle(test)

    return train, val, test


def save_splits(train: List[Dict], val: List[Dict], test: List[Dict]):
    """Save train/val/test splits to JSON files."""
    output_dir = DATA_DIR / "splits"
    output_dir.mkdir(parents=True, exist_ok=True)

    # Save individual splits
    for name, data in [("train", train), ("val", val), ("test", test)]:
        path = output_dir / f"{name}.json"
        with open(path, "w") as f:
            json.dump(data, f, indent=2)
        print(f"✓ Saved {name} set: {path} ({len(data)} samples)")

    # Save combined file for reference
    combined_path = output_dir / "combined.json"
    with open(combined_path, "w") as f:
        json.dump({"train": train, "val": val, "test": test}, f, indent=2)

    # Print statistics
    print(f"\n📈 Split Statistics:")
    for name, data in [("Train", train), ("Val", val), ("Test", test)]:
        injections = sum(1 for s in data if s["is_injection"])
        benign = len(data) - injections
        print(f"   {name}: {len(data)} samples ({injections} inj, {benign} benign)")
        if len(data) > 0:
            print(f"         {injections/len(data)*100:.1f}% injections, {benign/len(data)*100:.1f}% benign")


def main():
    """Main execution."""
    print("=" * 70)
    print("JailGuard Phase 4: Dataset Preparation")
    print("=" * 70)

    ensure_data_dir()

    # Determine which datasets to download
    import argparse
    parser = argparse.ArgumentParser(description="Prepare datasets for JailGuard training")
    parser.add_argument(
        "--datasets",
        nargs="+",
        default=["trustairlab", "jailbreakbench"],
        help="Datasets to download: trustairlab, jailbreakbench, or 'all'",
    )
    parser.add_argument("--seed", type=int, default=42, help="Random seed for splits")
    parser.add_argument("--skip-download", action="store_true", help="Skip downloading datasets")
    args = parser.parse_args()

    random.seed(args.seed)

    # Download datasets
    print("\n" + "=" * 70)
    print("STEP 1: Download Datasets")
    print("=" * 70)

    datasets_to_use = []
    if "all" in args.datasets:
        datasets_to_use = list(DATASETS.keys())
    else:
        datasets_to_use = [d for d in args.datasets if d in DATASETS]

    for dataset_name in datasets_to_use:
        config = DATASETS[dataset_name]
        if not args.skip_download:
            download_dataset(dataset_name, config["url"], config["local_path"])

    # Load datasets
    print("\n" + "=" * 70)
    print("STEP 2: Load and Combine Datasets")
    print("=" * 70)

    all_injection_samples = []
    for dataset_name in datasets_to_use:
        config = DATASETS[dataset_name]
        if config["local_path"].exists():
            if dataset_name == "trustairlab":
                samples = load_trustairlab(config["local_path"])
            elif dataset_name == "jailbreakbench":
                samples = load_jailbreakbench(config["local_path"])
            else:
                samples = []

            all_injection_samples.extend(samples)

    # Add benign examples
    benign_samples = load_benign_examples()
    print(f"✓ Created {len(benign_samples)} benign examples")

    # Combine all samples
    combined_samples = combine_datasets(all_injection_samples, benign_samples)

    # Create stratified splits
    print("\n" + "=" * 70)
    print("STEP 3: Create Stratified Train/Val/Test Splits")
    print("=" * 70)

    train, val, test = stratified_split(
        combined_samples,
        train_ratio=TRAIN_SPLIT,
        val_ratio=VAL_SPLIT,
    )

    # Save splits
    print("\n" + "=" * 70)
    print("STEP 4: Save Splits")
    print("=" * 70)

    save_splits(train, val, test)

    print("\n" + "=" * 70)
    print("✅ Phase 4 Complete: Dataset Preparation")
    print("=" * 70)
    print(f"\n📁 Output: {DATA_DIR / 'splits'}")
    print(f"\nNext steps:")
    print(f"  1. Review splits: {DATA_DIR / 'splits'}")
    print(f"  2. Run Phase 5: Fine-tune models with 'cargo run --example train_fine_tuned'")


if __name__ == "__main__":
    main()
