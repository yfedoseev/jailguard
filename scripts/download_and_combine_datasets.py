#!/usr/bin/env python3
"""
Download and combine public prompt injection datasets for training.

Integrates multiple sources with unified taxonomy, 3-tier deduplication,
quality filtering, balancing to 200K, and canonical JSON schema output.

Datasets (Injection/Jailbreak):
1. deepset/prompt-injections (662 samples)
2. TrustAIRLab In-The-Wild (15,140 samples)
3. SPML Chatbot Injection (16,012 samples)
4. JailbreakBench Behaviors (200 samples)
5. JailbreakV-28K (28,000 jailbreak queries)
6. jackhhao/jailbreak-classification (527 jailbreak)
7. rubend18/ChatGPT-Jailbreak-Prompts (79 prompts)
8. lmsys/toxic-chat jailbreaking subset (~226 samples)

Datasets (Benign):
9. databricks/databricks-dolly-15k (15,011 samples)
10. OpenAssistant/oasst1 EN prompter messages (15,200 samples)
11. tatsu-lab/alpaca (52,002 instructions)
12. PKU-Alignment/BeaverTails safe prompts (~16K unique)
13. lmsys/toxic-chat clean subset (~9,400 samples)
14. Anthropic/hh-rlhf human prompts (up to 160K)

Total available: ~44K attacks + ~275K benign
Output: 200K balanced (100K attacks oversampled + 100K benign subsampled)

Usage:
    python3 scripts/download_and_combine_datasets.py
    python3 scripts/download_and_combine_datasets.py --output data/combined_200k.json
    python3 scripts/download_and_combine_datasets.py --skip-fuzzy --target-size 100000
"""

import json
import os
import sys
import time
import random
import hashlib
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from collections import defaultdict
from difflib import SequenceMatcher

# Add scripts dir to path for local imports
sys.path.insert(0, str(Path(__file__).parent))
from unified_schema import (
    ATTACK_TYPE_TO_IDX,
    infer_attack_type,
)

# ============================================================================
# CONFIGURATION
# ============================================================================

OUTPUT_PATH = Path(__file__).parent.parent / "data" / "combined_200k.json"

# Quality filter thresholds
MIN_TEXT_LENGTH = 10
MAX_TEXT_LENGTH = 10000
MAX_WHITESPACE_RATIO = 0.5
MAX_PUNCTUATION_RATIO = 0.8

# Dedup thresholds
FUZZY_THRESHOLD = 0.95
FUZZY_SAMPLE_LIMIT = 50000

# Balancing
DEFAULT_TARGET_SIZE = 200000
RANDOM_SEED = 42


# ============================================================================
# DATASET DOWNLOADERS - ATTACK SOURCES
# ============================================================================

def download_deepset() -> List[Dict]:
    """Load deepset/prompt-injections dataset (662 samples)."""
    print("\n  [1/14] deepset/prompt-injections")

    try:
        from datasets import load_dataset
        dataset = load_dataset("deepset/prompt-injections")
        samples = []

        for split in dataset:
            for item in dataset[split]:
                text = item.get("text", "")
                is_injection = item.get("label", 0) == 1

                attack_type = "Benign"
                if is_injection:
                    attack_type = infer_attack_type(text)

                samples.append({
                    "text": text,
                    "is_injection": is_injection,
                    "attack_type": attack_type,
                    "attack_type_idx": ATTACK_TYPE_TO_IDX.get(attack_type, 6),
                    "source": "deepset",
                })

        print(f"         {len(samples)} samples")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_trustailab() -> List[Dict]:
    """Download TrustAIRLab In-The-Wild Jailbreak Prompts (15,140 samples)."""
    print("\n  [2/14] TrustAIRLab In-The-Wild Jailbreak Prompts")

    samples = []
    configs = ['jailbreak_2023_12_25', 'regular_2023_12_25']

    try:
        from datasets import load_dataset

        for config in configs:
            try:
                dataset = load_dataset(
                    "TrustAIRLab/in-the-wild-jailbreak-prompts",
                    config,
                )

                count = 0
                for split in dataset:
                    for item in dataset[split]:
                        text = item.get("prompt", "")
                        is_jailbreak = 'jailbreak' in config

                        attack_type = "Benign"
                        if is_jailbreak:
                            attack_type = infer_attack_type(text)

                        samples.append({
                            "text": text,
                            "is_injection": is_jailbreak,
                            "attack_type": attack_type,
                            "attack_type_idx": ATTACK_TYPE_TO_IDX.get(attack_type, 6),
                            "source": "trustairlab",
                        })
                        count += 1

                print(f"         {config}: {count} samples")

            except Exception as e:
                print(f"         {config}: Error: {e}")
                continue

    except ImportError:
        print("         datasets library not available")

    print(f"         Total: {len(samples)} samples")
    return samples


