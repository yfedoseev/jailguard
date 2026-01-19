# Archived Examples

Historical examples from JailGuard development. These examples are preserved for reference but have been superseded by the core examples in the parent directory.

## Archive Structure

### **training_variants/** - Experimental Training Approaches (13 files)

Progressive experiments with different training methods during development.

**Files:**
- `train_with_backprop.rs` - Backpropagation experiments
- `train_with_backprop_proper.rs` - Improved backpropagation
- `train_with_backprop_real.rs` - Real backpropagation implementation
- `train_with_gradients.rs` - Gradient-based training
- `train_gradient_descent.rs` - Gradient descent experiments
- `train_with_minilm_embeddings.rs` - MiniLM embedding training
- `train_with_real_minilm_embeddings.rs` - Real MiniLM setup
- `train_minilm_proper.rs` - Improved MiniLM training
- `train_semantic_embeddings.rs` - Semantic embedding training
- `train_with_weight_updates.rs` - Weight update experiments
- `train_on_labeled_dataset.rs` - Labeled dataset training
- `real_training.rs` - Real training implementation
- `train_with_early_stopping.rs` - Early stopping experiments

**Status:** Archived - Superseded by `train_neural_binary.rs` (96.58% accuracy)

**Why archived:** Development iterations exploring different approaches. The final `train_neural_binary.rs` incorporates lessons from all these variants.

---

### **fine_tuning/** - Progressive Fine-Tuning Stages (7 files)

Seven-stage fine-tuning progression showing incremental improvements.

**Files:**
- `fine_tune_stage1.rs` through `fine_tune_stage7.rs`

**Status:** Archived - Superseded by single-stage `train_neural_binary.rs`

**Why archived:** Multi-stage approach was replaced by direct training with better regularization.

---

### **embeddings/** - Embedding Generation Methods (4 files)

Different approaches to computing and pre-computing embeddings.

**Files:**
- `generate_embeddings_native.rs` - Native ONNX-based generation
- `generate_embeddings_fast.rs` - Fast GPU-accelerated generation
- `precompute_embeddings.rs` - CPU-based precomputation
- `precompute_embeddings_fast.rs` - Fast precomputation variant

**Status:** Archived - Functionality now in Python scripts

**Why archived:** Embedding generation moved to `scripts/enhance_embeddings.py` for better Python integration with transformers library.

**Current approach:** Use Python to generate embeddings, load pre-computed embeddings in Rust training/inference.

---

### **utilities/** - Phase-Specific Utility Examples (5 files)

Development utilities for specific development phases.

**Files:**
- `phase1_dataset_extension.rs` - Phase 1 dataset operations
- `phase1_pretrained_embeddings.rs` - Phase 1 embedding setup
- `phase1_pretrained_integration.rs` - Phase 1 integration tests
- `debug_heuristics.rs` - Heuristic debugging utility
- `test_paraphrases.rs` - Paraphrase testing utility

**Status:** Archived - Phase-specific development utilities

**Why archived:** Specific to development phases; functionality integrated into main pipeline.

---

### **advanced/** - Advanced Features & Deployment (4 files)

Advanced features and deployment examples.

**Files:**
- `collection_daemon.rs` - Community data collection framework
- `deploy_collection_pipeline.rs` - Deployment pipeline for collection
- `performance_optimization_demo.rs` - Performance optimization examples
- `ensemble_stage6_integration.rs` - Fine-tuning stage integration

**Status:**
- Collection daemon: 🔬 Experimental (see docs/EXPERIMENTAL_FEATURES.md)
- Performance optimization: Research/optimization focused
- Ensemble integration: Superseded by `unified_api_ensemble_demo.rs`

**Why archived:** Research features and experimental integrations not yet production-ready.

---

### **deprecated/** - Deprecated Approaches (6 files)

Approaches that have been deprecated in favor of better alternatives.

**Files:**
- `train_multitask.rs` - Multi-task learning (deprecated since v1.1)
- `train_transformer.rs` - Transformer-based training (deprecated)
- `train_transformer_fast.rs` - Fast transformer training (deprecated)
- `train_ensemble_classifier.rs` - Ensemble classifier approach
- `train_minilm_expanded_dataset.rs` - Expanded dataset variant
- `train.rs` - Generic training skeleton

**Status:** ❌ Deprecated - Do not use in new projects

**Why archived:** Superseded by more effective approaches:
- Multi-task learning → `train_neural_binary.rs` (binary classifier, 96.58% accuracy)
- Transformer-based → MiniLM embeddings + neural network (faster, better accuracy)
- Old ensemble → `unified_api_ensemble_demo.rs` (96-98% accuracy)

**Migration:** See [../../MIGRATION_GUIDE.md](../../MIGRATION_GUIDE.md) for upgrade instructions.

---

## Current Examples

For current recommended examples, see [../README.md](../README.md).

---

## Why These Were Archived

The original 49 examples contained:
- **13 training variants** - Different approaches to the same problem
- **7 fine-tuning stages** - Progressive multi-stage approach
- **4 embedding methods** - Different embedding generation techniques
- **5 utilities** - Phase-specific development tools
- **4 advanced features** - Research and experimental features
- **6 deprecated approaches** - Superseded by v1.1 approach
- **4 core examples** - Kept in parent directory

**Result:** Reduced from 49 to 10 core examples while preserving all historical code for reference.

---

## Using Archived Examples

Archived examples still compile and run, but:

1. **They may not represent best practices** - They show experimental approaches
2. **They may use deprecated APIs** - Some types have been renamed
3. **They may have lower performance** - Superseded by better algorithms
4. **They should not be used in production** - Use core examples instead

If you want to understand how a specific approach works:
1. Look in the appropriate subdirectory
2. Read the comments in the source code
3. Check [../../MIGRATION_GUIDE.md](../../MIGRATION_GUIDE.md) for any API changes
4. Run with `cargo run --example <name>`

---

## How to Explore Archived Examples

### Want to understand an approach?
```bash
# Read the example
cat examples/archive/training_variants/train_with_backprop.rs

# Run it
cargo run --example train_with_backprop
```

### Want to see fine-tuning progression?
```bash
# See Stage 1-7 progression
ls -1 examples/archive/fine_tuning/

# Run any stage
cargo run --example fine_tune_stage3
```

### Want to compare embedding methods?
```bash
# See all embedding approaches
ls -1 examples/archive/embeddings/

# Run and compare
cargo run --example generate_embeddings_native
cargo run --example generate_embeddings_fast
```

---

## Learning from Archives

These examples are excellent for learning:

1. **Training approaches** - See `training_variants/` for different optimizers, learning rates, regularization techniques
2. **Architecture evolution** - See progression from simple to complex models
3. **Performance optimization** - See different engineering approaches
4. **API migration** - See how types and APIs changed from v1.0 to v1.1

---

## Compilation

All archived examples still compile:
```bash
cargo build --examples
```

To run a specific archived example:
```bash
cargo run --example <filename_without_extension>
```

---

## Questions?

- See [../README.md](../README.md) for current recommended examples
- See [../../GETTING_STARTED.md](../../GETTING_STARTED.md) for usage guide
- See [../../MIGRATION_GUIDE.md](../../MIGRATION_GUIDE.md) for v1.0 → v1.1 upgrade
- See [../../docs/EXPERIMENTAL_FEATURES.md](../../docs/EXPERIMENTAL_FEATURES.md) for research features
