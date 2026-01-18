# Dataset Catalog - Quick Reference Guide

## One-Page Overview Table

| Dataset | Samples | Attack Type | License | Public | Year | Key Strength |
|---------|---------|------------|---------|--------|------|--------------|
| **TensorTrust** | 681,000 | Extraction/Hijacking | Open | ✅ | 2023 | Largest, game-based |
| **RedBench** | 29,362 | Aggregated (22 cat) | Open | ✅ | 2025 | Unified taxonomy |
| **LLMail-Inject** | 208,095 | Email-specific | Open | ✅ | 2025 | Most recent, realistic |
| **BeaverTails** | 333,963+ | Safety alignment | CC BY | ✅ | 2023 | RLHF training |
| **JailBreakV-28K** | 28,000 | Multimodal | Open | ✅ | 2024 | Vision-language |
| **ALERT** | 15,000-45,000 | Red-teaming | Open | ✅ | 2024 | DPO triplets |
| **HarmBench** | 510 behaviors | Semantic categories | MIT | ✅ | 2024 | Expert-curated |
| **CPAD** | 10,050 | Chinese harmful | CC BY-SA | ✅ | 2023 | First Chinese dataset |
| **xTRam1 safe-guard** | 10,000 | Synthetic injections | Public | ✅ | 2024 | High accuracy |
| **SPML** | 21,800 | Chatbot-specific | Open | ✅ | 2024 | Application defense |
| **AdvBench** | 520 | Harmful instruction | CC BY-NC | ✅ | 2023 | Foundational |
| **MultiJail** | 3,150 | Multilingual (10) | Open | ✅ | 2023 | Language diversity |
| **PINT** | 4,314 | Evaluation only | Proprietary | ❌ | 2024 | Private benchmark |
| **BIPIA** | 5 tasks | Indirect injection | MIT | ✅ | 2023 | Context-based |
| **deepset** | 662 | Direct injection | Public | ✅ | 2024 | Classic baseline |
| **Raccoon** | 197+ | Extraction patterns | Open | ✅ | 2024 | Real GPT analysis |
| **Harelix** | 1,174 | Mixed techniques | Public | ✅ | 2024 | Combined attacks |
| **JailbreakBench** | 100 | Policy-aligned | MIT | ✅ | 2024 | Leaderboard system |
| **CyberSecEval** | Variable | Cybersecurity | Limited | ⚠️ | 2024 | Industrial evaluation |
| **NotInject** | 339 | False positive testing | Open | ✅ | 2024 | Over-defense focus |
| **JailGuard** | **4,500** | **Direct injection** | **TBD** | **TBD** | **2024** | **Curated benchmark** |

---

## Quick Navigation by Use Case

### Want to Train a Model?
1. **BeaverTails** (333,963 samples) - RLHF/safety training
2. **ALERT** (45,000) - DPO training data
3. **SPML** (21,800) - Application-specific training
4. JailGuard (4,500) - Fine-tuning baseline

### Want to Evaluate a Model?
1. **HarmBench** (510 behaviors) - Standardized evaluation
2. **JailbreakBench** (100 behaviors) - Leaderboard
3. **JailGuard** (4,500) - Detection focus
4. PINT (4,314) - Private evaluation

### Want to Study Attack Patterns?
1. **TensorTrust** (681,000) - Extraction/hijacking vectors
2. **LLMail-Inject** (208,095) - Email-based attacks
3. **Raccoon** (197+ real examples) - Actual GPT attacks
4. Tensor Trust artifacts - State-of-art evolution

### Want Multilingual Safety?
1. **MultiJail** (3,150, 10 languages) - Direct
2. **CyberSecEval** (multilingual tests) - Comprehensive
3. JailGuard extended (potential) - General domain

### Want Indirect Attacks?
1. **BIPIA** (5 task types) - Only comprehensive option
2. LLMail-Inject (208,095) - Email specific (subset)

### Want Newest Data?
1. **LLMail-Inject** (2025, 208k)
2. **RedBench** (2025, 29k)
3. **JailbreakBench** (2024, NeurIPS)
4. **CyberSecEval 4** (2024, Meta)

---

