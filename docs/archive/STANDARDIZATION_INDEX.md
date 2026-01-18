# Naming Standardization - Complete Index

## Overview

This standardization project fixes inconsistent naming across the JailGuard codebase (Phase 6.3, Phase 5d, phase6_, etc.) to use professional semantic versioning.

**Timeline**: 6.5 hours
**Risk**: Low
**Benefit**: High (ongoing clarity + maintainability)

---

## Phase 6 Naming Issues

### Current Problems

- **Phase 6.3** vs **Phase 6.1** (confusing decimal notation)
- **Phase 5d** (letter suffix, inconsistent with Phase 6)
- **phase6_binary** (snake_case files)
- **Phase6BinaryNetwork** (PascalCase types)
- Mix of naming styles throughout documentation
- No clear deprecation markers
- No migration guide for users

### Solution

Standardize to semantic versioning:
- **v1.0-baseline** (Phase 5d, deprecated)
- **v1.1-neural** (Phase 6.3, current)
- **v1.2+** (future versions)

---

## Planning Documents (In Order)

### 1. STANDARDIZATION_IMPACT_SUMMARY.md (Start Here)
**What**: Before/after comparison and impact analysis
**Length**: 1,500+ lines
**Purpose**: Understand why this matters
**Read time**: 10-15 minutes
**Key sections**:
- The Problem (current confusion)
- The Solution (how it will look)
- Comparison table
- Cost-benefit analysis
- Why it matters

**👉 Read this first to understand the impact**

---

### 2. NAMING_QUICK_REFERENCE.md (Use During Execution)
**What**: Quick lookup tables and commands
**Length**: 600+ lines
**Purpose**: Reference while making changes
**Key sections**:
- At-a-glance mapping (one-page summary)
- File changes, type changes, documentation changes
- Import changes examples
- Version numbering reference
- Search & replace commands
- Verification checklist

**👉 Use this while executing the plan**

---

### 3. NAMING_STANDARDIZATION_PLAN.md (Detailed Plan)
**What**: Complete 5-phase implementation plan
**Length**: 3,500+ lines
**Purpose**: Know exactly what to do
**Key sections**:
- Problem statement with specific issues
- Proposed solution details
- Naming convention guide
- Phase 1: Source Code (2 hours)
- Phase 2: Examples (1 hour)
- Phase 3: Documentation (2 hours)
- Phase 4: Configuration (0.5 hours)
- Phase 5: Verification (1 hour)
- Migration map
- Deprecation strategy
- Risk assessment
- Success criteria

**👉 Read this before starting execution**

---

## Task Tracking

### 26 Tasks (Organized by Phase)

**Using TodoWrite for tracking:**
```
NAMING STANDARDIZATION: Phase 1 - Source Code (6 tasks)
NAMING STANDARDIZATION: Phase 2 - Examples (4 tasks)
NAMING STANDARDIZATION: Phase 3 - Documentation (7 tasks)
NAMING STANDARDIZATION: Phase 4 - Configuration (3 tasks)
NAMING STANDARDIZATION: Phase 5 - Verification (6 tasks)
```

**Mark each task as:**
- ⏳ `pending` - Not started
- 🔄 `in_progress` - Currently working on
- ✅ `completed` - Done

**Update status as you progress through each phase**

---

## Quick Start Checklist

### Before Starting (30 minutes)
- [ ] Read STANDARDIZATION_IMPACT_SUMMARY.md (understand why)
- [ ] Read NAMING_STANDARDIZATION_PLAN.md (understand how)
- [ ] Keep NAMING_QUICK_REFERENCE.md open (use for reference)
- [ ] Get approval on naming scheme
- [ ] Create feature branch: `git checkout -b refactor/naming-standardization`

### Phase 1: Source Code (2 hours)
- [ ] Task 1.1: Rename 4 source files
- [ ] Task 1.2: Update module exports
- [ ] Task 1.3: Rename 10 struct types
- [ ] Task 1.4: Update test imports
- [ ] Task 1.5: Verify compilation
- [ ] Task 1.6: Run `cargo check --all`

### Phase 2: Examples (1 hour)
- [ ] Task 2.1: Rename 2 example files
- [ ] Task 2.2: Update imports
- [ ] Task 2.3: Update struct names
- [ ] Task 2.4: Add deprecation markers

