#!/usr/bin/env python3
"""
Stratified Dataset Splitting

Creates train/val/test splits while preserving class distribution:
- Train: 70% (140K samples)
- Val: 15% (30K samples)
- Test: 15% (30K samples)

Ensures each split maintains the original attack type distribution.

Usage:
    python3 scripts/dataset_split.py --input embeddings.json --output splits/
    python3 scripts/dataset_split.py --input data.json --seed 42 --output-dir splits/
"""

import json
import sys
import random
from pathlib import Path
from typing import Dict, List, Tuple
from collections import defaultdict


# ============================================================================
# SPLIT CONFIGURATION
# ============================================================================

TRAIN_RATIO = 0.70
VAL_RATIO = 0.15
TEST_RATIO = 0.15


# ============================================================================
# STRATIFIED SPLITTING
# ============================================================================

class StratifiedSplitter:
    """Create stratified train/val/test splits."""

    def __init__(self, seed: int = 42):
        """Initialize splitter with random seed."""
        random.seed(seed)

    def analyze_distribution(self, samples: List[Dict]) -> Dict[str, int]:
        """Analyze attack type distribution."""
        dist = defaultdict(int)
        for sample in samples:
            atype = sample.get("attack_type", "JailbreakPattern")
            dist[atype] += 1
        return dict(dist)

    def split_samples(
        self,
        samples: List[Dict],
        train_ratio: float = TRAIN_RATIO,
        val_ratio: float = VAL_RATIO,
    ) -> Tuple[List[Dict], List[Dict], List[Dict], Dict]:
        """
        Create stratified train/val/test splits.

        Preserves attack type distribution in each split.

        Returns:
            (train_samples, val_samples, test_samples, split_stats)
        """
        print(f"\n📊 Creating stratified splits (train={train_ratio*100:.0f}%, val={val_ratio*100:.0f}%, test={(1-train_ratio-val_ratio)*100:.0f}%)...")

        # Separate by attack type
        samples_by_type = defaultdict(list)
        for sample in samples:
            atype = sample.get("attack_type", "JailbreakPattern")
            samples_by_type[atype].append(sample)

        print(f"\nDistribution by attack type:")
        for atype in sorted(samples_by_type.keys()):
            count = len(samples_by_type[atype])
            pct = count / len(samples) * 100
            print(f"  {atype:20} {count:>7} ({pct:>5.1f}%)")

        # Split each type independently
        train = []
        val = []
        test = []
        split_stats = {}

        for atype, atype_samples in sorted(samples_by_type.items()):
            count = len(atype_samples)

            # Shuffle
            random.shuffle(atype_samples)

            # Calculate split indices
            train_size = int(count * train_ratio)
            val_size = int(count * val_ratio)

            # Split
            train_split = atype_samples[:train_size]
            val_split = atype_samples[train_size:train_size + val_size]
            test_split = atype_samples[train_size + val_size:]

            # Track statistics
            split_stats[atype] = {
                "total": count,
                "train": len(train_split),
                "val": len(val_split),
                "test": len(test_split),
                "train_pct": len(train_split) / count * 100,
                "val_pct": len(val_split) / count * 100,
                "test_pct": len(test_split) / count * 100,
            }

            train.extend(train_split)
            val.extend(val_split)
            test.extend(test_split)

        # Shuffle final splits
        random.shuffle(train)
        random.shuffle(val)
        random.shuffle(test)

        # Verify splits preserve distribution
        train_dist = self.analyze_distribution(train)
        val_dist = self.analyze_distribution(val)
        test_dist = self.analyze_distribution(test)

        print(f"\n✓ Train split: {len(train):,} samples")
        for atype in sorted(train_dist.keys()):
            pct = train_dist[atype] / len(train) * 100
            print(f"  {atype:20} {train_dist[atype]:>7} ({pct:>5.1f}%)")

        print(f"\n✓ Val split: {len(val):,} samples")
        for atype in sorted(val_dist.keys()):
            pct = val_dist[atype] / len(val) * 100
            print(f"  {atype:20} {val_dist[atype]:>7} ({pct:>5.1f}%)")

        print(f"\n✓ Test split: {len(test):,} samples")
        for atype in sorted(test_dist.keys()):
            pct = test_dist[atype] / len(test) * 100
            print(f"  {atype:20} {test_dist[atype]:>7} ({pct:>5.1f}%)")

        return train, val, test, split_stats

    def compute_split_metrics(
        self,
        original_dist: Dict[str, int],
        split_dists: Dict[str, Dict[str, int]],
    ) -> Dict:
        """Compute metrics about split quality."""
        metrics = {}

        for split_name, split_dist in split_dists.items():
            split_total = sum(split_dist.values())
            metrics[split_name] = {
                "total": split_total,
                "distribution": split_dist,
            }

            # Check distribution preservation
            max_deviation = 0
            for atype, count in split_dist.items():
                expected_pct = original_dist.get(atype, 0) / sum(original_dist.values()) * 100
                actual_pct = count / split_total * 100
                deviation = abs(expected_pct - actual_pct)
                max_deviation = max(max_deviation, deviation)

            metrics[split_name]["max_distribution_deviation"] = max_deviation

        return metrics


# ============================================================================
# UTILITIES
# ============================================================================

def load_samples(filepath: Path) -> List[Dict]:
    """Load samples from JSON."""
    print(f"📖 Loading samples from {filepath}...")

    with open(filepath) as f:
        data = json.load(f)

    if isinstance(data, list):
        samples = data
    elif isinstance(data, dict) and "samples" in data:
        samples = data["samples"]
    else:
        raise ValueError(f"Unexpected format in {filepath}")

    print(f"✓ Loaded {len(samples):,} samples")
    return samples


