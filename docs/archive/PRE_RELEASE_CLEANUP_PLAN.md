# Pre-Release Cleanup Plan for JailGuard v1.1.0

**Status**: Ready for execution
**Total Estimated Effort**: 8-10 hours
**Target Completion**: Before v1.1.0 release
**Priority Levels**: Critical (do before release), Important (should do), Nice-to-have (future)

---

## Executive Summary

JailGuard codebase is **production-ready** with excellent architecture and 430+ tests. However, the repository has accumulated **documentation sprawl** (87 MD files) and **example redundancy** (50 similar examples) from iterative development and phase-based work tracking.

**Action Required**: Archive historical documentation and consolidate examples to prepare for professional release.

---

## Phase 1: Critical Cleanup (MUST DO BEFORE RELEASE)

### 1.1 Remove Deprecated Examples
**Priority**: 🔴 Critical
**Effort**: 15 minutes
**Checklist**:

- [ ] Delete `examples/train_neural_multitask.rs` (deprecated, causes warnings)
- [ ] Verify no other code imports this example
- [ ] Update .gitignore if needed

**Commands**:
```bash
rm examples/train_neural_multitask.rs
grep -r "train_neural_multitask" . --include="*.rs" --include="*.md"  # Should be 0 results
```

**Why**:
- Uses `NeuralMultitaskNetwork` (deprecated)
- Generates compiler deprecation warnings
- Confuses users about which example to follow

---

### 1.2 Document Production-Ready Status
**Priority**: 🔴 Critical
**Effort**: 1 hour
**Deliverables**:

- [ ] Create `PRODUCTION_READY.md` (500 words)
  - What's production-ready: v1.1-neural with 96.58% accuracy
  - What's experimental: Agent module, collection module
  - Deprecations: v1.0-baseline, NeuralMultitaskNetwork
  - Migration path from v1.0 to v1.1
  - Known limitations

**Template**:
```markdown
# Production Ready Status - JailGuard v1.1

## ✅ Production Ready Components

### Detection Engine (v1.1-neural)
- **Accuracy**: 96.58% on 15,185 test samples
- **Latency**: <5ms CPU, <2ms GPU
- **Status**: Recommended for production use

### Unified API (JailGuard struct)
- 6-layer defense system
- All components tested
- Thread-safe with parking_lot

## ⚠️ Experimental Components

### Agent Module (src/agent/)
- PPO and DQN implementations
- Not integrated with unified API
- Status: Research/reference only

### Collection Module (src/collection/)
- Data collectors for various sources
- Not used in detector
- Status: Dataset curation tool only

## 🚫 Deprecated Components

### v1.0-baseline (Phase 5d)
- Feature-based detector, 84.62% accuracy
- **Status**: DEPRECATED
- **Migration**: Use v1.1-neural instead
- **Timeline**: Will be removed in v2.0

### NeuralMultitaskNetwork
- Multi-task learning approach
- **Status**: DEPRECATED (convergence issues)
- **Replacement**: Use NeuralBinaryNetwork
- **Timeline**: Will be removed in v2.0
```

**Location**: Add to root; link from README

---

### 1.3 Clean Up Cargo.toml Issues
**Priority**: 🔴 Critical
**Effort**: 30 minutes

**Issue**: `deny.toml` has `avoid-breaking-exported-api = false`

**Action**:
- [ ] Review `deny.toml` setting
- [ ] Set `avoid-breaking-exported-api = true` for v1.1.0
- [ ] Run `cargo deny check` to verify

**Why**: Prevents accidental API breaking changes before v2.0

---

### 1.4 Run Final Quality Checks
**Priority**: 🔴 Critical
**Effort**: 30 minutes

**Checklist**:
```bash
# Check formatting
[ ] cargo fmt --all -- --check

# Check linting (should be clean)
[ ] cargo clippy --all --all-features -- -D warnings

# Run full test suite
[ ] cargo test --all --release

# Build documentation
[ ] RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# Check security
[ ] cargo audit

# Check dependencies
[ ] cargo deny check

# Check unused deps
[ ] cargo udeps --all-targets  # Optional, install: cargo install cargo-udeps
```

**Expected Results**:
- ✅ Formatting: Pass (no changes needed)
- ✅ Clippy: Pass (except deprecation warnings, which are OK)
- ✅ Tests: 12/12 passing
- ✅ Docs: Build successfully
- ✅ Audit: 0 vulnerabilities
- ✅ Deny: 0 violations

---

## Phase 2: Important Cleanup (SHOULD DO FOR v1.1)

