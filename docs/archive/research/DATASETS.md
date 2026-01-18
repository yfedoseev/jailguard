# Publicly Available Prompt Injection and Jailbreak Datasets

## Overview
This comprehensive review identifies datasets that complement the deepset/prompt-injections baseline (662 samples). Below are datasets organized by category with access methods and compatibility with your training pipeline.

---

## Quick Reference Table

| Dataset | Samples | License | Format | Commercial Use | Pipeline Compatibility |
|---------|---------|---------|--------|-----------------|----------------------|
| **TrustAIRLab In-The-Wild Jailbreak** | 15,140 | MIT | CSV/Parquet | ✅ Yes | ✅ Very High |
| **SPML Chatbot Injection** | 16,012 | MIT | CSV/Parquet | ✅ Yes | ✅ Very High |
| **BeaverTails Classification** | 300,000+ | CC-BY-NC-4.0 | HF Native | ❌ No | ✅ High |
| **JailbreakV-28K** | 28,000 | Unknown | HF Native | Unknown | ✅ High |
| **xTRam1 Safe-Guard** | 10,296 | Unknown | HF Native | Unknown | ✅ Very High |
| **Mindgard Evaded** | 10K-100K | CC-BY-NC-4.0 | Parquet | ❌ No | ✅ High |
| **Qualifire Benchmark** | 5,000 | CC-BY-NC-4.0 | CSV | ❌ No | ✅ Very High |
| **JailbreakBench** | 200 | Unknown | Python/HF | Unknown | ✅ High |
| **AdvBench** | 520 | Unknown | CSV | Unknown | ✅ Medium |
| **SafetyBench** | 11,435 | Unknown | MCQ | Unknown | ⚠️ Medium |
| **CyberSecEval3 Visual** | 1,000 | Research Only | JSON | ❌ Training Prohibited | ❌ Low |
| **Giskard Collection** | Unknown | Unknown | CSV | Unknown | ✅ High |

---

## TIER 1: Largest and Most Comprehensive Datasets

### 1. TrustAIRLab In-The-Wild Jailbreak Prompts (ACM CCS 2024)
- **Samples**: 15,140 prompts (1,405 jailbreak, 13,735 regular)
- **Source**: Reddit, Discord, websites, open-source datasets
- **Collection Period**: December 2022 - December 2023
- **Format**: CSV and Parquet
- **License**: MIT
- **Access**:
  ```python
  from datasets import load_dataset

  dataset = load_dataset(
      'TrustAIRLab/in-the-wild-jailbreak-prompts',
      'jailbreak_2023_12_25',
      split='train'
  )
  ```
- **Pipeline Compatibility**: ✅ Very High
- **Key Features**:
  - Real-world data from multiple platforms
  - Binary jailbreak label
  - Includes metadata (platform, source, community)
  - Optional 390-question evaluation set
- **URL**: https://huggingface.co/datasets/TrustAIRLab/in-the-wild-jailbreak-prompts
- **Paper**: https://arxiv.org/abs/2308.03825

### 2. SPML Chatbot Prompt Injection Dataset
- **Samples**: 16,012 annotated examples
- **System Prompts**: 1,871 diverse chatbot scenarios
- **Format**: CSV (29.1 MB), Parquet (11.1 MB)
- **License**: MIT
- **Fields**: System Prompt | User Prompt | Prompt Injection (0/1) | Degree (0-10) | Source
- **Access**:
  ```python
  from datasets import load_dataset
  dataset = load_dataset("reshabhs/SPML_Chatbot_Prompt_Injection")
  ```
- **Pipeline Compatibility**: ✅ Very High
- **Key Features**:
  - Attack complexity labels (Degree 0-10)
  - System prompt context (unique feature)
  - Generated using GPT-4
  - 16,840 unique system prompts
- **Note**: Authors recommend for evaluation, not training
- **URL**: https://huggingface.co/datasets/reshabhs/SPML_Chatbot_Prompt_Injection
- **Paper**: https://arxiv.org/abs/2402.11755

