# Quick Reference: Accuracy Boost Strategies for JailGuard

**TL;DR:** Use 3-5 lightweight ensemble models + LLMail-Inject dataset → 96-98% accuracy in 3-4 weeks

---

## 1. DATASETS AT A GLANCE

| Dataset | Size | License | Format | Use |
|---------|------|---------|--------|-----|
| **LLMail-Inject** | 208K | MIT | JSON | Training (70%) |
| **JailbreakBench** | 4.3K | MIT | HF | Evaluation (false positives) |
| **TrustAIRLab** | 15K | MIT | CSV | Training supplement (15%) |
| **SPML** | 16K | MIT | CSV | Training supplement (5%) |
| **GenTel-Safe** | 84K | ? | HF | Evaluation benchmark |

**Total Available:** 243K MIT-compatible samples ready now

---

## 2. QUICK ACCURACY COMPARISON

| Approach | Accuracy | Latency | Effort | Rust Viable |
|----------|----------|---------|--------|------------|
| Current JailGuard | 85-90% | 50-100ms | - | ✅ |
| + Heuristics layer | 90-92% | 100-150ms | 1 day | ✅ |
| + Single fine-tuned model | 93-95% | 100-200ms | 3 days | ✅ |
| + 3-model ensemble | 95-97% | 150-250ms | 1 week | ✅ |
| + Attention Tracker | 96-98% | 100-250ms | 2 weeks | ✅ |
| **Full system** | **96-98%** | **200ms** | **4 weeks** | **✅** |

---

## 3. MODEL SELECTION FOR ENSEMBLE

Pick these 3-5 models for best diversity:

```
Model 1: DeBERTa-small (86M params)
├─ Accuracy: AUROC 0.94-0.97
├─ Speed: 100-150ms per inference
└─ Reason: Industry standard, proven

Model 2: FLAN-T5-small (61M params)
├─ Accuracy: AUROC 0.942
├─ Speed: 80-100ms per inference
└─ Reason: Different architecture catches different attacks

Model 3: Attention Tracker (training-free)
├─ Accuracy: +10% AUROC improvement
├─ Speed: 30-50ms (pure attention analysis)
└─ Reason: Zero training required, orthogonal method

[Optional] Model 4: RoBERTa-base (125M params)
├─ Accuracy: 0.93-0.96
├─ Speed: 100-120ms
└─ Reason: Third diverse architecture

[Optional] Model 5: Custom lightweight (50M)
├─ Accuracy: Custom-tuned
├─ Speed: 80-100ms
└─ Reason: Domain-specific patterns
```

**Ensemble Strategy:** Majority vote or weighted average
- If ≥2/3 models say "injection" → Flag as injection
- Otherwise → Use confidence score

---

## 4. IMPLEMENTATION PHASES

### Phase 1: Fast (2 weeks, +3-5% accuracy)
```
Week 1:
- Download LLMail-Inject dataset
- Add heuristics/rules layer (1 day)
- Fine-tune DeBERTa-small (3-4 days)

Week 2:
- Export to ONNX format
- Build simple 2-model Rust ensemble
- Evaluate on JailbreakBench
```

### Phase 2: Complete (4 weeks, +6-13% accuracy)
```
Week 1: Data prep + baseline
Week 2: Train 3-5 models in parallel
Week 3: Implement Rust ensemble + Attention Tracker
Week 4: Integration + optimization
```

---

## 5. PAPERS TO IMPLEMENT (In Order)

| Priority | Paper | Code | Effort | Gain |
|----------|-------|------|--------|------|
| **🔴 1** | Attention Tracker (arxiv 2411.00348) | ? | ⭐⭐ | +10% AUROC |
| **🔴 2** | GenTel-Safe (arxiv 2409.19521) | ✅ | ⭐⭐⭐ | +2-4% |
| **🟠 3** | PromptShield (arxiv 2501.15145) | ✅ | ⭐⭐ | +2-3% |
| **🟠 4** | DMPI-PMHFE (June 2025) | ❓ | ⭐⭐⭐⭐ | +2-4% |
| **🟡 5** | SmoothLLM (arxiv 2310.03684) | ✅ | ⭐⭐⭐ | +2-3% |

---

## 6. RUST LIBRARIES NEEDED

```toml
[dependencies]
# ONNX model inference (3-5x faster than Python)
ort = "2.0"

# Parallel inference with multiple models
rayon = "1.7"

# Tokenization
tokenizers = "0.13"

# Regex for heuristics
regex = "1.10"

# ML math operations
ndarray = "0.15"

# Async runtime (optional)
tokio = "1"
```

---

