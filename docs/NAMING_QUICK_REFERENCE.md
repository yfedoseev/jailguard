# Naming Standardization - Quick Reference

## At-a-Glance Mapping

### Source Files

```
OLD                                 NEW
────────────────────────────────────────────────────────────
phase6_binary_network.rs         → neural_binary_network.rs
phase6_multitask_network.rs      → neural_multitask_network.rs
phase6_data.rs                   → neural_data_loader.rs
phase6_trainer.rs                → neural_trainer.rs
```

### Example Files

```
OLD                                 NEW
────────────────────────────────────────────────────────────
phase6_binary_train_full.rs      → train_neural_binary.rs
phase6_train_full.rs             → train_neural_multitask.rs (deprecated)
```

### Documentation Files

```
OLD                                      NEW
────────────────────────────────────────────────────────────
PHASE_6_VERIFICATION.md              → NEURAL_NETWORK_VERIFICATION.md
PHASE_5_PROGRESS.md                  → BASELINE_DETECTOR_STATUS.md
RUNNING_GUIDE.md                     → GETTING_STARTED.md
ARCHITECTURE.md                      → NEURAL_NETWORK_ARCHITECTURE.md
PHASE_6_STATUS_REPORT.txt            → NEURAL_NETWORK_STATUS_REPORT.txt
PHASE_6_FINAL_SUMMARY.md             → NEURAL_NETWORK_FINAL_SUMMARY.md
(NEW) MIGRATION_GUIDE.md             ← Create this
```

### Rust Types

```
OLD                                  NEW
────────────────────────────────────────────────────────────
Phase6BinaryNetwork              → NeuralBinaryNetwork
Phase6MultiTaskNetwork           → NeuralMultitaskNetwork (deprecate)
Phase6DataLoader                 → NeuralDataLoader
Phase6Trainer                    → NeuralTrainer
Phase6TrainerConfig              → NeuralTrainerConfig
Phase6Metrics                    → NeuralTrainingMetrics
EmbeddingSample                  → NeuralEmbeddingSample
ForwardCache                     → NeuralForwardCache
LRSchedule                       → NeuralLRSchedule
```

### Text References

```
OLD                                  NEW
────────────────────────────────────────────────────────────
Phase 6.3                        → Neural Network v1.1 (Binary)
Phase 6.1                        → Neural Network v1.0 (Multi-task, deprecated)
Phase 6                          → Neural Network
Phase 5d                         → Baseline Detector
phase6_                          → neural_
Phase6                           → Neural
```

---

## Import Changes

### User Code Updates

```rust
// OLD
use jailguard::training::Phase6BinaryNetwork;
use jailguard::training::Phase6DataLoader;

// NEW
use jailguard::training::NeuralBinaryNetwork;
use jailguard::training::NeuralDataLoader;
```

### Module Declarations

```rust
// OLD (src/training/mod.rs)
pub mod phase6_binary_network;
pub mod phase6_data;
pub mod phase6_trainer;

pub use phase6_binary_network::Phase6BinaryNetwork;
pub use phase6_data::Phase6DataLoader;

// NEW (src/training/mod.rs)
pub mod neural_binary_network;
pub mod neural_data_loader;
pub mod neural_trainer;

pub use neural_binary_network::NeuralBinaryNetwork;
pub use neural_data_loader::NeuralDataLoader;
```

---

## Document Content Updates

### Title Updates

```
OLD: Phase 6 Neural Network - Running Guide
NEW: Getting Started with Neural Network Detector

OLD: Phase 6 Summary: Neural Network Training Infrastructure
NEW: Neural Network Training Infrastructure (v1.1)

OLD: How We Achieved 99.62% - ctreate guilde for testing and launching
NEW: How We Achieved 99.62% - Neural Network Training Guide
```

### Content Replacements

| Find | Replace |
|------|---------|
| `Phase 6.3` | `Neural Network v1.1 (Binary)` |
| `Phase 6.1` | `Neural Network v1.0 (Multi-task)` |
| `Phase 6` | `Neural Network` |
| `Phase 5d` | `Baseline Detector` |
| `phase6_` | `neural_` |
| `Phase6` | `Neural` |

### Code Comment Updates

```rust
// OLD
/// Phase 6.3 Binary Classification Training
/// Trains the Phase6BinaryNetwork on the complete 15,185 sample dataset.

// NEW
/// Neural Network Binary Classifier (v1.1)
/// Trains the NeuralBinaryNetwork on the complete 15,185 sample dataset.
```

---

## Version Numbering

### Product Versions

```
v1.0-baseline
├─ Feature-based detector
├─ 84.62% accuracy
├─ ~1 KB model size
└─ DEPRECATED (not recommended for new projects)

v1.1-neural
├─ Neural network detector
├─ 99.62% accuracy
├─ ~500 KB model size
├─ Sub-components:
│  ├─ v1.1-neural:multitask (deprecated)
│  ├─ v1.1-neural:infrastructure (data, training)
│  └─ v1.1-neural:binary (recommended)
└─ CURRENT (recommended)
```

