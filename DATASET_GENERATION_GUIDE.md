# Dataset Generation Guide - Phase 3-4 Execution

**Status:** Ready to execute
**Expected Time:** 5-7 hours total
**Output:** 200K balanced dataset with embeddings and train/val/test splits

---

## ⚙️ Complete Workflow

The correct order for dataset generation is:

```
Step 1: DOWNLOAD & INTEGRATE DATASETS
  └─ scripts/download_expansion_datasets.py
     ├─ SPML Chatbot Injection (16K samples)
     ├─ JailbreakBench Behaviors (4.3K samples)
     └─ Existing TrustAIRLab/deepset (15K samples)
     └─ Output: 35K+ combined raw data

Step 2: TAXONOMY MAPPING & QUALITY FILTERING
  └─ scripts/taxonomy_integration.py
     ├─ Map all samples to unified 8-class taxonomy
     ├─ Apply 3-tier deduplication (exact, fuzzy, semantic)
     ├─ Quality filters (text length, punctuation, etc.)
     └─ Output: expansion_combined_clean.json (validated)

Step 3: BALANCE & AUGMENT TO 200K
  └─ scripts/balanced_augmentation.py
     ├─ Undersample benign (reduce from 135K to 100K)
     ├─ Oversample minority attack types
     ├─ Pattern-based augmentation (template variations)
     ├─ T5 paraphrase augmentation (semantic preservation)
     └─ Output: expansion_balanced_200k.json (200K samples)

Step 4: GENERATE EMBEDDINGS (384-dimensional)
  └─ scripts/embedding_pipeline.py
     ├─ Batch processing (128 CPU, 512 GPU)
     ├─ Checkpoint every 10K samples (resumable)
     ├─ Model: all-MiniLM-L6-v2
     └─ Output: expansion_balanced_embeddings.json (200K + vectors)
     └─ Time: 16-20 hours (CPU) or 3-4 hours (GPU)

Step 5: CREATE TRAIN/VAL/TEST SPLITS
  └─ scripts/dataset_split.py
     ├─ Stratified splitting (preserve class distribution)
     ├─ 70% training (140K)
     ├─ 15% validation (30K)
     ├─ 15% testing (30K)
     └─ Output:
         ├─ splits_200k/train.json
         ├─ splits_200k/val.json
         ├─ splits_200k/test.json
         └─ splits_200k/combined.json
         └─ splits_200k/split_report.json
```

---

## 📋 Step-by-Step Execution Commands

### Step 1: Download & Integrate Datasets

```bash
# Navigate to project root
cd ~/projects/jailguard

# Download SPML and JailbreakBench datasets
python3 scripts/download_expansion_datasets.py --all

# Expected output:
# - data/expansion/spml_raw.jsonl (16K samples)
# - data/expansion/jailbreakbench_raw.json (4.3K samples)
# - data/expansion/expansion_combined_raw.json (35K+ samples)
```

**What happens:**
- Downloads 16K SPML samples from HuggingFace
- Downloads 4.3K JailbreakBench samples
- Combines with existing 15K data
- Performs 3-tier deduplication
- Applies quality filters
- **Output:** 11,121 valid samples (from 15,185 after filtering)

**Note:** If download URLs fail:
- Script will continue with existing data
- Can still use 15K existing + synthetic augmentation
- Synthetic alone can reach 200K target

---

### Step 2: Taxonomy Integration

```bash
# Map all samples to unified 8-class taxonomy
python3 scripts/taxonomy_integration.py \
  --input data/expansion/expansion_combined_raw.json \
  --output data/expansion/expansion_integrated.json

# Expected output:
# Unified 8-class taxonomy:
# - Benign (0)
# - RolePlay (1)
# - InstructionOverride (2)
# - ContextManipulation (3)
# - OutputManipulation (4)
# - EncodingAttack (5)
# - JailbreakPattern (6)
# - PromptLeaking (7)
```

**What happens:**
- Maps legacy taxonomy to unified 8-class system
- Uses heuristic inference for unlabeled samples
- Validates all samples against Pydantic schema
- Generates taxonomy report
- **Output:** expansion_integrated.json (35K+ classified samples)

---

### Step 3: Balance & Augment to 200K

