#!/usr/bin/env python3
"""
Dataset Expansion Downloader

Downloads and integrates large public datasets for JailGuard expansion:
- SPML Chatbot Injection (16K samples)
- JailbreakBench (4.3K samples)
- Combines with existing TrustAIRLab/deepset (15K samples)

Target: 35K+ real samples → 200K balanced with augmentation

Usage:
    python3 scripts/download_expansion_datasets.py --all
    python3 scripts/download_expansion_datasets.py --spml --jailbreakbench
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import hashlib

try:
    import requests
except ImportError:
    print("Installing requests...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "requests"])
    import requests

from unified_schema import convert_to_canonical_format, TrainingSample


# ============================================================================
# DATASET CONFIGURATIONS
# ============================================================================

DATA_DIR = Path(__file__).parent.parent / "data" / "expansion"
CURRENT_DATA_PATH = Path(__file__).parent.parent / "data" / "combined_minilm_embeddings_with_types.json"

DATASETS = {
    "spml": {
        "name": "SPML Chatbot Injection",
        "url": "https://huggingface.co/datasets/qiaoyu-zheng/semantic-jailbreak/raw/main/data/semantic_jailbreak_jailbreak.jsonl",
        "local_path": DATA_DIR / "spml_raw.jsonl",
        "format": "jsonl",
        "expected_count": 16000,
        "description": "Large dataset with attack complexity scores",
    },
    "jailbreakbench": {
        "name": "JailbreakBench Behaviors",
        "url": "https://huggingface.co/datasets/JailbreakBench/JBB-Behaviors/raw/main/data.json",
        "local_path": DATA_DIR / "jailbreakbench_raw.json",
        "format": "json",
        "expected_count": 4300,
        "description": "Recent (2024) behavioral jailbreak dataset",
    },
}


# ============================================================================
# DOWNLOAD UTILITIES
# ============================================================================

def ensure_data_dir():
    """Create data directories if they don't exist."""
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    print(f"✓ Data directory: {DATA_DIR}")


def download_file(name: str, url: str, local_path: Path, timeout: int = 60) -> bool:
    """Download file from URL with retry logic."""
    if local_path.exists():
        size_mb = local_path.stat().st_size / 1024 / 1024
        print(f"✓ {name} already exists ({size_mb:.1f}MB)")
        return True

    print(f"⬇️  Downloading {name} ({url[:60]}...)")
    try:
        response = requests.get(url, timeout=timeout, stream=True)
        response.raise_for_status()

        # Save with progress
        total_size = int(response.headers.get('content-length', 0))
        with open(local_path, 'wb') as f:
            downloaded = 0
            for chunk in response.iter_content(chunk_size=8192):
                if chunk:
                    f.write(chunk)
                    downloaded += len(chunk)
                    if total_size > 0:
                        progress = (downloaded / total_size) * 100
                        if downloaded % (1024 * 1024) == 0:  # Print every 1MB
                            print(f"  ... {progress:.1f}% ({downloaded / 1024 / 1024:.1f}MB)")

        size_mb = local_path.stat().st_size / 1024 / 1024
        print(f"✓ Downloaded {name} ({size_mb:.1f}MB)")
        return True

    except Exception as e:
        print(f"✗ Failed to download {name}: {e}")
        return False


# ============================================================================
# DATASET LOADERS
# ============================================================================

def load_spml_dataset(filepath: Path) -> List[Dict]:
    """
    Load SPML Chatbot Injection dataset.

    Format: JSONL with fields: text, label, perplexity, etc.
    All samples are attack samples.
    """
    print(f"\n📖 Loading SPML dataset from {filepath}...")
    samples = []

    try:
        with open(filepath, 'r') as f:
            for line_num, line in enumerate(f, 1):
                if not line.strip():
                    continue

                try:
                    item = json.loads(line)
                    text = item.get("text") or item.get("prompt")
                    if not text:
                        continue

                    sample = {
                        "text": text.strip(),
                        "is_injection": True,
                        "source": "spml",
                        "metadata": {
                            "complexity": item.get("perplexity"),  # Complexity indicator
                            "confidence": item.get("score"),  # Attack strength
                        }
                    }
                    samples.append(sample)

                except json.JSONDecodeError as e:
                    if line_num <= 5:  # Only warn on first few lines
                        print(f"  ⚠️  Skipping malformed JSON at line {line_num}")
                    continue

                if line_num % 5000 == 0:
                    print(f"  ... Loaded {line_num} samples")

        print(f"✓ Loaded {len(samples)} samples from SPML")
        return samples

    except FileNotFoundError:
        print(f"✗ File not found: {filepath}")
        return []
    except Exception as e:
        print(f"✗ Error loading SPML: {e}")
        return []


