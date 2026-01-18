# Changelog

All notable changes to JailGuard are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-01-18

### ✨ Major Features

#### Neural Network v1.1 Binary Classifier (NEW - RECOMMENDED)
- **Accuracy:** 96.58% on 15,185 sample test set (11.96% improvement over v1.0)
- **Architecture:** 384 → 256 → 128 → 1 (ReLU + Dropout + Sigmoid)
- **Performance:** <5ms GPU, <30ms CPU inference latency
- **Confidence Calibration:** ECE = 0.038 (well-calibrated)

#### Confidence Calibration (NEW)
- Temperature scaling for reliable confidence scores
- Well-calibrated predictions suitable for threshold-based decision making
- Prevents overconfident misclassifications

### 🔧 Improvements

#### Code Quality
- ✅ Comprehensive test suite: 652 tests passing, 0 failures
- ✅ Code formatting: All files pass `cargo fmt`
- ✅ Security linting: Clean `cargo clippy` with -D warnings
- ✅ Documentation: Complete API documentation with rustdoc
- ✅ Dependency audit: All dependencies verified with cargo-deny

#### Naming Standardization
- Renamed Phase-based types to version-based names for clarity:
  - `Phase6BinaryNetwork` → `NeuralBinaryNetwork`
  - `Phase6DataLoader` → `NeuralDataLoader`
  - `Phase6Trainer` → `NeuralTrainer`
  - `Phase6MultiTaskNetwork` → `NeuralMultitaskNetwork` (deprecated)
- Renamed documentation files for consistency
- Deprecated types still compile with warnings for backward compatibility

#### Documentation
- ✅ New: PRODUCTION_READY.md (comprehensive component status)
- ✅ New: NEURAL_NETWORK_VERIFICATION.md (accuracy proof)
- ✅ New: RELEASE_v0.1.0.md (release notes)
- ✅ New: docs/TRAINING_GUIDE.md (850+ lines, complete training guide)
- ✅ New: docs/EXPERIMENTAL_FEATURES.md (research features documentation)
- ✅ New: MIGRATION_GUIDE.md (v1.0 → v1.1 upgrade instructions)
- ✅ Reorganized: All documentation files moved to docs/ folder (95% cleaner root)
- ✅ Examples: Updated all examples to use new v1.1 API

#### Repository Organization
- Archived 40+ historical phase documentation (docs/archive/phases/)
- Archived 8 work session notes (docs/archive/sessions/)
- Archived 10+ research artifacts (docs/archive/research/)
- Archived 39 redundant examples (examples/archive/)
- Consolidated 6 redundant dataset files (67% reduction)
- Result: Clean, professional repository structure

### 🐛 Bug Fixes

#### Gradient Flow Issues
- **Fixed:** Gradient computation in backpropagation layers
- **Impact:** Enables proper training convergence
- **Test:** Added gradient flow validation tests

#### Overfitting Prevention
- **Fixed:** Added dropout regularization (0.2 rate)
- **Improvement:** Test accuracy improved from ~85% to 96.58%
- **Result:** Better generalization on unseen data

#### Learning Rate Scheduling
- **Fixed:** Implemented exponential decay with warmup
- **Improvement:** Smoother convergence, better final accuracy
- **Result:** Consistent training behavior across runs

#### Early Stopping Implementation
- **Fixed:** Proper validation-based stopping criteria
- **Impact:** Prevents divergence and overfitting
- **Default:** patience=5 epochs without improvement

### 📦 API Stability

#### Deprecated Components (Removal planned for v2.0.0)

1. **Multi-Task Learning Network**
   - Type: `NeuralMultitaskNetwork`
   - Reason: Gradient conflicts between tasks, lower accuracy
   - Replacement: Use `NeuralBinaryNetwork` (96.58% accuracy)
   - Deprecation marker: `#[deprecated(...)]`

2. **Baseline Detector (v1.0)**
   - Type: `BaselineDetector`
   - Accuracy: 84.62% (deprecated)
   - Replacement: Use `NeuralBinaryNetwork` (96.58% accuracy)
   - Reference: See PRODUCTION_READY.md for comparison

#### Deprecation Path
- All deprecated types still compile
- Compiler warnings point to replacements
- Clear migration path in MIGRATION_GUIDE.md
- Full removal in v2.0.0

### 📊 Performance Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Accuracy | 96.58% | >95% | ✅ |
| Precision | 97.12% | >95% | ✅ |
| Recall | 95.89% | >93% | ✅ |
| F1 Score | 96.49% | >94% | ✅ |
| ECE | 0.038 | <0.05 | ✅ |
| GPU Latency | <5ms | <5ms | ✅ |
| CPU Latency | <30ms | <30ms | ✅ |
| Model Size | 16MB | <20MB | ✅ |

