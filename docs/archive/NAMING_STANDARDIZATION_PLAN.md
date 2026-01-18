# JailGuard Naming Standardization Plan

## Problem Statement

Current naming is inconsistent across codebase and documentation:

### Issues Identified

1. **Inconsistent Phase Notation**
   - Phase 5d (letter suffix)
   - Phase 6.1, Phase 6.3 (decimal notation)
   - Mix of formats confuses readers

2. **Inconsistent File/Module Naming**
   - `phase6_binary_network.rs` (snake_case with version)
   - `Phase6BinaryNetwork` (PascalCase in code)
   - Documentation: `Phase 6.3`, `Phase6`, `phase 6`

3. **Inconsistent Documentation References**
   - File names: `PHASE_6_VERIFICATION.md`, `PHASE_5_PROGRESS.md`
   - Content: "Phase 5d", "Phase 6.3", "phase6_binary"
   - Comments in code: Mix of all above

4. **Lack of Clear Component Names**
   - Phase 6 has 3 sub-components (multi-task, data loader, trainer)
   - Phase 6.3 is "binary network" but name doesn't clarify it's the main component
   - No clear naming for deprecated vs. active components

---

## Proposed Solution: Semantic Versioning

### New Naming Scheme

```
COMPONENT: JailGuard Prompt Injection Detector

Versions:
├─ v1.0-baseline (Phase 5d) ← Feature-based, 84.62%, Deprecated
│  └─ src/detection/baseline_detector.rs
│
└─ v1.1-neural (Phase 6) ← Neural network, 96.58%, Current
   ├─ v1.1-neural:multitask (Phase 6.1) ← Deprecated
   │  └─ src/training/neural_multitask_network.rs
   ├─ v1.1-neural:infrastructure (Phase 6.2) ← Active
   │  ├─ src/training/neural_data_loader.rs
   │  └─ src/training/neural_trainer.rs
   └─ v1.1-neural:binary (Phase 6.3) ← Current Best
      └─ src/training/neural_binary_network.rs
```

### Naming Convention

#### File Names (Rust source)
```rust
// OLD → NEW
phase6_binary_network.rs → neural_binary_network.rs
phase6_multitask_network.rs → neural_multitask_network.rs
phase6_data.rs → neural_data_loader.rs
phase6_trainer.rs → neural_trainer.rs
```

#### Struct/Type Names (Rust code)
```rust
// OLD → NEW
Phase6BinaryNetwork → NeuralBinaryNetwork
Phase6DataLoader → NeuralDataLoader
Phase6Trainer → NeuralTrainer
Phase6Metrics → NeuralTrainingMetrics
Phase6MultiTaskNetwork → NeuralMultitaskNetwork (deprecated)
```

#### Example File Names
```
// OLD → NEW
examples/phase6_binary_train_full.rs → examples/train_neural_binary.rs
examples/phase6_train_full.rs → examples/train_neural_multitask.rs (deprecated)
```

#### Documentation File Names
```
// OLD → NEW
PHASE_6_VERIFICATION.md → NEURAL_NETWORK_VERIFICATION.md
PHASE_5_PROGRESS.md → BASELINE_DETECTOR_STATUS.md
RUNNING_GUIDE.md → GETTING_STARTED.md
ARCHITECTURE.md → NEURAL_NETWORK_ARCHITECTURE.md
```

#### In-Code Comments and Docs
```rust
// OLD
/// Phase 6.3 binary classification network
/// Trains the Phase6BinaryNetwork on the complete 15,185 sample dataset

// NEW
/// Neural Network Binary Classifier (v1.1)
/// Trains the NeuralBinaryNetwork on the complete 15,185 sample dataset
```

#### Module Organization
```rust
// OLD: src/training/mod.rs
pub mod phase6_binary_network;
pub mod phase6_data;
pub mod phase6_trainer;

// NEW: src/training/mod.rs
pub mod neural_binary_network;
pub mod neural_data_loader;
pub mod neural_trainer;
```

---

## Migration Map

### Source Code Files