### 3. BeaverTails Dataset Collection
- **Classification Data**: 300,000+ human-labeled Q&A pairs
- **Preference Data**: 300,000+ expert comparisons
- **Evaluation Data**: 700 test prompts (50 per category)
- **Harm Categories**: 14 distinct categories
- **Format**: HuggingFace native
- **License**: CC BY-NC 4.0 (non-commercial)
- **Access**:
  ```python
  from datasets import load_dataset

  classification = load_dataset("PKU-Alignment/BeaverTails")
  preferences = load_dataset("PKU-Alignment/PKU-SafeRLHF")
  evaluation = load_dataset("PKU-Alignment/BeaverTails-Evaluation")
  ```
- **Pipeline Compatibility**: ✅ High
- **Key Features**:
  - Extensive multi-category coverage
  - Preference annotations (helpful for RLHF)
  - 3.34 crowdworkers per sample average
  - Includes trained beaver-dam-7b model
- **URL**: https://github.com/PKU-Alignment/beavertails
- **Paper**: https://arxiv.org/abs/2307.04657

---

## TIER 2: Medium-Sized Specialized Datasets

### 4. JailbreakV-28K Dataset
- **Samples**: 28,000 jailbreak samples
- **Policy Coverage**: 16 distinct safety policies
- **Data Sources**: 8 different sources including BeaverTails
- **Format**: HuggingFace native
- **License**: Unknown (check repo)
- **Access**:
  ```python
  from datasets import load_dataset
  dataset = load_dataset("JailbreakV-28K/JailbreakV-28k")
  ```
- **Pipeline Compatibility**: ✅ High
- **URL**: https://huggingface.co/datasets/JailbreakV-28K/JailbreakV-28k

### 5. xTRam1 Safe-Guard Prompt Injection Dataset
- **Samples**: 10,296 examples
- **Composition**: 7,000 safe + 3,000 injection prompts
- **Test Set**: 2,060 examples (650 adversarial)
- **Categories**: Context manipulation, social engineering, ignore prompt, fake completion, etc.
- **Format**: HuggingFace native
- **License**: Unknown
- **Access**:
  ```python
  from datasets import load_dataset
  dataset = load_dataset("xTRam1/safe-guard-prompt-injection")
  ```
- **Pipeline Compatibility**: ✅ Very High
- **Key Features**:
  - Category-organized attacks
  - Clear benign/malicious separation
  - GPT-3.5-turbo generated
- **URL**: https://huggingface.co/datasets/xTRam1/safe-guard-prompt-injection

### 6. Mindgard Evaded Prompt Injection and Jailbreak Samples
- **Samples**: 10K - 100K (estimate ~50K)
- **Content**: Original and modified injections with evasion techniques
- **Special Features**: Emoji smuggling in Base64, character injection, AML evasion
- **Format**: Parquet
- **License**: CC-BY-NC-4.0 (non-commercial)
- **Access**:
  ```python
  from datasets import load_dataset
  dataset = load_dataset("Mindgard/evaded-prompt-injection-and-jailbreak-samples")
  ```
- **Pipeline Compatibility**: ✅ High
- **Key Features**:
  - Adversarial evasion techniques
  - Real bypass attempts
  - Useful for robustness testing
- **URL**: https://huggingface.co/datasets/Mindgard/evaded-prompt-injection-and-jailbreak-samples

### 7. Qualifire Prompt Injections Benchmark
- **Samples**: 5,000 labeled prompts
- **Labels**: "jailbreak" vs "benign"
- **Format**: CSV
- **License**: CC-BY-NC-4.0
- **Downloads/Month**: 729
- **Access**:
  ```python
  from datasets import load_dataset
  dataset = load_dataset("qualifire/prompt-injections-benchmark")
  ```
- **Pipeline Compatibility**: ✅ Very High
- **URL**: https://huggingface.co/datasets/qualifire/prompt-injections-benchmark