def download_spml() -> List[Dict]:
    """Download SPML Chatbot Prompt Injection Dataset (16,012 samples)."""
    print("\n  [3/14] SPML Chatbot Prompt Injection")

    try:
        from datasets import load_dataset
        dataset = load_dataset("reshabhs/SPML_Chatbot_Prompt_Injection")
        samples = []

        for split in dataset:
            for item in dataset[split]:
                text = item.get("User Prompt", "") or ""
                is_injection = bool(item.get("Prompt injection", 0))
                complexity = item.get("Degree", 0)

                attack_type = "Benign"
                if is_injection:
                    attack_type = infer_attack_type(text)

                samples.append({
                    "text": text,
                    "is_injection": is_injection,
                    "attack_type": attack_type,
                    "attack_type_idx": ATTACK_TYPE_TO_IDX.get(attack_type, 6),
                    "source": "spml",
                    "metadata": {"complexity": complexity},
                })

        print(f"         {len(samples)} samples")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_jailbreakbench() -> List[Dict]:
    """Download JailbreakBench Behaviors dataset (200 samples)."""
    print("\n  [4/14] JailbreakBench Behaviors")

    try:
        from datasets import load_dataset
        dataset = load_dataset("JailbreakBench/JBB-Behaviors", "behaviors")
        samples = []

        for split in dataset:
            is_harmful = split == "harmful"
            for item in dataset[split]:
                text = item.get("Goal", "")
                if not text or len(text) < MIN_TEXT_LENGTH:
                    continue

                category = item.get("Category", "")

                if is_harmful:
                    attack_type = infer_attack_type(text)
                else:
                    attack_type = "Benign"

                samples.append({
                    "text": text.strip(),
                    "is_injection": is_harmful,
                    "attack_type": attack_type,
                    "attack_type_idx": ATTACK_TYPE_TO_IDX.get(attack_type, 6),
                    "source": "jailbreakbench",
                    "metadata": {"category": category},
                })

        print(f"         {len(samples)} samples")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_jailbreakv_28k() -> List[Dict]:
    """Download JailbreakV-28K jailbreak queries (28,000 samples)."""
    print("\n  [5/14] JailbreakV-28K")

    try:
        from datasets import load_dataset
        dataset = load_dataset(
            "JailbreakV-28K/JailBreakV-28k",
            "JailBreakV_28K",
            split="JailBreakV_28K",
        )
        samples = []

        for item in dataset:
            text = item.get("jailbreak_query", "")
            if not text or len(text.strip()) < MIN_TEXT_LENGTH:
                continue

            fmt = item.get("format", "")
            policy = item.get("policy", "")

            attack_type = infer_attack_type(text)

            samples.append({
                "text": text.strip(),
                "is_injection": True,
                "attack_type": attack_type,
                "attack_type_idx": ATTACK_TYPE_TO_IDX.get(attack_type, 6),
                "source": "jailbreakv28k",
                "metadata": {"format": fmt, "policy": policy},
            })

        print(f"         {len(samples)} samples")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_jackhhao_jailbreak() -> List[Dict]:
    """Download jackhhao/jailbreak-classification (1,044 samples)."""
    print("\n  [6/14] jackhhao/jailbreak-classification")

    try:
        from datasets import load_dataset
        dataset = load_dataset("jackhhao/jailbreak-classification", split="train")
        samples = []

        for item in dataset:
            text = item.get("prompt", "")
            if not text or len(text.strip()) < MIN_TEXT_LENGTH:
                continue

            is_jailbreak = item.get("type", "") == "jailbreak"

            attack_type = "Benign"
            if is_jailbreak:
                attack_type = infer_attack_type(text)

            samples.append({
                "text": text.strip(),
                "is_injection": is_jailbreak,
                "attack_type": attack_type,
                "attack_type_idx": ATTACK_TYPE_TO_IDX.get(attack_type, 6),
                "source": "jackhhao",
            })

        print(f"         {len(samples)} samples")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_rubend18_jailbreak() -> List[Dict]:
    """Download rubend18/ChatGPT-Jailbreak-Prompts (79 prompts)."""
    print("\n  [7/14] rubend18/ChatGPT-Jailbreak-Prompts")

    try:
        from datasets import load_dataset
        dataset = load_dataset("rubend18/ChatGPT-Jailbreak-Prompts", split="train")
        samples = []

        for item in dataset:
            text = item.get("Prompt", "")
            if not text or len(text.strip()) < MIN_TEXT_LENGTH:
                continue

            attack_type = infer_attack_type(text)

            samples.append({
                "text": text.strip(),
                "is_injection": True,
                "attack_type": attack_type,
                "attack_type_idx": ATTACK_TYPE_TO_IDX.get(attack_type, 6),
                "source": "rubend18",
            })

        print(f"         {len(samples)} samples")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_toxic_chat() -> List[Dict]:
    """Download lmsys/toxic-chat (10K samples, mixed)."""
    print("\n  [8/14] lmsys/toxic-chat")

    try:
        from datasets import load_dataset
        samples = []

        for split_name in ["train", "test"]:
            dataset = load_dataset("lmsys/toxic-chat", "toxicchat0124", split=split_name)

            for item in dataset:
                text = item.get("user_input", "")
                if not text or len(text.strip()) < MIN_TEXT_LENGTH:
                    continue

                is_jailbreaking = bool(item.get("jailbreaking", 0))

                if is_jailbreaking:
                    attack_type = infer_attack_type(text)
                else:
                    attack_type = "Benign"

                samples.append({
                    "text": text.strip(),
                    "is_injection": is_jailbreaking,
                    "attack_type": attack_type,
                    "attack_type_idx": ATTACK_TYPE_TO_IDX.get(attack_type, 6),
                    "source": "toxic_chat",
                })

        inj = sum(1 for s in samples if s["is_injection"])
        print(f"         {len(samples)} samples ({inj} jailbreaking, {len(samples) - inj} benign)")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