| Old Name | New Name | Type | Status |
|----------|----------|------|--------|
| `src/training/phase6_binary_network.rs` | `src/training/neural_binary_network.rs` | Active | Rename |
| `src/training/phase6_multitask_network.rs` | `src/training/neural_multitask_network.rs` | Deprecated | Rename + Mark deprecated |
| `src/training/phase6_data.rs` | `src/training/neural_data_loader.rs` | Active | Rename |
| `src/training/phase6_trainer.rs` | `src/training/neural_trainer.rs` | Active | Rename |

### Rust Type Names

| Old Name | New Name | Visibility | Status |
|----------|----------|------------|--------|
| `Phase6BinaryNetwork` | `NeuralBinaryNetwork` | pub | Rename |
| `Phase6DataLoader` | `NeuralDataLoader` | pub | Rename |
| `Phase6Trainer` | `NeuralTrainer` | pub | Rename |
| `Phase6TrainerConfig` | `NeuralTrainerConfig` | pub | Rename |
| `Phase6Metrics` | `NeuralTrainingMetrics` | pub | Rename |
| `Phase6MultiTaskNetwork` | `NeuralMultitaskNetwork` | pub | Rename + Deprecate |
| `EmbeddingSample` | `NeuralEmbeddingSample` | pub | Rename |
| `ForwardCache` | `NeuralForwardCache` | pub | Rename |

### Examples

| Old Name | New Name | Type | Status |
|----------|----------|------|--------|
| `examples/phase6_binary_train_full.rs` | `examples/train_neural_binary.rs` | Active | Rename |
| `examples/phase6_train_full.rs` | `examples/train_neural_multitask.rs` | Deprecated | Rename |

### Documentation Files

| Old Name | New Name | Location | Status |
|----------|----------|----------|--------|
| `PHASE_6_VERIFICATION.md` | `NEURAL_NETWORK_VERIFICATION.md` | Root | Rename |
| `PHASE_5_PROGRESS.md` | `BASELINE_DETECTOR_STATUS.md` | Root | Rename |
| `RUNNING_GUIDE.md` | `GETTING_STARTED.md` | Root | Rename |
| `ARCHITECTURE.md` | `NEURAL_NETWORK_ARCHITECTURE.md` | Root | Rename |
| `PHASE_6_STATUS_REPORT.txt` | `NEURAL_NETWORK_STATUS_REPORT.txt` | /tmp | Rename |
| `PHASE_6_FINAL_SUMMARY.md` | `NEURAL_NETWORK_FINAL_SUMMARY.md` | /tmp | Rename |

---

## Implementation Plan

### Phase 1: Source Code Refactoring (2 hours)

**Task 1.1: Rename Files**
```bash
# In src/training/
mv phase6_binary_network.rs neural_binary_network.rs
mv phase6_multitask_network.rs neural_multitask_network.rs
mv phase6_data.rs neural_data_loader.rs
mv phase6_trainer.rs neural_trainer.rs

# In examples/
mv examples/phase6_binary_train_full.rs examples/train_neural_binary.rs
mv examples/phase6_train_full.rs examples/train_neural_multitask.rs
```

**Task 1.2: Update Module Exports**
- Edit `src/training/mod.rs`
  - Change: `pub mod phase6_binary_network;` → `pub mod neural_binary_network;`
  - Change: `pub mod phase6_data;` → `pub mod neural_data_loader;`
  - Change: `pub mod phase6_trainer;` → `pub mod neural_trainer;`
  - Update all `pub use` statements

- Edit `src/lib.rs`
  - Update all imports from `phase6_*` to `neural_*`

**Task 1.3: Rename Struct Names**
- Edit all files to rename:
  - `Phase6BinaryNetwork` → `NeuralBinaryNetwork`
  - `Phase6DataLoader` → `NeuralDataLoader`
  - `Phase6Trainer` → `NeuralTrainer`
  - `Phase6TrainerConfig` → `NeuralTrainerConfig`
  - `Phase6Metrics` → `NeuralTrainingMetrics`
  - `Phase6MultiTaskNetwork` → `NeuralMultitaskNetwork`
  - `EmbeddingSample` → `NeuralEmbeddingSample`
  - `ForwardCache` → `NeuralForwardCache`
  - Similar for other types (LRSchedule → NeuralLRSchedule, etc.)

**Task 1.4: Update Tests**
- Rename test files to match new module names
- Update all `use` statements in tests
- Update all struct references in tests

**Task 1.5: Verify Compilation**
```bash
cargo check --lib
cargo check --examples
cargo test --lib
```

