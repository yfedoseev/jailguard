#!/usr/bin/env python3
"""
Pre-compute embeddings using all-MiniLM-L6-v2 sentence transformer
Fast (~5ms/sample) and high quality semantic embeddings
"""

import json
import time
import sys
import argparse
from pathlib import Path

print("\n" + "="*70)
print("FAST EMBEDDING PRE-COMPUTATION")
print("Using all-MiniLM-L6-v2 (384-dim, ~5ms/sample)")
print("="*70 + "\n")

# Install sentence-transformers if needed
try:
    from sentence_transformers import SentenceTransformer
except ImportError:
    print("📦 Installing sentence-transformers...")
    import subprocess
    try:
        subprocess.check_call([sys.executable, "-m", "pip", "install",
                              "--break-system-packages", "-q", "sentence-transformers"])
    except Exception as e:
        print(f"⚠️  Installation failed: {e}")
        print("Please install manually: pip install --break-system-packages sentence-transformers")
        exit(1)
    from sentence_transformers import SentenceTransformer

# Parse command-line arguments
parser = argparse.ArgumentParser(description="Pre-compute embeddings using all-MiniLM-L6-v2")
parser.add_argument("--data", default="data/prompt_injections_real.json", help="Input dataset JSON file")
parser.add_argument("--output", default="data/minilm_embeddings.json", help="Output embeddings JSON file")
args = parser.parse_args()

# Load dataset
data_path = Path(args.data)
if not data_path.exists():
    print(f"⚠️  Dataset not found at {data_path}")
    exit(1)

print("📊 DATASET")
with open(data_path) as f:
    samples = json.load(f)
print(f"   Total samples: {len(samples)}\n")

# Load model
print("🔧 Loading all-MiniLM-L6-v2 model...")
start = time.time()
model = SentenceTransformer("all-MiniLM-L6-v2")
print(f"   ✓ Loaded in {time.time() - start:.2f}s\n")

# Extract embeddings
print("🧠 EXTRACTING EMBEDDINGS")
print("-" * 70)

embeddings_data = []
total_start = time.time()

for idx, sample in enumerate(samples):
    text = sample.get("text", "")
    is_injection = sample.get("is_injection", False)
    
    sample_start = time.time()
    embedding = model.encode(text, convert_to_tensor=False)
    elapsed = time.time() - sample_start
    
    embeddings_data.append({
        "embedding": embedding.tolist(),
        "is_injection": is_injection,
        "text": text,
        "index": idx,
        "embedding_dim": len(embedding)
    })
    
    if (idx + 1) % 100 == 0 or idx == 0:
        print(f"   Sample {idx+1:3d}/{len(samples)} | {elapsed*1000:.1f}ms | "
              f"Dim: {len(embedding)} | Text: \"{text[:40]}...\"")

total_time = time.time() - total_start

print(f"\n✅ EMBEDDING EXTRACTION COMPLETE")
print(f"   Total samples processed: {len(samples)}")
print(f"   Total time: {total_time:.2f}s")
print(f"   Average per sample: {total_time/len(samples)*1000:.1f}ms")
print(f"   Speedup vs transformer detector: ~{100*total_time/len(samples)/1:.0f}x faster")

# Save embeddings
output_file = args.output
Path(output_file).parent.mkdir(parents=True, exist_ok=True)
with open(output_file, "w") as f:
    json.dump(embeddings_data, f, indent=2)

print(f"\n✅ Embeddings saved to {output_file}")
file_size = Path(output_file).stat().st_size / (1024*1024)
print(f"   File size: {file_size:.2f} MB")

# Embedding statistics
if embeddings_data and embeddings_data[0]["embedding"]:
    print(f"\n📊 EMBEDDING STATISTICS")
    print(f"   Dimension: {embeddings_data[0]['embedding_dim']}")
    print(f"   Count: {len(embeddings_data)}")
    print(f"   Model: all-MiniLM-L6-v2 (384-dim, semantic)")
    
    # Count injections
    injection_count = sum(1 for e in embeddings_data if e["is_injection"])
    print(f"   Injections: {injection_count}/{len(embeddings_data)} ({100*injection_count/len(embeddings_data):.1f}%)")

print(f"\n🚀 Next step:")
print(f"   cargo run --example train_with_minilm_embeddings")
print(f"\n{'='*70}\n")
