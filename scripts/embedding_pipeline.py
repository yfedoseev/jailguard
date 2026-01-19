#!/usr/bin/env python3
"""
Batched Embedding Generation Pipeline

Generates 384-dimensional embeddings for 200K samples with:
- Batching for memory efficiency (batch_size=128 or 512 if GPU)
- Checkpointing every 10K samples for resumability
- Progress tracking and time estimates
- GPU support (automatic detection)

Uses all-MiniLM-L6-v2 model (384-dimensional, ~5ms per sample on CPU)

Usage:
    python3 scripts/embedding_pipeline.py --input augmented_200k.json --output embeddings.json
    python3 scripts/embedding_pipeline.py --input data.json --batch-size 512  # GPU mode
    python3 scripts/embedding_pipeline.py --resume checkpoint_150000.json  # Resume from failure
"""

import json
import sys
import time
from pathlib import Path
from typing import List, Dict, Optional, Tuple
from datetime import datetime, timedelta

try:
    from sentence_transformers import SentenceTransformer
    import torch
except ImportError:
    print("Installing required packages...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "sentence-transformers", "torch"])
    from sentence_transformers import SentenceTransformer
    import torch


# ============================================================================
# EMBEDDING CONFIGURATION
# ============================================================================

MODEL_NAME = "all-MiniLM-L6-v2"
EMBEDDING_DIM = 384
CHECKPOINT_INTERVAL = 10000  # Save checkpoint every 10K samples
DEFAULT_BATCH_SIZE_CPU = 128
DEFAULT_BATCH_SIZE_GPU = 512


# ============================================================================
# EMBEDDING PIPELINE
# ============================================================================

class EmbeddingPipeline:
    """Generate embeddings with checkpointing and resumability."""

    def __init__(self, batch_size: Optional[int] = None, device: Optional[str] = None):
        """
        Initialize embedding pipeline.

        Args:
            batch_size: Batch size (auto-detect if None)
            device: Device to use ('cuda' or 'cpu', auto-detect if None)
        """
        # Auto-detect device
        if device is None:
            self.device = "cuda" if torch.cuda.is_available() else "cpu"
        else:
            self.device = device

        # Auto-detect batch size
        if batch_size is None:
            self.batch_size = DEFAULT_BATCH_SIZE_GPU if self.device == "cuda" else DEFAULT_BATCH_SIZE_CPU
        else:
            self.batch_size = batch_size

        print(f"🔧 Loading {MODEL_NAME} model...")
        print(f"   Device: {self.device.upper()}")
        print(f"   Batch size: {self.batch_size}")

        self.model = SentenceTransformer(MODEL_NAME, device=self.device)
        print(f"✓ Model loaded with embedding dimension: {EMBEDDING_DIM}")

    def embed_batch(self, texts: List[str]) -> List[List[float]]:
        """
        Embed a batch of texts.

        Returns list of 384-dimensional embeddings.
        """
        embeddings = self.model.encode(texts, batch_size=self.batch_size, show_progress_bar=False)
        return embeddings.tolist()

    def process_samples(
        self,
        samples: List[Dict],
        start_index: int = 0,
        output_path: Optional[Path] = None,
    ) -> Tuple[List[Dict], Dict]:
        """
        Process samples and add embeddings.

        Args:
            samples: List of samples to embed
            start_index: Start index (for resuming from checkpoint)
            output_path: Path to save checkpoint file

        Returns:
            (processed_samples, statistics)
        """
        print(f"\n📊 Processing {len(samples):,} samples (starting from index {start_index})...")

        processed = []
        stats = {
            "total_samples": len(samples),
            "start_index": start_index,
            "successful": 0,
            "failed": 0,
            "start_time": datetime.now(),
            "errors": [],
        }

        # Process in batches
        for batch_start in range(start_index, len(samples), self.batch_size):
            batch_end = min(batch_start + self.batch_size, len(samples))
            batch = samples[batch_start:batch_end]

            # Extract texts
            texts = [s.get("text", "") for s in batch]

            try:
                # Embed batch
                embeddings = self.embed_batch(texts)

                # Add embeddings to samples
                for i, (sample, embedding) in enumerate(zip(batch, embeddings)):
                    processed_sample = sample.copy()
                    processed_sample["embedding"] = embedding
                    processed_sample["embedding_dim"] = EMBEDDING_DIM
                    processed_sample["index"] = batch_start + i
                    processed.append(processed_sample)

                stats["successful"] += len(batch)

                # Progress update
                elapsed = (datetime.now() - stats["start_time"]).total_seconds()
                rate = stats["successful"] / elapsed if elapsed > 0 else 0
                remaining = (len(samples) - batch_end) / rate if rate > 0 else 0

                if (batch_end - start_index) % (self.batch_size * 5) == 0:  # Every 5 batches
                    pct = (batch_end - start_index) / (len(samples) - start_index) * 100
                    eta = datetime.now() + timedelta(seconds=remaining)
                    print(f"  [{pct:>5.1f}%] {batch_end - start_index:>7} samples "
                          f"({rate:>6.1f} samples/sec, ETA: {eta.strftime('%H:%M:%S')})")

                # Save checkpoint
                if output_path and (batch_end - start_index) % CHECKPOINT_INTERVAL == 0:
                    self._save_checkpoint(processed, batch_end, output_path)

            except Exception as e:
                print(f"  ✗ Error processing batch [{batch_start}:{batch_end}]: {e}")
                stats["failed"] += len(batch)
                stats["errors"].append({
                    "batch": f"{batch_start}:{batch_end}",
                    "error": str(e),
                })

        stats["end_time"] = datetime.now()
        stats["duration"] = (stats["end_time"] - stats["start_time"]).total_seconds()

        return processed, stats

    def _save_checkpoint(self, processed: List[Dict], index: int, output_path: Path):
        """Save checkpoint at specified index."""
        checkpoint_path = output_path.parent / f"checkpoint_{index}.json"

        with open(checkpoint_path, 'w') as f:
            json.dump(processed, f)

        size_mb = checkpoint_path.stat().st_size / 1024 / 1024
        print(f"  💾 Checkpoint saved: {checkpoint_path.name} ({size_mb:.1f}MB)")


# ============================================================================
# DATASET UTILITIES
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


def find_checkpoint(output_path: Path) -> Optional[Tuple[Path, int]]:
    """Find most recent checkpoint file."""
    checkpoint_dir = output_path.parent
    checkpoints = sorted(checkpoint_dir.glob("checkpoint_*.json"))

    if checkpoints:
        latest = checkpoints[-1]
        # Extract index from filename
        index_str = latest.stem.split("_")[-1]
        try:
            index = int(index_str)
            return latest, index
        except ValueError:
            pass

    return None


def load_checkpoint(checkpoint_path: Path) -> List[Dict]:
    """Load samples from checkpoint."""
    print(f"📂 Loading checkpoint from {checkpoint_path}...")

    with open(checkpoint_path) as f:
        samples = json.load(f)

    print(f"✓ Loaded {len(samples):,} samples from checkpoint")
    return samples


# ============================================================================
# MAIN
# ============================================================================

def main():
    """Main execution."""
    import argparse

    parser = argparse.ArgumentParser(description="Generate embeddings with checkpointing")
    parser.add_argument("--input", type=Path, help="Input samples JSON")
    parser.add_argument("--output", type=Path, required=True, help="Output embeddings JSON")
    parser.add_argument("--batch-size", type=int, help="Batch size (auto-detect if not specified)")
    parser.add_argument("--device", choices=["cpu", "cuda"], help="Device (auto-detect if not specified)")
    parser.add_argument("--resume", type=Path, help="Resume from checkpoint file")
    parser.add_argument("--checkpoint-interval", type=int, default=CHECKPOINT_INTERVAL, help="Checkpoint interval")
    parser.add_argument("--skip-validation", action="store_true", help="Skip embedding validation")

    args = parser.parse_args()

    print("=" * 80)
    print("🚀 JailGuard Embedding Generation Pipeline")
    print("=" * 80)

    # Initialize pipeline
    pipeline = EmbeddingPipeline(batch_size=args.batch_size, device=args.device)

    # Load or resume
    processed = []
    start_index = 0

    if args.resume:
        # Resume from checkpoint
        if not args.resume.exists():
            print(f"❌ Checkpoint not found: {args.resume}")
            return 1

        processed = load_checkpoint(args.resume)
        start_index = len(processed)
        print(f"Resuming from index {start_index}")

        # Load original samples for remaining data
        if not args.input:
            print("❌ Error: --input required when resuming")
            return 1

        all_samples = load_samples(args.input)
        remaining_samples = all_samples[start_index:]

    elif args.input:
        # Start fresh
        all_samples = load_samples(args.input)
        remaining_samples = all_samples
    else:
        print("❌ Error: --input or --resume required")
        return 1

    # Process samples
    print(f"\n{'='*80}")
    print("STEP 1: Generate Embeddings")
    print(f"{'='*80}")

    new_processed, stats = pipeline.process_samples(
        remaining_samples,
        start_index=start_index,
        output_path=args.output,
    )

    processed.extend(new_processed)

    # Validate embeddings
    if not args.skip_validation:
        print(f"\n{'='*80}")
        print("STEP 2: Validate Embeddings")
        print(f"{'='*80}")

        invalid_count = 0
        for sample in processed:
            embedding = sample.get("embedding")
            if not embedding or len(embedding) != EMBEDDING_DIM:
                invalid_count += 1

        if invalid_count > 0:
            print(f"⚠️  Found {invalid_count} invalid embeddings")
        else:
            print(f"✓ All {len(processed)} embeddings are valid ({EMBEDDING_DIM}-dimensional)")

    # Save output
    print(f"\n{'='*80}")
    print("STEP 3: Save Results")
    print(f"{'='*80}")

    args.output.parent.mkdir(parents=True, exist_ok=True)

    with open(args.output, 'w') as f:
        json.dump(processed, f, indent=2)

    output_size_mb = args.output.stat().st_size / 1024 / 1024
    print(f"✓ Saved {len(processed):,} samples to {args.output}")
    print(f"  File size: {output_size_mb:.1f}MB")

    # Print statistics
    print(f"\n{'='*80}")
    print("📊 PIPELINE STATISTICS")
    print(f"{'='*80}")
    print(f"Successful: {stats['successful']:,}")
    print(f"Failed: {stats['failed']:,}")
    print(f"Duration: {stats['duration']:.1f} seconds ({stats['duration']/60:.1f} minutes)")

    if stats['duration'] > 0:
        rate = stats['successful'] / stats['duration']
        print(f"Rate: {rate:.1f} samples/second")
        print(f"Device: {pipeline.device.upper()}")
        print(f"Batch size: {pipeline.batch_size}")

    if stats['errors']:
        print(f"\n⚠️  Errors ({len(stats['errors'])}):")
        for error in stats['errors'][:5]:
            print(f"  [{error['batch']}] {error['error']}")

    print(f"\n✅ Embedding generation complete!")

    return 0


if __name__ == "__main__":
    sys.exit(main())