### 2.1 Archive Historical Documentation
**Priority**: 🟡 Important
**Effort**: 2-3 hours
**Scope**: 70+ MD files → 40

**Step 1: Create Archive Directory**
```bash
mkdir -p docs/archive
```

**Step 2: Archive Phase-Based Documentation**
Move these files to `docs/archive/`:
```
PHASE_1_*.md (5 files)
PHASE_2_*.md (8 files)
PHASE_3_*.md (3 files)
PHASE_4_*.md (6 files)
PHASE_5_*.md (2 files)
PHASE_6_*.md (2 files)
PHASE_8_PLAN.md
```

**Why**:
- These document completed development phases
- Not needed for production use
- Can be referenced for historical context but shouldn't clutter root

**Step 3: Archive Session Documentation**
Move to `docs/archive/`:
```
SESSION_*.md (8 files)
PROJECT_STATUS_*.md (5 files)
CONTINUATION_SESSION_*.md (4 files)
```

**Why**: Work-in-progress session notes, not user documentation

**Step 4: Consolidate Research Documentation**
Keep in root but consolidate:

Current:
- ACCURACY_BOOST_RESEARCH_2026.md
- SOTA_RESEARCH_2024_2026.md
- JAILGUARD_COMPETITIVE_ANALYSIS.md
- RESEARCH_PHASE_1.md
- RESEARCH_VISION_PHASE_2.md

Action:
- [ ] Consolidate into single `RESEARCH_AND_BENCHMARKING.md`
- [ ] Archive individual research files to `docs/archive/`
- [ ] Link from README under "Research" section

**Commands**:
```bash
# Create archive structure
mkdir -p docs/archive/phases docs/archive/sessions docs/archive/research

# Archive phase docs
mv PHASE_*.md docs/archive/phases/

# Archive session docs
mv SESSION_*.md docs/archive/sessions/
mv CONTINUATION_*.md docs/archive/sessions/
mv PROJECT_STATUS_*.md docs/archive/sessions/

# Review research docs
# - Consolidate into RESEARCH_AND_BENCHMARKING.md
# - Archive originals
```

**Deliverable**: Create `docs/archive/README.md`:
```markdown
# Archive Documentation

This directory contains historical documentation from JailGuard development.

## Contents

### Phases
- `phases/` - Documentation from development phases (Phase 1-8)
- Use for historical context only

### Sessions
- `sessions/` - Session notes and work-in-progress documentation
- Use for development history reference

### Research
- `research/` - Archived research files
- See main `RESEARCH_AND_BENCHMARKING.md` for consolidated information
```

---

### 2.2 Archive Redundant Dataset Documentation
**Priority**: 🟡 Important
**Effort**: 1 hour

**Current Files**:
- DATASET_CATALOG.md (comprehensive)
- DATASET_QUICK_REFERENCE.md
- DATASET_ROADMAP_2026.md
- DATASET_EXTENSION_STRATEGY.md
- DATASETS.md

**Action**:
- [ ] Keep: `DATASET_CATALOG.md` (main reference)
- [ ] Keep: `DATASET_QUICK_REFERENCE.md` (link from GETTING_STARTED.md)
- [ ] Archive: DATASET_ROADMAP_2026.md, DATASET_EXTENSION_STRATEGY.md, DATASETS.md
  ```bash
  mv DATASET_ROADMAP_2026.md DATASET_EXTENSION_STRATEGY.md DATASETS.md docs/archive/
  ```

---

### 2.3 Consolidate Quick References
**Priority**: 🟡 Important
**Effort**: 1.5 hours

**Current Files**:
- NAMING_QUICK_REFERENCE.md (comprehensive)
- QUICK_START.md
- QUICK_REFERENCE_ACCURACY_BOOST.md

**Action**:
- [ ] Consolidate into `QUICK_REFERENCE.md` (single file)
  - Quick start instructions
  - Common commands
  - Examples guide
  - Links to detailed docs
- [ ] Archive: QUICK_START.md, QUICK_REFERENCE_ACCURACY_BOOST.md

**Structure**:
```markdown
# Quick Reference

## Getting Started (5 minutes)
[Quick start content from QUICK_START.md]

## Common Commands
[cargo commands, running examples]

## Naming Quick Reference
[Links to NAMING_QUICK_REFERENCE.md or embed key items]

## Dataset Quick Reference
[Links to DATASET_QUICK_REFERENCE.md or embed key items]

## Examples Guide
[Which example to use for what purpose]
```

---

### 2.4 Create Release Notes for v1.1.0
**Priority**: 🟡 Important
**Effort**: 1 hour

**File**: `RELEASE_v1.1.0.md`

