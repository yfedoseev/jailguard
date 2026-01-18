# Phase 8: ML Fine-tuning & SOTA Integration Plan

## Executive Summary

**Current State**: 82-87% accuracy (heuristics-only)
**Target**: 95%+ accuracy (close gap to GenTel-Shield @ 97.63%)
**Gap to Close**: 10-15 percentage points
**Strategy**: Fine-tune transformer + integrate pre-trained models

---

## Root Cause Analysis: Why Heuristics Cap at 87%

### Fundamental Limitations of Regex Patterns

1. **Semantic Understanding**
   - Pattern: `"Ignore your instructions"` → detected ✅
   - Paraphrase: `"Disregard your prior directives"` → missed ❌
   - Root cause: Regex doesn't understand synonyms

2. **Context Sensitivity**
   - Direct: `"Act as a hacker"` → detected ✅
   - Embedded: `"...and act as a hacker in the simulation..."` → missed ❌
   - Root cause: No contextual analysis

3. **Adversarial Obfuscation**
   - Clean: `"ignore|your|instructions"` with pipes → evades ❌
   - Cause: Word boundaries break on special chars
   - ML solution: Semantic embeddings robust to perturbation

4. **Novel Attack Patterns**
   - Known patterns: detected by curated rules ✅
   - Unknown variations: requires generalization ❌
   - ML solution: Learned representations generalize better

### Why Pre-trained Models Reach 97%+

**GenTel-Shield (97.63%)**:
- Fine-tuned on 100k+ jailbreak examples
- Transformer-based (semantic understanding)
- Multi-task learning (7-way attack classification)
- Adversarial training (30% augmented examples)

**ProtectAI DeBERTa (94.2%)**:
- RoBERTa-base with disentangled attention
- Capture long-range dependencies
- Debias attention weights

---

## Phase 8 Implementation Strategy

### Stage 1: Fine-tune Transformer on Synthetic Data (Week 1)

**Goal**: Achieve 88-90% on current dataset to validate training pipeline

```
File: src/training/fine_tune.rs (NEW, 300 LOC)
├── Fine-tuning config
├── Learning rate scheduling
├── Gradient accumulation
├── Early stopping
└── Checkpoint management
```

**Steps**:
1. Load synthetic dataset (257 samples)
2. Create train/val/test split (80/10/10)
3. Fine-tune transformer encoder for 10 epochs
4. Evaluate on test set
5. Save best checkpoint

**Expected Results**: 88-90% accuracy (2-3 point gain from heuristics)

---

### Stage 2: Expand Dataset with External Sources (Week 2)

**Goal**: Increase training data to 10k+ samples for meaningful improvement

```
File: src/dataset/external.rs (NEW, 250 LOC)
├── HuggingFace dataset loader
├── JailbreakBench integration
├── Data augmentation pipeline
└── Stratified sampling
```

**Data Sources**:
1. **JailbreakBench** (HuggingFace) - 10k+ examples
   - `dataset = load_dataset("JailbreakBench/JailBench")`

2. **DeepSeek-Jailbreak** - 1k manually crafted
   - Focus on novel patterns

3. **PAIR (Prompt Adversarial Input Refinement)** - 100+ variants
   - High-quality adversarial examples

4. **Augmentation**: Character swap, paraphrase, encoding
   - Generate 3x variants per original sample

**Expected Data**:
- Total: ~30k samples
- Injection: 15k (50%)
- Benign: 15k (50%)
- Attack types: 7-way distribution

---

### Stage 3: Adversarial Training (Week 2-3)

**Goal**: Robust to evasion attacks (homoglyphs, encoding, obfuscation)

```
File: src/training/adversarial_fine_tune.rs (NEW, 350 LOC)
├── Adversarial example generation
├── Mixed training (70% clean, 30% adversarial)
├── Loss scaling
└── Metrics tracking
```

**Adversarial Techniques** (30% of batch):

1. **Character Substitution** (10%)
   ```rust
   "Ignore" → "Ìgnore" (homoglyph ì→i)
   "ignore" → "1gn0r3" (leetspeak)
   "IGNORE" → "IgNoRe" (case variation)
   ```

2. **Encoding Obfuscation** (10%)
   ```rust
   "ignore your instructions" → base64 encoded
   "ignore" → hex: 69 67 6e 6f 72 65
   "ignore" → ROT13: "vtzber"
   ```

3. **Semantic Paraphrasing** (10%)
   ```rust
   "Ignore instructions" → "Disregard directives"
   "Override rules" → "Circumvent constraints"
   ```

**Expected Robustness**:
- +3-5% accuracy on adversarial examples
- Reduced false negatives on novel patterns