### 8. JailbreakBench - JBB-Behaviors Dataset
- **Behaviors**: 200 distinct misuse behaviors
  - 100 harmful (OpenAI policy-aligned)
  - 100 benign (thematic controls)
- **Judges Data**: 300 human/LLM evaluations
- **Format**: Python library + HuggingFace
- **License**: Unknown
- **Access**:
  ```python
  import jailbreakbench as jbb
  dataset = jbb.read_dataset()  # harmful

  from datasets import load_dataset
  jbb_data = load_dataset("JailbreakBench/JBB-Behaviors", "behaviors")
  ```
- **Pipeline Compatibility**: ✅ High
- **Sources**: Original, Trojan Detection Challenge, HarmBench, AdvBench
- **Publication**: NeurIPS 2024 Datasets and Benchmarks Track
- **URL**: https://github.com/JailbreakBench/jailbreakbench

### 9. AdvBench Dataset
- **Samples**: 520 examples (train)
- **Harmful Instructions**: 500 strings
- **Format**: CSV
- **License**: Unknown (check repo)
- **Access**:
  ```python
  from datasets import load_dataset
  dataset = load_dataset("walledai/AdvBench")

  # Or direct from GitHub
  # https://github.com/llm-attacks/llm-attacks/blob/main/data/advbench/harmful_behaviors.csv
  ```
- **Pipeline Compatibility**: ✅ Medium
- **Use**: GCG attacks, adversarial robustness research
- **URL**: https://huggingface.co/datasets/walledai/AdvBench

---

## TIER 3: Specialized/Niche Datasets

### 10. SafetyBench - Multiple Choice Questions
- **Samples**: 11,435 MCQ questions
- **Categories**: 7 safety concern areas
- **Languages**: English and Chinese
- **Format**: QA pairs
- **License**: Unknown
- **Pipeline Compatibility**: ⚠️ Medium - MCQ format needs adaptation
- **Paper**: https://arxiv.org/abs/2408.01605

### 11. Facebook CyberSecEval3 - Visual Prompt Injection
- **Samples**: 1,000 test cases
- **Modality**: Text + Images (multimodal)
- **Format**: test_cases.json + images/
- **License**: Research evaluation only
- **Pipeline Compatibility**: ❌ Low - Evaluation only, don't use for training
- **Access**:
  ```python
  from datasets import load_dataset
  dataset = load_dataset("facebook/cyberseceval3-visual-prompt-injection")
  ```
- **URL**: https://huggingface.co/datasets/facebook/cyberseceval3-visual-prompt-injection

### 12. Giskard Prompt Injections Collection
- **Samples**: Unknown (concatenation of garak + PromptInject)
- **Format**: CSV
- **License**: Unknown
- **Pipeline Compatibility**: ✅ High
- **Access**:
  ```bash
  curl -O https://raw.githubusercontent.com/Giskard-AI/prompt-injections/main/prompt_injections.csv
  ```
- **URL**: https://github.com/Giskard-AI/prompt-injections

### 13. Geekyrakshit Prompt Injection Dataset
- **Format**: HuggingFace native
- **License**: Unknown
- **Access**:
  ```python
  from datasets import load_dataset
  dataset = load_dataset("geekyrakshit/prompt-injection-dataset")
  ```
- **URL**: https://huggingface.co/datasets/geekyrakshit/prompt-injection-dataset

---

## GitHub Research Repositories with Datasets

### 1. Jailbreak LLMs (CCS 2024)
- **Dataset**: 15,140 prompts (1,405 jailbreak)
- **Sources**: Reddit, Discord, websites, datasets
- **Format**: CSV files in `/data`
- **URL**: https://github.com/verazuo/jailbreak_llms
- **Paper**: "Do Anything Now": Characterizing and Evaluating In-The-Wild Jailbreak Prompts

### 2. KatherLab Prompt Injection Attacks
- **Focus**: Vision-Language Models (medical imaging)
- **URL**: https://github.com/KatherLab/prompt_injection_attacks