**Content**:
```markdown
# JailGuard v1.1.0 Release Notes

## 🎉 Major Features

### Neural Network Detector (v1.1-neural)
- **Accuracy**: 96.58% (up from 84.62% in v1.0)
- **Architecture**: 2-layer neural network with dropout
- **Performance**: <5ms CPU latency, <2ms GPU
- **Real evaluation**: 15,185 samples, 80/10/10 split

### Unified API Improvements
- 6-layer defense-in-depth architecture
- Improved error handling
- Thread-safe with parking_lot

### Documentation Standardization
- Semantic versioning (v1.0-baseline, v1.1-neural)
- Comprehensive migration guide
- Clear deprecation markers

## 📋 Changes

### New
- Neural network binary classifier (96.58% accuracy)
- Migration guide for v1.0 → v1.1 users
- PRODUCTION_READY.md status document

### Improved
- Cleaner documentation structure
- Consolidated examples
- Better naming conventions

### Deprecated
- v1.0-baseline (feature-based, 84.62% accuracy)
- NeuralMultitaskNetwork (convergence issues)
- Phase-based naming (Phase 6.3, Phase 5d)

### Fixed
- [List any bug fixes if applicable]

## 🔄 Migration Guide

If you're using v1.0-baseline:
1. See MIGRATION_GUIDE.md for detailed upgrade instructions
2. Update imports from `Phase6*` to `Neural*`
3. Run tests to verify compatibility
4. Deploy with confidence in 96.58% accuracy

## 📚 Documentation

- **[GETTING_STARTED.md](GETTING_STARTED.md)** - Quick start guide
- **[NEURAL_NETWORK_ARCHITECTURE.md](NEURAL_NETWORK_ARCHITECTURE.md)** - Technical details
- **[PRODUCTION_READY.md](PRODUCTION_READY.md)** - Production status & limitations
- **[MIGRATION_GUIDE.md](MIGRATION_GUIDE.md)** - Upgrade from v1.0

## 🧪 Testing

- 430+ tests passing ✅
- Multi-platform CI/CD (Ubuntu, macOS, Windows)
- Security audit: 0 vulnerabilities ✅
- Performance benchmarks: Meeting targets ✅

## 📦 Installation

```bash
cargo add jailguard@1.1.0
```

## 🙏 Acknowledgments

Thanks to the community for feedback and contributions.

## 📄 License

Dual-licensed under MIT OR Apache-2.0
```

---

## Phase 3: Nice-to-Have Improvements (POST-v1.1)

### 3.1 Archive Redundant Examples
**Priority**: 🟢 Nice-to-have
**Effort**: 2-3 hours
**Scope**: Reduce from 50 to ~10 core examples

**Keep** (Core Examples):
- `train_neural_binary.rs` - Main training example
- `production_inference.rs` - Production usage
- `ensemble_demo.rs` - Ensemble voting
- `api_server.rs` - REST API
- `evaluate_detector.rs` - Model evaluation
- `spotlighting_demo.rs` - Spotlighting layer
- `custom_config.rs` - Configuration options

**Archive** (Variants, 40+ files):
- `train_*.rs` variants (keep train_neural_binary, archive others)
- `fine_tune_stage*.rs` (archive all 7 stages)
- `*_embeddings*.rs` variants (archive variants)
- `ensemble_*` variants (keep ensemble_demo, archive others)
- `phase*.rs` files (archive all)

**Commands**:
```bash
mkdir -p examples/archive
# Archive files
mv examples/train_*.rs examples/archive/  # CAREFUL: Keep train_neural_binary
mv examples/fine_tune*.rs examples/archive/
mv examples/*embeddings*.rs examples/archive/
# ... etc
```

### 3.2 Simplify Training Module Documentation
**Priority**: 🟢 Nice-to-have
**Effort**: 2 hours

**Action**:
- [ ] Create `TRAINING_GUIDE.md` (comprehensive guide)
  - Which training approach to use
  - How to train the binary classifier
  - How to do fine-tuning
  - How to implement adversarial training
  - How to use online learning
- [ ] Link from README
- [ ] Reference instead of exploring 24 submodules

---

### 3.3 Document Agent & Collection Modules
**Priority**: 🟢 Nice-to-have
**Effort**: 1 hour

**Action**:
- [ ] Create `EXPERIMENTAL_FEATURES.md`
  - Agent module (PPO/DQN) - Status and usage
  - Collection module - Dataset curation tool
  - External models - GenTelShield, ProtectAI integration
  - Make clear these are not integrated with main detector

---

### 3.4 Clarify Detector Module Versions
**Priority**: 🟢 Nice-to-have
**Effort**: 1 hour