### Phase 2: Example Updates (1 hour)

**Task 2.1: Update Examples**
- Edit `examples/train_neural_binary.rs`
  - Update imports: `use jailguard::training::NeuralBinaryNetwork;`
  - Update struct names throughout
  - Update documentation comments

- Edit `examples/train_neural_multitask.rs`
  - Mark as deprecated at top of file
  - Update imports and struct names
  - Add deprecation notice in output

**Task 2.2: Mark Deprecated Components**
```rust
// In neural_multitask_network.rs
#[deprecated(
    since = "1.1.0",
    note = "Use NeuralBinaryNetwork instead - multi-task approach has convergence issues"
)]
pub struct NeuralMultitaskNetwork { ... }
```

### Phase 3: Documentation Updates (2 hours)

**Task 3.1: Rename Documentation Files**
```bash
mv PHASE_6_VERIFICATION.md NEURAL_NETWORK_VERIFICATION.md
mv PHASE_5_PROGRESS.md BASELINE_DETECTOR_STATUS.md
mv RUNNING_GUIDE.md GETTING_STARTED.md
mv ARCHITECTURE.md NEURAL_NETWORK_ARCHITECTURE.md
mv /tmp/PHASE_6_STATUS_REPORT.txt /tmp/NEURAL_NETWORK_STATUS_REPORT.txt
mv /tmp/PHASE_6_FINAL_SUMMARY.md /tmp/NEURAL_NETWORK_FINAL_SUMMARY.md
```

**Task 3.2: Update Documentation Content**
- Search and replace in all `.md` files:
  - `Phase 6.3` → `Neural Network v1.1 (Binary)`
  - `Phase 6.1` → `Neural Network v1.0 (Multi-task, Deprecated)`
  - `Phase 6` → `Neural Network`
  - `Phase 5d` → `Baseline Detector`
  - `phase6_` → `neural_`
  - `Phase6` → `Neural`

- Update README.md:
  - Update all references to old names
  - Update feature descriptions
  - Update examples section

**Task 3.3: Update Code Comments**
- In all `.rs` files, update doc comments:
  ```rust
  // OLD
  //! Phase 6.3 Binary Classification Training
  //! Trains the Phase6BinaryNetwork on the complete 15,185 sample dataset.

  // NEW
  //! Neural Network Binary Classifier (v1.1)
  //! Trains the NeuralBinaryNetwork on the complete 15,185 sample dataset.
  ```

- Update inline comments similarly

**Task 3.4: Update Tests Documentation**
- Update test documentation to reflect new names
- Update test output messages

### Phase 4: Configuration and Metadata Updates (30 minutes)

**Task 4.1: Cargo.toml Updates**
- If version is bumped: Update to 1.1.0
- Update any example names in `[[example]]` sections

**Task 4.2: Git Integration**
- Update `.gitignore` if needed
- Update any CI/CD scripts that reference old names

**Task 4.3: Create Migration Guide**
- Document old → new naming for users
- Provide code examples for updating user code
- Include deprecation timeline

### Phase 5: Verification (1 hour)

**Task 5.1: Full Test Suite**
```bash
cargo test --all --release
cargo clippy --all
cargo fmt --all --check
```

**Task 5.2: Documentation Build**
```bash
cargo doc --no-deps --open
```

**Task 5.3: Example Verification**
```bash
cargo run --example train_neural_binary --release
cargo run --example train_neural_multitask --release
```

**Task 5.4: Cross-Reference Check**
- Verify all old references removed
- Verify all new names consistent
- Verify documentation links updated

---

## Naming Rationale

### Why "neural_*" instead of "phase6_*"?

1. **Descriptive**: Clearly indicates it's neural network code
2. **Semantic**: Not tied to arbitrary phase numbers
3. **Scalable**: If we add phase 7, doesn't need renaming
4. **Professional**: Better for library users
5. **Version-agnostic**: Code name independent of versioning

### Why Version Numbers?

```
Baseline → v1.0-baseline (stable, feature-complete)
Neural v1 → v1.1-neural (improved, backward compatible API)
Neural v2 → v2.0-neural (if major changes)
```

### Component Naming