```bash
# Generate balanced 200K dataset with augmentation
python3 scripts/balanced_augmentation.py \
  --input data/expansion/expansion_integrated.json \
  --output data/expansion/expansion_balanced_200k.json \
  --patterns-and-paraphrase

# This will:
# 1. Run pattern-based augmentation (2-3 hours)
# 2. Run T5 paraphrase augmentation (8-10 hours)
# 3. Balance to target distribution
```

**Target Distribution:**
- **Benign:** 100,000 (50%)
- **RolePlay:** 14,000 (7%)
- **InstructionOverride:** 14,000 (7%)
- **ContextManipulation:** 14,000 (7%)
- **OutputManipulation:** 14,000 (7%)
- **EncodingAttack:** 14,000 (7%)
- **JailbreakPattern:** 14,000 (7%)
- **PromptLeaking:** 14,000 (7%)
- **Total:** 200,000 (100%)

**Options:**
```bash
# Pattern-based only (faster, 2-3 hours)
python3 scripts/balanced_augmentation.py \
  --input data/expansion/expansion_integrated.json \
  --output data/expansion/expansion_balanced_200k.json \
  --patterns-only

# Both pattern and paraphrase (comprehensive, 10-13 hours)
python3 scripts/balanced_augmentation.py \
  --input data/expansion/expansion_integrated.json \
  --output data/expansion/expansion_balanced_200k.json \
  --patterns-and-paraphrase
```

**Output:** expansion_balanced_200k.json (200K samples with metadata)

---

### Step 4: Generate Embeddings (Longest Step!)

```bash
# Generate 384-dimensional embeddings
# CPU version (16-20 hours)
python3 scripts/embedding_pipeline.py \
  --input data/expansion/expansion_balanced_200k.json \
  --output data/expansion/expansion_balanced_embeddings.json \
  --device cpu \
  --batch-size 128

# GPU version (3-4 hours) - if GPU available
python3 scripts/embedding_pipeline.py \
  --input data/expansion/expansion_balanced_200k.json \
  --output data/expansion/expansion_balanced_embeddings.json \
  --device cuda \
  --batch-size 512
```

**Features:**
- Auto-detects GPU availability
- Checkpoints every 10K samples
- Resumable on failure
- Progress tracking with ETA
- Memory-efficient batching

**Checkpoint behavior:**
```bash
# If interrupted, can resume:
python3 scripts/embedding_pipeline.py \
  --input data/expansion/expansion_balanced_200k.json \
  --output data/expansion/expansion_balanced_embeddings.json \
  --resume  # Resumes from last checkpoint
```

**Output:** expansion_balanced_embeddings.json
- 200K samples
- Each with 384-dim embedding vector
- File size: ~156MB

---

### Step 5: Create Train/Val/Test Splits

```bash
# Create stratified train/val/test splits
python3 scripts/dataset_split.py \
  --input data/expansion/expansion_balanced_embeddings.json \
  --output splits_200k \
  --train-ratio 0.70 \
  --val-ratio 0.15 \
  --test-ratio 0.15

# Output files:
# - splits_200k/train.json (140K samples, 70%)
# - splits_200k/val.json (30K samples, 15%)
# - splits_200k/test.json (30K samples, 15%)
# - splits_200k/combined.json (200K all samples)
# - splits_200k/split_report.json (statistics)
```

**Stratification:** Preserves class distribution in each split

Expected split distribution:
```
Training Set (140K):
  - Benign: 70,000
  - RolePlay: 9,800
  - InstructionOverride: 9,800
  - ContextManipulation: 9,800
  - OutputManipulation: 9,800
  - EncodingAttack: 9,800
  - JailbreakPattern: 9,800
  - PromptLeaking: 9,800

Validation Set (30K):
  - Benign: 15,000
  - Each attack type: 2,100

Test Set (30K):
  - Same distribution as validation
```

---

## 📊 Expected Output Structure

After all steps, you'll have:

```
data/
├── expansion/
│   ├── spml_raw.jsonl                      (16K raw)
│   ├── jailbreakbench_raw.json             (4.3K raw)
│   ├── expansion_combined_raw.json         (35K raw)
│   ├── expansion_integrated.json           (35K integrated)
│   ├── expansion_balanced_200k.json        (200K balanced)
│   └── expansion_balanced_embeddings.json  (200K + embeddings)
└── training/
    └── splits_200k/
        ├── train.json                      (140K)
        ├── val.json                        (30K)
        ├── test.json                       (30K)
        ├── combined.json                   (200K all)
        └── split_report.json               (statistics)
```