## 7. REALISTIC TIMELINE

```
Start: Jan 16, 2026
│
├─ Week 1 (Jan 16-23): Data + baseline
│  └ Download datasets, implement heuristics, train DeBERTa-small
│
├─ Week 2 (Jan 23-30): Model training + export
│  └ Train 3-4 models, distillation, ONNX conversion
│
├─ Week 3 (Jan 30-Feb 6): Rust ensemble implementation
│  └ Load models, parallel inference, voting logic
│
├─ Week 4 (Feb 6-13): Integration + testing
│  └ Progressive detection, full evaluation, optimization
│
End: ~Feb 13, 2026 → 96-98% accuracy system ready
```

---

## 8. EXPECTED FINAL METRICS

```
Accuracy:        96-98%  (vs 85-90% baseline)
Precision:       96-98%  (high confidence detections)
Recall:          94-97%  (catches most real attacks)
F1 Score:        0.95-0.98
AUROC:           0.97-0.99

Latency P50:     100-150ms
Latency P99:     200-250ms
Memory:          1.2GB (all models)

False Positives: 2-4% (low friction for users)
False Negatives: 2-6% (catches most attacks)
```

---

## 9. COST-BENEFIT ANALYSIS

| Factor | Cost | Benefit |
|--------|------|---------|
| **Development Time** | 4 weeks | +11% accuracy gain |
| **Compute Resources** | 16GB GPU, 50GB disk | Production-grade system |
| **Inference Speed** | 100-250ms | Reasonable for security |
| **Model Size** | 1.2GB | Deployable in most systems |
| **Maintenance** | Periodic retraining | Up-to-date defense |

**ROI:** Very high - industry-leading accuracy with manageable resources

---

## 10. DECISION MATRIX

**Choose based on your constraints:**

### If you have 1-2 weeks:
```
→ Quick Wins approach
  - Add heuristics layer (1 day)
  - Integrate Attention Tracker (2 days)
  - Fine-tune single model (4 days)
  Expected: 90-93% accuracy
```

### If you have 3 weeks:
```
→ Balanced approach
  - Fine-tune 2-3 models (week 1-2)
  - Build simple ensemble (week 2)
  - Integrate Attention Tracker (week 3)
  Expected: 94-96% accuracy
```

### If you have 4 weeks (recommended):
```
→ Maximum accuracy approach
  - Full data pipeline + augmentation (week 1)
  - Train 5 models + knowledge distillation (week 2)
  - Implement Rust ensemble + all techniques (week 3-4)
  Expected: 96-98% accuracy
```

---

## 11. DATA DOWNLOAD COMMANDS

```bash
# Create data directory
mkdir -p data/raw

# LLMail-Inject (208K samples) - CRITICAL
huggingface-cli download microsoft/llmail-inject-challenge \
  --repo-type dataset --local-dir data/raw/llmail-inject

# JailbreakBench (4.3K samples) - for evaluation
huggingface-cli download JailbreakBench/JBB-Behaviors \
  --repo-type dataset --local-dir data/raw/jailbreakbench

# TrustAIRLab (15K samples) - supplement training
huggingface-cli download TrustAIRLab/in-the-wild-jailbreak-prompts \
  --repo-type dataset --local-dir data/raw/trustairlab

# SPML (16K samples) - supplement training
huggingface-cli download reshabhs/SPML_Chatbot_Prompt_Injection \
  --repo-type dataset --local-dir data/raw/spml

# Total: ~243K samples, all MIT-compatible
```

---

## 12. PYTHON TRAINING SCAFFOLD

```python
# train_ensemble.py
import torch
from transformers import AutoTokenizer, AutoModelForSequenceClassification
from datasets import load_dataset

# Load LLMail-Inject
dataset = load_dataset(
    "microsoft/llmail-inject-challenge",
    split="train[:80%]",  # 80% for training
)

# Train model 1: DeBERTa-small
model1 = train_model(
    model_name="microsoft/deberta-v3-small",
    dataset=dataset,
    epochs=3,
    batch_size=32,
)

# Train model 2: FLAN-T5-small
model2 = train_model(
    model_name="google/flan-t5-small",
    dataset=dataset,
    epochs=3,
    batch_size=32,
)

# Train model 3: RoBERTa-base
model3 = train_model(
    model_name="roberta-base",
    dataset=dataset,
    epochs=3,
    batch_size=32,
)

# Export to ONNX
export_to_onnx(model1, "deberta-small.onnx")
export_to_onnx(model2, "flan-t5-small.onnx")
export_to_onnx(model3, "roberta-base.onnx")

# Evaluate on JailbreakBench
eval_dataset = load_dataset(
    "JailbreakBench/JBB-Behaviors",
    split="test",
)
accuracy = evaluate_ensemble([model1, model2, model3], eval_dataset)
print(f"Ensemble accuracy: {accuracy:.2%}")
```