### Phase 3: Documentation (2 hours)
- [ ] Task 3.1: Rename 8+ doc files
- [ ] Task 3.2: Update documentation content
- [ ] Task 3.3: Update code comments
- [ ] Task 3.4: Update test documentation
- [ ] Task 3.5: Update code comments
- [ ] Task 3.6: Create MIGRATION_GUIDE.md
- [ ] Task 3.7: Update README.md

### Phase 4: Configuration (30 minutes)
- [ ] Task 4.1: Update Cargo.toml
- [ ] Task 4.2: Review .gitignore
- [ ] Task 4.3: Update CI/CD scripts

### Phase 5: Verification (1 hour)
- [ ] Task 5.1: Run `cargo test --all --release`
- [ ] Task 5.2: Run `cargo clippy --all`
- [ ] Task 5.3: Build docs: `cargo doc --no-deps`
- [ ] Task 5.4: Test examples
- [ ] Task 5.5: Search for old names
- [ ] Task 5.6: Create PR

---

## Key Naming Changes

### At a Glance

```
Old                          →  New
────────────────────────────────────────
phase6_binary_network.rs     →  neural_binary_network.rs
phase6_data.rs               →  neural_data_loader.rs
phase6_trainer.rs            →  neural_trainer.rs

Phase6BinaryNetwork          →  NeuralBinaryNetwork
Phase6DataLoader             →  NeuralDataLoader
Phase6Trainer                →  NeuralTrainer

PHASE_6_VERIFICATION.md      →  NEURAL_NETWORK_VERIFICATION.md
RUNNING_GUIDE.md             →  GETTING_STARTED.md
ARCHITECTURE.md              →  NEURAL_NETWORK_ARCHITECTURE.md

Phase 6.3                    →  Neural Network v1.1 (Binary)
Phase 6.1                    →  Neural Network v1.0 (Multi-task)
Phase 5d                     →  Baseline Detector v1.0
```

**Full details in NAMING_QUICK_REFERENCE.md**

---

## Execution Timeline

### Option 1: Single Developer (1-2 days)
```
Day 1 Morning:   Phase 1 + Phase 2 (3 hours)
Day 1 Afternoon: Phase 3 (2 hours) + Phase 4 (0.5 hours)
Day 2 Morning:   Phase 5 (1 hour)
Day 2 Afternoon: PR review + merge
```

### Option 2: Multiple Developers (6-8 hours)
```
Dev 1:          Phase 1 (2 hours) + Phase 2 (1 hour)
Dev 2 (parallel): Phase 3 (2 hours)
Dev 1:          Phase 4 (0.5 hours)
Dev 1 or 2:     Phase 5 (1 hour)
```

---

## Risk & Mitigation

### Risk Level: LOW

| Component | Risk | Mitigation |
|-----------|------|-----------|
| File renames | Low | Use `git mv` (preserves history) |
| Type renames | Low | Compiler verifies all references |
| Module exports | Medium | Full test suite + review |
| Deprecation | Medium | Clear warnings + migration guide |
| Version bump | Medium | Release notes included |

### Mitigation Strategy
✅ Use feature branch for all changes
✅ Run full tests before/after
✅ Comprehensive PR for review
✅ Migration guide for users
✅ Clear deprecation warnings

---

## Success Criteria

After standardization, you should be able to:

- ✅ Find files: Search for `neural_` (all neural network files)
- ✅ Find types: All start with `Neural` (clear ownership)
- ✅ Know version: "v1.1" (clear semantic versioning)
- ✅ Know deprecation: `#[deprecated]` attribute (compiler warns)
- ✅ Find examples: `examples/train_neural_*.rs` (clear naming)
- ✅ Find docs: `NEURAL_*.md` (consistent prefixes)
- ✅ Migrate code: `MIGRATION_GUIDE.md` (clear path)
- ✅ No warnings: No deprecation about name changes

---

## Commands to Implement Changes

### Search for old names
```bash
# Find all old names
grep -r "phase6_\|Phase6\|Phase 6" src/ examples/ *.md

# Find files to rename
find src -name "phase6_*"
find examples -name "phase6_*"
```

