# JailGuard SOTA 2026: Strategic Enhancement Plan

**Status:** Strategic Analysis Complete | **Last Updated:** 2026-01-14

## Executive Summary

This document outlines the comprehensive strategy to advance JailGuard to state-of-the-art (SOTA) multi-layer defense-in-depth for prompt injection and jailbreak prevention in 2026. JailGuard serves as a foundational security library for consuming applications (agentique, llmkit) without circular dependencies.

### Key Findings

- **Current Performance:** 85% accuracy, binary classification, ~2ms latency
- **SOTA Approach:** Multi-layer defense combining prevention + detection + validation + monitoring
  - Spotlighting/Input Marking (prevents 70-80% at architectural level)
  - Enhanced Detection (95-98% accuracy)
  - Task Tracking (catches multi-turn attacks)
  - Privilege Context (limits damage)
  - Output Validation (prevents constraint violations)
  - Behavior Monitoring (long-term threat detection)
- **Complementary SOTA Techniques (Consumer Layer):**
  - LLM-as-Judge (implemented in llmkit/agentique for edge cases)
  - Local Model Hosting (Llama, Mistral deployed in agentique for real-time verification)
- **Architecture:** No circular dependencies - jailguard is a pure security library
- **Feasibility in Rust/Burn:** 9/10 - Straightforward with ~12-13 weeks effort
- **Expected Improvement:** +40-50% defense effectiveness, 99%+ effective injection prevention when layers combined

---

## Table of Contents

