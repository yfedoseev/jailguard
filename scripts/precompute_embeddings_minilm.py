#!/usr/bin/env python3
"""
Pre-compute embeddings using all-MiniLM-L6-v2 sentence transformer.

Supports batch encoding (10-50x faster), checkpointing for crash recovery,
and large datasets (200K+).

Usage:
    python3 scripts/precompute_embeddings_minilm.py --data data/combined.json
    python3 scripts/precompute_embeddings_minilm.py --data data/balanced_200k.json --batch-size 256 --resume
    python3 scripts/precompute_embeddings_minilm.py --data data/input.json --output data/output.json
"""

import json
import time
import sys
import argparse
from pathlib import Path
from typing import List, Dict, Optional

# ============================================================================
# CONFIGURATION
# ============================================================================

DEFAULT_BATCH_SIZE = 256
CHECKPOINT_INTERVAL = 10000  # Checkpoint every 10K samples
MODEL_NAME = "all-MiniLM-L6-v2"
EMBEDDING_DIM = 384


# ============================================================================
# EMBEDDING ENGINE
# ============================================================================

class BatchEmbedder:
    """Batch embedding generator with checkpointing."""

    def __init__(self, batch_size: int = DEFAULT_BATCH_SIZE):
        self.batch_size = batch_size
        self.model = None

    def load_model(self):
        """Load the sentence transformer model."""
        try:
            from sentence_transformers import SentenceTransformer
        except ImportError:
            print("Installing sentence-transformers...")
            import subprocess
            subprocess.check_call([
                sys.executable, "-m", "pip", "install",
                "--break-system-packages", "-q", "sentence-transformers",
            ])
            from sentence_transformers import SentenceTransformer

        print(f"Loading {MODEL_NAME} model...")
        start = time.time()
        self.model = SentenceTransformer(MODEL_NAME)
        print(f"  Model loaded in {time.time() - start:.2f}s")

    def encode_batch(self, texts: List[str]) -> List[List[float]]:
        """Encode a batch of texts to embeddings."""
        if self.model is None:
            self.load_model()

        embeddings = self.model.encode(
            texts,
            batch_size=self.batch_size,
            show_progress_bar=False,
            convert_to_numpy=True,
        )
        return [emb.tolist() for emb in embeddings]

    def encode_dataset(
        self,
        samples: List[Dict],
        checkpoint_path: Optional[Path] = None,
        resume: bool = False,
    ) -> List[Dict]:
        """
        Encode all samples with batch processing and checkpointing.

        Args:
            samples: List of sample dicts with 'text' field
            checkpoint_path: Path for checkpoint files
            resume: Whether to resume from last checkpoint

        Returns:
            List of samples with 'embedding' field added
        """
        if self.model is None:
            self.load_model()

        total = len(samples)
        start_idx = 0

        # Resume from checkpoint if available
        if resume and checkpoint_path and checkpoint_path.exists():
            print(f"  Resuming from checkpoint: {checkpoint_path}")
            with open(checkpoint_path) as f:
                checkpoint_data = json.load(f)
            start_idx = checkpoint_data.get("processed_count", 0)
            # Load already-processed samples
            for i in range(min(start_idx, len(samples))):
                if i < len(checkpoint_data.get("embeddings", [])):
                    samples[i]["embedding"] = checkpoint_data["embeddings"][i]
                    samples[i]["embedding_dim"] = EMBEDDING_DIM
            print(f"  Resuming from sample {start_idx}")

        print(f"\n  Encoding {total - start_idx} samples (batch_size={self.batch_size})...")
        print(f"  {'='*60}")

        total_start = time.time()

        for batch_start in range(start_idx, total, self.batch_size):
            batch_end = min(batch_start + self.batch_size, total)
            batch_texts = [s.get("text", "") for s in samples[batch_start:batch_end]]

            batch_start_time = time.time()
            embeddings = self.encode_batch(batch_texts)
            batch_elapsed = time.time() - batch_start_time

            # Assign embeddings to samples
            for i, emb in enumerate(embeddings):
                samples[batch_start + i]["embedding"] = emb
                samples[batch_start + i]["embedding_dim"] = EMBEDDING_DIM

            # Progress reporting
            processed = batch_end
            elapsed = time.time() - total_start
            rate = (processed - start_idx) / elapsed if elapsed > 0 else 0
            remaining = (total - processed) / rate if rate > 0 else 0

            if processed % 1000 < self.batch_size or processed == total:
                print(
                    f"  [{processed:>7}/{total}] "
                    f"{processed / total * 100:>5.1f}% | "
                    f"{rate:.0f} samples/s | "
                    f"ETA: {remaining / 60:.1f}m | "
                    f"Batch: {batch_elapsed * 1000:.0f}ms"
                )

            # Checkpoint
            if (
                checkpoint_path
                and processed % CHECKPOINT_INTERVAL < self.batch_size
                and processed < total
            ):
                self._save_checkpoint(samples, processed, checkpoint_path)

        total_elapsed = time.time() - total_start
        print(f"\n  Encoding complete!")
        print(f"  Total time: {total_elapsed:.1f}s ({total_elapsed / 60:.1f}m)")
        print(f"  Average: {total_elapsed / max(total - start_idx, 1) * 1000:.1f}ms/sample")
        print(f"  Throughput: {max(total - start_idx, 1) / total_elapsed:.0f} samples/s")

        return samples

    def _save_checkpoint(self, samples: List[Dict], processed: int, path: Path):
        """Save checkpoint for crash recovery."""
        checkpoint = {
            "processed_count": processed,
            "embeddings": [
                s.get("embedding") for s in samples[:processed]
            ],
        }
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, 'w') as f:
            json.dump(checkpoint, f)
        print(f"  [Checkpoint saved: {processed} samples]")