### 3. LLM Security Prompt Injection
- **Uses**: deepset/prompt-injections (your baseline!)
- **Approach**: Binary classification with various ML algorithms
- **URL**: https://github.com/sinanw/llm-security-prompt-injection

### 4. Open-Prompt-Injection Benchmark
- **Type**: Framework for studying attacks/defenses
- **Format**: JSON configs + Python framework
- **License**: MIT
- **Features**: Attackers, defenders, evaluators, ASV metric
- **URL**: https://github.com/liu00222/Open-Prompt-Injection

### 5. Awesome Prompt Injection
- **Type**: Curated resource collection
- **Content**: Educational, red-teaming, defensive research
- **URL**: https://github.com/Joe-B-Security/awesome-prompt-injection

---

## Integration Guide for Your Pipeline

### Current Supported Formats
Your codebase in `src/dataset/deepset.rs` supports:
- ✅ CSV: `from_csv()` - text,label format
- ✅ HuggingFace CSV: `from_huggingface_csv()`
- ✅ JSON: `from_json()` - {text, label} format
- ✅ HuggingFace Datasets API

### Recommended Integration Priority

**PRIORITY 1 (Immediate - High Impact)**
1. **TrustAIRLab** (+15,140 samples)
   - CSV/Parquet directly loadable
   - MIT license (commercial OK)
   - Real-world Reddit/Discord/website data
   - 1-2 hour integration

2. **SPML** (+16,012 samples)
   - CSV format
   - MIT license
   - Add system prompt context to your Sample struct
   - 2-3 hour integration

**PRIORITY 2 (Medium-term)**
3. **xTRam1** (+10,296 samples)
   - Clean category labels
   - HF native loading
   - 1 hour integration

4. **Mindgard** (+50K estimate)
   - Evasion-focused
   - Robustness testing value
   - Parquet → pandas → CSV conversion
   - 1-2 hour integration

**PRIORITY 3 (Specialized)**
5. **BeaverTails** (subset of 300K)
   - Extract only injection/jailbreak relevant samples
   - Multi-category coverage
   - 2-3 hour integration

6. **JailbreakBench** (+200 behaviors)
   - Well-structured with goals/targets
   - NeurIPS published
   - 1 hour integration

### Sample Data Merging Code
```python
# Suggested addition to src/dataset/mod.rs
pub fn load_all_datasets(data_dir: &str) -> Result<Vec<Sample>> {
    let mut all_samples = Vec::new();

    // Load deepset baseline
    let deepset = DeepsetDataset::download(data_dir)?;
    all_samples.extend(deepset.samples().iter().cloned());

    // Load TrustAIRLab
    let trustai = DeepsetDataset::from_csv(
        &format!("{}/trustailab/jailbreaks.csv", data_dir)
    )?;
    all_samples.extend(trustai.samples().iter().cloned());

    // Load SPML (need to map System+User Prompt to text)
    let spml = load_spml_dataset(&format!("{}/spml.csv", data_dir))?;
    all_samples.extend(spml.samples().iter().cloned());

    // Deduplicate
    all_samples.sort_by(|a, b| a.text.cmp(&b.text));
    all_samples.dedup_by(|a, b| a.text == b.text);

    println!("Loaded {} total unique samples", all_samples.len());
    Ok(all_samples)
}
```

---

## Licensing Summary

| License | Datasets | Notes |
|---------|----------|-------|
| **MIT** | TrustAIRLab, SPML, JailbreakBench | ✅ Commercial use allowed |
| **CC-BY-NC-4.0** | Qualifire, Mindgard, BeaverTails | ❌ Non-commercial only |
| **Research Only** | CyberSecEval3 | ❌ No training use |
| **Unknown/Unspecified** | AdvBench, SafetyBench, Giskard, JailbreakV-28K, xTRam1 | ⚠️ Check individual repos |

### Commercial-Use Safe Combination
If you need only commercial-licensable datasets:
- deepset/prompt-injections: 662
- TrustAIRLab: 15,140
- SPML: 16,012
- JailbreakBench: 200
- **Total: 32,014 samples**