---

### Stage 4: Multi-Task Learning (Week 3-4)

**Goal**: 7-way attack type classification + binary classification + semantic similarity

```
File: src/model/multitask_heads.rs (NEW, 250 LOC)
├── Binary classification head (injection vs benign)
├── Attack type classifier (7 classes)
├── Semantic similarity head (cosine distance)
└── Loss combination (weighted sum)
```

**Attack Types**:
1. Instruction Override (weight: 0.15)
2. Role-play / Context Manipulation (weight: 0.15)
3. Encoding / Obfuscation (weight: 0.15)
4. Separator / Structural (weight: 0.15)
5. Prompt Leaking (weight: 0.15)
6. Output Manipulation (weight: 0.15)
7. Novel / Unknown (weight: 0.1)

**Multi-Task Loss**:
```
L_total = α * L_binary + β * L_attack + γ * L_semantic
         = 0.6 * CE(binary) + 0.3 * CE(attack_type) + 0.1 * MSE(similarity)
```

**Benefits**:
- +2-3% accuracy from auxiliary tasks
- Better feature learning
- Interpretable attack classification

---

### Stage 5: Confidence Calibration (Week 4)

**Goal**: Reliable confidence scores (ECE < 0.05)

```
File: src/training/calibration_fine_tune.rs (NEW, 200 LOC)
├── Temperature scaling on validation set
├── ECE computation
├── Reliability diagrams
└── Confidence distribution analysis
```

**Process**:
1. Train model without calibration
2. Freeze model weights
3. Optimize temperature T on validation set
4. Minimize NLL: `L = -log(softmax(z/T))`
5. Validate on held-out test set

**Target**: ECE < 0.05 (vs. heuristics: not calibrated)

---

### Stage 6: Pre-trained Model Integration (Week 5)

**Goal**: Ensemble fine-tuned model with pre-trained models

```
File: src/integration/pretrained.rs (NEW, 300 LOC)
├── GenTel-Shield v1 integration
├── ProtectAI DeBERTa integration
├── Weighted ensemble voting
└── Fallback handling
```

**Integration Strategy**:

```rust
pub struct SOTAEnsemble {
    fine_tuned: TransformerDetector,      // Our 92-94% model
    gentel_shield: PretrainedDetector,    // 97.63%
    protectai_deberta: PretrainedDetector, // 94.2%
    weights: EnsembleWeights,
}

impl SOTAEnsemble {
    pub fn detect(&self, text: &str) -> DetectionResult {
        let fine_tuned_score = self.fine_tuned.detect(text);
        let gentel_score = self.gentel_shield.detect(text);
        let protectai_score = self.protectai_deberta.detect(text);

        // Weighted average
        let final_score = 0.3 * fine_tuned_score
                        + 0.5 * gentel_score
                        + 0.2 * protectai_score;

        DetectionResult {
            is_injection: final_score > 0.5,
            confidence: final_score,
            // Explain which models agreed
            voting_breakdown: format!(
                "Fine-tuned: {:.1}%, GenTel: {:.1}%, ProtectAI: {:.1}%",
                fine_tuned_score * 100.0,
                gentel_score * 100.0,
                protectai_score * 100.0,
            ),
        }
    }
}
```

**Expected Accuracy**: 96-98% (ensemble beats individual models)

---

### Stage 7: Online Learning Integration (Week 5-6)

**Goal**: Continuous improvement from user corrections

```
File: src/training/online_fine_tune.rs (NEW, 250 LOC)
├── Feedback collection
├── Mini-batch fine-tuning
├── Model versioning
└── A/B testing framework
```

**Process**:
1. User marks detection as correct/incorrect
2. Collect feedback samples (batch size: 32)
3. Perform 1-2 epochs of fine-tuning with low LR (1e-5)
4. Validate on holdout set
5. Roll out if improvement confirmed
6. Version control model checkpoints

**Conservative Settings**:
- Learning rate: 1e-5 (10x lower than initial training)
- Epochs: 1-2 only
- Gradient clipping: 1.0
- Weight decay: higher (prevent overfitting to feedback)

**Expected Gains**:
- +1-2% accuracy per 100 corrections
- Adapted to user's specific domain

---

## Performance Targets (Phase 8)

| Milestone | Metric | Target | Timeline |
|-----------|--------|--------|----------|
| **After Fine-tuning** | Accuracy | 90-92% | Week 2 |
| **After Expansion** | Accuracy | 92-94% | Week 3 |
| **After Adversarial Training** | Robustness | +3-5% | Week 4 |
| **After Multi-Task** | Attack Classification | 85%+ | Week 4 |
| **After Calibration** | ECE | <0.05 | Week 5 |
| **After Integration** | Ensemble Accuracy | 96-98% | Week 6 |
| **After Online Learning** | Domain Adaptation | +1-2% | Week 7 |