# ============================================================================
# DATASET DOWNLOADERS - BENIGN SOURCES
# ============================================================================

def download_dolly() -> List[Dict]:
    """Download databricks/databricks-dolly-15k (15,011 benign samples)."""
    print("\n  [9/14] databricks/databricks-dolly-15k")

    try:
        from datasets import load_dataset
        dataset = load_dataset("databricks/databricks-dolly-15k", split="train")
        samples = []

        for item in dataset:
            text = item.get("instruction", "")
            context = item.get("context", "")
            # Combine instruction + context for richer text
            if context and len(context.strip()) > 0:
                full_text = f"{text}\n\nContext: {context}"
            else:
                full_text = text

            if not full_text or len(full_text.strip()) < MIN_TEXT_LENGTH:
                continue

            samples.append({
                "text": full_text.strip(),
                "is_injection": False,
                "attack_type": "Benign",
                "attack_type_idx": 0,
                "source": "dolly",
                "metadata": {"category": item.get("category", "")},
            })

        print(f"         {len(samples)} samples")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_openassistant() -> List[Dict]:
    """Download OpenAssistant/oasst1 EN prompter messages (15,200 samples)."""
    print("\n  [10/14] OpenAssistant/oasst1")

    try:
        from datasets import load_dataset
        samples = []

        for split_name in ["train", "validation"]:
            dataset = load_dataset("OpenAssistant/oasst1", split=split_name)

            for item in dataset:
                # Only take initial user prompts in English
                if item.get("role") != "prompter":
                    continue
                if item.get("lang") != "en":
                    continue
                # Only root messages (no replies)
                if item.get("parent_id") is not None:
                    continue

                text = item.get("text", "")
                if not text or len(text.strip()) < MIN_TEXT_LENGTH:
                    continue

                samples.append({
                    "text": text.strip(),
                    "is_injection": False,
                    "attack_type": "Benign",
                    "attack_type_idx": 0,
                    "source": "openassistant",
                })

        print(f"         {len(samples)} samples")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_alpaca() -> List[Dict]:
    """Download tatsu-lab/alpaca (52,002 benign instructions)."""
    print("\n  [11/14] tatsu-lab/alpaca")

    try:
        from datasets import load_dataset
        dataset = load_dataset("tatsu-lab/alpaca", split="train")
        samples = []

        for item in dataset:
            text = item.get("instruction", "")
            inp = item.get("input", "")
            if inp and len(inp.strip()) > 0:
                full_text = f"{text}\n\nInput: {inp}"
            else:
                full_text = text

            if not full_text or len(full_text.strip()) < MIN_TEXT_LENGTH:
                continue

            samples.append({
                "text": full_text.strip(),
                "is_injection": False,
                "attack_type": "Benign",
                "attack_type_idx": 0,
                "source": "alpaca",
            })

        print(f"         {len(samples)} samples")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_beavertails() -> List[Dict]:
    """Download PKU-Alignment/BeaverTails unique prompts as benign.

    These are harmful-content requests but NOT prompt injection.
    Teaching the model that harmful content != injection is valuable.
    """
    print("\n  [12/14] PKU-Alignment/BeaverTails (unique prompts)")

    try:
        from datasets import load_dataset
        dataset = load_dataset("PKU-Alignment/BeaverTails", split="30k_train")
        samples = []
        seen_prompts = set()

        for item in dataset:
            text = item.get("prompt", "")
            if not text or len(text.strip()) < MIN_TEXT_LENGTH:
                continue

            # Deduplicate prompts (many rows share the same prompt)
            normalized = text.strip().lower()
            if normalized in seen_prompts:
                continue
            seen_prompts.add(normalized)

            # These are harmful queries but NOT injection/jailbreak techniques.
            # Label as benign from injection perspective.
            samples.append({
                "text": text.strip(),
                "is_injection": False,
                "attack_type": "Benign",
                "attack_type_idx": 0,
                "source": "beavertails",
            })

        print(f"         {len(samples)} unique prompts")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_hh_rlhf() -> List[Dict]:
    """Extract human prompts from Anthropic/hh-rlhf (up to 160K)."""
    print("\n  [13/14] Anthropic/hh-rlhf (human prompts)")

    try:
        from datasets import load_dataset
        dataset = load_dataset("Anthropic/hh-rlhf", split="train")
        samples = []
        seen = set()

        for item in dataset:
            chosen = item.get("chosen", "")
            # Extract first human turn
            if "\n\nHuman:" not in chosen:
                continue

            parts = chosen.split("\n\nHuman:")
            if len(parts) < 2:
                continue

            text = parts[1].split("\n\nAssistant:")[0].strip()
            if not text or len(text) < MIN_TEXT_LENGTH:
                continue

            # Deduplicate
            normalized = text.lower().strip()
            if normalized in seen:
                continue
            seen.add(normalized)

            samples.append({
                "text": text,
                "is_injection": False,
                "attack_type": "Benign",
                "attack_type_idx": 0,
                "source": "hh_rlhf",
            })

        print(f"         {len(samples)} unique prompts")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