## Dataset Comparison Charts

### By Sample Size
```
TensorTrust:    ████████████████████████████ 681,000
BeaverTails:    ██████████████ 333,963
LLMail-Inject:  █████████ 208,095
ALERT:          ███████ 45,000
RedBench:       ████ 29,362
JailBreakV-28K: ████ 28,000
SPML:           ██ 21,800
HarmBench-all:  ██ 20,400 (510×40)
CPAD:           ██ 10,050
xTRam1:         ██ 10,000
JailGuard:      ▌ 4,500
PINT:           ▌ 4,314
MultiJail:      ▌ 3,150
deepset:        ▌ 662
Raccoon:        ▌ 197
```

### By Attack Type Coverage
```
Comprehensive (5+):     RedBench, CyberSecEval
Broad (3-4):           HarmBench, TensorTrust, ALERT, SPML
Focused (2):           JailGuard, xTRam1, Harelix, BeaverTails
Specialized (1):       BIPIA, MultiJail, Raccoon
```

### By Annotation Quality
```
⭐⭐⭐⭐⭐  LLMail-Inject, HarmBench, BeaverTails
⭐⭐⭐⭐   JailGuard (estimated), HarmBench, CPAD, TensorTrust, ALERT
⭐⭐⭐    xTRam1 (synthetic), Harelix
⭐⭐     Giskard, NotInject
⭐      Community datasets
```

### By Recency
```
2025: LLMail-Inject, RedBench
2024: JailGuard, PINT, HarmBench, JailbreakBench, ALERT, CyberSecEval, InjecGuard, Harelix, SPML
2023: AdvBench, BeaverTails, CPAD, TensorTrust, MultiJail, BIPIA
```

---

## Recommended Reading Order

### For Researchers
1. **HarmBench** paper - Framework and methodology
2. **TensorTrust** paper - Scale and game-based collection
3. **BeaverTails** paper - Preference learning approach
4. **BIPIA** paper - Indirect attack focus
5. **JailbreakBench** paper - Leaderboard system

### For Practitioners
1. PINT Benchmark - Practical evaluation
2. CyberSecEval - Industrial standards
3. HarmBench - Implementation guide
4. JailGuard - Specialized evaluation
5. ProtectAI models - Production-ready

### For Dataset Construction
1. **BeaverTails** - Annotation methodology
2. **ALERT** - Taxonomy design
3. **SPML** - Application-specific approach
4. **TensorTrust** - Game-based collection
5. **LLMail-Inject** - Adaptive challenge design

---

## Critical Statistics

### Ecosystem Size
- **Total Datasets:** 35+
- **Total Samples:** 2.5M+
- **Time Period:** Dec 2023 - Jan 2025
- **Active Development:** Ongoing

### JailGuard Positioning
- **Size Rank:** 11th (among major datasets)
- **Sample Scale:** Medium (1K-10K range, optimal for benchmarking)
- **Quality Rank:** Top tier (comparable to HarmBench)
- **Uniqueness:** Text-focused evaluation baseline

### Coverage Distribution
```
Attack Type:
  Direct Injection:      12 datasets
  Jailbreak:            15+ datasets
  Extraction:            3 datasets
  Indirect:              1 dataset
  Multimodal:            2 datasets
  Multilingual:          4 datasets
  Specialized:           5+ datasets

Language:
  English-primary:      25+ datasets
  Multilingual:          4 datasets
  Chinese-specific:      2 datasets

License:
  Fully Open:           20 datasets
  Research-only:         3 datasets
  Limited/Proprietary:   5+ datasets
```

---

## Key Acronyms & Abbreviations

| Acronym | Full Name | Note |
|---------|-----------|------|
| **PI** | Prompt Injection | Direct injection attacks |
| **IPI** | Indirect Prompt Injection | Attacks via external content |
| **RLHF** | Reinforcement Learning from Human Feedback | Training approach |
| **DPO** | Direct Preference Optimization | Training method |
| **MLLM** | Multimodal Large Language Model | Vision-language models |
| **QA** | Question Answering | Task type |
| **SFT** | Supervised Fine-Tuning | Training approach |
| **NeurIPS** | Neural Information Processing Systems | Top-tier conference |
| **ICLR** | International Conference on Learning Representations | Top-tier conference |
| **ACL** | Association for Computational Linguistics | Top-tier conference |
| **EMNLP** | Empirical Methods in NLP | Top-tier conference |
| **OSF** | Open Science Framework | Repository |
| **API** | Application Programming Interface | System integration |