---

## Deprecation Notices

### Code Markers

```rust
#[deprecated(
    since = "1.1.0",
    note = "Use NeuralBinaryNetwork instead - multi-task approach has convergence issues. \
             See MIGRATION_GUIDE.md for details."
)]
pub struct NeuralMultitaskNetwork {
    // ...
}
```

### Documentation Notices

```markdown
> **⚠️  Deprecated**
>
> This document describes the Phase 6.1 (Neural Network v1.0 - Multi-task) approach,
> which is deprecated due to convergence issues.
>
> **Recommended**: Use [Neural Network v1.1 (Binary)](GETTING_STARTED.md) instead.
>
> See [Migration Guide](MIGRATION_GUIDE.md) for updating your code.
```

---

## Examples Reference

### Recommended Examples

- `cargo run --example train_neural_binary --release` ✅ RECOMMENDED
  - Trains the best-performing neural network
  - 99.62% accuracy
  - Full training pipeline

### Deprecated Examples

- `cargo run --example train_neural_multitask --release` ⚠️ DEPRECATED
  - For reference/comparison only
  - Poor convergence
  - Use binary version instead

---

## Search & Replace Commands

### Linux/Mac

```bash
# In source code
find src -type f -name "*.rs" -exec sed -i 's/Phase6BinaryNetwork/NeuralBinaryNetwork/g' {} \;
find src -type f -name "*.rs" -exec sed -i 's/phase6_binary/neural_binary/g' {} \;

# In documentation
find . -name "*.md" -exec sed -i 's/Phase 6\.3/Neural Network v1.1 (Binary)/g' {} \;
find . -name "*.md" -exec sed -i 's/Phase 5d/Baseline Detector/g' {} \;
```

### Windows (PowerShell)

```powershell
# In source code
Get-ChildItem -Path src -Include *.rs -Recurse | ForEach-Object {
    (Get-Content $_) -replace 'Phase6BinaryNetwork', 'NeuralBinaryNetwork' | Set-Content $_
}

# In documentation
Get-ChildItem -Path . -Include *.md -Recurse | ForEach-Object {
    (Get-Content $_) -replace 'Phase 6\.3', 'Neural Network v1.1 (Binary)' | Set-Content $_
}
```

---

## Verification Checklist

After completing all renamings:

- [ ] `cargo check --all` compiles without errors
- [ ] `cargo test --all` passes all tests
- [ ] `cargo clippy --all` has no new warnings
- [ ] `cargo fmt --all` formats correctly
- [ ] `cargo doc --no-deps` builds documentation
- [ ] Examples run: `cargo run --example train_neural_binary`
- [ ] No references to old names in code: `grep -r "phase6_\|Phase6\|Phase 6" src/`
- [ ] No references to old names in docs: `grep -r "Phase 6\|phase6_" *.md`
- [ ] Git history preserved (use `git mv` for file renames)

---

## Quick Commands

### Find Old Names

```bash
# Find all old names in code
grep -r "phase6_\|Phase6\|Phase 6\." src/ examples/

# Find all old names in docs
grep -r "Phase 6\|phase6_\|Phase 5d" *.md

# Count occurrences
grep -r "Phase 6" . --include="*.rs" --include="*.md" | wc -l
```

### Rename Files (git-aware)

```bash
# Use git mv to preserve history
git mv src/training/phase6_binary_network.rs src/training/neural_binary_network.rs
git mv examples/phase6_binary_train_full.rs examples/train_neural_binary.rs
```

### Build and Test

```bash
# Full verification
cargo test --all --release && \
cargo clippy --all && \
cargo fmt --all --check && \
cargo doc --no-deps

# Run examples
cargo run --example train_neural_binary --release
```

---

## File Status

### To Be Renamed

✏️ = Source files
🔧 = Config files
📚 = Documentation

| Status | Count | Category |
|--------|-------|----------|
| ✏️ Source files | 4 | `src/training/*.rs` |
| ✏️ Example files | 2 | `examples/*.rs` |
| 📚 Doc files | 8+ | Root `*.md` files |
| 🔧 Config files | 2 | `Cargo.toml`, etc. |
| **TOTAL** | **16+** | **All types** |

---

## Timeline

| Phase | Duration | Tasks |
|-------|----------|-------|
| 1 | 2 hours | Code refactoring (files + structs) |
| 2 | 1 hour | Examples + deprecation |
| 3 | 2 hours | Documentation updates |
| 4 | 30 min | Configuration + metadata |
| 5 | 1 hour | Testing + verification |
| **TOTAL** | **6.5 hours** | **All phases** |

---

## Next Steps

1. ✅ Review this plan (`NAMING_STANDARDIZATION_PLAN.md`)
2. ✅ Approve naming scheme
3. 🚀 Start execution:
   ```bash
   git checkout -b refactor/naming-standardization
   # Follow Phase 1-5 in order
   ```
4. 📝 Create pull request
5. ✅ Review and merge