def download_easyjailbreak() -> List[Dict]:
    """Download EasyJailbreak datasets (AdvBench, ForbiddenQuestion, etc).

    These are harmful queries (NOT injection techniques).
    Labeled as benign from injection perspective.
    """
    print("\n  [14/14] Lemhf14/EasyJailbreak_Datasets")

    try:
        from datasets import load_dataset
        samples = []

        for config in ["AdvBench", "ForbiddenQuestion", "MaliciousInstruct", "QuestionList"]:
            try:
                dataset = load_dataset(
                    "Lemhf14/EasyJailbreak_Datasets",
                    config,
                    split="train",
                )

                count = 0
                for item in dataset:
                    text = item.get("query", "")
                    if not text or len(text.strip()) < MIN_TEXT_LENGTH:
                        continue

                    # Harmful queries but NOT injection technique
                    samples.append({
                        "text": text.strip(),
                        "is_injection": False,
                        "attack_type": "Benign",
                        "attack_type_idx": 0,
                        "source": f"easyjailbreak_{config.lower()}",
                    })
                    count += 1

                print(f"         {config}: {count} samples")

            except Exception as e:
                print(f"         {config}: Error: {e}")

        print(f"         Total: {len(samples)} samples")
        return samples

    except Exception as e:
        print(f"         Error: {e}")
        return []