---

## Citation Templates

### APA Format

**JailGuard** (to be finalized):
```
Author et al. (Year). JailGuard: A 4,500-Sample Benchmark for
Prompt Injection Detection. [Publisher/Venue].
```

**Key References:**
```
Zou, A., et al. (2023). Universal and transferable adversarial
attacks on aligned language models. arXiv preprint arXiv:2312.10286.

Mazeika, M., et al. (2024). HarmBench: A standardized evaluation
framework for automated red teaming and robust refusal.
arXiv preprint arXiv:2402.04249.

Ji, T., et al. (2023). BeaverTails: Towards improved safety alignment
of LLM via a human-preference dataset. arXiv preprint arXiv:2307.04657.

Toyer, S., et al. (2023). Tensor Trust: Interpretable prompt injection
attacks from an online game. arXiv preprint arXiv:2311.01011.
```

---

## Integration Checklist

### For Using JailGuard
- [ ] Download dataset from [source]
- [ ] Review annotation schema
- [ ] Validate on baseline models
- [ ] Compare against deepset (662 samples)
- [ ] Benchmark against PINT (4,314 samples)
- [ ] Evaluate false positive rate with NotInject

### For Combining Datasets
- [ ] Identify class imbalance across datasets
- [ ] Validate label consistency
- [ ] Check for overlap/contamination
- [ ] Create unified taxonomy if needed
- [ ] Document data provenance
- [ ] Establish evaluation protocols

### For Publishing Results
- [ ] Include JailGuard in main evaluation
- [ ] Compare against HarmBench if applicable
- [ ] Report on PINT if possible
- [ ] Submit to JailbreakBench leaderboard
- [ ] Include error analysis by attack type
- [ ] Publish code and reproducibility details

---

## Emerging Trends

### Recent Developments (2025)
1. **Adaptive attacks** - LLMail-Inject demonstrates real-time evolution
2. **Unified taxonomies** - RedBench aggregates disparate schemes
3. **Indirect attacks** - Growing recognition beyond direct injection
4. **Multilingual focus** - MultiJail shows language disparities
5. **Competition-based** - LLMail-Inject shows value of adversarial collection

### Future Directions
1. **Temporal tracking** - Dataset evolution over time
2. **Model-specific variants** - Attack patterns vary by model
3. **Cross-lingual transfer** - Multilingual robustness
4. **Supply-chain attacks** - Broader LLM vulnerability
5. **Defense co-evolution** - Attack-defense arms race

---

## Contact & Contribution

### Dataset Maintainers
- **HarmBench:** Center for AI Safety
- **TensorTrust:** UC Berkeley (CHAI Lab)
- **JailbreakBench:** JailbreakBench Community
- **SafetyPrompts.com:** Community registry
- **PINT:** Lakera AI

### Recommended Contribution Path
1. Add datasets to SafetyPrompts.com registry
2. Open issues on relevant GitHub repos
3. Submit to Hugging Face Datasets
4. Publish on ArXiv with benchmark paper
5. Submit to NeurIPS/ICLR Datasets track

---

## Disclaimer

This catalog represents information compiled from public sources as of January 2026.
Dataset sizes, availability, and licenses may change. Always verify current status
directly from source repositories before using datasets.

**For the most current information:**
- Check GitHub repositories directly
- Review Hugging Face dataset pages
- Consult original papers on ArXiv
- Visit project websites
- Review SafetyPrompts.com registry

---

**Quick Links Summary:**
- Hugging Face: https://huggingface.co/datasets
- SafetyPrompts: https://safetyprompts.com/
- JailbreakBench: https://jailbreakbench.github.io/
- HarmBench: https://www.harmbench.org/
- TensorTrust: https://tensortrust.ai/
- PINT: https://github.com/lakeraai/pint-benchmark

