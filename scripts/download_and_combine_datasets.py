#!/usr/bin/env python3
"""
Download and combine MIT-licensed prompt injection datasets for training.

Datasets:
1. TrustAIRLab In-The-Wild: 15,140 samples
2. SPML Chatbot Injection: 16,012 samples

Total: ~31,352 samples (47x expansion from 662 baseline)
"""

import json
import os
from pathlib import Path
from datasets import load_dataset
import time

def download_trustailab():
    """Download TrustAIRLab In-The-Wild Jailbreak Prompts."""
    print("\n" + "="*70)
    print("📥 Downloading TrustAIRLab In-The-Wild Jailbreak Prompts")
    print("="*70)
    print("   Source: ACM CCS 2024 research")
    print("   Samples: 15,140 (1,405 jailbreak, 13,735 benign)")
    print("   License: MIT (commercial safe)")

    samples = []
    configs = ['jailbreak_2023_12_25', 'regular_2023_12_25']

    for config in configs:
        try:
            print(f"   Loading config: {config}...", end=" ")
            dataset = load_dataset("TrustAIRLab/in-the-wild-jailbreak-prompts", config, trust_remote_code=True)

            for split in dataset:
                for item in dataset[split]:
                    is_jailbreak = 'jailbreak' in config
                    samples.append({
                        "text": item.get("prompt", ""),
                        "is_injection": is_jailbreak,
                        "attack_type": "jailbreak" if is_jailbreak else "benign",
                        "source": "trustailab",
                    })
            print(f"✅ {len([s for s in samples if s['source']=='trustailab'])} samples")
        except Exception as e:
            print(f"⚠️  {e}")
            continue

    trustailab_samples = [s for s in samples if s['source']=='trustailab']
    print(f"✅ Downloaded {len(trustailab_samples)} total samples")
    return trustailab_samples

def download_spml():
    """Download SPML Chatbot Prompt Injection Dataset."""
    print("\n" + "="*70)
    print("📥 Downloading SPML Chatbot Prompt Injection Dataset")
    print("="*70)
    print("   Samples: 16,012 annotated examples")
    print("   Features: Attack complexity labels (0-10)")
    print("   License: MIT (commercial safe)")

    try:
        dataset = load_dataset("reshabhs/SPML_Chatbot_Prompt_Injection", trust_remote_code=True)
        samples = []

        for split in dataset:
            for item in dataset[split]:
                samples.append({
                    "text": item.get("input", ""),
                    "is_injection": item.get("is_injection", False),
                    "attack_type": "injection" if item.get("is_injection") else "benign",
                    "complexity": item.get("complexity", 0),
                    "source": "spml",
                })

        print(f"✅ Downloaded {len(samples)} samples")
        return samples
    except Exception as e:
        print(f"⚠️  Error downloading SPML: {e}")
        return []

def load_existing():
    """Load existing deepset/prompt-injections dataset."""
    print("\n" + "="*70)
    print("📥 Loading Existing deepset/prompt-injections Dataset")
    print("="*70)

    try:
        dataset = load_dataset("deepset/prompt-injections", trust_remote_code=True)
        samples = []

        for split in dataset:
            for item in dataset[split]:
                samples.append({
                    "text": item.get("text", ""),
                    "is_injection": item.get("label", 0) == 1,
                    "attack_type": "injection" if item.get("label", 0) == 1 else "benign",
                    "source": "deepset",
                })

        print(f"✅ Loaded {len(samples)} samples")
        return samples
    except Exception as e:
        print(f"⚠️  Error loading deepset: {e}")
        return []

def combine_datasets(datasets_list):
    """Combine multiple datasets."""
    print("\n" + "="*70)
    print("🔄 Combining Datasets")
    print("="*70)

    combined = []
    for samples in datasets_list:
        if samples:
            combined.extend(samples)

    # Remove exact duplicates (same text) but keep legitimate variants
    seen = set()
    unique = []
    duplicates = 0

    for sample in combined:
        text_key = sample["text"].strip()

        # Only skip if exact duplicate
        if text_key not in seen:
            seen.add(text_key)
            unique.append(sample)
        else:
            duplicates += 1

    print(f"   Total samples (before dedup): {len(combined)}")
    print(f"   Exact duplicates removed: {duplicates}")
    print(f"   Final unique samples: {len(unique)}")

    # Class distribution
    injections = sum(1 for s in unique if s["is_injection"])
    benign = len(unique) - injections

    print(f"   Injections: {injections} ({injections/len(unique)*100:.1f}%)")
    print(f"   Benign: {benign} ({benign/len(unique)*100:.1f}%)")

    # Filter out empty samples
    non_empty = [s for s in unique if s["text"].strip()]
    print(f"   Non-empty samples: {len(non_empty)}")

    return non_empty