def load_jailbreakbench_dataset(filepath: Path) -> List[Dict]:
    """
    Load JailbreakBench Behaviors dataset.

    Format: JSON with list of behavior objects.
    All samples are attack samples.
    """
    print(f"\n📖 Loading JailbreakBench dataset from {filepath}...")
    samples = []

    try:
        with open(filepath, 'r') as f:
            data = json.load(f)

        # Handle different format variations
        items = data
        if isinstance(data, dict):
            # Try common keys
            for key in ["behaviors", "data", "samples", "jailbreaks"]:
                if key in data:
                    items = data[key]
                    break

        if not isinstance(items, list):
            print(f"✗ Unexpected format: expected list, got {type(items)}")
            return samples

        for i, item in enumerate(items):
            text = None
            if isinstance(item, dict):
                # Try common keys for text
                for key in ["behavior", "prompt", "text", "jailbreak"]:
                    if key in item and isinstance(item[key], str):
                        text = item[key]
                        break

                # Category/category_name might indicate attack type
                category = item.get("category") or item.get("category_name", "")
            else:
                text = str(item)
                category = ""

            if text and len(text) > 10:
                sample = {
                    "text": text.strip(),
                    "is_injection": True,
                    "source": "jailbreakbench",
                    "metadata": {
                        "category": category,
                    }
                }
                samples.append(sample)

            if (i + 1) % 1000 == 0:
                print(f"  ... Loaded {i + 1} samples")

        print(f"✓ Loaded {len(samples)} samples from JailbreakBench")
        return samples

    except FileNotFoundError:
        print(f"✗ File not found: {filepath}")
        return []
    except Exception as e:
        print(f"✗ Error loading JailbreakBench: {e}")
        return []


def load_existing_data() -> List[Dict]:
    """Load existing combined dataset."""
    print(f"\n📖 Loading existing data from {CURRENT_DATA_PATH}...")

    try:
        with open(CURRENT_DATA_PATH, 'r') as f:
            data = json.load(f)

        if not isinstance(data, list):
            print(f"✗ Unexpected format in existing data")
            return []

        # Convert to canonical format if needed
        samples = []
        for sample in data:
            samples.append({
                "text": sample.get("text", ""),
                "is_injection": sample.get("is_injection", True),
                "source": sample.get("source", "deepset"),
                "attack_type": sample.get("attack_type"),
            })

        print(f"✓ Loaded {len(samples)} samples from existing data")
        return samples

    except FileNotFoundError:
        print(f"⚠️  Existing data not found")
        return []
    except Exception as e:
        print(f"✗ Error loading existing data: {e}")
        return []


# ============================================================================
# DEDUPLICATION & QUALITY FILTERING
# ============================================================================

def deduplicate_exact(samples: List[Dict]) -> Tuple[List[Dict], int]:
    """
    Remove exact duplicate samples (case-insensitive, whitespace-normalized).

    Returns: (unique_samples, duplicates_removed)
    """
    print(f"\n🔍 Running exact deduplication on {len(samples)} samples...")

    seen = {}
    unique = []
    duplicates = 0

    for sample in samples:
        # Normalize text
        normalized = sample.get("text", "").lower().strip()
        # Remove extra whitespace
        normalized = " ".join(normalized.split())

        if normalized not in seen:
            seen[normalized] = True
            unique.append(sample)
        else:
            duplicates += 1

    print(f"✓ Removed {duplicates} exact duplicates ({duplicates/len(samples)*100:.1f}%)")
    return unique, duplicates


