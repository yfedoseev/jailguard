#!/usr/bin/env python3
"""
Convert all-MiniLM-L6-v2 from HuggingFace to ONNX format for native Rust inference.

This is a one-time conversion that enables 10-50x faster embedding generation.

Usage:
    python3 scripts/convert_to_onnx.py

Output:
    models/minilm-l6-v2.onnx (~140 MB)
"""

import os
from pathlib import Path

print("\n" + "="*70)
print("CONVERTING all-MiniLM-L6-v2 TO ONNX")
print("One-time conversion for native Rust inference")
print("="*70 + "\n")

# Install optimum if needed
try:
    from optimum.onnxruntime import ORTModelForFeatureExtraction
except ImportError:
    print("📦 Installing optimum and onnxruntime...")
    import subprocess
    import sys
    try:
        subprocess.check_call([
            sys.executable, "-m", "pip", "install", "-q",
            "optimum[onnxruntime]", "sentence-transformers"
        ])
    except Exception as e:
        print(f"⚠️  Installation failed: {e}")
        print("Please install manually:")
        print("  pip install optimum[onnxruntime] sentence-transformers")
        exit(1)
    from optimum.onnxruntime import ORTModelForFeatureExtraction

from transformers import AutoTokenizer

print("🔄 Downloading model from HuggingFace...")
model_id = "sentence-transformers/all-MiniLM-L6-v2"

# Load tokenizer
print("   Loading tokenizer...")
tokenizer = AutoTokenizer.from_pretrained(model_id)

# Convert to ONNX
print("   Converting to ONNX...")
ort_model = ORTModelForFeatureExtraction.from_pretrained(
    model_id,
    export=True,
    provider="CUDAExecutionProvider" if os.name == "cuda" else "CPUExecutionProvider"
)

# Create output directory
output_dir = Path("models")
output_dir.mkdir(exist_ok=True)

# Save model
output_path = output_dir / "minilm-l6-v2.onnx"
print(f"\n💾 Saving to {output_path}...")
ort_model.save_pretrained(str(output_dir))

# Verify
if output_path.exists():
    size_mb = output_path.stat().st_size / (1024 * 1024)
    print(f"\n✅ Conversion complete!")
    print(f"   Model saved: {output_path}")
    print(f"   Size: {size_mb:.1f} MB")
    print(f"\n🚀 Next step:")
    print(f"   cargo run --example generate_embeddings_native --features onnx-embeddings --release")
else:
    print(f"\n❌ Failed to create {output_path}")
    exit(1)

print(f"\n{='='*70}\n")