# ============================================================================
# 3-TIER DEDUPLICATION
# ============================================================================

def normalize_text(text: str) -> str:
    """Normalize text for comparison: lowercase, strip, collapse whitespace."""
    return " ".join(text.lower().strip().split())


def tier1_exact_dedup(samples: List[Dict]) -> Tuple[List[Dict], int]:
    """Tier 1: Exact match deduplication (hash-based, O(n))."""
    print("\n  Tier 1: Exact match deduplication...")

    seen = set()
    unique = []
    removed = 0

    for sample in samples:
        key = normalize_text(sample.get("text", ""))
        text_hash = hashlib.md5(key.encode()).hexdigest()

        if text_hash not in seen:
            seen.add(text_hash)
            unique.append(sample)
        else:
            removed += 1

    print(f"    Removed {removed} exact duplicates ({removed / len(samples) * 100:.1f}%)")
    return unique, removed


def tier2_fuzzy_dedup(
    samples: List[Dict], threshold: float = FUZZY_THRESHOLD
) -> Tuple[List[Dict], int]:
    """Tier 2: Fuzzy match deduplication (SequenceMatcher, O(n^2))."""
    print(f"\n  Tier 2: Fuzzy deduplication (threshold={threshold})...")

    if len(samples) > FUZZY_SAMPLE_LIMIT:
        print(f"    Dataset too large ({len(samples)}) for O(n^2) fuzzy dedup.")
        print(f"    Running on first {FUZZY_SAMPLE_LIMIT} samples only.")
        head = samples[:FUZZY_SAMPLE_LIMIT]
        tail = samples[FUZZY_SAMPLE_LIMIT:]
    else:
        head = samples
        tail = []

    unique = []
    removed = 0
    unique_texts = []

    for i, sample in enumerate(head):
        text = normalize_text(sample.get("text", ""))

        is_dup = False
        window = unique_texts[-200:]
        for existing_text in window:
            ratio = SequenceMatcher(None, text, existing_text).ratio()
            if ratio >= threshold:
                is_dup = True
                removed += 1
                break

        if not is_dup:
            unique.append(sample)
            unique_texts.append(text)

        if (i + 1) % 10000 == 0:
            print(f"    ... Processed {i + 1}/{len(head)} samples")

    unique.extend(tail)
    print(f"    Removed {removed} fuzzy duplicates ({removed / len(head) * 100:.1f}%)")
    return unique, removed


def run_deduplication(
    samples: List[Dict], skip_fuzzy: bool = False
) -> Tuple[List[Dict], Dict]:
    """Run deduplication pipeline."""
    print("\n" + "=" * 70)
    print("Running Deduplication")
    print("=" * 70)

    stats = {"initial_count": len(samples)}

    samples, exact_removed = tier1_exact_dedup(samples)
    stats["tier1_exact_removed"] = exact_removed
    stats["after_tier1"] = len(samples)

    if not skip_fuzzy:
        samples, fuzzy_removed = tier2_fuzzy_dedup(samples)
        stats["tier2_fuzzy_removed"] = fuzzy_removed
    else:
        print("\n  Tier 2: Fuzzy deduplication (skipped)")
        stats["tier2_fuzzy_removed"] = 0
    stats["after_tier2"] = len(samples)

    total_removed = stats["initial_count"] - len(samples)
    stats["total_removed"] = total_removed
    stats["dedup_rate"] = total_removed / stats["initial_count"] * 100 if stats["initial_count"] > 0 else 0

    print(f"\n  Total removed: {total_removed} ({stats['dedup_rate']:.1f}%)")
    print(f"  Final count: {len(samples)}")

    return samples, stats


# ============================================================================
# QUALITY FILTERING
# ============================================================================