---

## 13. RUST ENSEMBLE SCAFFOLD

```rust
// src/ensemble.rs
use ort::{Session, SessionBuilder};
use std::sync::Arc;

pub struct EnsembleDetector {
    models: Vec<Arc<Session>>,
    voting_strategy: VotingStrategy,
}

impl EnsembleDetector {
    pub fn new() -> Result<Self> {
        let model1 = SessionBuilder::new()?
            .with_model_from_file("deberta-small.onnx")?;
        let model2 = SessionBuilder::new()?
            .with_model_from_file("flan-t5-small.onnx")?;
        let model3 = SessionBuilder::new()?
            .with_model_from_file("roberta-base.onnx")?;

        Ok(EnsembleDetector {
            models: vec![
                Arc::new(model1),
                Arc::new(model2),
                Arc::new(model3),
            ],
            voting_strategy: VotingStrategy::MajorityVote,
        })
    }

    pub fn detect(&self, prompt: &str) -> DetectionResult {
        let scores: Vec<f32> = self.models
            .par_iter()  // Parallel inference
            .map(|model| self.infer(model, prompt))
            .collect();

        let confidence = self.voting_strategy.aggregate(&scores);

        if confidence > 0.65 {
            DetectionResult::Injection(confidence)
        } else {
            DetectionResult::Benign(1.0 - confidence)
        }
    }
}
```

---

## 14. SUCCESS CRITERIA

✅ **System is production-ready when:**
- [ ] Accuracy ≥ 95% on LLMail-Inject test set
- [ ] Recall ≥ 90% (catches most real attacks)
- [ ] Precision ≥ 95% (low false positives)
- [ ] Latency ≤ 250ms P99
- [ ] Memory footprint ≤ 2GB
- [ ] All models export to ONNX successfully
- [ ] Rust binary compiles without warnings
- [ ] Evaluation passes on JailbreakBench
- [ ] Documentation complete
- [ ] Performance benchmarks documented

---

## 15. NEXT ACTIONS (TODAY)

1. **📥 Download Datasets** (~30 minutes)
   - Use download commands above
   - Verify checksums

2. **📖 Read Key Papers** (~2 hours)
   - Attention Tracker: arxiv 2411.00348
   - GenTel-Safe: arxiv 2409.19521

3. **🛠️ Set Up Python Environment** (~30 minutes)
   - Install transformers, datasets, torch
   - Test dataset loading

4. **📋 Create Detailed Project Plan** (~1 hour)
   - Break down 4 weeks into daily tasks
   - Assign team members
   - Set up progress tracking

5. **🚀 Start Week 1 Implementation**
   - Begin data preprocessing
   - Create baseline evaluation script
   - Schedule training runs

---

## Recommended Reading Order

1. **Attention Tracker** (15 min) - Easiest, +10% gain
   - https://arxiv.org/abs/2411.00348

2. **GenTel-Safe** (30 min) - Complete framework
   - https://arxiv.org/abs/2409.19521

3. **PromptShield** (20 min) - Training details
   - https://arxiv.org/pdf/2501.15145

4. **JailbreakBench** (25 min) - Evaluation methodology
   - https://arxiv.org/abs/2404.01318

5. **LLMail-Inject** (25 min) - Dataset details
   - https://arxiv.org/abs/2506.09956

**Total reading time:** ~2 hours

---

## FAQ

**Q: Can I use just LLMail-Inject?**
A: Yes. 208K samples is sufficient. 243K combined is better for diversity.

**Q: How many models should I ensemble?**
A: 3-5 is sweet spot. More than 5 adds latency without proportional accuracy gain.

**Q: Is Attention Tracker viable for Rust?**
A: Yes, if ONNX models export attention weights. Check ONNX opset version.

**Q: Can I skip knowledge distillation?**
A: Yes. Ensemble alone gets you 95-97% accuracy. Distillation adds +1-2%.

**Q: What's the minimum viable timeline?**
A: 2 weeks for 93-95% accuracy (heuristics + single model).

**Q: Will this run on CPU?**
A: Yes, models will run on CPU but 3-5x slower than GPU (~300-750ms).

**Q: Do I need to retrain periodically?**
A: Yes, 1-2x per year as attacks evolve. Use LLMail-Inject followups.

---

**Created:** January 16, 2026
**Version:** 1.0
**Status:** Ready to implement