### 🔒 Security

- ✅ Passed `cargo audit` - No known vulnerabilities
- ✅ Passed `cargo deny check` - License compliance verified
- ✅ Clippy: No critical warnings
- ✅ Documentation: All public APIs documented
- ✅ Unsafe code: 4 justified uses, all audited

#### Known Issues (Non-Critical)
- RUSTSEC-2026-0002: lru IterMut soundness (upgrade planned for v1.1.1)
- RUSTSEC-2024-0436: paste unmaintained (no security impact)
- RUSTSEC-2025-0141: bincode unmaintained (no security impact)

### 📚 Documentation

**New Files:**
- docs/TRAINING_GUIDE.md (850+ lines)
- docs/EXPERIMENTAL_FEATURES.md (600+ lines)
- examples/README.md (400+ lines)
- examples/archive/README.md (350+ lines)
- docs/archive/README.md (250+ lines)
- CONTRIBUTING.md
- CODE_OF_CONDUCT.md
- SECURITY.md

**Total new documentation:** 3,450+ lines

---

## [1.0.0] - 2025-12-15

### ✨ Features

- **6-Layer Defense Architecture** - Defense-in-depth protection against prompt injection
  1. Spotlighting - Input boundary marking
  2. Detection - Multi-task threat detection (84.62% accuracy)
  3. Task Tracking - Behavioral drift detection
  4. Privilege Context - Resource access control
  5. Output Validation - Secret detection and sanitization
  6. Behavior Monitoring - Attack campaign detection

- **Baseline Detector** - Feature-based detection (84.62% accuracy)
  - Rule-based heuristics
  - Regex patterns
  - Simple and fast

- **Multi-Task Learning** - Three-task classification
  - Binary injection detection
  - Attack type classification
  - Semantic similarity scoring

- **Confidence Calibration** - Reliable prediction scores

- **Online Learning** - Feedback-based model updates

- **Adversarial Training** - Character, encoding, and paraphrase attacks

- **GPU Support** - WGPU acceleration

### 🔧 Improvements

- Comprehensive test suite (430+ tests)
- Docker and Docker Compose support
- API server with REST endpoints
- Monitoring integration (Prometheus/Grafana)
- Configuration options (lenient, strict, output-only modes)

### 📦 Dependencies

- Burn 0.16+ (deep learning framework)
- Serde (serialization)
- Tokio (async runtime)
- All dependencies audited and verified

---

## Upgrade Guide

### From v1.0.0 to v0.1.0

**Type Renames:**
```rust
// Old (v1.0)
use jailguard::training::Phase6BinaryNetwork;
let detector = Phase6BinaryNetwork::new();

// New (v1.1)
use jailguard::training::NeuralBinaryNetwork;
let detector = NeuralBinaryNetwork::new(0.01);  // learning_rate parameter added
```

**Recommended Upgrade:**
```rust
// Old: 84.62% accuracy
let detector = BaselineDetector::new();

// New: 96.58% accuracy (11.96% improvement!)
let detector = NeuralBinaryNetwork::new(0.01);
```

See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for complete upgrade instructions.

---

## Project History

- **Phase 1** - Pre-trained embedding integration (Dec 2025)
- **Phase 2** - Backpropagation implementation (Dec 2025)
- **Phase 3** - Batch training and multi-task learning (Dec 2025)
- **Phase 4** - Early stopping and optimization (Dec 2025)
- **Phase 5** - Baseline detector development (84.62%) (Dec 2025)
- **Phase 6** - Neural network development (96.58%) (Jan 2026)
- **Phase 7-8** - Fine-tuning and production readiness (Jan 2026)
- **Phase 9** - SOTA validation (Jan 2026)
- **v1.0** - Initial release (Dec 2025)
- **v1.1** - Major improvements and naming standardization (Jan 2026) ← Current

---

## Future Roadmap

### v1.2 (Planned)
- Enhanced adversarial training techniques
- Production-ready feedback learning
- Improved attention tracker integration
- Distributed training support

### v2.0 (Future)
- Remove deprecated v1.0 components
- Agent module production readiness (if validation criteria met)
- Multilingual support
- Enhanced ensemble methods

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Reporting bugs
- Suggesting features
- Contributing code
- Documentation improvements

## License

Licensed under either of:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

---

**[Full commit history](https://github.com/yfedoseev/jailguard/commits/main)**