**Action**:
- [ ] Document in README which detector to use:
  - Use: `JailGuard` unified API (recommended)
  - Use: `NeuralBinaryNetwork` for custom training
  - Reference: Other detectors for benchmarking
- [ ] Create deprecation plan for old detector versions

---

## Phase 4: Continuous Improvement

### 4.1 Documentation Linting
**Setup**: Add to CI/CD
```bash
# Check for broken links (optional)
cargo install cargo-deadlinks
cargo deadlinks --check-fragments

# Check for duplicate sections
# (manual review or custom script)
```

### 4.2 Regular Archive Reviews
**Frequency**: Quarterly
**Action**: Review `docs/archive/` for content to move back to main or delete

### 4.3 Example Maintenance Policy
**Policy**:
- Keep only examples that are actively tested
- Archive experimental examples
- Update examples with each major version

---

## Execution Checklist

### Pre-Execution Verification
- [ ] Current branch is `main` and up-to-date
- [ ] All tests passing: `cargo test --all --release`
- [ ] No uncommitted changes: `git status` is clean
- [ ] Create feature branch: `git checkout -b cleanup/pre-release-v1.1`

### Phase 1 Execution (Critical - ~2.5 hours)
- [ ] 1.1: Remove deprecated examples
- [ ] 1.2: Create PRODUCTION_READY.md
- [ ] 1.3: Configure deny.toml
- [ ] 1.4: Run final quality checks

### Phase 2 Execution (Important - ~5 hours)
- [ ] 2.1: Archive historical documentation
- [ ] 2.2: Archive dataset documentation
- [ ] 2.3: Consolidate quick references
- [ ] 2.4: Create RELEASE_v1.1.0.md

### Phase 3 & 4 (Post-Release)
- [ ] Schedule for after v1.1.0 release
- [ ] Archive examples (low priority)
- [ ] Document experimental features
- [ ] Clarify detector versions

### Post-Execution Verification
- [ ] All tests still passing
- [ ] Documentation builds without warnings
- [ ] No broken links in README
- [ ] Cargo check clean

### Create Pull Request
```bash
git add -A
git commit -m "cleanup: Pre-release repository cleanup for v1.1.0

- Remove deprecated NeuralMultitaskNetwork example
- Archive phase-based and session documentation
- Consolidate research and quick reference docs
- Add PRODUCTION_READY.md for status clarity
- Create RELEASE_v1.1.0.md release notes
- Configure deny.toml to prevent API breaking changes
- Improve documentation structure for production release"

git push origin cleanup/pre-release-v1.1
```

---

## Success Criteria

After cleanup, the repository should:

✅ **Documentation**
- [ ] ~40 root-level MD files (down from 87)
- [ ] Clear navigation in README
- [ ] Links to archived docs in docs/archive/
- [ ] No broken internal links

✅ **Examples**
- [ ] No deprecated examples that cause warnings
- [ ] 7-10 core examples with clear purposes
- [ ] All examples in examples/ are actively maintained

✅ **Code Quality**
- [ ] All tests passing (430+)
- [ ] No compiler warnings
- [ ] No security vulnerabilities
- [ ] All dependencies checked

✅ **Release Readiness**
- [ ] PRODUCTION_READY.md defines what's production-ready
- [ ] RELEASE_v1.1.0.md documents changes
- [ ] MIGRATION_GUIDE.md helps v1.0 users upgrade
- [ ] README clearly points to main documentation

---

## Appendix: File-by-File Cleanup List

### Archive to docs/archive/phases/ (15 files)
```
PHASE_1_COMPLETION.md
PHASE_1_COMPLETION_SUMMARY.md
PHASE_1_EVALUATION_PLAN.md
PHASE_1_EVALUATION_RESULTS.md
PHASE_1_STAGE5_CALIBRATION.md
PHASE_1_STATUS_CHECKLIST.md
PHASE_1_TRAINING_VALIDATION.md
PHASE_2_DEBERTA.md
PHASE_2_IMPLEMENTATION_PLAN.md
PHASE_2_WEEK1_IMPLEMENTATION.md
PHASE_2_WEEK2_COMPLETION.md
PHASE_2_WEEK3_COMPLETION.md
PHASE_2_WEEK4_COMPLETION.md
PHASE_3_IMPLEMENTATION_OUTLINE.md
PHASE_3_TRAINING_SUMMARY.md
PHASE_4_COMPLETE_SUMMARY.md
PHASE_4_FINAL_SUMMARY.md
PHASE_4_SEMANTIC_EMBEDDINGS.md
PHASE_4B_ADAM_OPTIMIZER.md
PHASE_4C_ADVERSARIAL_TRAINING.md
PHASE_4D_EARLY_STOPPING.md
PHASE_5_PROGRESS.md
PHASE_6_FINAL_SUMMARY.md
PHASE_8_PLAN.md
```