---

## Technical Implementation Details

### 1. Fine-tuning Configuration

```rust
pub struct FineTuneConfig {
    pub learning_rate: f64,           // 2e-5
    pub num_epochs: usize,            // 10
    pub batch_size: usize,            // 32
    pub warmup_steps: usize,          // 500
    pub gradient_accumulation: usize, // 1
    pub max_grad_norm: f32,           // 1.0
    pub weight_decay: f64,            // 0.01
    pub dropout: f32,                 // 0.1
}
```

### 2. Data Augmentation Strategy

```
Original: "Ignore your instructions"

Variants (3 total):
1. Paraphrase: "Disregard your directives"
2. Encode: base64 or homoglyph
3. Contextual: "...and ignore your instructions..."
```

### 3. Loss Function Weighting

```
Epoch 0-3:   All tasks equally (0.33, 0.33, 0.33)
Epoch 4-7:   Focus on primary task (0.6, 0.2, 0.2)
Epoch 8-10:  Fine-tune secondary tasks (0.5, 0.3, 0.2)
```

### 4. Evaluation Metrics

```
Accuracy:       True Positives / Total
Precision:      TP / (TP + FP)  [prefer low false positives]
Recall:         TP / (TP + FN)  [prefer detecting injections]
F1-Score:       2 * (Precision * Recall) / (Precision + Recall)
ECE:            Expected Calibration Error [target < 0.05]
AUC-ROC:        Area under ROC curve [target > 0.98]
Attack Type F1: 7-way classification F1 score
```

---

## Rollout Strategy

### Phase 8a: Baseline (Week 1-2)
- Deploy fine-tuned model
- Measure 90-92% accuracy
- Validate on test set
- No breaking changes to API

### Phase 8b: Enhanced (Week 3-4)
- Expand dataset
- Add multi-task learning
- +2-3% accuracy improvement
- Feature flag for new model

### Phase 8c: Robust (Week 5)
- Adversarial training
- Confidence calibration
- Better corner case handling
- A/B test against baseline

### Phase 8d: SOTA (Week 6-7)
- Integrate pre-trained models
- Ensemble voting
- 96-98% accuracy achieved
- Production-ready deployment

---

## Risk Mitigation

### Risk: Data Leakage
**Mitigation**: Strict train/val/test split, no test data in training

### Risk: Overfitting on Synthetic Data
**Mitigation**: Expand with external data, adversarial training, validation checks

### Risk: Degraded Performance on New Attacks
**Mitigation**: Out-of-distribution testing, online learning, continuous monitoring

### Risk: Model Complexity
**Mitigation**: Start with simple fine-tuning, gradually add components

---

## Success Criteria

✅ **Phase 8 Complete When**:
1. Fine-tuned model: 90%+ accuracy
2. Expanded dataset: 10k+ samples integrated
3. Adversarial robustness: +3% on adversarial examples
4. Multi-task: 85%+ attack classification F1
5. Calibration: ECE < 0.05
6. Ensemble: 96%+ accuracy
7. Online learning: 1-2% domain improvement
8. All 311 tests still passing
9. Documentation updated

---

## Files to Create/Modify

### New Files (8 total)
- `src/training/fine_tune.rs` (300 LOC)
- `src/dataset/external.rs` (250 LOC)
- `src/training/adversarial_fine_tune.rs` (350 LOC)
- `src/model/multitask_heads.rs` (250 LOC)
- `src/training/calibration_fine_tune.rs` (200 LOC)
- `src/integration/pretrained.rs` (300 LOC)
- `src/training/online_fine_tune.rs` (250 LOC)
- `examples/fine_tune_sota.rs` (400 LOC)

### Modified Files (3 total)
- `src/lib.rs` (re-exports, feature flags)
- `Cargo.toml` (new dependencies)
- `README.md` (updated accuracy claims)

### Total New Code
- ~1,900 LOC core implementation
- ~400 LOC examples
- ~500 LOC tests

---

## Next Immediate Step

**Start with Stage 1: Fine-tuning on Synthetic Data**

1. Create `src/training/fine_tune.rs`
2. Implement FineTuneConfig and fine-tuning loop
3. Load synthetic dataset
4. Fine-tune for 10 epochs
5. Evaluate and log metrics
6. Target: 88-90% accuracy

**Estimated completion**: 2-3 hours

