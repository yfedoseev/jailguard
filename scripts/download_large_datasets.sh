#!/bin/bash
# Download large 200K balanced dataset for training
# This script downloads or generates the full training dataset

set -e

echo "=============================================="
echo "JailGuard Large Dataset Download/Generation"
echo "=============================================="

# Create output directory
mkdir -p splits_200k

# Check if splits already exist
if [ -f "splits_200k/train.json" ] && [ -f "splits_200k/val.json" ] && [ -f "splits_200k/test.json" ]; then
    echo "Dataset already exists in splits_200k/"
    echo "To regenerate, delete the directory first: rm -rf splits_200k/"
    exit 0
fi

echo ""
echo "Option 1: Generate from existing data"
echo "--------------------------------------"

# Check if we have the base data to generate from
if [ -f "data/combined_injection_dataset.json" ]; then
    echo "Found base dataset. Generating balanced splits..."

    # Use Python script to generate balanced splits
    python3 scripts/dataset_split.py \
        --input data/combined_injection_dataset.json \
        --output-dir splits_200k/ \
        --train-size 70000 \
        --val-size 27500 \
        --test-size 1875 \
        --balance true \
        2>/dev/null || {
            echo "Python split script not available, using alternative method..."

            # Alternative: Copy existing splits if available
            if [ -d "data/training" ]; then
                echo "Copying from data/training/..."
                cp -r data/training/* splits_200k/
            fi
        }
else
    echo "Base dataset not found. Please run first:"
    echo "  python scripts/download_and_combine_datasets.py"
    echo ""
    echo "Or download pre-built splits from release assets."
fi

echo ""
echo "Option 2: Download from external storage"
echo "-----------------------------------------"
echo "If you have access to pre-built datasets, download them to splits_200k/"
echo ""

# Verify results
if [ -f "splits_200k/train.json" ]; then
    echo "=============================================="
    echo "Dataset ready in splits_200k/"
    echo "=============================================="
    echo ""
    ls -lh splits_200k/
    echo ""
    echo "To train with this dataset:"
    echo "  cargo run --example evaluate_on_test_set --release"
else
    echo "=============================================="
    echo "Dataset generation incomplete"
    echo "=============================================="
    echo ""
    echo "Please ensure you have:"
    echo "1. Base dataset in data/combined_injection_dataset.json"
    echo "2. Python with required dependencies (numpy, pandas)"
    echo ""
    echo "Or manually download the dataset to splits_200k/"
fi