### Archive to docs/archive/sessions/ (17 files)
```
SESSION_COMPLETION_JANUARY_17.md
SESSION_EXTENDED_JANUARY_17_FINAL.md
SESSION_JANUARY_17_FINAL_ENSEMBLE_INTEGRATION.md
SESSION_SUMMARY_PHASE_1_IMPLEMENTATION.md
CONTINUATION_SESSION_*.md (4 files)
PROJECT_STATUS.md
PROJECT_STATUS_JANUARY_2026.md
CURRENT_PROGRESS.md
IMPLEMENTATION_PROGRESS.md
IMPLEMENTATION_QUICK_START.md
NEXT_PHASE_GUIDE.md
IMPLEMENTATION_ROADMAP_2026.md
```

### Consolidate/Keep (10 files)
```
README.md (UPDATE with new structure) ✅
GETTING_STARTED.md (Keep) ✅
MIGRATION_GUIDE.md (Keep) ✅
NEURAL_NETWORK_ARCHITECTURE.md (Keep) ✅
NEURAL_NETWORK_VERIFICATION.md (Keep) ✅
BASELINE_DETECTOR_STATUS.md (Keep) ✅
PRODUCTION_READY.md (CREATE NEW) ✅
RELEASE_v1.1.0.md (CREATE NEW) ✅
QUICK_REFERENCE.md (CONSOLIDATE 3 files) ✅
RESEARCH_AND_BENCHMARKING.md (CONSOLIDATE) ✅
DATASET_CATALOG.md (Keep) ✅
DATASET_QUICK_REFERENCE.md (Keep) ✅
CONFIGURATION_GUIDE.md (Keep) ✅
LIBRARY_INTEGRATION_GUIDE.md (Keep) ✅
STANDARDIZATION_INDEX.md (Archive or keep?)
NAMING_STANDARDIZATION_PLAN.md (Archive)
```

### Delete (Safe to delete after archiving)
```
DATASETS.md (covered in DATASET_CATALOG.md)
DATASET_ROADMAP_2026.md
DATASET_EXTENSION_STRATEGY.md
QUICK_START.md (content in QUICK_REFERENCE.md)
QUICK_REFERENCE_ACCURACY_BOOST.md
ACCURACY_BOOST_RESEARCH_2026.md (content in RESEARCH_AND_BENCHMARKING.md)
README_ACCURACY_RESEARCH.md
SOTA_RESEARCH_2024_2026.md (keep key findings in RESEARCH_AND_BENCHMARKING.md)
```

### Delete Example Files (Safe to delete)
```
examples/train_neural_multitask.rs (DEPRECATED)
# Plus 40+ other variant examples (archive to examples/archive/)
```

---

## Timeline

| Phase | Task | Duration | Deadline |
|-------|------|----------|----------|
| 1 | Critical cleanup (remove deprecated code, final QA) | 2.5 hours | **Before release** |
| 2 | Important cleanup (archive docs, consolidate) | 5 hours | **Before release** |
| Release | Tag v1.1.0 and announce | - | **After phases 1-2** |
| 3 | Nice-to-have improvements | 5-6 hours | **Post-v1.1.0** |
| 4 | Continuous improvement setup | 2 hours | **Post-v1.1.0** |

**Total Pre-Release**: 7.5 hours (Phases 1-2)
**Total Full**: 15-16 hours (all phases)

---

## Questions & Answers

**Q: Should we delete old documentation or archive it?**
A: Archive in `docs/archive/` for historical reference. Never delete - context may be useful later.

**Q: What about the Agent module (PPO/DQN)?**
A: Document as experimental. Can either deprecate or integrate in future versions.

**Q: Should we keep all 50 examples or just 7-10?**
A: Keep 7-10 core examples. Archive variants for reference. Reduces cognitive load for new users.

**Q: When should we remove deprecated components completely?**
A: v2.0 release. For v1.1, keep with clear deprecation warnings and migration guides.

**Q: How do we ensure docs don't proliferate again?**
A: Establish guidelines: (1) Archive old session docs, (2) Consolidate research findings, (3) Test examples regularly

---

## Contact & Next Steps

- **Status**: Ready for execution
- **Approval needed**: Review this plan before cleanup begins
- **Questions**: Ask before starting Phase 1

Ready to proceed with Phase 1 (critical cleanup) ✅
