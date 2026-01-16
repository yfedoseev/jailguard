#!/bin/bash

# Monitor embedding generation and automatically run training when complete

EMBEDDINGS_FILE="data/combined_minilm_embeddings.json"
COMBINED_DATASET="data/combined_injection_dataset.json"
CHECK_INTERVAL=10  # Check every 10 seconds
MAX_WAIT=36000     # 10 hours max wait

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║  Monitoring Embedding Generation & Automatic Training            ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""

if [ ! -f "$COMBINED_DATASET" ]; then
    echo "❌ Error: Combined dataset not found"
    exit 1
fi

echo "📊 Monitoring for embeddings: $EMBEDDINGS_FILE"
echo "⏱️  Check interval: ${CHECK_INTERVAL}s"
echo "⏱️  Max wait time: ${MAX_WAIT}s"
echo ""

ELAPSED=0
while [ ! -f "$EMBEDDINGS_FILE" ] && [ $ELAPSED -lt $MAX_WAIT ]; do
    # Check if embedding process is still running
    if pgrep -f "precompute_embeddings_minilm" > /dev/null; then
        ELAPSED=$((ELAPSED + CHECK_INTERVAL))
        PERCENT=$((ELAPSED * 100 / MAX_WAIT))
        MINS=$((ELAPSED / 60))
        echo "⏳ Waiting... ${MINS}m elapsed (${PERCENT}%) - $(date +%H:%M:%S)"
        sleep $CHECK_INTERVAL
    else
        if [ ! -f "$EMBEDDINGS_FILE" ]; then
            echo "❌ Embedding process finished but file not found!"
            exit 1
        fi
        break
    fi
done

if [ ! -f "$EMBEDDINGS_FILE" ]; then
    echo "❌ Timeout waiting for embeddings"
    exit 1
fi

echo ""
echo "✅ EMBEDDINGS READY!"
FILE_SIZE=$(du -h "$EMBEDDINGS_FILE" | cut -f1)
echo "📁 $EMBEDDINGS_FILE ($FILE_SIZE)"
echo ""

# Wait a moment for file write to complete
sleep 2

echo "🏋️  STARTING TRAINING ON EXPANDED DATASET..."
echo "═══════════════════════════════════════════════════════════════════════"
echo ""

# Build and run training
cargo build --example train_minilm_expanded_dataset --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

echo ""
echo "Running training..."
echo ""

cargo run --example train_minilm_expanded_dataset --release

echo ""
echo "═══════════════════════════════════════════════════════════════════════"
echo "✅ TRAINING COMPLETED!"
echo ""
echo "Results are shown above. Compare:"
echo "  Baseline (662 samples):   78.9%"
echo "  Expanded (15,185 samples): [SEE ABOVE]"
echo ""