# ============================================================================
# MAIN
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Pre-compute embeddings using all-MiniLM-L6-v2"
    )
    parser.add_argument(
        "--data",
        default="data/combined_injection_dataset.json",
        help="Input dataset JSON file",
    )
    parser.add_argument(
        "--output",
        default=None,
        help="Output embeddings JSON file (default: replaces input extension with _embeddings.json)",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=DEFAULT_BATCH_SIZE,
        help=f"Batch size for encoding (default: {DEFAULT_BATCH_SIZE})",
    )
    parser.add_argument(
        "--resume",
        action="store_true",
        help="Resume from last checkpoint",
    )
    parser.add_argument(
        "--no-checkpoint",
        action="store_true",
        help="Disable checkpointing",
    )
    args = parser.parse_args()

    print("\n" + "=" * 70)
    print("  BATCH EMBEDDING PRE-COMPUTATION")
    print(f"  Model: {MODEL_NAME} ({EMBEDDING_DIM}-dim)")
    print(f"  Batch size: {args.batch_size}")
    print("=" * 70 + "\n")

    # Load dataset
    data_path = Path(args.data)
    if not data_path.exists():
        print(f"  Dataset not found at {data_path}")
        return 1

    print(f"  Loading dataset from {data_path}...")
    with open(data_path) as f:
        samples = json.load(f)
    print(f"  Loaded {len(samples):,} samples\n")

    # Determine output path
    if args.output:
        output_path = Path(args.output)
    else:
        stem = data_path.stem
        if stem.endswith("_embeddings"):
            output_path = data_path  # Overwrite
        else:
            output_path = data_path.parent / f"{stem}_embeddings.json"

    # Checkpoint path
    checkpoint_path = None
    if not args.no_checkpoint:
        checkpoint_path = data_path.parent / f".{data_path.stem}_checkpoint.json"

    # Encode
    embedder = BatchEmbedder(batch_size=args.batch_size)
    samples = embedder.encode_dataset(
        samples,
        checkpoint_path=checkpoint_path,
        resume=args.resume,
    )

    # Validate embeddings
    valid_count = sum(1 for s in samples if s.get("embedding") and len(s["embedding"]) == EMBEDDING_DIM)
    print(f"\n  Validation: {valid_count}/{len(samples)} samples have valid {EMBEDDING_DIM}-dim embeddings")

    if valid_count != len(samples):
        print(f"  WARNING: {len(samples) - valid_count} samples missing embeddings!")

    # Save
    print(f"\n  Saving to {output_path}...")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, 'w') as f:
        json.dump(samples, f)  # No indent for large files (saves ~50% space)

    file_size = output_path.stat().st_size / (1024 * 1024)
    print(f"  File size: {file_size:.1f} MB")

    # Clean up checkpoint
    if checkpoint_path and checkpoint_path.exists():
        checkpoint_path.unlink()
        print(f"  Checkpoint cleaned up")

    # Statistics
    injection_count = sum(1 for s in samples if s.get("is_injection"))
    print(f"\n  Statistics:")
    print(f"    Dimension:   {EMBEDDING_DIM}")
    print(f"    Count:       {len(samples):,}")
    print(f"    Injections:  {injection_count:,}/{len(samples):,} ({100 * injection_count / len(samples):.1f}%)")
    print(f"    Model:       {MODEL_NAME}")

    print(f"\n  Next step:")
    print(f"    python3 scripts/dataset_split.py --input {output_path} --output splits_200k/")
    print(f"\n{'='*70}\n")

    return 0


if __name__ == "__main__":
    sys.exit(main())