def quality_filter(samples: List[Dict]) -> Tuple[List[Dict], Dict]:
    """Apply quality filters to remove low-quality samples."""
    print("\n" + "=" * 70)
    print("Quality Filtering")
    print("=" * 70)

    filtered = []
    stats = {
        "too_short": 0,
        "too_long": 0,
        "excessive_whitespace": 0,
        "mostly_punctuation": 0,
        "empty_text": 0,
        "valid": 0,
    }

    for sample in samples:
        text = sample.get("text", "")

        if not text or not text.strip():
            stats["empty_text"] += 1
            continue

        if len(text) < MIN_TEXT_LENGTH:
            stats["too_short"] += 1
            continue

        if len(text) > MAX_TEXT_LENGTH:
            stats["too_long"] += 1
            continue

        whitespace_ratio = sum(c.isspace() for c in text) / len(text)
        if whitespace_ratio > MAX_WHITESPACE_RATIO:
            stats["excessive_whitespace"] += 1
            continue

        punct_ratio = sum(c in "!?.,:;'\"" for c in text) / len(text)
        if punct_ratio > MAX_PUNCTUATION_RATIO:
            stats["mostly_punctuation"] += 1
            continue

        stats["valid"] += 1
        filtered.append(sample)

    print(f"  Results:")
    for reason, count in stats.items():
        pct = count / len(samples) * 100 if samples else 0
        print(f"    {reason:25} {count:>6} ({pct:>5.1f}%)")

    return filtered, stats


# ============================================================================
# BALANCING
# ============================================================================

def balance_dataset(
    samples: List[Dict],
    target_size: int = DEFAULT_TARGET_SIZE,
    seed: int = RANDOM_SEED,
) -> List[Dict]:
    """Balance dataset to target size with 50/50 attack/benign split.

    Strategy:
    - Oversample attacks if fewer than target/2
    - Subsample benign to match
    """
    print("\n" + "=" * 70)
    print(f"Balancing Dataset to {target_size:,} samples (50/50)")
    print("=" * 70)

    rng = random.Random(seed)

    attacks = [s for s in samples if s["is_injection"]]
    benign = [s for s in samples if not s["is_injection"]]

    half = target_size // 2
    print(f"\n  Available: {len(attacks):,} attacks, {len(benign):,} benign")
    print(f"  Target:    {half:,} attacks, {half:,} benign")

    # Handle attacks: oversample if needed
    if len(attacks) >= half:
        print(f"  Attacks:   Subsampling {len(attacks):,} -> {half:,}")
        rng.shuffle(attacks)
        balanced_attacks = attacks[:half]
    else:
        print(f"  Attacks:   Oversampling {len(attacks):,} -> {half:,} ({half / len(attacks):.1f}x)")
        # First include all unique, then oversample
        balanced_attacks = list(attacks)
        remaining = half - len(attacks)
        while remaining > 0:
            batch = min(remaining, len(attacks))
            balanced_attacks.extend(rng.sample(attacks, batch))
            remaining -= batch

    # Handle benign: subsample if needed
    if len(benign) >= half:
        print(f"  Benign:    Subsampling {len(benign):,} -> {half:,}")
        rng.shuffle(benign)
        balanced_benign = benign[:half]
    else:
        print(f"  Benign:    Oversampling {len(benign):,} -> {half:,} ({half / len(benign):.1f}x)")
        balanced_benign = list(benign)
        remaining = half - len(benign)
        while remaining > 0:
            batch = min(remaining, len(benign))
            balanced_benign.extend(rng.sample(benign, batch))
            remaining -= batch

    combined = balanced_attacks + balanced_benign
    rng.shuffle(combined)

    print(f"\n  Final: {len(combined):,} samples")

    # Print attack type distribution
    by_type = defaultdict(int)
    for s in combined:
        by_type[s["attack_type"]] += 1

    print(f"\n  Attack type distribution:")
    for atype in sorted(by_type.keys()):
        count = by_type[atype]
        pct = count / len(combined) * 100
        print(f"    {atype:<25} {count:>8} ({pct:>5.1f}%)")

    return combined


# ============================================================================
# COMBINATION & OUTPUT
# ============================================================================