### Rename files (Git-aware)
```bash
# Use git mv to preserve history
git mv src/training/phase6_binary_network.rs src/training/neural_binary_network.rs
git mv src/training/phase6_data.rs src/training/neural_data_loader.rs
git mv src/training/phase6_trainer.rs src/training/neural_trainer.rs
git mv src/training/phase6_multitask_network.rs src/training/neural_multitask_network.rs
git mv examples/phase6_binary_train_full.rs examples/train_neural_binary.rs
git mv examples/phase6_train_full.rs examples/train_neural_multitask.rs
```

### Find and replace
```bash
# In code files
find src -name "*.rs" -exec sed -i 's/Phase6BinaryNetwork/NeuralBinaryNetwork/g' {} \;

# In documentation
find . -name "*.md" -exec sed -i 's/Phase 6\.3/Neural Network v1.1/g' {} \;
```

### Verify
```bash
# Test compilation
cargo check --all

# Run tests
cargo test --all --release

# Lint
cargo clippy --all

# Format
cargo fmt --all
```

---

## Related Files

### Documentation Files (Project Root)
- ✅ NAMING_STANDARDIZATION_PLAN.md (3,500+ lines)
- ✅ NAMING_QUICK_REFERENCE.md (600+ lines)
- ✅ STANDARDIZATION_IMPACT_SUMMARY.md (1,500+ lines)
- ✅ STANDARDIZATION_INDEX.md (this file)

### Phase 6 Documentation (Existing)
- NEURAL_NETWORK_VERIFICATION.md (formerly PHASE_6_VERIFICATION.md)
- GETTING_STARTED.md (formerly RUNNING_GUIDE.md)
- NEURAL_NETWORK_ARCHITECTURE.md (formerly ARCHITECTURE.md)

### To Be Created
- MIGRATION_GUIDE.md (for users updating code)

---

## Next Steps

### Immediate (Approval)
1. Review STANDARDIZATION_IMPACT_SUMMARY.md (why)
2. Review NAMING_STANDARDIZATION_PLAN.md (how)
3. Approve naming scheme
4. ✅ Create feature branch

### Short-term (Execution)
1. Execute Phase 1-5 (6.5 hours)
2. Create pull request
3. Get team review
4. Merge to main

### Release
1. Tag as v1.1.0
2. Update release notes
3. Announce standardization

### Long-term (Maintenance)
1. Keep naming consistent for future versions
2. Use semantic versioning (v1.2, v2.0, etc.)
3. Clear deprecation path for changes

---

## Document Relationship Map

```
STANDARDIZATION_INDEX.md (You are here)
│
├─→ STANDARDIZATION_IMPACT_SUMMARY.md
│   (Read first - understand the problem & solution)
│
├─→ NAMING_QUICK_REFERENCE.md
│   (Use during execution - quick lookup tables)
│
└─→ NAMING_STANDARDIZATION_PLAN.md
    (Read before execution - detailed 5-phase plan)
    ├─ Phase 1: Source Code Refactoring (2 hours)
    ├─ Phase 2: Example Updates (1 hour)
    ├─ Phase 3: Documentation Updates (2 hours)
    ├─ Phase 4: Configuration Updates (0.5 hours)
    └─ Phase 5: Verification & PR (1 hour)

Task Tracking (TodoWrite)
└─ 26 specific tasks to mark as in_progress/completed
```

---

## Quick Facts

| Metric | Value |
|--------|-------|
| **Total Effort** | 6.5 hours |
| **Risk Level** | Low |
| **Files to Rename** | 16+ |
| **Type Names to Change** | 10 |
| **Documentation Files** | 8+ |
| **Tasks to Track** | 26 |
| **Long-term Benefit** | High |
| **Backward Compatibility** | Yes (with deprecation warnings) |

---

## Summary

### Current State ❌
- Inconsistent naming (Phase 6.3, Phase 5d, phase6_)
- No clear version numbers
- No deprecation markers
- No migration guide

### Future State ✅
- Consistent semantic versioning (v1.0, v1.1, v2.0)
- Clear component naming (Neural, Baseline)
- Explicit deprecation with compiler warnings
- Migration guide for users

### Getting There
1. Read the plans (30 minutes)
2. Execute 5 phases (6.5 hours)
3. Review & merge (1 hour)
4. Total: 8 hours

---

**Status**: Ready to execute
**Approval needed**: Naming scheme + resource allocation
**Contact**: See NAMING_STANDARDIZATION_PLAN.md for detailed Q&A

