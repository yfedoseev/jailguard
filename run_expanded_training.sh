#!/bin/bash

# Run training on expanded dataset once embeddings are available
# This script waits for embeddings to be generated, then trains the model

set -e

EMBEDDINGS_FILE="data/combined_minilm_embeddings.json"
COMBINED_DATASET="data/combined_injection_dataset.json"

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║ JailGuard Expanded Dataset Training Pipeline                      ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""

# Check if combined dataset exists
if [ ! -f "$COMBINED_DATASET" ]; then
    echo "❌ Error: Combined dataset not found at $COMBINED_DATASET"
    exit 1
fi

echo "📊 Dataset Status:"
echo "   Combined dataset: ✅ Ready ($COMBINED_DATASET, $(du -h "$COMBINED_DATASET" | cut -f1))"
echo ""

# Check if embeddings exist, if not wait
if [ ! -f "$EMBEDDINGS_FILE" ]; then
    echo "⏳ Embeddings file not found. Checking embedding generation status..."
    echo ""

    # Check if embedding generation process is running
    if pgrep -f "precompute_embeddings_minilm" > /dev/null; then
        echo "   ⏳ Embedding generation in progress..."
        echo "   Monitor with: tail -f /tmp/*.output | grep -i sample"
        echo ""
        echo "   Waiting for embeddings to complete..."
        echo "   (This may take several hours for 15,185 samples)"
        echo ""

        # Wait for embeddings file to appear
        while [ ! -f "$EMBEDDINGS_FILE" ]; do
            sleep 30
            if pgrep -f "precompute_embeddings_minilm" > /dev/null; then
                echo "   Still processing... ($(date +%H:%M:%S))"
            else
                echo "   Embedding process finished but file not found. Check for errors."
                exit 1
            fi
        done
    else
        echo "   ❌ Embedding generation not running. Start with:"
        echo "      python3 scripts/precompute_embeddings_minilm.py --data $COMBINED_DATASET --output $EMBEDDINGS_FILE"
        exit 1
    fi
fi

echo "   Embeddings file: ✅ Ready ($EMBEDDINGS_FILE, $(du -h "$EMBEDDINGS_FILE" | cut -f1))"
echo ""

# Build training example
echo "🏗️  Building training example..."
cargo build --example train_minilm_expanded_dataset --release 2>&1 | grep -E "(Compiling|Finished|error)" || true
echo ""

# Run training
echo "🏋️  Running training on expanded dataset..."
echo "   Architecture: 384 → 256 (ReLU) → 2 (softmax)"
echo "   Samples: 15,185 (9,111 train, 3,037 val, 3,037 test)"
echo "   Baseline accuracy (662 samples): 78.9%"
echo "   Expected accuracy (15,185 samples): 82-87%"
echo "   Training time: ~30-60 seconds"
echo ""

cargo run --example train_minilm_expanded_dataset --release

echo ""
echo "✅ Training completed!"
