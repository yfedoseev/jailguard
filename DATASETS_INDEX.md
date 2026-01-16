# JailGuard Prompt Injection & Jailbreak Datasets - Research Index

This directory contains comprehensive research on publicly available datasets to expand your training data beyond the baseline 662-sample deepset/prompt-injections dataset.

## Quick Navigation

### For Busy People (TL;DR)
- **Summary**: See `DATASETS_QUICK_REFERENCE.txt` (2-3 min read)
- **Immediate Action**: Run `examples/download_datasets.sh`
- **Quick Stats**: ~31K MIT-licensed samples available immediately

### For Integration
- **Full Details**: See `DATASETS.md` (30-45 min read)
- **Download Script**: `examples/download_datasets.sh`
- **Integration Guide**: Section "Integration Guide for Your Pipeline" in `DATASETS.md`

### For Decision Making
- **Licensing Summary**: `DATASETS.md` - "Licensing Summary" section
- **Tier Comparison**: `DATASETS_QUICK_REFERENCE.txt` - Dataset tables
- **Recommendations**: `DATASETS.md` - "TIER 1", "TIER 2" sections

## File Descriptions

### 1. DATASETS.md (Main Document - 17 KB)
**Purpose**: Comprehensive research reference document

**Contents**:
- Overview and tier classification of 14+ datasets
- Detailed specifications for each dataset (samples, license, format, access)
- Integration code examples
- Licensing breakdown
- Recommended integration priority
- 4-week implementation roadmap
- Citation information
- Research links

**Best For**: 
- Getting complete information about any dataset
- Understanding integration options
- Verification before implementation
- Citation and reference information

**Key Sections**:
- TIER 1: Largest and Most Comprehensive Datasets
- TIER 2: Medium-Sized Specialized Datasets
- TIER 3: Specialized/Niche Datasets
- TIER 4: Focused Collections
- GitHub Research Repositories with Datasets
- Integration Guide for Your Pipeline
- Estimated Total Dataset Size

---

### 2. DATASETS_QUICK_REFERENCE.txt (Summary - 8.3 KB)
**Purpose**: Quick lookup and decision-making guide

**Contents**:
- At-a-glance comparison tables
- Tier-organized dataset summaries
- Key statistics and metrics
- Licensing breakdown
- 4-week integration roadmap
- Quick start commands

**Best For**:
- Quick decisions about which datasets to use
- Comparing datasets side-by-side
- Planning implementation timeline
- Getting started immediately

**Structure**:
- Tier 1 Datasets (Recommended immediate)
- Tier 2 Datasets (Secondary)
- Tier 3 Datasets (Tertiary)
- Total Dataset Potential
- Licensing Breakdown
- Integration Roadmap
- Quick Start Command

---

### 3. DATASETS_INDEX.md (This File)
**Purpose**: Navigation and quick reference guide

**Contents**:
- File descriptions
- Quick start instructions
- Key statistics
- Most important datasets highlighted

---

### 4. examples/download_datasets.sh (4.6 KB)
**Purpose**: Automated dataset download script

**Features**:
- Automatic download from HuggingFace and GitHub
- Color-coded progress output
- Handles multiple data sources
- Error handling and fallbacks
- Summary report

**Usage**:
```bash
cd data/external
bash ../../examples/download_datasets.sh .
```

**Downloads**:
- TrustAIRLab (15,140 samples)
- SPML (16,012 samples)
- xTRam1 (10,296 samples)
- Giskard (variable)
- JailbreakBench (200 samples)
- AdvBench (520 samples)

---

## Key Statistics at a Glance

### BASELINE
- deepset/prompt-injections: **662 samples**

### TIER 1 (Immediate - MIT Licensed)
| Dataset | Samples | License | Impact |
|---------|---------|---------|--------|
| TrustAIRLab | 15,140 | MIT | Real-world data |
| SPML | 16,012 | MIT | System prompt context |
| **Total** | **31,152** | **MIT** | **47x expansion** |

### TIER 2 (Secondary)
| Dataset | Samples | License | Specialty |
|---------|---------|---------|-----------|
| JailbreakV-28K | 28,000 | Unknown | 16 policies |
| Mindgard | ~50,000 | CC-NC | Evasion techniques |
| xTRam1 | 10,296 | Unknown | Categorized attacks |
| BeaverTails | 300,000+ | CC-NC | Harm categories |
| JailbreakBench | 200 | Unknown | NeurIPS 2024 |
| **Subtotal** | **~390K** | Mixed | Specialized |

### TOTAL AVAILABLE
- **Without dedup**: 125K+ samples
- **After 15-20% dedup**: 101K-108K samples
- **140x+ expansion** of baseline

### COMMERCIAL SAFE
- TrustAIRLab + SPML: 31,352 samples (47x)
- MIT licensed, production-ready

---

## Recommended Quick Start (< 1 Hour)

### Step 1: Read Overview (5 min)
```bash
cat DATASETS_QUICK_REFERENCE.txt
```

