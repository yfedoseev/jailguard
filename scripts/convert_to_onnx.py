#!/usr/bin/env python3
"""
Convert all-MiniLM-L6-v2 from HuggingFace to ONNX format for native Rust inference.

This is a one-time conversion that enables 10-50x faster embedding generation.

Usage:
    python3 scripts/convert_to_onnx.py

Output:
    models/model.onnx
    models/tokenizer.json
"""

import os
import sys
from pathlib import Path

print("\n" + "="*70)
print("CONVERTING all-MiniLM-L6-v2 TO ONNX")
print("One-time conversion for native Rust inference")
print("="*70 + "\n")

# Install optimum if needed
try:
    from optimum.onnxruntime import ORTModelForFeatureExtraction
except ImportError:
    print("Installing optimum and onnxruntime...")
    import subprocess
    try:
        subprocess.check_call([
            sys.executable, "-m", "pip", "install", "-q",
            "optimum[onnxruntime]", "sentence-transformers"
        ])
    except Exception as e:
        print(f"Installation failed: {e}")
        print("Please install manually:")
        print("  pip install optimum[onnxruntime] sentence-transformers")
        sys.exit(1)
    from optimum.onnxruntime import ORTModelForFeatureExtraction

from transformers import AutoTokenizer

model_id = "sentence-transformers/all-MiniLM-L6-v2"

# Load tokenizer
print("Loading tokenizer...")
tokenizer = AutoTokenizer.from_pretrained(model_id)

# Convert to ONNX
print("Converting to ONNX...")
ort_model = ORTModelForFeatureExtraction.from_pretrained(
    model_id,
    export=True,
    provider="CPUExecutionProvider"
)

# Create output directory
output_dir = Path("models")
output_dir.mkdir(exist_ok=True)

# Save ONNX model
print(f"Saving to {output_dir}/...")
ort_model.save_pretrained(str(output_dir))

# Save tokenizer as tokenizer.json (needed by Rust tokenizers crate)
tokenizer.save_pretrained(str(output_dir))

# Verify output files
model_path = output_dir / "model.onnx"
tokenizer_path = output_dir / "tokenizer.json"

if model_path.exists() and tokenizer_path.exists():
    model_size = model_path.stat().st_size / (1024 * 1024)
    tok_size = tokenizer_path.stat().st_size / 1024
    print(f"\nConversion complete!")
    print(f"  Model:     {model_path} ({model_size:.1f} MB)")
    print(f"  Tokenizer: {tokenizer_path} ({tok_size:.1f} KB)")
    print(f"\nNext step:")
    print(f"  cargo run --bin onnx_embedding_generation --features onnx --release -- \\")
    print(f"    --input data/combined.json \\")
    print(f"    --output data/combined_onnx_embeddings.json \\")
    print(f"    --model-dir models/")
else:
    print(f"\nFailed to create model files")
    if not model_path.exists():
        # optimum may save as different name, check what's there
        onnx_files = list(output_dir.glob("*.onnx"))
        print(f"  ONNX files found: {onnx_files}")
    sys.exit(1)

print(f"\n{'='*70}\n")