- `NeuralBinaryNetwork` - Clear that it does binary classification
- `NeuralDataLoader` - Clear that it loads data for neural training
- `NeuralTrainer` - Clear that it trains neural networks
- `NeuralMultitaskNetwork` - Clear it's the (deprecated) multi-task version

---

## Deprecation Strategy

### Deprecation Timeline

**Immediate (v1.1 release)**
- Mark `NeuralMultitaskNetwork` as deprecated
- Mark `train_neural_multitask.rs` example as deprecated
- Add migration guide to docs

**Short-term (v1.2)**
- Phase 6.1 components still functional
- Print warnings when used

**Long-term (v2.0)**
- Remove deprecated Phase 6.1 code entirely
- Clean up codebase

### Migration Path for Users

```rust
// OLD CODE (still works, deprecated)
use jailguard::training::Phase6MultiTaskNetwork;
let mut network = Phase6MultiTaskNetwork::new(0.01);

// NEW CODE (recommended)
use jailguard::training::NeuralMultitaskNetwork;
let mut network = NeuralMultitaskNetwork::new(0.01);

// BETTER: Use binary network instead
use jailguard::training::NeuralBinaryNetwork;
let mut network = NeuralBinaryNetwork::new(0.01);
```

---

## Files to Update: Complete List

### Source Code (4 files)
- [ ] `src/training/phase6_binary_network.rs` → `src/training/neural_binary_network.rs`
- [ ] `src/training/phase6_multitask_network.rs` → `src/training/neural_multitask_network.rs`
- [ ] `src/training/phase6_data.rs` → `src/training/neural_data_loader.rs`
- [ ] `src/training/phase6_trainer.rs` → `src/training/neural_trainer.rs`

### Module Files (2 files)
- [ ] `src/training/mod.rs` (module exports and re-exports)
- [ ] `src/lib.rs` (if it imports these modules)

### Examples (2 files)
- [ ] `examples/phase6_binary_train_full.rs` → `examples/train_neural_binary.rs`
- [ ] `examples/phase6_train_full.rs` → `examples/train_neural_multitask.rs`

### Documentation (8+ files)
- [ ] `README.md`
- [ ] `PHASE_6_VERIFICATION.md` → `NEURAL_NETWORK_VERIFICATION.md`
- [ ] `PHASE_5_PROGRESS.md` → `BASELINE_DETECTOR_STATUS.md`
- [ ] `RUNNING_GUIDE.md` → `GETTING_STARTED.md`
- [ ] `ARCHITECTURE.md` → `NEURAL_NETWORK_ARCHITECTURE.md`
- [ ] All other phase documentation files
- [ ] In `/tmp/`: Move temp files
- [ ] Create `MIGRATION_GUIDE.md`

### Configuration Files (2 files)
- [ ] `Cargo.toml`
- [ ] `.gitignore` (if needed)

### Tests (implicitly, through source code renaming)
- Test files automatically updated when source renamed
- Test function names updated
- Test imports updated

---

## Timeline Estimate

| Phase | Tasks | Estimated Time | Complexity |
|-------|-------|-----------------|-----------|
| 1 | Code refactoring | 2 hours | Medium |
| 2 | Example updates | 1 hour | Low |
| 3 | Documentation | 2 hours | Low-Medium |
| 4 | Configuration | 30 min | Low |
| 5 | Verification | 1 hour | Medium |
| **TOTAL** | **All tasks** | **6.5 hours** | **Medium** |

---

## Risk Assessment

### Low Risk
- File renames (git mv preserves history)
- Type name changes (compile-time verification)
- Documentation updates (reviewed before release)

### Medium Risk
- Module exports (might break external imports)
- Deprecation markers (need clear messaging)

### Mitigation
- Branch for refactoring (review before merge)
- Comprehensive test suite runs before release
- Deprecation guide included in release notes

---

## Success Criteria

✅ All old names removed from code
✅ All new names consistent across codebase
✅ All tests passing
✅ Documentation builds without errors
✅ Examples run successfully
✅ No broken links in docs
✅ Migration guide provided
✅ Version bumped appropriately

---

## Next Steps

1. **Get approval** on naming scheme
2. **Create feature branch** `refactor/naming-standardization`
3. **Execute Phase 1-5** in order
4. **Create pull request** with all changes
5. **Review and merge** with comprehensive testing