def deduplicate_fuzzy(samples: List[Dict], threshold: float = 0.95) -> Tuple[List[Dict], int]:
    """
    Remove fuzzy duplicates (similar texts, high character-level similarity).

    Uses SequenceMatcher for O(n²) but high accuracy.
    """
    print(f"\n🔍 Running fuzzy deduplication (threshold={threshold})...")

    try:
        from difflib import SequenceMatcher
    except ImportError:
        print("⚠️  difflib not available, skipping fuzzy dedup")
        return samples, 0

    unique = []
    duplicates = 0

    for i, sample in enumerate(samples):
        text1 = sample.get("text", "").lower()

        # Check similarity with existing unique samples
        is_duplicate = False
        for unique_sample in unique:
            text2 = unique_sample.get("text", "").lower()
            similarity = SequenceMatcher(None, text1, text2).ratio()

            if similarity >= threshold:
                is_duplicate = True
                duplicates += 1
                break

        if not is_duplicate:
            unique.append(sample)

        if (i + 1) % 5000 == 0:
            print(f"  ... Processed {i + 1}/{len(samples)} samples")

    print(f"✓ Removed {duplicates} fuzzy duplicates ({duplicates/len(samples)*100:.1f}%)")
    return unique, duplicates


def quality_filter(samples: List[Dict]) -> Tuple[List[Dict], Dict]:
    """
    Apply quality filters to samples.

    Filters:
    1. Text length: 10-2000 characters
    2. No excessive whitespace
    3. Not mostly punctuation
    """
    print(f"\n🔍 Applying quality filters...")

    filtered = []
    stats = {
        "too_short": 0,
        "too_long": 0,
        "excessive_whitespace": 0,
        "mostly_punctuation": 0,
        "valid": 0,
    }

    for sample in samples:
        text = sample.get("text", "")

        # Check length
        if len(text) < 10:
            stats["too_short"] += 1
            continue
        if len(text) > 2000:
            stats["too_long"] += 1
            continue

        # Check whitespace ratio
        whitespace_ratio = sum(c.isspace() for c in text) / len(text)
        if whitespace_ratio > 0.5:
            stats["excessive_whitespace"] += 1
            continue

        # Check punctuation ratio
        punct_ratio = sum(c in "!?.,:;'\"" for c in text) / len(text)
        if punct_ratio > 0.8:
            stats["mostly_punctuation"] += 1
            continue

        stats["valid"] += 1
        filtered.append(sample)

    print(f"✓ Quality filtering results:")
    for reason, count in stats.items():
        pct = count / len(samples) * 100 if samples else 0
        print(f"  {reason}: {count} ({pct:.1f}%)")

    return filtered, stats


# ============================================================================
# COMBINATION & OUTPUT
# ============================================================================

def combine_datasets(
    existing: List[Dict],
    spml: List[Dict],
    jailbreakbench: List[Dict],
) -> Tuple[List[Dict], Dict]:
    """
    Combine all datasets with deduplication and quality filtering.

    Returns: (combined_samples, statistics)
    """
    print(f"\n{'='*80}")
    print("📊 COMBINING DATASETS")
    print(f"{'='*80}")

    # Combine
    all_samples = existing + spml + jailbreakbench
    print(f"\n✓ Combined {len(existing)} existing + {len(spml)} SPML + {len(jailbreakbench)} JailbreakBench")
    print(f"  Total: {len(all_samples)} samples")

    # Deduplicate exact
    unique, exact_dupes = deduplicate_exact(all_samples)

    # Deduplicate fuzzy (optional, slower)
    # unique, fuzzy_dupes = deduplicate_fuzzy(unique, threshold=0.95)

    # Quality filter
    filtered, quality_stats = quality_filter(unique)

    # Statistics
    stats = {
        "total_input": len(all_samples),
        "after_exact_dedup": len(unique),
        "exact_duplicates": exact_dupes,
        "after_quality_filter": len(filtered),
        "quality_stats": quality_stats,
        "by_source": {
            "existing": sum(1 for s in filtered if s.get("source") == "deepset"),
            "spml": sum(1 for s in filtered if s.get("source") == "spml"),
            "jailbreakbench": sum(1 for s in filtered if s.get("source") == "jailbreakbench"),
        },
        "injection_rate": sum(1 for s in filtered if s.get("is_injection", True)) / len(filtered) if filtered else 0,
    }

    return filtered, stats