---

## ⏱️ Timeline Breakdown

| Step | Operation | CPU Time | GPU Time |
|------|-----------|----------|----------|
| 1 | Download & integrate | 10-30 min | 10-30 min |
| 2 | Taxonomy integration | 5-10 min | 5-10 min |
| 3 | Balance & augment | 10-13 hours | 10-13 hours |
| 4 | Embedding generation | 16-20 hours | 3-4 hours |
| 5 | Split datasets | 1-2 min | 1-2 min |
| **TOTAL** | | **26-44 hours** | **13-17 hours** |

**Optimization Tips:**
- GPU makes embedding step 4-6x faster
- Steps 1-3 can run on CPU overnight
- Step 4 (embeddings) is the bottleneck
- Can parallelize if you have multiple machines

---

## 🔄 Alternative: Existing Data Only

If external dataset URLs are inaccessible, use existing 15K + synthetic:

```bash
# Step 1: Skip downloading, use existing data
# (already have data/combined_minilm_embeddings_with_types.json)

# Step 2: Integrate existing taxonomy
python3 scripts/taxonomy_integration.py \
  --input data/combined_minilm_embeddings_with_types.json \
  --output data/expansion/expansion_integrated.json

# Step 3: Balance & augment existing data to 200K
python3 scripts/balanced_augmentation.py \
  --input data/expansion/expansion_integrated.json \
  --output data/expansion/expansion_balanced_200k.json \
  --patterns-and-paraphrase

# Steps 4-5: Same as above
```

This approach:
- Uses existing 15K + synthetic 185K
- Still reaches 200K balanced target
- Maintains distribution across 8 classes
- Preserves baseline accuracy (no regression)

---

## ✅ Quality Validation

Each step includes validation:

**Step 1-2:**
- Check for duplicates
- Validate text length (10-2000 chars)
- Verify taxonomy assignment
- Ensure Pydantic schema compliance

**Step 3:**
- Balance ratios match targets
- Per-class diversity (avoid 100% identical)
- Synthetic quality scoring

**Step 4:**
- Embedding dimensions: 384
- Vector magnitude: ~1.0 (normalized)
- No NaN values

**Step 5:**
- Class distribution preserved in splits
- No overlap between train/val/test
- Total: 200K samples

---

## 🚀 Next: Model Training

Once you have splits_200k/:

```bash
# Train neural model on 200K balanced dataset
cargo run --example train_neural_binary -- \
  --data splits_200k/train.json \
  --val-data splits_200k/val.json \
  --epochs 50

# Produces: predictions on test set
```

Then evaluate with Phase 6 framework:

```bash
# Run comprehensive evaluation
cargo run --example comprehensive_evaluation
# Shows all 4 evaluation dimensions
```

---

## 🐛 Troubleshooting

### Download fails
```
Error: 404 Not Found
→ External URLs might be unavailable
→ Use existing data only (see Alternative section)
```

### Embedding generation slow
```
→ CPU: Expected 16-20 hours
→ GPU: Much faster (3-4 hours if available)
→ Can resume from checkpoint if interrupted
```

### Out of memory
```
→ Reduce batch size: --batch-size 64
→ Process in multiple runs
→ Use GPU if available (more VRAM)
```

### T5 model download slow
```
→ First run downloads ~500MB model
→ Cached after first download
→ Can manually pre-download with:
   python3 -c "from transformers import T5ForConditionalGeneration; \
              T5ForConditionalGeneration.from_pretrained('t5-small')"
```

---

## 📝 Summary

The complete pipeline transforms:
- **Input:** 15K existing + 20K external + synthetic
- **Output:** 200K balanced, clean, embedded dataset
- **Time:** 13-44 hours (GPU much faster)
- **Quality:** Validated at each step

Once complete, you have a production-ready dataset for:
- ✅ Training the 8-class model
- ✅ Evaluating with Phase 6 framework
- ✅ Comparing to SOTA (GenTel-Shield, PromptShield)

---

**Status:** Ready to execute at any time
**Next:** Run Step 1 when ready to start dataset generation