### Step 2: Download Tier 1 Data (20-30 min)
```bash
mkdir -p data/external
cd data/external
bash ../../examples/download_datasets.sh .
cd ../..
```

### Step 3: Review Details (15-20 min)
```bash
# Read the comprehensive guide
head -n 500 DATASETS.md
```

### Step 4: Plan Integration (10 min)
- Decide which tier(s) you need
- Check licensing for your use case
- Plan 4-week implementation

---

## Most Important Datasets

### If You Only Pick One Tier:
**PRIORITY 1 - Do This First**
- TrustAIRLab (15,140) + SPML (16,012) = 31,152 samples
- MIT licensed (commercial OK)
- 47x baseline expansion
- ~2-4 hours integration
- Real-world + synthetic balance

---

## Integration Status

### What You Have Now
- ✅ Deepset loader in `src/dataset/deepset.rs`
- ✅ CSV parsing support
- ✅ JSON loading support
- ✅ HuggingFace Datasets API ready

### What You Can Easily Add
- CSV-based datasets (TrustAIRLab, SPML, xTRam1)
- HuggingFace Datasets API loads (all major datasets)
- Parquet support (via pandas intermediary)

### Implementation Effort
- Priority 1: 4-6 hours
- Priority 2: 6-8 hours
- Priority 3: 4-6 hours
- Total: 14-20 hours for full integration

---

## License Summary

| Type | Datasets | Count | Notes |
|------|----------|-------|-------|
| **MIT** | TrustAIRLab, SPML, JailbreakBench | 3 | ✅ Commercial OK |
| **CC-BY-NC-4.0** | Mindgard, BeaverTails, Qualifire | 3 | ❌ Non-commercial only |
| **Research Only** | CyberSecEval3 | 1 | ❌ Evaluation only |
| **Unknown** | JailbreakV-28K, xTRam1, AdvBench, others | 5+ | ⚠️ Check before use |

---

## Key Insights

1. **31K commercial-safe samples** available immediately (MIT licensed)
2. **TrustAIRLab** has real-world data from Reddit, Discord, websites (2022-2023)
3. **SPML** provides synthetic data with attack complexity labels (0-10)
4. **Mindgard** specializes in evasion techniques (character injection, emoji smuggling)
5. **100K+ total samples** available after deduplication
6. **Well-researched sources**: CCS 2024, NeurIPS 2024, ArXiv publications
7. **Easy integration**: Your codebase already supports needed formats

---

## Next Steps

### Immediate (This Week)
- [ ] Run `examples/download_datasets.sh` to get Tier 1 data
- [ ] Read `DATASETS.md` for integration details
- [ ] Update `src/dataset/` to load new datasets

### Short-term (Week 1-2)
- [ ] Integrate TrustAIRLab + SPML into training
- [ ] Test baseline improvements
- [ ] Document data loading in README

### Medium-term (Week 2-4)
- [ ] Add Tier 2 datasets (xTRam1, Mindgard)
- [ ] Implement deduplication logic
- [ ] Benchmark performance improvements

### Long-term (Week 4+)
- [ ] Add BeaverTails subset for multi-task learning
- [ ] Setup evaluation on CyberSecEval3
- [ ] Generate training statistics report

---

## References & Links

### Primary Datasets (Recommended)
1. [TrustAIRLab](https://huggingface.co/datasets/TrustAIRLab/in-the-wild-jailbreak-prompts)
2. [SPML](https://huggingface.co/datasets/reshabhs/SPML_Chatbot_Prompt_Injection)
3. [JailbreakV-28K](https://huggingface.co/datasets/JailbreakV-28K/JailbreakV-28k)
4. [xTRam1](https://huggingface.co/datasets/xTRam1/safe-guard-prompt-injection)

### Academic Papers
- [CCS 2024] "Do Anything Now": https://arxiv.org/abs/2308.03825
- [SPML DSL] https://arxiv.org/abs/2402.11755
- [BeaverTails] https://arxiv.org/abs/2307.04657
- [NeurIPS 2024] JailbreakBench: https://github.com/JailbreakBench/jailbreakbench

### Research Repositories
- [Jailbreak LLMs]: https://github.com/verazuo/jailbreak_llms
- [Open-Prompt-Injection]: https://github.com/liu00222/Open-Prompt-Injection
- [Giskard]: https://github.com/Giskard-AI/prompt-injections

---

## Questions?

- **For dataset details**: See `DATASETS.md`
- **For quick comparison**: See `DATASETS_QUICK_REFERENCE.txt`
- **For download help**: Run `examples/download_datasets.sh --help`
- **For licensing clarification**: Check individual dataset pages

---

## Document Version
- **Created**: January 15, 2026
- **Last Updated**: January 15, 2026
- **Research Scope**: HuggingFace, GitHub, Academic Databases
- **Datasets Cataloged**: 14+ public datasets
- **Total Samples Available**: 100K-125K (after dedup)