def normalize_to_canonical(samples: List[Dict]) -> List[Dict]:
    """Normalize all samples to canonical JSON schema format."""
    print("\n" + "=" * 70)
    print("Normalizing to Canonical Schema")
    print("=" * 70)

    canonical = []

    for i, sample in enumerate(samples):
        text = sample.get("text", "").strip()
        is_injection = sample.get("is_injection", False)

        attack_type = sample.get("attack_type")
        if not attack_type or attack_type in ("unknown", "injection", "jailbreak", "benign"):
            if is_injection:
                attack_type = infer_attack_type(text)
            else:
                attack_type = "Benign"

        attack_type_idx = ATTACK_TYPE_TO_IDX.get(attack_type, 6)

        # Ensure consistency
        if attack_type == "Benign" and is_injection:
            attack_type = infer_attack_type(text)
            attack_type_idx = ATTACK_TYPE_TO_IDX.get(attack_type, 6)
        elif attack_type != "Benign" and not is_injection:
            attack_type = "Benign"
            attack_type_idx = 0

        source = sample.get("source", "unknown")
        metadata = sample.get("metadata")

        canonical_sample = {
            "index": i,
            "text": text,
            "is_injection": is_injection,
            "attack_type": attack_type,
            "attack_type_idx": attack_type_idx,
            "source": source,
        }

        if metadata:
            canonical_sample["metadata"] = metadata

        canonical.append(canonical_sample)

    print(f"  Normalized {len(canonical)} samples")

    return canonical


def print_statistics(samples: List[Dict]):
    """Print comprehensive dataset statistics."""
    print("\n" + "=" * 70)
    print("Dataset Statistics")
    print("=" * 70)

    total = len(samples)
    injections = sum(1 for s in samples if s.get("is_injection", False))
    benign = total - injections

    print(f"\n  Total samples:    {total:,}")
    print(f"  Injections:       {injections:,} ({injections / total * 100:.1f}%)")
    print(f"  Benign:           {benign:,} ({benign / total * 100:.1f}%)")

    # By source
    by_source = defaultdict(lambda: {"total": 0, "injections": 0})
    for s in samples:
        src = s.get("source", "unknown")
        by_source[src]["total"] += 1
        if s.get("is_injection", False):
            by_source[src]["injections"] += 1

    print(f"\n  By Source:")
    print(f"    {'Source':<25} {'Total':>8} {'Injections':>12} {'Benign':>8}")
    print(f"    {'-' * 57}")
    for source in sorted(by_source.keys()):
        stats = by_source[source]
        ben = stats["total"] - stats["injections"]
        inj_pct = stats["injections"] / stats["total"] * 100 if stats["total"] > 0 else 0
        print(f"    {source:<25} {stats['total']:>8} {stats['injections']:>7} ({inj_pct:>5.1f}%) {ben:>7}")

    # By attack type
    by_type = defaultdict(int)
    for s in samples:
        by_type[s.get("attack_type", "unknown")] += 1

    print(f"\n  By Attack Type:")
    print(f"    {'Attack Type':<25} {'Count':>8} {'Percentage':>10}")
    print(f"    {'-' * 48}")
    for atype in sorted(by_type.keys()):
        count = by_type[atype]
        pct = count / total * 100
        print(f"    {atype:<25} {count:>8} ({pct:>5.1f}%)")


def save_dataset(samples: List[Dict], output_path: Path):
    """Save combined dataset to JSON."""
    print(f"\n  Saving to: {output_path}")
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open(output_path, 'w') as f:
        json.dump(samples, f)

    file_size_mb = os.path.getsize(output_path) / (1024 * 1024)
    print(f"  File size: {file_size_mb:.1f} MB")
    print(f"  Samples: {len(samples):,}")


# ============================================================================
# MAIN
# ============================================================================

