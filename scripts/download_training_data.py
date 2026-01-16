#!/usr/bin/env python3
"""Download and prepare real training data from deepset/prompt-injections dataset."""

import json
import sys
from pathlib import Path

try:
    from datasets import load_dataset
except ImportError:
    print("Installing huggingface datasets...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "-q", "datasets"])
    from datasets import load_dataset

def classify_attack_type(text):
    """Simple heuristic-based attack type classification."""
    text_lower = text.lower()

    if "ignore" in text_lower or "disregard" in text_lower or "forget" in text_lower:
        return "InstructionOverride"
    elif "role" in text_lower or "pretend" in text_lower or "act as" in text_lower:
        return "RoleplayInjection"
    elif "context" in text_lower or "remember" in text_lower:
        return "ContextManipulation"
    elif "output" in text_lower or "format" in text_lower or "response" in text_lower:
        return "OutputManipulation"
    elif any(x in text_lower for x in ["base64", "encode", "hex", "rot13", "url"]):
        return "EncodingObfuscation"
    elif any(x in text_lower for x in ["jailbreak", "unrestricted", "developer mode", "dan"]):
        return "JailbreakPatterns"
    else:
        return "Benign"

def download_and_prepare_data():
    """Download dataset and prepare for training."""
    print("Downloading deepset/prompt-injections dataset...")

    try:
        # Load the dataset
        ds = load_dataset("deepset/prompt-injections", split="train")
        print(f"Loaded {len(ds)} samples from training set")

        # Convert to JailGuard format
        training_samples = []
        for idx, item in enumerate(ds):
            text = item.get("text", "")
            if not text:
                continue

            # Determine if injection based on label
            is_injection = item.get("label", "") == "injection"

            sample = {
                "text": text,
                "is_injection": is_injection,
                "attack_type": classify_attack_type(text),
                "expected_output": None,
            }
            training_samples.append(sample)

            if (idx + 1) % 100 == 0:
                print(f"  Processed {idx + 1} samples...")

        # Save to JSON
        output_dir = Path(__file__).parent.parent / "data"
        output_dir.mkdir(exist_ok=True)

        output_file = output_dir / "training_data.json"
        with open(output_file, "w") as f:
            json.dump(training_samples, f, indent=2)

        print(f"\n✅ Saved {len(training_samples)} samples to {output_file}")

        # Print statistics
        injection_count = sum(1 for s in training_samples if s["is_injection"])
        benign_count = len(training_samples) - injection_count

        print(f"\nDataset Statistics:")
        print(f"  Total samples: {len(training_samples)}")
        print(f"  Injections: {injection_count} ({injection_count/len(training_samples)*100:.1f}%)")
        print(f"  Benign: {benign_count} ({benign_count/len(training_samples)*100:.1f}%)")

        # Print attack type distribution
        attack_types = {}
        for sample in training_samples:
            attack_type = sample["attack_type"]
            attack_types[attack_type] = attack_types.get(attack_type, 0) + 1

        print(f"\nAttack Type Distribution:")
        for attack_type, count in sorted(attack_types.items(), key=lambda x: -x[1]):
            print(f"  {attack_type}: {count}")

        return output_file

    except Exception as e:
        print(f"Error downloading dataset: {e}")
        print("Falling back to synthetic data generation...")
        return None

if __name__ == "__main__":
    data_file = download_and_prepare_data()
    if data_file:
        print(f"\n📊 Data ready at: {data_file}")
        sys.exit(0)
    else:
        sys.exit(1)