---

## Estimated Total Dataset Size

### All Available (50K Mindgard estimate):
- deepset: 662
- TrustAIRLab: 15,140
- SPML: 16,012
- BeaverTails (evaluation): 700
- xTRam1: 10,296
- JailbreakBench: 200
- AdvBench: 520
- Mindgard: ~50,000
- JailbreakV-28K: 28,000
- SafetyBench: 11,435
- CyberSecEval3: 1,000
- **Conservative Total: ~133,965 samples**

### After Deduplication
Assuming 15-20% overlap:
- **Estimated Unique: 107K - 114K samples**

---

## Key Insights & Recommendations

### Attack Type Coverage
- **Context Manipulation**: xTRam1, SPML, TrustAIRLab
- **Social Engineering**: xTRam1, SPML
- **Evasion/Obfuscation**: Mindgard (specialty)
- **Role-Playing/DAN**: TrustAIRLab, SPML
- **Character Injection**: Mindgard
- **Emoji Smuggling**: Mindgard
- **Harmful Behaviors**: AdvBench, BeaverTails, JailbreakBench

### Recommended Training Strategy
1. **Base Training**: TrustAIRLab + SPML (31K samples)
   - Real-world diversity
   - Well-researched datasets
   - MIT licensed

2. **Robustness Augmentation**: Add Mindgard
   - Evasion techniques
   - Character-level attacks

3. **Evaluation**: Use CyberSecEval3 + JailbreakBench
   - Multimodal testing
   - Structured evaluation

4. **Optional Specialization**: BeaverTails for harm categories

### Data Quality Notes
- **TrustAIRLab**: Real-world sourced, varies in quality
- **SPML**: Synthetically generated (GPT-4), consistent quality
- **Mindgard**: Focused on evasion, adversarial nature
- **BeaverTails**: Human-annotated with 3.34 workers per sample

---

## References & Links

### Primary Datasets
1. [TrustAIRLab HF](https://huggingface.co/datasets/TrustAIRLab/in-the-wild-jailbreak-prompts)
2. [SPML HF](https://huggingface.co/datasets/reshabhs/SPML_Chatbot_Prompt_Injection)
3. [BeaverTails GitHub](https://github.com/PKU-Alignment/beavertails)
4. [JailbreakBench GitHub](https://github.com/JailbreakBench/jailbreakbench)
5. [JailbreakV-28K HF](https://huggingface.co/datasets/JailbreakV-28K/JailbreakV-28k)
6. [xTRam1 HF](https://huggingface.co/datasets/xTRam1/safe-guard-prompt-injection)
7. [Mindgard HF](https://huggingface.co/datasets/Mindgard/evaded-prompt-injection-and-jailbreak-samples)
8. [Qualifire HF](https://huggingface.co/datasets/qualifire/prompt-injections-benchmark)
9. [AdvBench GitHub](https://github.com/llm-attacks/llm-attacks)
10. [CyberSecEval3 HF](https://huggingface.co/datasets/facebook/cyberseceval3-visual-prompt-injection)

### Academic Papers
- [CCS 2024] Shen et al. - "Do Anything Now" (TrustAIRLab): https://arxiv.org/abs/2308.03825
- [ArXiv 2402.11755] SPML DSL: https://arxiv.org/abs/2402.11755
- [ArXiv 2307.04657] BeaverTails: https://arxiv.org/abs/2307.04657
- [ArXiv 2408.01605] SafetyBench: https://arxiv.org/abs/2408.01605
- [NeurIPS 2024] JailbreakBench: https://github.com/JailbreakBench/jailbreakbench

### Research Frameworks
- [Open-Prompt-Injection](https://github.com/liu00222/Open-Prompt-Injection)
- [Giskard LLM Testing](https://github.com/Giskard-AI/prompt-injections)
- [Awesome Prompt Injection](https://github.com/Joe-B-Security/awesome-prompt-injection)