def save_split(samples: List[Dict], output_path: Path, split_name: str):
    """Save split to JSON."""
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open(output_path, 'w') as f:
        json.dump(samples, f, indent=2)

    size_mb = output_path.stat().st_size / 1024 / 1024
    print(f"✓ Saved {split_name}: {output_path}")
    print(f"  {len(samples):,} samples ({size_mb:.1f}MB)")


def generate_split_report(
    train: List[Dict],
    val: List[Dict],
    test: List[Dict],
    split_stats: Dict,
    output_path: Path,
):
    """Generate detailed split report."""
    report = {
        "timestamp": str(Path(__file__).stat().st_mtime),
        "split_configuration": {
            "train_ratio": TRAIN_RATIO,
            "val_ratio": VAL_RATIO,
            "test_ratio": TEST_RATIO,
        },
        "split_sizes": {
            "train": len(train),
            "val": len(val),
            "test": len(test),
            "total": len(train) + len(val) + len(test),
        },
        "per_type_statistics": split_stats,
        "embedding_statistics": {
            "train": {
                "with_embedding": sum(1 for s in train if "embedding" in s),
                "embedding_dim": next((s.get("embedding_dim") for s in train if "embedding" in s), None),
            },
            "val": {
                "with_embedding": sum(1 for s in val if "embedding" in s),
                "embedding_dim": next((s.get("embedding_dim") for s in val if "embedding" in s), None),
            },
            "test": {
                "with_embedding": sum(1 for s in test if "embedding" in s),
                "embedding_dim": next((s.get("embedding_dim") for s in test if "embedding" in s), None),
            },
        },
    }

    report_path = output_path.parent / "split_report.json"
    with open(report_path, 'w') as f:
        json.dump(report, f, indent=2)

    print(f"\n✓ Split report saved: {report_path}")


# ============================================================================
# MAIN
# ============================================================================

def main():
    """Main execution."""
    import argparse

    parser = argparse.ArgumentParser(description="Create stratified train/val/test splits")
    parser.add_argument("--input", type=Path, required=True, help="Input samples JSON")
    parser.add_argument("--output", type=Path, required=True, help="Output directory for splits")
    parser.add_argument("--train-ratio", type=float, default=TRAIN_RATIO, help="Train ratio (default: 0.70)")
    parser.add_argument("--val-ratio", type=float, default=VAL_RATIO, help="Val ratio (default: 0.15)")
    parser.add_argument("--seed", type=int, default=42, help="Random seed")

    args = parser.parse_args()

    # Validate ratios
    if abs(args.train_ratio + args.val_ratio + (1 - args.train_ratio - args.val_ratio) - 1.0) > 0.01:
        print(f"❌ Error: Ratios must sum to 1.0 (got {args.train_ratio + args.val_ratio + (1-args.train_ratio-args.val_ratio):.2f})")
        return 1

    print("=" * 80)
    print("🚀 JailGuard Dataset Stratified Splitting")
    print("=" * 80)

    # Load samples
    samples = load_samples(args.input)

    # Create splits
    print(f"\n{'='*80}")
    print("STEP 1: Create Stratified Splits")
    print(f"{'='*80}")

    splitter = StratifiedSplitter(seed=args.seed)
    train, val, test, split_stats = splitter.split_samples(
        samples,
        train_ratio=args.train_ratio,
        val_ratio=args.val_ratio,
    )

    # Save splits
    print(f"\n{'='*80}")
    print("STEP 2: Save Splits")
    print(f"{'='*80}")

    args.output.mkdir(parents=True, exist_ok=True)

    save_split(train, args.output / "train.json", "Train")
    save_split(val, args.output / "val.json", "Val")
    save_split(test, args.output / "test.json", "Test")

    # Save combined file
    combined = {"train": train, "val": val, "test": test}
    combined_path = args.output / "combined.json"
    with open(combined_path, 'w') as f:
        json.dump(combined, f, indent=2)
    print(f"✓ Saved combined: {combined_path}")

    # Generate report
    print(f"\n{'='*80}")
    print("STEP 3: Generate Report")
    print(f"{'='*80}")

    generate_split_report(train, val, test, split_stats, args.output / "report.json")

    # Print statistics
    print(f"\n{'='*80}")
    print("📊 SPLIT STATISTICS")
    print(f"{'='*80}")

    total = len(train) + len(val) + len(test)
    print(f"\nOverall:")
    print(f"  Total: {total:,}")
    print(f"  Train: {len(train):,} ({len(train)/total*100:.1f}%)")
    print(f"  Val:   {len(val):,} ({len(val)/total*100:.1f}%)")
    print(f"  Test:  {len(test):,} ({len(test)/total*100:.1f}%)")

    # Check for embeddings
    train_with_emb = sum(1 for s in train if "embedding" in s)
    val_with_emb = sum(1 for s in val if "embedding" in s)
    test_with_emb = sum(1 for s in test if "embedding" in s)

    if train_with_emb > 0:
        print(f"\nEmbeddings:")
        print(f"  Train: {train_with_emb}/{len(train)} samples have embeddings ({train_with_emb/len(train)*100:.1f}%)")
        print(f"  Val:   {val_with_emb}/{len(val)} samples have embeddings ({val_with_emb/len(val)*100:.1f}%)")
        print(f"  Test:  {test_with_emb}/{len(test)} samples have embeddings ({test_with_emb/len(test)*100:.1f}%)")

    print(f"\n✅ Dataset splitting complete!")
    print(f"Output directory: {args.output}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