def save_combined_dataset(samples, output_path):
    """Save combined dataset to JSON."""
    print("\n" + "="*70)
    print("💾 Saving Combined Dataset")
    print("="*70)

    output_dir = Path(output_path).parent
    output_dir.mkdir(parents=True, exist_ok=True)

    # Normalize format for training
    normalized = []
    for i, sample in enumerate(samples):
        normalized.append({
            "index": i,
            "text": sample["text"],
            "is_injection": sample["is_injection"],
            "attack_type": sample.get("attack_type", "unknown"),
            "source": sample.get("source", "unknown"),
            "embedding_dim": 384,  # Will be filled by embedding script
        })

    with open(output_path, 'w') as f:
        json.dump(normalized, f, indent=2)

    file_size_mb = os.path.getsize(output_path) / (1024 * 1024)
    print(f"✅ Saved to: {output_path}")
    print(f"   File size: {file_size_mb:.1f} MB")
    print(f"   Samples: {len(normalized)}")

def generate_statistics(samples):
    """Generate dataset statistics."""
    print("\n" + "="*70)
    print("📊 Dataset Statistics")
    print("="*70)

    by_source = {}
    for sample in samples:
        source = sample.get("source", "unknown")
        if source not in by_source:
            by_source[source] = {"total": 0, "injections": 0, "benign": 0}

        by_source[source]["total"] += 1
        if sample["is_injection"]:
            by_source[source]["injections"] += 1
        else:
            by_source[source]["benign"] += 1

    print("\nBy Source:")
    print(f"   {'Source':<20} {'Total':>8} {'Injections':>12} {'Benign':>8}")
    print("   " + "-"*52)
    for source, stats in sorted(by_source.items()):
        inj_pct = stats["injections"] / stats["total"] * 100 if stats["total"] > 0 else 0
        ben_pct = stats["benign"] / stats["total"] * 100 if stats["total"] > 0 else 0
        print(f"   {source:<20} {stats['total']:>8} {stats['injections']:>7} ({inj_pct:>5.1f}%) {stats['benign']:>7} ({ben_pct:>5.1f}%)")

    total = sum(s["total"] for s in by_source.values())
    total_inj = sum(s["injections"] for s in by_source.values())
    total_ben = total - total_inj

    print("   " + "-"*52)
    inj_pct = total_inj / total * 100 if total > 0 else 0
    print(f"   {'TOTAL':<20} {total:>8} {total_inj:>7} ({inj_pct:>5.1f}%) {total_ben:>7} ({100-inj_pct:>5.1f}%)")

def main():
    print("\n" + "╔" + "="*68 + "╗")
    print("║" + " "*68 + "║")
    print("║" + "  Download & Combine Prompt Injection Datasets".center(68) + "║")
    print("║" + "  Expanding Training Data for JailGuard SOTA".center(68) + "║")
    print("║" + " "*68 + "║")
    print("╚" + "="*68 + "╝\n")

    output_path = "data/combined_injection_dataset.json"

    print("📋 Available Datasets (MIT Licensed):")
    print("   1. deepset/prompt-injections (662 samples)")
    print("   2. TrustAIRLab (15,140 samples)")
    print("   3. SPML Chatbot (16,012 samples)")
    print("   Total: 31,352+ samples\n")

    start_time = time.time()

    # Download datasets
    datasets_to_combine = []

    # Load existing (baseline)
    existing = load_existing()
    if existing:
        datasets_to_combine.append(existing)

    # Download new datasets
    trustailab = download_trustailab()
    if trustailab:
        datasets_to_combine.append(trustailab)

    spml = download_spml()
    if spml:
        datasets_to_combine.append(spml)

    if not datasets_to_combine:
        print("\n❌ No datasets downloaded successfully")
        return

    # Combine
    combined = combine_datasets(datasets_to_combine)

    # Save
    save_combined_dataset(combined, output_path)

    # Statistics
    generate_statistics(combined)

    elapsed = time.time() - start_time
    print("\n" + "="*70)
    print(f"✅ Complete! Total time: {elapsed:.1f}s")
    print("="*70)
    print(f"\nNext steps:")
    print(f"1. Compute embeddings for combined dataset:")
    print(f"   python3 scripts/precompute_embeddings_minilm.py --data {output_path}")
    print(f"\n2. Retrain on expanded data:")
    print(f"   cargo run --example train_minilm_with_gradients --release")
    print(f"\n3. Compare results with baseline (78.9% accuracy)")

if __name__ == "__main__":
    main()