def save_combined_dataset(samples: List[Dict], output_dir: Path):
    """Save combined dataset to JSON."""
    output_dir.mkdir(parents=True, exist_ok=True)

    output_file = output_dir / "expansion_combined_raw.json"
    with open(output_file, 'w') as f:
        json.dump(samples, f, indent=2)

    print(f"\n✓ Saved combined dataset: {output_file}")
    return output_file


def save_statistics(stats: Dict, output_dir: Path):
    """Save combination statistics."""
    stats_file = output_dir / "expansion_statistics.json"
    with open(stats_file, 'w') as f:
        json.dump(stats, f, indent=2)

    print(f"✓ Saved statistics: {stats_file}")


# ============================================================================
# MAIN
# ============================================================================

def main():
    """Main execution."""
    import argparse

    parser = argparse.ArgumentParser(description="Download and combine expansion datasets")
    parser.add_argument("--all", action="store_true", help="Download all datasets")
    parser.add_argument("--spml", action="store_true", help="Download SPML dataset")
    parser.add_argument("--jailbreakbench", action="store_true", help="Download JailbreakBench")
    parser.add_argument("--skip-download", action="store_true", help="Skip downloading")
    parser.add_argument("--output", type=Path, default=DATA_DIR, help="Output directory")

    args = parser.parse_args()

    print("=" * 80)
    print("🚀 JailGuard Dataset Expansion Downloader")
    print("=" * 80)

    ensure_data_dir()

    # Determine which datasets to download
    datasets_to_download = set()
    if args.all:
        datasets_to_download = {"spml", "jailbreakbench"}
    else:
        if args.spml:
            datasets_to_download.add("spml")
        if args.jailbreakbench:
            datasets_to_download.add("jailbreakbench")

    if not datasets_to_download and not args.skip_download:
        print("\nNo datasets specified. Use --all, --spml, or --jailbreakbench")
        return 1

    # Download datasets
    print(f"\n{'='*80}")
    print("STEP 1: Download Datasets")
    print(f"{'='*80}")

    for dataset_name in sorted(datasets_to_download):
        config = DATASETS[dataset_name]
        if not args.skip_download:
            download_file(config["name"], config["url"], config["local_path"])

    # Load datasets
    print(f"\n{'='*80}")
    print("STEP 2: Load Datasets")
    print(f"{'='*80}")

    existing = load_existing_data()
    spml = load_spml_dataset(DATASETS["spml"]["local_path"]) if "spml" in datasets_to_download else []
    jailbreakbench = load_jailbreakbench_dataset(DATASETS["jailbreakbench"]["local_path"]) if "jailbreakbench" in datasets_to_download else []

    # Combine & deduplicate
    combined, stats = combine_datasets(existing, spml, jailbreakbench)

    # Save outputs
    print(f"\n{'='*80}")
    print("STEP 3: Save Combined Dataset")
    print(f"{'='*80}")

    save_combined_dataset(combined, args.output)
    save_statistics(stats, args.output)

    # Print final statistics
    print(f"\n{'='*80}")
    print("📈 FINAL STATISTICS")
    print(f"{'='*80}")
    print(f"Total samples: {stats['total_input']:,}")
    print(f"After deduplication: {stats['after_exact_dedup']:,}")
    print(f"After quality filter: {stats['after_quality_filter']:,}")
    print(f"Injection rate: {stats['injection_rate']*100:.1f}%")
    print(f"\nBreakdown by source:")
    for source, count in stats["by_source"].items():
        pct = count / stats['after_quality_filter'] * 100 if stats['after_quality_filter'] > 0 else 0
        print(f"  {source:20} {count:>6} ({pct:>5.1f}%)")

    print(f"\n✅ Dataset expansion complete!")
    print(f"Output: {args.output}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
