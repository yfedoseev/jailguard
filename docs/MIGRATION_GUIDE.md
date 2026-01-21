# Migration Guide: Updating to Neural Network v1.1

This guide helps you migrate from the old Phase 6 naming conventions to the standardized Neural Network v1.1 naming.

## Overview

JailGuard has adopted semantic versioning for clarity and maintainability:

- **v1.0-baseline**: Feature-based detector (Phase 5d) - 84.62% accuracy
- **v1.1-neural**: Neural network detector (Phase 6.3) - 99.62% accuracy ✅ RECOMMENDED

## What Changed

### Old vs New Type Names

| Old (Phase 6) | New (Neural v1.1) | Status |
|---|---|---|
| `Phase6BinaryNetwork` | `NeuralBinaryNetwork` | ✅ Recommended, use this |
| `Phase6MultiTaskNetwork` | `NeuralMultitaskNetwork` | ⚠️ Deprecated, converges poorly |
| `Phase6DataLoader` | `NeuralDataLoader` | ✅ Updated |
| `Phase6Trainer` | `NeuralTrainer` | ✅ Updated |
| `Phase6TrainerConfig` | `NeuralTrainerConfig` | ✅ Updated |
| `Phase6Metrics` | `NeuralTrainingMetrics` | ✅ Updated |
| `EmbeddingSample` | `NeuralEmbeddingSample` | ✅ Updated |
| `LRSchedule` | `NeuralLRSchedule` | ✅ Updated |

### Old vs New File Names

| Old | New | Location |
|---|---|---|
| `phase6_binary_network.rs` | `neural_binary_network.rs` | src/training/ |
| `phase6_multitask_network.rs` | `neural_multitask_network.rs` | src/training/ |
| `phase6_data.rs` | `neural_data_loader.rs` | src/training/ |
| `phase6_trainer.rs` | `neural_trainer.rs` | src/training/ |
| `phase6_binary_train_full.rs` | `train_neural_binary.rs` | examples/ |
| `phase6_train_full.rs` | `train_neural_multitask.rs` | examples/ |

### Old vs New Documentation Files

| Old | New |
|---|---|
| `PHASE_6_VERIFICATION.md` | `NEURAL_NETWORK_VERIFICATION.md` |
| `PHASE_5_PROGRESS.md` | `BASELINE_DETECTOR_STATUS.md` |
| `RUNNING_GUIDE.md` | `GETTING_STARTED.md` |
| `ARCHITECTURE.md` | `NEURAL_NETWORK_ARCHITECTURE.md` |

## Updating Your Code

### Step 1: Update Imports

**Before:**
```rust
use jailguard::training::{
    Phase6BinaryNetwork, Phase6DataLoader, Phase6Trainer, Phase6TrainerConfig,
};
```

**After:**
```rust
use jailguard::training::{
    NeuralBinaryNetwork, NeuralDataLoader, NeuralTrainer, NeuralTrainerConfig,
};
```

### Step 2: Update Type Names

**Before:**
```rust
let mut network = Phase6BinaryNetwork::new(0.01);
let config = Phase6TrainerConfig::default();
let loader = Phase6DataLoader::load_from_file("data.json")?;
let mut trainer = Phase6Trainer::new(config);
```

**After:**
```rust
let mut network = NeuralBinaryNetwork::new(0.01);
let config = NeuralTrainerConfig::default();
let loader = NeuralDataLoader::load_from_file("data.json")?;
let mut trainer = NeuralTrainer::new(config);
```

### Step 3: Update Training Loop

**Before:**
```rust
for epoch in 0..num_epochs {
    let metrics = trainer.train_epoch(&loader, epoch)?;
    println!("Epoch {}: {:.4}", epoch, metrics.train_loss);
}
```

**After:**
```rust
for epoch in 0..num_epochs {
    let metrics = trainer.train_epoch(&loader, epoch)?;
    println!("Epoch {}: {:.4}", epoch, metrics.train_loss);
}
```

No change to the training loop itself - just the type names.

## Deprecation Warnings

When using the old multi-task approach, you'll see:

```
warning: use of deprecated struct `NeuralMultitaskNetwork`
  --> src/main.rs:10:5
   |
   | let net = NeuralMultitaskNetwork::new(0.01);
   |           ^^^^^^^^^^^^^^^^^^^^^
   |
   = note: since 1.1.0: Multi-task approach has convergence issues.
     Use NeuralBinaryNetwork instead. See MIGRATION_GUIDE.md for details.
```

### Why the Multi-Task Version is Deprecated

The multi-task learning approach (trying to predict binary classification AND attack type AND semantic similarity simultaneously) has known issues:

1. **Poor convergence**: The three loss terms compete with each other
2. **Lower accuracy**: Achieves only ~90% vs 99.62% for binary approach
3. **Training instability**: Loss can oscillate unpredictably
4. **Not recommended for production**: Use binary classification instead

### Recommended: Use Binary Classification Instead

The binary classification approach (NeuralBinaryNetwork) is:
- ✅ More stable (achieves 99.62% accuracy)
- ✅ Faster to train
- ✅ Simpler to debug
- ✅ Better for production use

**Migration path:**

1. Replace `Phase6MultiTaskNetwork` with `NeuralBinaryNetwork`
2. Remove attack type and semantic similarity prediction code
3. Focus on binary classification (injection vs benign)
4. Run tests to verify accuracy

## Version Numbers

Going forward, versions will use semantic versioning:

```
vMAJOR.MINOR-COMPONENT

v1.0-baseline  → Phase 5d (deprecated)
v1.1-neural    → Neural Network (current, recommended)
v1.2-enhanced  → Future improvements
v2.0-transformer → Next major version

Naming pattern:
- v{version}-{component}
- All source files: neural_*
- All types: Neural*
- Clear deprecation markers with #[deprecated]
```

## Documentation

Updated documentation is available in:

- **GETTING_STARTED.md** - How to use the detector (replaces RUNNING_GUIDE.md)
- **NEURAL_NETWORK_ARCHITECTURE.md** - Technical details (replaces ARCHITECTURE.md)
- **NEURAL_NETWORK_VERIFICATION.md** - Proof of real model (replaces PHASE_6_VERIFICATION.md)
- **BASELINE_DETECTOR_STATUS.md** - Previous version info (replaces PHASE_5_PROGRESS.md)

## Quick Reference

### Running Examples

**Train the binary classifier:**
```bash
cargo run --example train_neural_binary --release
```

**Train the multi-task version (deprecated):**
```bash
cargo run --example train_neural_multitask --release
```

### Checking Your Code

Find all old references:
```bash
# Search for old names
grep -r "Phase6\|phase6_\|Phase 6" src/ examples/

# Should return empty (or only deprecation markers)
```

## FAQ

### Q: Can I still use Phase6MultiTaskNetwork?

**A:** Yes, but with deprecation warnings. It's marked as `#[deprecated]` and the compiler will warn you. For new projects, use `NeuralBinaryNetwork` instead.

### Q: Do I need to reprocess my training data?

**A:** No. The embedding format hasn't changed. All existing data files work with the new types.

### Q: What about saved models?

**A:** Models trained with the old code can still be used. The internal weight matrix format is identical between `Phase6BinaryNetwork` and `NeuralBinaryNetwork` - it's just a rename.

### Q: Is there backward compatibility?

**A:** Partial. The module structure changed, so imports need updating. But the underlying data structures and algorithms are identical.

### Q: When will the old names be removed?

**A:** Planned for v2.0. There will be a deprecation period (v1.1 - v1.2+) before complete removal.

## Getting Help

If you encounter issues:

1. Check the updated documentation files
2. Look at the examples: `examples/train_neural_binary.rs`
3. Review the architecture: `NEURAL_NETWORK_ARCHITECTURE.md`
4. Search for your struct/function name in the renamed files

## Summary

- **4 files renamed** in src/training/
- **2 examples renamed** in examples/
- **4 documentation files renamed** in project root
- **10 struct/type names updated**
- **Deprecation warnings for v1.0-baseline**
- **Binary classification (v1.1) is recommended**

Migration typically takes **5-10 minutes** for most projects.