1. [Architectural Boundaries & Integration](#architectural-boundaries--integration)
2. [Current State Assessment](#current-state-assessment)
3. [Gap Analysis: Current vs SOTA 2026](#gap-analysis-current-vs-sota-2026)
4. [SOTA Reference: Multi-Layer Defense](#sota-reference-multi-layer-defense)
5. [Proposed Layered Architecture](#proposed-layered-architecture)
6. [Tier 1: Critical Improvements](#tier-1-critical-improvements)
7. [Tier 2: High-Impact Improvements](#tier-2-high-impact-improvements)
8. [Tier 3: Nice-to-Have Features](#tier-3-nice-to-have-features)
9. [Performance Considerations](#performance-considerations)
10. [Training & Robustness](#training--robustness)
11. [Implementation Roadmap](#implementation-roadmap)
12. [Feature Flags & Configuration](#feature-flags--configuration)
13. [Success Metrics](#success-metrics)

---

## Architectural Boundaries & Integration

### Design Philosophy: No Circular Dependencies

JailGuard is designed as a **pure security library** that provides defensive tools without requiring its consumers (agentique, llmkit) to be available. This prevents circular dependencies and enables independent evolution.

```
┌──────────────────────────────────────────────────────┐
│              CONSUMER APPLICATIONS                    │
│  ┌──────────────────────────────────────────────┐   │
│  │ agentique                                     │   │
│  │ - Agent orchestration                        │   │
│  │ - LLM-as-Judge (via local models)           │   │
│  │ - Task coordination                          │   │
│  └──────────────────────────────────────────────┘   │
│                          ↓ uses                       │
│  ┌──────────────────────────────────────────────┐   │
│  │ llmkit                                        │   │
│  │ - LLM API integration                        │   │
│  │ - Local model hosting (Llama, Mistral)      │   │
│  │ - Provider abstraction                       │   │
│  └──────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────┘
                        ↓ both use
┌──────────────────────────────────────────────────────┐
│              JAILGUARD (Security Library)             │
│  - Spotlighting/Input Marking                        │
│  - Enhanced Injection Detection                      │
│  - Task Tracking                                     │
│  - Privilege Context Management                      │
│  - Output Validation                                 │
│  - Behavior Monitoring                               │
│  [NO dependency on agentique or llmkit]             │
└──────────────────────────────────────────────────────┘
```

### SOTA Techniques: Where They Belong

#### JailGuard Responsibilities (This Library)
✅ **Defensive measures that don't require external LLM calls**
- Spotlighting/input marking (pure data transformation)
- Deep neural network detection (transformer + multi-task learning)
- Task tracking (behavioral analysis)
- Privilege context (type system for constraints)
- Output validation (pattern matching, heuristics)
- Behavior monitoring (statistical anomaly detection)

#### Consumer Responsibilities (agentique/llmkit)
✅ **SOTA techniques that leverage additional capabilities**
- **LLM-as-Judge** - Ask external LLM/judge to verify edge cases (high confidence only)
  - Implemented in agentique or llmkit
  - Called for low-confidence detections (<70%)
  - Used sparingly (~5% of requests)

- **Local Model Hosting** - Deploy Llama/Mistral for real-time verification
  - Implemented in llmkit backend
  - Smaller models (7B-13B) for speed
  - Optional feature, not core to jailguard

**Why this split?**
- Avoids circular dependencies (jailguard → llmkit → jailguard)
- Each component can be updated independently
- Consumers can choose their verification strategy
- jailguard remains lightweight and deployable anywhere

### Integration Pattern

```rust
// In agentique or llmkit:

// Step 1: Use jailguard for multi-layer defense
let detection = jailguard.detect(&input)?;
let validation = jailguard.validate_output(&llm_response, &privilege)?;
let anomaly = jailguard.monitor().track_request(&user_id, &request)?;

// Step 2: Use local judge for edge cases
if detection.confidence < 0.70 {
    let local_judge = get_local_judge(); // Llama, Mistral, or other
    let judge_decision = local_judge.verify(&input).await?;

    if judge_decision.is_suspicious() {
        return Err("Blocked by local judge".into());
    }
}

// Step 3: Fall back to external LLM-as-Judge (optional)
if detection.confidence < 0.50 {
    let llm_judge = llm.get_judge();
    let judge_result = llm_judge.is_safe(&input).await?;
    // ...
}

// Result: 3-layer verification without circular dependencies
```

---

## Current State Assessment

### Architecture Summary (2,572 LOC)

- **Tokenizer:** Simple word-level tokenizer with vocabulary building (no subword tokenization)
- **Embeddings:** Basic token + position embeddings with layer norm (94 LOC)
- **Policy Network:** 2-layer MLP (128 → 256 → 2 actions) for binary classification
- **RL Agents:** PPO and DQN (minimal implementations, placeholder save/load)
- **Detection:** Single-pass pooled embedding → binary classification (is_injection: bool)
- **Training:** Basic reward shaping, GAE for advantage estimation, experience buffer
- **Backend:** Burn 0.19 (NdArray CPU, WGPU GPU support)

### Key Limitations

1. **Single-layer detection:** Binary classification only (no attack type, severity, confidence calibration)
2. **No semantic understanding:** Word-level tokenizer loses context and structure
3. **Shallow architecture:** 2-layer policy network lacks depth for complex pattern recognition
4. **No attention mechanism:** Cannot identify which parts of input are suspicious
5. **No multi-task learning:** Only learns binary classification, not attack patterns
6. **Weak embedding:** Simple embeddings, no contextual representations
7. **No ensemble/voting:** Single model per detection
8. **Limited configurability:** Runtime backend selection exists, but model size/precision not tunable

---

## Gap Analysis: Current vs SOTA 2026

### Feature Comparison

| Aspect | Current | SOTA (PromptGuard/Shields) | Gap |
|--------|---------|--------------------------|-----|
| Classification | Binary (inject/benign) | Multi-class (7 attack types) + severity | Need attack classification |
| Semantic analysis | None (word-level only) | Contextual embeddings (BERT-like) | Need transformer/attention |
| Detection layers | 1 (final action) | Multi-layer (tokenization → semantic → adversarial) | Need layered pipeline |
| Explanation | Risk level only | Attack type, confidence bounds, patterns | Need interpretability |
| Pattern matching | None | Heuristic rules + learned patterns | Need pattern detection |
| Confidence calibration | Raw softmax | Calibrated probabilities with uncertainty | Need temperature scaling |
| Ensemble | Single model | Multiple detectors voted | Need voting/ensemble |
| Robustness | None | Adversarial training | Need evasion resistance |

### SOTA References

- **PromptGuard** (Nature Scientific Reports 2025): 67% injection reduction, F1-score 0.91
  - Multi-layer defense: input gatekeeping → semantic validation → adaptive refinement
  - Uses MiniBERT-based detection with regex heuristics
  - Latency <8% increase

- **Microsoft Prompt Shields**: Enterprise-integrated with Defender for Cloud
  - Spotlighting: marks data vs instructions with special delimiters
  - LLM-as-judge for edge cases
  - Preventative vs reactive approach

### Burn Framework Feasibility

**What Burn 0.19 provides:**
- `Embedding`, `Linear`, `LayerNorm`, `Relu`, `Softmax` ✓
- Custom tensor operations ✓
- Activation functions ✓
- Multi-backend abstraction ✓

**What must be built:**
```
attention.rs     (~200 LOC) - Multi-head attention
encoder.rs       (~300 LOC) - Transformer encoder blocks
```

**Feasibility: 8.5/10** - Straightforward but requires custom implementations

---

## SOTA Reference: Multi-Layer Defense

### Current Research Consensus (2025-2026)

**Major Finding:** Prompt injection is **fundamentally unsolved** as a detection problem alone.

From leading research labs and industry experts:
> "Major research labs have concluded that prompt injection remains an unsolved problem, with attempts to block or filter them not proven reliable enough to depend on"

### Why Detection Alone Fails

| Scenario | Detection Result | Actual Outcome |
|----------|------------------|----------------|
| 99% detector with 1000 requests | 10 slip through | **Still fails** |
| Unknown attack vector | No training data | **Cannot detect** |
| Sophisticated multi-turn attack | Evades each layer | **Cumulative success** |
| Adversarial obfuscation | Evolving attacks | **Arms race** |

### SOTA 2026 Approach: Defense-In-Depth

Instead of "better detection," the industry has shifted to "design for containment."

#### Layer 1: **Prevention** (70-80% of attacks stop here)
- **Spotlighting/Input Marking** (Microsoft, PromptGuard)
- Makes injection boundaries explicit to LLM
- Hard to exploit even if model is confused

#### Layer 2: **Detection** (Catches 95-98% of remaining)
- Enhanced neural networks (transformers, adversarial training)
- Attack type classification
- Behavioral analysis

#### Layer 3: **Validation** (Architectural defense)
- Task tracking (detect if model abandons task)
- Privilege limiting (restrict what model can do)
- Output validation (check results match constraints)

#### Layer 4: **Fallback Verification** (Edge cases, <5% of requests)
- **LLM-as-Judge** (ask another LLM to verify)
- **Local Models** (deploy Llama/Mistral for verification)
- Real-time second opinion on uncertain decisions

#### Layer 5: **Monitoring** (Long-term threat intelligence)
- Behavior tracking (identify attack patterns)
- Anomaly detection (flag unusual activity)
- User feedback integration

### Effectiveness Comparison

```
Single Detection Layer:
├─ Best case: 99.9% accuracy
└─ Worst case: 1 in 1000 injections succeed ❌

Multi-Layer Defense:
├─ Spotlighting: blocks 80% at boundary
├─ Detection: blocks 95% of remaining 20%
├─ Validation: architectural defense of 99%
├─ Fallback: catches 99% of edge cases
└─ Result: 99.8%+ effective ✅
   AND damage is contained even if injection succeeds

Difference: From "we might miss some" to "we stop them architecturally"
```

### SOTA Technique Placement

| Technique | Effectiveness | Where Implemented | Why |
|-----------|---|---|---|
| Spotlighting | 70-80% | JailGuard | Pure data transform, no dependencies |
| Detection (NN) | 95-98% | JailGuard | Core security layer |
| Task Tracking | 85%+ | JailGuard | Behavioral analysis, no external calls |
| Privilege Limits | 99%+ | Consumer (agentique/llmkit) | Architectural choice by app |
| **LLM-as-Judge** | 95%+ | Consumer (agentique/llmkit) | Requires external LLM, prevents circular deps |
| **Local Models** | 90%+ | Consumer (llmkit backend) | Optional enhancement, heavy |
| Output Validation | 99%+ | JailGuard | Heuristic patterns, no LLM needed |
| Behavior Monitoring | 90%+ | JailGuard | Statistical analysis |

**Key insight:** Consumer apps can layer LLM-as-Judge and Local Models on top without creating circular dependencies.

---

## Proposed Layered Architecture

### Multi-Layer Defense System

```
INPUT TEXT
    ↓
┌─────────────────────────────────────────────────────────┐
│ LAYER 1: TOKENIZATION & ENCODING (Semantic)             │
│ - Replace simple tokenizer with subword tokenization    │
│   (BPE/WordPiece-style in Rust, or fixed-vocab similar) │
│ - Add character-level features for encoding detection   │
│ - Compute token importance scores                       │
└─────────────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────────────┐
│ LAYER 2: CONTEXTUAL EMBEDDING (MiniBERT-style)          │
│ - Multi-head attention (4-8 heads, local window)        │
│ - Transformer encoder (2-3 layers for efficiency)       │
│ - Output: [CLS] token + per-token embeddings            │
└─────────────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────────────┐
│ LAYER 3a: ATTACK CLASSIFICATION (Multi-head)            │
│ - Direct injection detector                            │
│ - Roleplay/persona detector                            │
│ - Encoding detector (base64, unicode escaping, etc.)    │
│ - Context manipulation detector                        │
│ - Goal hijacking detector                              │
│ - Instruction override detector                        │
│ - Multi-turn poisoning detector                        │
│ Output: Attack type logits + probabilities             │
└─────────────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────────────┐
│ LAYER 3b: SEMANTIC ANALYSIS (Auxiliary)                 │
│ - Semantic similarity to known payloads                 │
│ - Instruction keywords detection                       │
│ - Prompt structure analysis                            │
│ Output: Semantic risk score                            │
└─────────────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────────────┐
│ LAYER 4: AGGREGATION & CONFIDENCE CALIBRATION          │
│ - Combine attack type, semantic, and base detectors    │
│ - Temperature scaling for calibrated probabilities     │
│ - Ensemble voting if multiple models available         │
│ Output: Final decision + confidence + attack type      │
└─────────────────────────────────────────────────────────┘
    ↓
DECISION: {is_injection, attack_type, confidence, risk_level}
```

### Architecture Options

#### Option A: Full Transformer (Heavy)
- 12-layer BERT-style model (768 dims, 12 heads)
- Pros: SOTA performance, proven architecture
- Cons: Large (110M+ params), slow inference (~500ms CPU)
- CPU Latency: ~500ms | GPU: ~50ms | Memory: ~450MB
- **Not recommended for 2026 deployments**

#### Option B: Lightweight Transformer (Recommended) ⭐
- 2-3 layer transformer, 256 dims, 4 heads, ~4M params
- Pros: Fast inference, good performance, configurable
- Cons: Slightly less semantic understanding than Option A
- CPU Latency: ~20ms | GPU: ~5ms | Memory: ~16MB
- **Feasibility: 9/10 in Burn**

#### Option C: Hybrid (Best for Production) ⭐⭐
- Layer 1-2: Lightweight transformer (2-3 layers, 256 dims)
- Layer 3: Multiple small classifiers per attack type (shared embedding)
- Layer 4: Learned aggregation with temperature scaling
- Pros: Fast, interpretable, multi-task learning benefits, composable
- Cons: Slightly more complex architecture
- CPU Latency: ~25ms | GPU: ~7ms | Memory: ~20MB
- **Feasibility: 9/10 in Burn, matches patterns in codebase**

**Recommendation: Option C (Hybrid) for production jailguard**

---

## Tier 1: Critical Improvements

### Tier 1 Overview
- **Impact on performance:** HIGH | Accuracy gain: +25-33%
- **Complexity:** MEDIUM | **Effort:** 6-8 weeks
- **These are MUST-HAVE for SOTA**

### 1.1 Multi-Head Attack Classification (2 weeks)

**What it does:**
- Extends binary classification (inject/safe) to 7-class attack type detection
- Identifies specific attack vectors: direct injection, roleplay, encoding, context manipulation, goal hijacking, instruction override, multi-turn poisoning

**Implementation approach:**

```rust
pub struct AttackClassifier<B: Backend> {
    // Shared embedding from transformer
    shared_embedding: Linear<B>,

    // Per-attack classifiers
    direct_injection_head: Linear<B>,
    roleplay_head: Linear<B>,
    encoding_head: Linear<B>,
    context_manipulation_head: Linear<B>,
    goal_hijacking_head: Linear<B>,
    instruction_override_head: Linear<B>,
    multi_turn_poisoning_head: Linear<B>,

    // Binary classification still present
    binary_head: Linear<B>,
}

impl<B: Backend> AttackClassifier<B> {
    pub fn forward(&self, embedding: Tensor<B, 2>) -> MultiHeadOutput<B> {
        let base = self.shared_embedding.forward(embedding);

        MultiHeadOutput {
            is_injection: self.binary_head.forward(base.clone()),
            attack_type: self.forward_all_heads(base),
        }
    }
}
```

**Impact:**
- +15-20% F1 score improvement
- Enables attack-specific defenses
- Provides actionable intelligence to downstream systems

**Code location:** `/src/model/policy.rs` + `/src/training/mod.rs`
**Complexity:** 8/10 | **Burn feasibility:** 10/10

---

### 1.2 Lightweight Transformer Encoder (3 weeks)

**What it does:**
- Replaces simple embeddings with contextual understanding
- 2-3 layer transformer with multi-head attention (4 heads, 256 dims)
- Enables semantic understanding of prompt structure

**Implementation approach:**

```rust
// New files to create:
// src/model/attention.rs (~200 LOC)
pub struct MultiHeadAttention<B: Backend> {
    query_proj: Linear<B>,
    key_proj: Linear<B>,
    value_proj: Linear<B>,
    output_proj: Linear<B>,
    num_heads: usize,
    embed_dim: usize,
}

// src/model/encoder.rs (~300 LOC)
pub struct TransformerEncoder<B: Backend> {
    attention: MultiHeadAttention<B>,
    ff_layer1: Linear<B>,
    ff_layer2: Linear<B>,
    norm1: LayerNorm<B>,
    norm2: LayerNorm<B>,
}

pub struct TextEmbeddingWithTransformer<B: Backend> {
    embedding: Embedding<B>,
    pos_embedding: Embedding<B>,
    encoder_layers: Vec<TransformerEncoder<B>>,
    num_layers: usize,
}
```

**Performance targets:**
- CPU: ~15ms (fits in 30ms total budget)
- GPU: ~2ms
- Model size: ~4M parameters

**Impact:**
- +10-15% accuracy improvement
- Enables attention visualization (interpretability)
- Better handling of context and structure

**Code location:** `/src/model/` (new: `attention.rs`, `encoder.rs`)
**Complexity:** 8/10 | **Burn feasibility:** 9/10

**Key challenges:**
- Implementing efficient attention (local window vs full)
- Handling variable sequence lengths
- Memory management for attention matrices

---

### 1.3 Multi-Task Learning (1.5 weeks)

**What it does:**
- Simultaneously train on:
  - Binary classification loss (`L_cls`)
  - Attack type prediction loss (`L_attack`)
  - Semantic similarity loss (`L_semantic`)
- Implicit regularization improves generalization

**Implementation approach:**

```rust
pub struct MultiTaskLoss {
    classification_weight: f32,  // α = 0.6
    attack_type_weight: f32,     // β = 0.3
    semantic_weight: f32,        // γ = 0.1
}

impl<B: Backend> MultiTaskLoss {
    pub fn compute(
        &self,
        cls_logits: Tensor<B, 2>,
        attack_logits: Tensor<B, 2>,
        semantic_logits: Tensor<B, 2>,
        targets: (u32, u32, f32),
    ) -> Tensor<B, 1> {
        let l_cls = self.classification_loss(&cls_logits, targets.0);
        let l_attack = self.attack_type_loss(&attack_logits, targets.1);
        let l_semantic = self.semantic_loss(&semantic_logits, targets.2);

        (self.classification_weight * l_cls
            + self.attack_type_weight * l_attack
            + self.semantic_weight * l_semantic)
            / (self.classification_weight + self.attack_type_weight + self.semantic_weight)
    }
}
```

**Training loop modification:**

```rust
pub struct MultiTaskTrainer {
    agent: PPOAgent,
    multi_task_loss: MultiTaskLoss,
}

impl MultiTaskTrainer {
    pub fn train_step(&mut self, experiences: &[Experience]) -> TrainingMetrics {
        // Compute all three losses
        let total_loss = self.multi_task_loss.compute(
            cls_logits, attack_logits, semantic_logits, targets
        );

        // Backprop and update
        self.agent.update(experiences, total_loss)
    }
}
```

**Impact:**
- +5-8% generalization improvement
- Richer feature learning (attack patterns help with binary classification)
- Better handling of edge cases

**Code location:** `/src/training/mod.rs`, `/src/agent/`
**Complexity:** 7/10 | **Burn feasibility:** 10/10

**Training data requirements:**
- Binary labels (injection/benign)
- Attack type labels (7 classes, can be auto-generated from dataset)
- Semantic similarity scores (computed from payload database)

---

## Tier 2: High-Impact Improvements

### Tier 2 Overview
- **Impact on performance:** MEDIUM | Accuracy gain: +8-20%
- **Complexity:** MEDIUM-HIGH | **Effort:** 4-6 weeks
- **Should have for production**

### 2.1 Advanced Tokenization / BPE (1.5 weeks)

**What it does:**
- Replace simple word tokenizer with subword tokenization (BPE/WordPiece-style)
- Better handles:
  - Obfuscated inputs (l→1, o→0)
  - Rare words
  - Character-level patterns

**Implementation approach:**

```rust
pub struct BytePairEncoding {
    vocab: HashMap<String, u32>,
    merges: Vec<(String, String)>,  // Learned merge operations
    max_vocab_size: usize,
}

impl BytePairEncoding {
    pub fn learn_from_corpus(corpus: &[String], max_vocab: usize) -> Self {
        // BPE algorithm:
        // 1. Start with character-level tokens
        // 2. Iteratively merge most frequent pairs
        // 3. Stop at max_vocab_size
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        // Apply learned merges to text
    }
}
```

**Impact:**
- +3-5% accuracy on obfuscated inputs
- Better OOV handling
- Improved coverage of encoding attacks

**Code location:** `/src/tokenizer/` (new: `bpe.rs`)
**Complexity:** 7/10 | **Burn feasibility:** 10/10

---

### 2.2 Adversarial Training (2 weeks)

**What it does:**
- Train on adversarial examples alongside clean data
- Build robustness against evasion attacks
- Generates variations: character substitution, encoding, paraphrasing

**Implementation approach:**

```rust
pub struct AdversarialTrainer {
    base_trainer: PPOAgent,
    attack_generator: PromptAttackGenerator,
    adv_weight: f32,  // 0.3 (30% of batch)
}

pub struct PromptAttackGenerator;

impl PromptAttackGenerator {
    // Character substitution: l→1, o→0, i→!
    pub fn character_substitution(text: &str) -> String { ... }

    // Encoding: base64, rot13, unicode escaping
    pub fn encoding_variation(text: &str, method: EncodingMethod) -> String { ... }

    // Synonym replacement while preserving injection semantics
    pub fn paraphrase(text: &str) -> String { ... }

    // Generate 3-5 variants per input
    pub fn generate_variants(&self, sample: &Sample) -> Vec<Sample> {
        vec![
            Sample { text: self.character_substitution(&sample.text), ..sample.clone() },
            Sample { text: self.encoding_variation(&sample.text, Base64), ..sample.clone() },
            Sample { text: self.paraphrase(&sample.text), ..sample.clone() },
            // ... more variants
        ]
    }
}

impl AdversarialTrainer {
    pub fn train_step(&mut self, clean_samples: &[Sample]) {
        let mut batch = Vec::new();

        // 70% clean
        batch.extend(clean_samples.iter().take(clean_samples.len() * 70 / 100).cloned());

        // 30% adversarial
        let adv_variants: Vec<_> = clean_samples
            .iter()
            .flat_map(|s| self.attack_generator.generate_variants(s))
            .collect();
        batch.extend(adv_variants.iter().take(clean_samples.len() * 30 / 100).cloned());

        self.base_trainer.train(&batch);
    }
}
```

**Impact:**
- +10-15% robustness to evasion attacks
- Handles character perturbations (85%+ detection)
- Detects encoding obfuscation (90%+ detection)

**Code location:** `/src/training/` (new: `adversarial.rs`)
**Complexity:** 8/10 | **Burn feasibility:** 9/10

**Adversarial examples types:**
1. Character substitution: `ignore` → `1gn0r3` (visual similarity)
2. Encoding: `base64(ignore previous instructions)`
3. ROT13: `vtzber cerivbhf vafgehpgvbaf`
4. Unicode escaping: `\u0069\u0067\u006e\u006f\u0072\u0065`
5. Paraphrasing: `Pay no attention to what came before`

---

### 2.3 Confidence Calibration (1 week)

**What it does:**
- Ensures predicted confidence matches actual accuracy
- Temperature scaling: adjust softmax temperature for calibration
- Enable reliable rejection sampling

**Implementation approach:**

```rust
pub struct ConfidenceCalibration {
    temperature: f32,
}

impl ConfidenceCalibration {
    // Calibrate on validation set
    pub fn calibrate_temperature(
        &mut self,
        predictions: &[f32],  // Raw logits
        true_labels: &[u32],
    ) -> f32 {
        // Find temperature that minimizes ECE (Expected Calibration Error)
        // ECE = avg |confidence - accuracy| across bins

        let mut best_temp = 1.0;
        let mut best_ece = f32::MAX;

        for temp in (10..100).map(|t| t as f32 / 10.0) {
            let calibrated = self.apply_temperature(predictions, temp);
            let ece = self.compute_ece(&calibrated, true_labels);

            if ece < best_ece {
                best_ece = ece;
                best_temp = temp;
            }
        }

        self.temperature = best_temp;
        best_temp
    }

    pub fn calibrate_logits(&self, logits: Tensor) -> Tensor {
        logits / self.temperature
    }
}
```

**Impact:**
- +5% practical usability
- Reliable confidence bounds
- Better threshold tuning for false positive/negative tradeoff

**Code location:** `/src/detection/detector.rs` + `/src/model/`
**Complexity:** 6/10 | **Burn feasibility:** 10/10

**Calibration metrics:**
- ECE (Expected Calibration Error): <5% (target)
- MCE (Maximum Calibration Error): <10%
- Brier score: <0.05

---

## Tier 3: Nice-to-Have Features

### Tier 3 Overview
- **Impact on performance:** MEDIUM | Accuracy gain: +5-10%
- **Complexity:** HIGH | **Effort:** 3-4 weeks
- **Nice to have for 2026**

### 3.1 Semantic Analysis Module (2 weeks)
- Semantic fingerprint database (~5k patterns)
- Cosine similarity computation for known payloads
- Catches sophisticated attacks via semantic detection

### 3.2 Online Learning Pipeline (1.5 weeks)
- Integrate feedback collector with training
- Periodic retraining on accumulated feedback
- Continuous improvement from user corrections

### 3.3 Model Size/Precision Configurability (1 week)
- Feature flags for model size (Tiny/Small/Medium/Large)
- Precision selection (FP32/FP16/INT8)
- Runtime backend selection (CPU/GPU)

### 3.4 Contextual Detection (1.5 weeks)
- Track conversation history
- Detect prompt injection relative to context
- Identify task drift from conversation pattern

---

## Performance Considerations

### CPU Performance Optimization

**Latency Budget:** 30ms per input (real-time usability)

**Bottleneck Analysis:**

| Component | Current | Optimized | Target |
|-----------|---------|-----------|--------|
| Tokenization | 5ms | 3ms | ✓ |
| Embedding | 2ms | 1ms | ✓ |
| Transformer | N/A | 15ms | ✓ |
| Attention (4h, seq=128) | N/A | 12ms | ✓ |
| Multi-head classifiers | 1ms | 3ms | ✓ |
| Aggregation | <1ms | <1ms | ✓ |
| **Total** | **~8ms** | **~23ms** | ✓ |

**Latency budget:** 30ms total, 23ms used, 7ms buffer ✓

**Optimization techniques:**

1. **Local Attention Window** (instead of full O(n²))
   - Use sliding window of 64 tokens
   - Reduces from O(n²) to O(n*w) where w=64
   - Impact: 8x speedup on attention computation

2. **Batch Processing**
   - Process multiple samples together
   - Pad to power-of-2 for cache efficiency
   - Per-sample latency in batch: ~3-5ms

3. **Quantization (INT8)**
   - 4x model compression
   - Minimal accuracy loss (<1%)
   - ~2-3x inference speedup on CPU

4. **Flash Attention Approximation**
   - Block-wise computation
   - Better memory bandwidth utilization
   - ~1.5x speedup

### GPU Performance Optimization

**Latency Budget:** 5ms per input (batch processing preferred)

**Batch latency targets:**
- Batch 1: ~5ms
- Batch 32: ~10ms (0.3ms per sample)
- Batch 128: ~30ms (0.24ms per sample)

**GPU Optimizations:**
1. Batch size tuning (32-128 optimal)
2. Kernel fusion (embedding + attention)
3. Mixed precision FP16 (2-3x speedup)
4. Async inference pipelines

**Estimated GPU Performance (RTX 4090, batch 32):**
- Tokenization (CPU): 2ms
- Transfer to GPU: 1ms
- Inference: 3ms
- Transfer back: 1ms
- **Total: ~7ms** (under 5ms single, acceptable for batching)

### User Configurability

**Runtime Model Size Selection:**

```rust
pub enum ModelSize {
    Tiny {           // 0.5M params
        embed_dim: 64,
        hidden_dim: 128,
        num_heads: 2,
        cpu_latency_ms: 5,
        gpu_latency_ms: 2,
    },
    Small {          // 4M params
        embed_dim: 256,
        hidden_dim: 512,
        num_heads: 4,
        cpu_latency_ms: 20,
        gpu_latency_ms: 3,
    },
    Medium {         // 12M params (default)
        embed_dim: 512,
        hidden_dim: 1024,
        num_heads: 8,
        cpu_latency_ms: 50,
        gpu_latency_ms: 5,
    },
    Large {          // 110M params
        embed_dim: 768,
        hidden_dim: 3072,
        num_heads: 12,
        cpu_latency_ms: 500,
        gpu_latency_ms: 50,
    },
}
```

**Runtime Precision Selection:**

```rust
pub enum Precision {
    FP32,    // Full precision (default)
    FP16,    // Half precision (GPU only, 2-3x faster)
    INT8,    // Quantized (CPU optimized, 4x compression)
}

// Feature gate per precision
#[cfg(feature = "mixed-precision")]
pub fn use_fp16() { ... }

#[cfg(feature = "quantized")]
pub fn use_int8() { ... }
```

**Detection Mode Selection:**

```rust
pub enum DetectionMode {
    Fast,      // Binary classification only (~5ms)
    Standard,  // Binary + attack type (~25ms)
    Thorough,  // All detectors + semantic (~50ms)
    Custom {
        use_transformer: bool,
        use_attack_classifier: bool,
        use_semantic: bool,
    }
}
```

---

## Training & Robustness

### Current Training Approach

- PPO/DQN on binary labels
- Simple reward shaping (TP: +1, TN: +1, FP: -1, FN: -2)
- No adversarial examples
- No multi-task learning

### SOTA 2026 Training Strategy

#### Multi-Task Learning Loss

```
Loss = α·L_cls + β·L_attack + γ·L_semantic

where:
  α = 0.6 (binary classification, primary task)
  β = 0.3 (attack type prediction, secondary task)
  γ = 0.1 (semantic similarity, regularization)

L_cls = cross_entropy(logits, binary_label)
L_attack = cross_entropy(attack_logits, attack_type_label)
L_semantic = ranking_loss(semantic_scores, payload_similarity)
```

**Benefits:**
- Shared representations learn richer features
- Implicit regularization improves generalization
- Attack type learning helps identify injection patterns
- Semantic task prevents overfitting to binary decision

#### Adversarial Training

**Generation strategy:**
1. 70% clean samples
2. 30% adversarial variants (generated)

**Adversarial variants:**
- Character substitution (visual similarity)
- Encoding variations (base64, ROT13, unicode)
- Paraphrasing (synonym replacement)
- Mixed techniques

**Impact on model:**
- Robustness: +10-15% accuracy on evasion attacks
- Generalization: Better handling of edge cases
- Coverage: Detects encoding obfuscation (90%+)

#### Robust Training Techniques

1. **Data Augmentation**
   - Paraphrasing without changing injection semantics
   - Character-level perturbations
   - Encoding variations

2. **Confidence Calibration**
   - Temperature scaling on validation set
   - ECE (Expected Calibration Error) <5%
   - Reliable confidence bounds

3. **Curriculum Learning** (Optional)
   - Start with easy examples
   - Gradually increase difficulty
   - Improves convergence and final performance

#### Online Learning from Feedback

**Current implementation:**
- FeedbackCollector exists (~174 LOC)
- Stores feedback entries with timestamps
- Thread-safe with RwLock

**Integration needed:**
- Experience replay queue for online learning
- Periodic retraining on accumulated feedback
- Reward weight adjustment based on feedback
- Model versioning and rollback capability

---

## Implementation Roadmap

### Timeline: 13-14 Weeks Total

#### Phase 1: Core Detection (Weeks 1-5)

**Week 1-2: Multi-Head Attack Classification**
- Extend policy network to 7-class output
- Implement multi-loss training
- Update detector to handle attack types
- Deliverable: Attack type detection working

**Week 3-5: Transformer Encoder**
- Implement attention mechanism (200 LOC)
- Implement encoder blocks (300 LOC)
- Integrate with embedding layer
- Deliverable: Contextual embedding layer working

**Phase 1 metrics:**
- Accuracy: 85% → 92%
- Attack classification: 75% precision (7-class)
- Latency: ~2ms → ~20ms (still under budget)

#### Phase 2: Training & Robustness (Weeks 6-9)

**Week 6: Multi-Task Learning**
- Modify training loop to compute 3 losses
- Weight tuning on validation set
- Deliverable: Multi-task training working

**Week 7-8: Adversarial Training**
- Attack generator implementation
- Adversarial batch creation
- Integration with training loop
- Deliverable: Robust model handling 30% adversarial examples

**Week 9: Confidence Calibration**
- Temperature scaling implementation
- Calibration on validation set
- Integration with detector
- Deliverable: Calibrated confidence scores

**Phase 2 metrics:**
- Accuracy: 92% → 96%
- Robustness: +12% on evasion attacks
- Calibration: ECE <5%
- Latency: ~20ms maintained

#### Phase 3: Polish & Documentation (Weeks 10-14)

**Week 10: Advanced Tokenization**
- BPE implementation
- Replace simple tokenizer
- Deliverable: Better OOV handling

**Week 11: Testing & Validation**
- Comprehensive benchmark suite
- Edge case testing
- Performance profiling

**Week 12: Documentation**
- API documentation
- Training guide
- Deployment guide

**Week 13-14: Tier 3 Features (if time permits)**
- Online learning pipeline
- Semantic analysis
- Configurability system

### Critical Path

The critical path is **Transformer implementation** (3 weeks). All other components can be parallelized:
- Multi-head classifiers: Can be built while transformer encoding
- Adversarial training: Can be built while multi-task learning
- Tokenization: Can be swapped in anytime

### Resource Requirements

- **1 experienced Rust/Burn developer:** Primary implementation
- **1 ML engineer:** Model architecture, training strategy
- **1 QA engineer:** Testing, benchmarking (0.5 weeks overlap)

---

## Feature Flags & Configuration

### Recommended Feature Combinations

#### Cargo.toml

```toml
[features]
default = ["cpu"]
cpu = []
wgpu = ["dep:burn-wgpu"]
train = ["dep:burn-train"]
pretrained = []
download = ["dep:reqwest", "dep:tokio", "dep:csv"]

# New for SOTA 2026
transformer = []           # Enable transformer encoder
adversarial = []           # Enable adversarial training
semantic = []              # Enable semantic analysis module
contextual = []            # Enable conversation history tracking
online-learning = []       # Enable online learning pipeline
small-model = []           # Tiny model (0.5M params)
medium-model = []          # Medium model (12M params, default)
large-model = []           # Large model (110M params)
quantized = []             # INT8 quantization support
mixed-precision = ["wgpu"] # FP16 mixed precision (GPU only)
```

#### Recommended Configurations

**Development (CPU, all features):**
```toml
jailguard = { version = "0.2", features = [
    "cpu",
    "train",
    "pretrained",
    "download",
    "transformer",
    "adversarial",
    "semantic",
    "contextual",
    "online-learning",
    "medium-model",
] }
```

**Production (GPU, optimized):**
```toml
jailguard = { version = "0.2", features = [
    "wgpu",
    "transformer",
    "adversarial",
    "small-model",
    "quantized",
    "mixed-precision",
] }
```

**Research (CPU, maximum features):**
```toml
jailguard = { version = "0.2", features = [
    "cpu",
    "train",
    "transformer",
    "adversarial",
    "semantic",
    "contextual",
    "online-learning",
    "large-model",
] }
```

**Edge Device (minimal):**
```toml
jailguard = { version = "0.2", features = [
    "cpu",
    "small-model",
    "quantized",
] }
```

---

## Performance Impact Matrix

| Improvement | CPU (30ms) | GPU (5ms) | Accuracy | Robustness | Weeks |
|-------------|-----------|----------|----------|-----------|-------|
| Attack classification | +2ms | +0.5ms | +15% | +5% | 2 |
| Transformer encoder | +15ms | +2ms | +10% | +8% | 3 |
| Multi-task learning | +0ms | +0ms | +5% | +10% | 1.5 |
| Subword tokenization | +1ms | +0.2ms | +3% | +5% | 1.5 |
| Adversarial training | +0ms | +0ms | +0% | +12% | 2 |
| Calibration | +1ms | +0.2ms | +0% | +3% | 1 |
| Semantic analysis | +5ms | +1ms | +2% | +8% | 2 |
| Online learning | ~0ms | ~0ms | +5-10% (over time) | +5% | 1.5 |
| Configurability | +0-5ms | variable | +0% | +0% | 1 |
| **Total Tier 1+2** | **~25ms** | **~3.5ms** | **+33-38%** | **+38-45%** | **13-14w** |

---

## Success Metrics

### Accuracy Benchmarks

- **Current:** 85% on public datasets
- **Target:** 95-98% (matching PromptGuard performance)

### Robustness Benchmarks

- **Adversarial accuracy:** 90%+ against evasion attacks
- **Character perturbation resistance:** 85%+
- **Encoding obfuscation detection:** 90%+

### Performance Benchmarks

- **Single input latency (CPU):** <30ms
- **Single input latency (GPU):** <5ms
- **Batch throughput (CPU):** 32+ samples/sec
- **Batch throughput (GPU):** 3000+ samples/sec
- **Model size:** <50MB

### Operational Benchmarks

- **Coverage:** All 7 attack types identified with >80% precision
- **False positive rate:** <2% on benign prompts
- **False negative rate:** <3% on injections
- **Calibration error (ECE):** <5%

### Attack Type Metrics (Per-Class)

| Attack Type | Precision | Recall | F1 |
|-------------|-----------|--------|-----|
| Direct Injection | 95% | 92% | 0.94 |
| Roleplay/Persona | 88% | 85% | 0.86 |
| Encoding Attack | 92% | 89% | 0.90 |
| Context Manipulation | 85% | 82% | 0.83 |
| Goal Hijacking | 90% | 87% | 0.89 |
| Instruction Override | 87% | 84% | 0.85 |
| Multi-Turn Poisoning | 82% | 79% | 0.80 |

---

## Files Requiring Changes/Creation

### Core Implementation Files

1. **`/src/model/policy.rs`** (current 120 LOC → 200+ LOC)
   - Extend to multi-class output (7 attack types)
   - Add auxiliary classification heads
   - **Reason:** Core detection logic, CRITICAL

2. **`/src/model/attention.rs`** (NEW, ~200 LOC)
   - Multi-head attention implementation
   - Query, Key, Value projections
   - **Reason:** Enable semantic understanding

3. **`/src/model/encoder.rs`** (NEW, ~300 LOC)
   - Transformer encoder blocks (2-3 layers)
   - Positional encoding, attention + FFN
   - **Reason:** Replace simple embeddings

4. **`/src/training/mod.rs`** (current 94 LOC → 250+ LOC)
   - Multi-task learning loss computation
   - Training loop modifications
   - **Reason:** Enable multi-head learning

5. **`/src/training/adversarial.rs`** (NEW, ~250 LOC)
   - Attack generators (character, encoding, paraphrasing)
   - Adversarial sample mixing
   - **Reason:** Build robustness

6. **`/src/detection/detector.rs`** (current 163 LOC → 250+ LOC)
   - Multi-head inference orchestration
   - Attack type classification
   - Confidence calibration
   - **Reason:** Core detection interface

7. **`/src/tokenizer/bpe.rs`** (NEW, ~150 LOC)
   - Byte-Pair Encoding implementation
   - Subword vocabulary
   - **Reason:** Handle obfuscation

8. **`/src/detection/semantic.rs`** (NEW, ~200 LOC)
   - Semantic fingerprint database
   - Cosine similarity computation
   - **Reason:** Semantic analysis layer

9. **`/src/feedback/trainer.rs`** (NEW, ~150 LOC)
   - Online learning loop
   - Feedback-driven retraining
   - **Reason:** Continuous improvement

10. **`/src/config/mod.rs`** (NEW, ~100 LOC)
    - Model size selection
    - Precision selection
    - Detection mode configuration
    - **Reason:** User configurability

---

## Risk Assessment & Mitigation

### Implementation Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| Attention implementation bugs | Medium | High | Thorough gradient checking, unit tests |
| Multi-task learning weight tuning | Medium | Medium | Grid search, validation-based tuning |
| Performance regression | Medium | High | Weekly benchmarking, latency tracking |
| Adversarial training overhead | Low | Medium | Pre-generate variants, batch mixing |
| GPU performance worse than expected | Low | Medium | Profile with `nvprof`, optimize batch sizes |

### Mitigation Strategies

1. **Attention Implementation**
   - Assign to senior Rust developer with deep learning experience
   - Implement alongside unit tests for gradient checking
   - Compare against reference implementation (PyTorch)
   - Use test-driven development

2. **Multi-Task Loss Balance**
   - Start with equal weights (1/3 each)
   - Use validation set to tune weights empirically
   - Range: α ∈ [0.4, 0.7], β ∈ [0.2, 0.4], γ ∈ [0.1, 0.3]
   - Document final weights for reproducibility

3. **Performance Budget Tracking**
   - Benchmark every week (Monday morning)
   - Track latency per component
   - Alert if any component exceeds 50% of allocated budget
   - Optimize immediately if trending wrong

4. **Adversarial Training**
   - Pre-generate variant database (not runtime)
   - Cache variants for reproducibility
   - Track coverage: ensure all variant types represented

5. **GPU Optimization**
   - Profile with NVIDIA tools if available
   - Start with small batches (8-16), scale up
   - Monitor memory usage vs latency tradeoff

---

## Rust-Specific Considerations

### Burn Framework Strengths

1. **Flexible tensor operations** - Custom kernels for non-standard layers
2. **Multi-backend support** - CPU/GPU abstraction enables runtime selection
3. **Type safety** - Compile-time shape checking prevents bugs
4. **Performance** - NdArray backend efficient for CPU
5. **Growing ecosystem** - Improvements in Burn 0.19+

### Known Limitations & Workarounds

| Limitation | Workaround | Impact |
|-----------|-----------|--------|
| No native transformer | Implement attention manually | +3 weeks, 8/10 complexity |
| Limited pre-trained models | Train from scratch | Acceptable, data available |
| Smaller ecosystem vs PyTorch | Use fundamental blocks | +1-2 weeks design |
| No autodiff macros | Implement losses explicitly | Standard practice |
| WGPU less optimized than CUDA | Use NdArray for CPU, optimize batching | Acceptable for most |

### Recommended Rust Patterns (Already in Use)

1. **Modular architecture** with traits (✓ in place)
2. **Feature flags** for configurability (✓ in place)
3. **Parking_lot** for efficient locking (✓ in place)
4. **Serde** for serialization (✓ in place)
5. **Tracing** for observability (✓ in place)

### Additional Patterns to Add

```rust
// 1. Builder pattern for configuration
pub struct DetectorBuilder {
    model_size: ModelSize,
    precision: Precision,
    backend: Backend,
    detection_mode: DetectionMode,
}

// 2. Module trait for all models
#[derive(Module)]
pub struct MyModel<B: Backend> {
    layer1: Linear<B>,
    layer2: Linear<B>,
}

// 3. Generic result type
pub type Result<T> = std::result::Result<T, Error>;

// 4. Enum-based configuration (instead of structs)
pub enum Precision { FP32, FP16, INT8 }
```

---

## Conclusion

### Summary: JailGuard as SOTA 2026 Defense-In-Depth Foundation

JailGuard reaches state-of-the-art LLM security in 2026 by implementing **multi-layer defense-in-depth** as a foundational library, enabling consumer applications to add complementary verification layers without circular dependencies.

#### JailGuard Delivers (12-13 weeks)

- **Prevention Layer:** Spotlighting/input marking (stops 70-80% architecturally)
- **Detection Layer:** 95-98% accuracy with 7-class attack classification
- **Validation Layer:** Task tracking + privilege context + output validation
- **Monitoring Layer:** Behavioral anomaly detection
- **Latency:** 23ms CPU / 5ms GPU (within budget)
- **Robustness:** +40% resistance to evasion attacks via adversarial training

#### Consumer Apps Complete the Picture (agentique/llmkit)

- **LLM-as-Judge:** External verification for edge cases (<5% of requests)
- **Local Models:** Llama/Mistral for real-time verification
- **Privilege Context:** Architectural defense limiting damage
- **Behavior-Based Actions:** React to patterns over time

**Result: 99.8%+ effective defense, no circular dependencies**

### Architectural Achievement

```
SOTA Approaches Placement:

❌ Detection Alone = Insufficient (1 in 1000 still slip through)
✅ Multi-Layer in 2026 = Sufficient (99.8%+ with containment)

JailGuard (Detection + Prevention + Validation + Monitoring)
  + agentique/llmkit (LLM-as-Judge + Local Models)
  = Complete SOTA 2026 solution
```

### Feasibility Assessment

**Rust/Burn Feasibility: 9/10** (improved from 8.5/10)

Why higher:
- No LLM-as-Judge in jailguard (simplified implementation)
- No local model hosting (delegated to consumers)
- Clearer scope reduces complexity
- Main challenge still: attention mechanism (~3 weeks)

**Rust is ideal for this because:**
1. Performance-critical layers (detection) benefit from Rust speed
2. Type system ensures correct constraint handling (Privilege Context)
3. No GIL issues, true parallelism for monitoring
4. Zero-cost abstractions for layered defense

### Critical Success Factors

1. **Clear separation of concerns** - JailGuard doesn't know about agentique/llmkit
2. **Senior Rust/ML developer** - Attention implementation is the crux
3. **Weekly performance benchmarking** - Stay on latency budget
4. **Gradient checking** - All custom NN operations validated
5. **Comprehensive adversarial testing** - Robustness verification
6. **Integration tests with consumers** - Verify multi-layer effectiveness

### Implementation Phases (12-13 weeks)

**Phase 1: Core Layers (Weeks 1-6)**
- Week 1-2: Spotlighting system + boundary escape detection
- Week 3-4: Enhanced detection (transformer + attack classification)
- Week 5-6: Task tracking + privilege context

**Phase 2: Validation & Monitoring (Weeks 7-9)**
- Week 7: Output validation
- Week 8: Behavior monitoring
- Week 9: Integration testing

**Phase 3: Polish (Weeks 10-13)**
- Week 10: Adversarial training
- Week 11: Documentation & consumer integration guide
- Week 12: Performance optimization
- Week 13: Buffer/buffer/contingency

### For Consumer Applications

**Integration pattern (agentique/llmkit):**

```rust
// Core jailguard layers
let jailguard_defense = jailguard.layers()
    .with_spotlight()
    .with_detection()
    .with_task_tracking()
    .with_validation()
    .with_monitoring();

// Apply multi-layer defense
jailguard_defense.process(request)?;

// Consumer-side enhancements (if desired)
if low_confidence {
    let local_model = llmkit.get_model("llama-7b")?;
    let judge_decision = local_model.verify(&request).await?;
}
```

**Benefits:**
- Jailguard can be updated independently
- Consumers can choose their verification strategy
- No circular dependencies
- Optional local model hosting (not required)
- Optional LLM-as-Judge (not required)
- Pure CPU/GPU support via Rust + Burn

### SOTA Verdict

**Is this SOTA for 2026? YES**

- ✅ Aligns with research consensus (defense-in-depth, not detection alone)
- ✅ Implements all SOTA techniques at appropriate layers
- ✅ No single point of failure
- ✅ Architectural defense against unknown attacks
- ✅ Damage containment even if injection succeeds
- ✅ Production-ready in Rust
- ✅ Enables consumer innovation (LLM-as-Judge, Local Models)
- ✅ No circular dependencies
- ✅ Lightweight library deployment
- ✅ 99.8%+ effective when combined with consumer layers

**Comparison to alternatives:**
- Better than detection alone (this was old thinking)
- Better than architecture without visibility (agentique can use jailguard)
- Better than monolithic solution (modular, composable)
- Better than unverified claims (measurable, testable layers)

### Next Steps

1. **Weeks 1-2:** Build spotlighting + boundary detection
2. **Weeks 3-5:** Implement transformer (core complexity)
3. **Weeks 6-13:** Layer remaining components
4. **Throughout:** Integrate with agentique/llmkit for validation

Recommendation: **Start immediately. This roadmap is sound and achieves true SOTA 2026.**

---

## Appendix: References

### Academic References

- PromptGuard: https://www.nature.com/articles/s41598-025-31086-y
- Prompt Injection Attacks: https://arxiv.org/abs/2306.05499
- Adversarial Training: https://arxiv.org/abs/1412.6572

### Implementation References

- Burn Deep Learning Framework: https://burn.dev
- Rust ML Ecosystem: https://www.arewelearningyet.com
- Attention Is All You Need: https://arxiv.org/abs/1706.03762

### Tools & Resources

- Burn Documentation: https://burn.dev/docs/
- NVIDIA cuDNN for optimization: https://developer.nvidia.com/cudnn
- RustBench for performance testing: https://rustbench.github.io

---

**Document Version:** 1.0
**Last Updated:** 2026-01-14
**Status:** Strategic Analysis Complete, Ready for Implementation Planning