def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Download and combine prompt injection datasets"
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=OUTPUT_PATH,
        help="Output JSON file path",
    )
    parser.add_argument(
        "--skip-fuzzy",
        action="store_true",
        help="Skip fuzzy deduplication (faster)",
    )
    parser.add_argument(
        "--target-size",
        type=int,
        default=DEFAULT_TARGET_SIZE,
        help="Target dataset size after balancing (default: 200000)",
    )
    parser.add_argument(
        "--no-balance",
        action="store_true",
        help="Skip balancing, output all unique samples",
    )
    parser.add_argument(
        "--skip-large-benign",
        action="store_true",
        help="Skip large benign datasets (alpaca, hh-rlhf) for faster testing",
    )

    args = parser.parse_args()

    print("\n" + "=" * 70)
    print("  JailGuard Dataset Download & Combination Pipeline v2")
    print("  14 Real Public Datasets | 3-Tier Dedup | 200K Balanced Output")
    print("=" * 70)

    start_time = time.time()

    # ====================================================================
    # STEP 1: Download all datasets
    # ====================================================================
    print("\n" + "=" * 70)
    print("STEP 1: Download Datasets")
    print("=" * 70)

    all_samples = []

    # Attack sources
    for downloader in [
        download_deepset,
        download_trustailab,
        download_spml,
        download_jailbreakbench,
        download_jailbreakv_28k,
        download_jackhhao_jailbreak,
        download_rubend18_jailbreak,
        download_toxic_chat,
    ]:
        samples = downloader()
        if samples:
            all_samples.extend(samples)

    # Benign sources
    for downloader in [
        download_dolly,
        download_openassistant,
    ]:
        samples = downloader()
        if samples:
            all_samples.extend(samples)

    if not args.skip_large_benign:
        for downloader in [
            download_alpaca,
            download_beavertails,
            download_hh_rlhf,
        ]:
            samples = downloader()
            if samples:
                all_samples.extend(samples)

    # EasyJailbreak (benign from injection perspective)
    samples = download_easyjailbreak()
    if samples:
        all_samples.extend(samples)

    inj = sum(1 for s in all_samples if s["is_injection"])
    ben = len(all_samples) - inj
    print(f"\n  Raw total: {len(all_samples):,} samples ({inj:,} attacks, {ben:,} benign)")

    # ====================================================================
    # STEP 2: Deduplicate & Quality Filter
    # ====================================================================
    print("\n" + "=" * 70)
    print("STEP 2: Deduplicate & Quality Filter")
    print("=" * 70)

    all_samples, dedup_stats = run_deduplication(all_samples, skip_fuzzy=args.skip_fuzzy)
    all_samples, quality_stats = quality_filter(all_samples)

    inj = sum(1 for s in all_samples if s["is_injection"])
    ben = len(all_samples) - inj
    print(f"\n  After dedup+filter: {len(all_samples):,} ({inj:,} attacks, {ben:,} benign)")

    # ====================================================================
    # STEP 3: Balance to target size
    # ====================================================================
    if not args.no_balance:
        all_samples = balance_dataset(all_samples, target_size=args.target_size)

    # ====================================================================
    # STEP 4: Normalize to canonical schema
    # ====================================================================
    print("\n" + "=" * 70)
    print("STEP 3: Normalize to Canonical Schema")
    print("=" * 70)

    canonical = normalize_to_canonical(all_samples)

    # Print statistics
    print_statistics(canonical)

    # ====================================================================
    # STEP 5: Save
    # ====================================================================
    print("\n" + "=" * 70)
    print("STEP 4: Save Dataset")
    print("=" * 70)

    save_dataset(canonical, args.output)

    # Save stats
    stats_path = args.output.parent / "combination_statistics.json"
    stats = {
        "dedup_stats": dedup_stats,
        "quality_stats": quality_stats,
        "final_count": len(canonical),
        "target_size": args.target_size,
    }
    with open(stats_path, 'w') as f:
        json.dump(stats, f, indent=2, default=str)
    print(f"  Stats saved to: {stats_path}")

    elapsed = time.time() - start_time

    print("\n" + "=" * 70)
    print(f"  Complete! Total time: {elapsed:.1f}s ({elapsed / 60:.1f} min)")
    print("=" * 70)

    print(f"\nNext steps:")
    print(f"  1. Generate ONNX embeddings (Rust):")
    print(f"     cargo run --bin onnx_embedding_generation --features onnx --release -- \\")
    print(f"       --input {args.output} --output data/combined_200k_onnx_embeddings.json \\")
    print(f"       --model-dir models/ --batch-size 64")
    print(f"  2. Train model:")
    print(f"     cargo run --bin train_neural_binary --release")

    return 0


if __name__ == "__main__":
    sys.exit(main())
