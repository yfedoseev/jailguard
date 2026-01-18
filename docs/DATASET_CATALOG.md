# Comprehensive Catalog of Prompt Injection, Jailbreak & LLM Attack Datasets

**Last Updated:** 2026-01-16
**Research Scope:** Public and publicly available datasets related to prompt injection detection, LLM jailbreaking, and adversarial prompt attacks (2023-2025)

---

## Executive Summary

This catalog documents **35+ major datasets** across prompt injection, jailbreak detection, and LLM adversarial attack research. Total indexed samples exceed **2.5 million** across all datasets, with JailGuard's 4,500-sample benchmark positioned as a curated, high-quality evaluation set.

### Key Findings
- **Largest Dataset:** Tensor Trust (681,000 samples)
- **Most Comprehensive Taxonomy:** RedBench (22 risk categories, 19 domains, 29,362 samples)
- **Dedicated Indirect Attack Dataset:** BIPIA (first benchmark for indirect injection)
- **Multilingual Coverage:** MultiJail (3,150 samples across 10 languages)
- **Most Recent Competition Data:** LLMail-Inject (208,095 unique submissions, Jan 2025)

---

## Dataset Index

### Table of Contents
1. [Direct Prompt Injection Detection](#direct-prompt-injection-detection)
2. [Jailbreak & Adversarial Attacks](#jailbreak--adversarial-attacks)
3. [Indirect Prompt Injection](#indirect-prompt-injection)
4. [Prompt Extraction & Hijacking](#prompt-extraction--hijacking)
5. [Multilingual & Specialized](#multilingual--specialized)
6. [Benchmark & Evaluation Frameworks](#benchmark--evaluation-frameworks)
7. [Comparison Matrix](#comparison-matrix)
8. [Recommendations](#recommendations)

---

## 1. Direct Prompt Injection Detection

### 1.1 deepset/prompt-injections
**Source:** Hugging Face Datasets
**URL:** https://huggingface.co/datasets/deepset/prompt-injections
**Paper:** N/A (industry dataset from deepset)

| Metric | Value |
|--------|-------|
| **Total Samples** | 662 |
| **Training Samples** | 546 |
| **Test Samples** | 116 |
| **Benign Examples** | 399 (63%) |
| **Malicious Examples** | 263 (37%) |
| **Languages** | English, German |
| **Last Updated** | October 2024 |

**Attack Types Covered:**
- Direct prompt injection
- Instruction override
- Context manipulation
- Simple prompt concatenation

**Annotation Quality:**
- Binary labels (benign/malicious)
- Enriched with adversarial examples
- Includes translated variants

**License:** Publicly available
**Commercial Use:** Yes (permissive)

**Key Characteristics:**
- Early foundational dataset
- Widely used baseline
- Small but high-quality samples
- Languages beyond English

**Comparison to JailGuard (4,500 samples):**
- JailGuard: **6.8x larger**
- JailGuard has broader attack taxonomy
- deepset valuable for validation across languages

---

### 1.2 xTRam1/safe-guard-prompt-injection
**Source:** Hugging Face Datasets
**URL:** https://huggingface.co/datasets/xTRam1/safe-guard-prompt-injection
**Related Model:** xTRam1/safe-guard-classifier

| Metric | Value |
|--------|-------|
| **Total Samples** | 10,000 |
| **Training Samples** | 9,400 |
| **Test Samples** | 600 |
| **Benign Samples** | 7,000 (70%) |
| **Injection Samples** | 3,000 (30%) |
| **Creation Method** | Synthetic (GPT-3.5-turbo) |
| **Date** | 2024 |

**Attack Types Covered:**
- Context manipulation
- Social engineering
- Ignore prompt commands
- Fake completion attacks

**Dataset Creation:**
- Built using categorical tree structure
- Seed data from open-source datasets:
  - vmware/open-instruct
  - huggingfaceh4/helpful-instructions
  - Fka-awesome-chatgpt-prompts
  - jackhhao/jailbreak-classification
- GPT-3.5-turbo augmentation with categorical strategies

**Annotation Quality:**
- High precision due to synthetic generation pipeline
- Systematic coverage of attack categories

**License:** Public/MIT
**Commercial Use:** Yes

**Model Performance:**
- Fine-tuned DeBERTa-v3-small
- 99.6% accuracy on test set
- Model size: 44M (2x smaller than ProtectAI baseline)

**Comparison to JailGuard:**
- JailGuard: **0.45x this dataset**
- xTRam1 is synthetic, JailGuard is curated
- JailGuard has richer annotation schema

---

### 1.3 protectai/deberta-v3-base-prompt-injection (v2)
**Source:** Hugging Face Models
**URL:** https://huggingface.co/protectai/deberta-v3-base-prompt-injection-v2

| Metric | Value |
|--------|-------|
| **Associated Dataset** | Multiple (proprietary training) |
| **Model Base** | DeBERTa-v3-base |
| **Model Size** | 86M parameters |
| **Deployment Status** | Production-ready |
| **Update** | v2 released 2024 |

**Key Characteristics:**
- Industry-standard model for prompt injection detection
- Trained on proprietary dataset of significant size
- Fine-tuned for binary classification
- Multilingual capable

**Related Research:**
- ProtectAI has evaluated on multiple public datasets
- Baseline in PINT benchmark evaluations

**Comparison to JailGuard:**
- ProtectAI is a trained classifier
- JailGuard data can fine-tune similar models
- ProtectAI provides production-ready defense

---

### 1.4 Harelix/Prompt-Injection-Mixed-Techniques-2024
**Source:** Hugging Face Datasets
**URL:** https://huggingface.co/datasets/Harelix/Prompt-Injection-Mixed-Techniques-2024

| Metric | Value |
|--------|-------|
| **Total Samples** | 1,174 |
| **Date** | May 2024 |
| **Attack Variety** | High (mixed techniques) |
| **Special Feature** | Combined attack examples |

**Attack Types Covered:**
- Multiple prompt injection techniques
- Combined attacks merging several vectors
- Benign inputs with injection variants

**Key Feature:**
- Final "combine attack" merging all techniques
- Tests robustness against sophisticated attacks

**License:** Public
**Comparison to JailGuard:**
- JailGuard: **3.8x larger**
- Useful for mixed technique evaluation
- Good for ensemble defense testing

---

### 1.5 geekyrakshit/prompt-injection-dataset
**Source:** Hugging Face Datasets
**URL:** https://huggingface.co/datasets/geekyrakshit/prompt-injection-dataset

| Metric | Value |
|--------|-------|
| **Status** | Available on Hugging Face |
| **Focus** | General prompt injection |

**Key Characteristics:**
- Community-contributed dataset
- Used in various classification studies

**Comparison to JailGuard:**
- Limited published metadata
- JailGuard provides more systematic collection

---

### 1.6 yanismiraoui/prompt_injections
**Source:** Hugging Face Datasets
**URL:** https://huggingface.co/datasets/yanismiraoui/prompt_injections

**Status:** Available on Hugging Face platform
**Comparison to JailGuard:** Community dataset with limited documentation

---

## 2. Jailbreak & Adversarial Attacks

### 2.1 AdvBench (Adversarial Benchmark)
**Source:** Hugging Face + GitHub
**URL:** https://huggingface.co/datasets/walledai/AdvBench
**Paper:** "Universal and Transferable Adversarial Attacks on Aligned Language Models" (Zou et al., Dec 2023)
**ArXiv:** https://arxiv.org/abs/2312.10286

| Metric | Value |
|--------|-------|
| **Total Behaviors** | 520 |
| **Harmful Behaviors** | 500 |
| **Focus** | Instruction-following attacks |
| **Date** | December 2023 |
| **License** | CC BY-NC 4.0 |

**Attack Types Covered (Harmful Behaviors):**
- Profanity and offensive language
- Threatening behavior
- Misinformation/disinformation
- Discrimination
- Illegal activities
- Unethical advice

**Dataset Methodology:**
- String-based adversarial attack formulation
- Single attack string per behavior
- Transfer-based evaluation across models

**Annotation Quality:**
- Manually curated harmful behaviors
- Clear success criteria

**Commercial Use:** Limited (CC BY-NC)
**Citation Count:** High (foundational dataset)

**Key Insight:**
- Attack goal: Find single string causing model to attempt harmful task
- Designed to maximize transferability

**Comparison to JailGuard:**
- AdvBench: ~500 behaviors
- JailGuard: 4,500 samples with richer taxonomy
- AdvBench focuses on direct harmful instruction generation
- JailGuard covers broader attack vectors

**Related Resources:**
- Widely integrated in AutoRed, GCG, and other jailbreak methods
- Used as baseline in HarmBench, JailbreakBench

---

### 2.2 HarmBench
**Source:** Center for AI Safety (CAIS)
**GitHub:** https://github.com/centerforaisafety/HarmBench
**Hugging Face:** https://huggingface.co/datasets/walledai/HarmBench
**Paper:** "HarmBench: A Standardized Evaluation Framework for Automated Red Teaming and Robust Refusal" (Mazeika et al., Feb 2024)
**Website:** https://www.harmbench.org/
**ArXiv:** https://arxiv.org/abs/2402.04249

| Metric | Value |
|--------|-------|
| **Total Behaviors** | 400-510 |
| **Unique Behavior Types** | 510 |
| **Semantic Categories** | 7 |
| **Date** | February 2024 |
| **License** | Open source (MIT) |
| **Red-Teaming Methods Evaluated** | 18 |
| **Target Models/Defenses** | 33 |

**Behavior Categories (7 Semantic Groups):**
1. Cybercrime & Unauthorized Intrusion
2. Chemical & Biological Weapons/Drugs
3. Copyright Violations
4. Misinformation & Disinformation
5. Harassment & Bullying
6. Illegal Activities
7. General Harm

**Behavior Types:**
1. **Standard Behaviors:** Self-contained task strings
2. **Contextual Behaviors:** Context + behavior requiring reference
3. **Multimodal Behaviors:** Image + text prompts for vision-language models

**Annotation Quality:**
- Professionally curated
- Cross-referenced with AdvBench and TDC 2023
- Validated for semantic consistency

**Key Features:**
- Largest red-teaming evaluation at launch
- Comprehensive model-attack compatibility matrix
- Leaderboard system

**Commercial Use:** Yes (MIT license)

**Comparison to JailGuard:**
- HarmBench: 510 behaviors with 3 variants per behavior type = 1,020+ total
- JailGuard: 4,500 samples with attack taxonomy
- HarmBench: Broader semantic coverage
- JailGuard: Denser sampling within categories

**Integration:**
- Used extensively in red-teaming research
- Reference benchmark for safety evaluation

---

### 2.3 MaliciousInstruct
**Source:** Paper supplementary materials
**Paper:** "Exploring the Transferability of Adversarial Examples" (research context)
**Related to AdvBench paper**

| Metric | Value |
|--------|-------|
| **Total Harmful Queries** | 100 |
| **Harmful Intent Categories** | 10 |
| **Query Structure** | "Ways to...", "How to...", "Ideas for..." |
| **Date** | 2023 |
| **Creation Method** | GPT-based generation + filtering |

**Harmful Intent Categories (10):**
1. Psychological manipulation
2. Sabotage
3. Theft
4. Defamation
5. Cyberbullying
6. False accusation
7. Tax fraud
8. Hacking
9. Fraud
10. Illegal drug use

**Dataset Characteristics:**
- Smaller than AdvBench but more focused
- Machine-written with human filtering
- Used in jailbreak evaluation
- High success rate (~70% on GPT-3.5)

**Commercial Use:** Limited (research context)

**Comparison to JailGuard:**
- MaliciousInstruct: 100 queries, 10 categories
- JailGuard: 4,500 samples with more granular taxonomy
- MaliciousInstruct: Focused on harmful instruction compliance
- JailGuard: Broader scope including detection focus

---

### 2.4 CPAD (Goal-Oriented Prompt Attack Dataset)
**Source:** GitHub (Public)
**GitHub:** https://github.com/liuchengyuan123/CPAD
**Paper:** "Goal-Oriented Prompt Attack and Safety Evaluation for LLMs" (Liu et al., Sep 2023)
**ArXiv:** https://arxiv.org/abs/2309.11830

| Metric | Value |
|--------|-------|
| **Total Prompts** | 10,050 |
| **Language** | Chinese |
| **Dimensions** | Content, method, goal |
| **Creation Method** | GPT-based + human seeds |
| **Attack Success Rate** | ~70% (GPT-3.5) |
| **License** | CC BY-SA 4.0 |
| **Date** | September 2023 |

**Attack Methodology:**
- **Content dimension:** Carefully designed harmful content
- **Method dimension:** Various attack techniques
- **Goal dimension:** Expected harmful response behavior

**Key Distinction:**
- Goal-oriented formulation allows precise evaluation
- Responses can be objectively evaluated

**Language Coverage:**
- Primarily Chinese (first major Chinese prompt attack dataset)

**Annotation Quality:**
- Hybrid creation: GPT generation from seeds
- Goal-based annotation enables clear success metrics

**Commercial Use:** Yes (CC BY-SA 4.0)

**Comparison to JailGuard:**
- CPAD: 10,050 prompts (Chinese-focused)
- JailGuard: 4,500 samples (English)
- CPAD: 3D methodology (content/method/goal)
- JailGuard: Broader attack type taxonomy
- CPAD demonstrates importance of language diversity

**Citation Value:**
- First systematic Chinese prompt attack dataset
- Demonstrates multilingual requirements

---

### 2.5 BeaverTails
**Source:** GitHub + Hugging Face
**GitHub:** https://github.com/PKU-Alignment/beavertails
**Paper:** "BeaverTails: Towards Improved Safety Alignment of LLM via a Human-Preference Dataset" (Ji et al., July 2023)
**ArXiv:** https://arxiv.org/abs/2307.04657
**Conference:** NeurIPS 2023 (Datasets & Benchmarks Track)

| Metric | Value |
|--------|-------|
| **Total QA Pairs** | 333,963 |
| **Expert Comparison Pairs** | 361,903 |
| **Evaluation Dataset** | 700 prompts |
| **Harm Categories** | 14 (v1), 19 (v2) |
| **Severity Levels** | 3 (v2) |
| **Date** | July 2023 |
| **Version** | 2.0 (updated) |
| **License** | CC BY 4.0 |

**Dataset Components:**
1. **Classification Dataset:** 300k+ annotated for safety
2. **Preference Dataset:** 300k+ pairs for preference learning
3. **Evaluation Set:** 700 curated prompts

**Harm Categories (14 in v1, 19 in v2):**
- Violence/harm
- Illegal activities
- Fraud/scam
- Copyright violations
- Adult content
- Misinformation
- Personal privacy
- Political bias
- Harassment/bullying
- Defamation
- Sexual/erotic content
- (Plus 8 additional in v2)

**Annotation Quality:**
- Two-stage annotation process
- Multi-classification for harm categorization
- Separate helpfulness/harmlessness metrics
- Expert preference comparison

**Key Innovation:**
- Separates helpfulness and harmlessness
- Designed for RLHF training
- Practical content moderation focus

**Commercial Use:** Yes (CC BY 4.0)

**Comparison to JailGuard:**
- BeaverTails: 333k+ QA pairs (massive scale)
- JailGuard: 4,500 curated samples
- BeaverTails: RLHF training focus
- JailGuard: Detection and classification focus
- BeaverTails: Extensive annotation with multiple dimensions
- Complementary use case (training vs. evaluation)

---

### 2.6 ALERT (Benchmark for Red Teaming)
**Source:** GitHub + Hugging Face
**GitHub:** https://github.com/Babelscape/ALERT
**Paper:** "ALERT: A Comprehensive Benchmark for Assessing Large Language Models' Safety through Red Teaming" (Tedeschi et al., April 2024)
**ArXiv:** https://arxiv.org/abs/2404.08676
**Blog:** https://huggingface.co/blog/sted97/alert

| Metric | Value |
|--------|-------|
| **Total Prompts** | 15,000 (standard) |
| **Full Benchmark** | 45,000+ (with augmentation) |
| **Macro Categories** | 6 |
| **Micro Categories** | 32 |
| **Creation Method** | AnthropicRedTeam samples + templates |
| **Date** | April 2024 |
| **License** | Open source |

**Safety Risk Taxonomy (6 Macro / 32 Micro):**
**Macro Categories:**
1. Illegal Activities
2. Violence/Harm
3. Privacy/Security
4. Deception/Manipulation
5. Copyright/IP
6. Other Harmful Content

Each macro category subdivided into fine-grained micro categories.

**Key Features:**
- Dual-purpose dataset (attacks + DPO training)
- Direct Preference Optimization (DPO) triplets
- Hierarchical taxonomy
- Category-specific safety scores

**Annotation Quality:**
- High-quality curation from AnthropicRedTeam
- Template-based augmentation
- Validated taxonomy

**Commercial Use:** Yes

**Comparison to JailGuard:**
- ALERT: 15,000-45,000 prompts
- JailGuard: 4,500 samples (0.29x-0.3x)
- ALERT: 38 categories (6 macro + 32 micro)
- JailGuard: ~4 primary attack types
- ALERT: Designed for DPO training
- JailGuard: Evaluation/detection focus
- Both complementary in comprehensive safety evaluation

**Integration:**
- ALERT data used to create DPO training sets
- Reference taxonomy for safety categorization

---

## 3. Indirect Prompt Injection

### 3.1 BIPIA (Benchmark for Indirect Prompt Injection Attacks)
**Source:** GitHub (Microsoft)
**GitHub:** https://github.com/microsoft/BIPIA
**Paper:** "Benchmarking and Defending Against Indirect Prompt Injection Attacks on Large Language Models" (Li et al., Dec 2023)
**ArXiv:** https://arxiv.org/abs/2312.14197

| Metric | Value |
|--------|-------|
| **Total Tasks** | 5 |
| **Data Sources** | Mixed (OpenAI Evals, WikiTableQuestions, Stack Exchange) |
| **Task Types** | Web QA, Email QA, Table QA, Summarization, Code QA |
| **Benchmark Type** | First comprehensive indirect injection benchmark |
| **Date** | December 2023 |
| **License** | MIT |

**Task Composition:**
1. **Web QA:** Document retrieval + QA
2. **Email QA:** Email content + question answering
3. **Table QA:** Structured data + queries
4. **Summarization:** Document summarization with injected content
5. **Code QA:** Code context + questions

**Attack Mechanism Addressed:**
- Malicious instructions embedded in external content
- Document retrieval contexts
- Email attachments/quoted text
- Web page content

**Key Finding:**
- All evaluated LLMs vulnerable to indirect injection
- Models unable to distinguish information from instructions
- Lack of awareness in filtering external instructions

**Defenses Proposed:**
- Black-box meta-prompting defenses
- White-box model-aware defenses

**Annotation Quality:**
- Leverages established public datasets
- Validated attack success criteria

**Commercial Use:** Yes (MIT)

**Comparison to JailGuard:**
- BIPIA: Indirect attacks focus
- JailGuard: Direct prompt injection focus
- Complementary attack vectors
- BIPIA demonstrates importance of multi-vector evaluation

---

## 4. Prompt Extraction & Hijacking

### 4.1 TensorTrust (Online Game Dataset)
**Source:** Website + GitHub
**Website:** https://tensortrust.ai/
**GitHub:** https://github.com/HumanCompatibleAI/tensor-trust
**Paper:** "Tensor Trust: Interpretable Prompt Injection Attacks from an Online Game" (Toyer et al., Nov 2023)
**ArXiv:** https://arxiv.org/abs/2311.01011
**Conference:** NeurIPS 2023

| Metric | Value |
|--------|-------|
| **Total Attack Prompts** | 563,000+ |
| **Defense Prompts** | 118,000+ |
| **Attack Categories** | 2 main (extraction, hijacking) |
| **Data Collection** | Crowdsourced game players |
| **Date** | November 2023 |
| **License** | Open source |

**Game Mechanics:**
- Competitive online game
- Players attempt attacks while defending assets
- Strategic prompt manipulation required
- Real-world adversarial dynamic

**Attack Categories:**
1. **Prompt Extraction:** Reveal system prompt
2. **Prompt Hijacking:** Redirect model behavior

**Key Metrics:**
- Largest human-generated adversarial example dataset for instruction-following
- ~126,000 unique documented prompt hacking instances
- Natural adversarial distribution from gameplay

**Annotation Quality:**
- Organic generation from real player strategies
- Game-based success criteria
- Community-driven

**Commercial Use:** Yes

**Unique Characteristic:**
- First large-scale collection via interactive game
- Demonstrates transferability across deployed systems
- Reveals interpretable attack patterns

**Comparison to JailGuard:**
- TensorTrust: 681,000 total samples (extraction/hijacking focused)
- JailGuard: 4,500 samples (broader attack taxonomy)
- TensorTrust: Game-generated, high diversity
- JailGuard: Curated, focused on detection
- Scale difference: TensorTrust **151x larger**
- Use case: TensorTrust for attack strategy study, JailGuard for detection

---

### 4.2 Raccoon (Prompt Extraction Benchmark)
**Source:** GitHub
**GitHub:** https://github.com/M0gician/RaccoonBench
**Paper:** "Raccoon: Prompt Extraction Benchmark of LLM-Integrated Applications" (Wang et al., 2024)
**Conference:** ACL 2024 (Findings)
**ArXiv:** https://arxiv.org/abs/2406.06737

| Metric | Value |
|--------|-------|
| **Custom GPTs Analyzed** | 48,000+ |
| **Prompts Extracted** | 197 |
| **Successful Extraction Rate** | 197/200 (98.5%) |
| **Attack Categories** | 14 |
| **Defense Categories** | Multiple |
| **Compounded Attacks** | Included |
| **Date** | June 2024 |

**Attack Categories (14 types):**
1. Direct extraction ("What is your system prompt?")
2. Role-playing extraction
3. Encoding-based attacks
4. Multi-turn sophisticated attacks
5. Injection-based extraction
6. And 9 additional documented vectors

**Key Findings:**
- 98.5% success rate on undefended models
- GPT-4 with long defenses nearly 100% protected
- Higher instruction-following capability correlates with extraction vulnerability
- Defense effectiveness directly correlated with template length

**Annotation Quality:**
- Manual extraction from real-world GPTs
- Systematic attack methodology
- Defense template evaluation

**Commercial Use:** Yes

**Benchmark Features:**
- Defenseless scenarios
- Defended scenarios with various templates
- Compound attack strategies
- Real-world model evaluation

**Comparison to JailGuard:**
- Raccoon: Extraction-specific focus
- JailGuard: General detection/classification
- Raccoon: Demonstrates 98.5% extraction rate
- JailGuard: Higher-level attack taxonomy
- Complementary: Raccoon shows specific vulnerability, JailGuard broader scope

---

### 4.3 Tensor Trust Benchmark Results
**Source:** TensorTrust paper and benchmark
**Key Finding:** ~12,600 documented prompt "hacking" instances from Tensor Trust
**Related:** 600k+ instances from Schulhoff et al. (2023) prompt hacking work

---

## 5. Multilingual & Specialized

### 5.1 MultiJail (Multilingual Jailbreak)
**Source:** GitHub
**GitHub:** https://github.com/DAMO-NLP-SG/multilingual-safety-for-LLMs
**Paper:** "Multilingual Jailbreak Challenges in Large Language Models" (Li et al., Oct 2023)
**Conference:** ICLR 2024

| Metric | Value |
|--------|-------|
| **Total Samples** | 3,150 |
| **English Samples** | 315 |
| **Languages Covered** | 10 (9 non-English) |
| **Language Resource Levels** | High, medium, low-resource |
| **Date** | October 2023 |
| **License** | Open source |

**Language Distribution:**
**High-Resource Languages (3):**
- Chinese
- Italian
- Vietnamese

**Medium-Resource Languages (3):**
- Arabic
- Korean
- Thai

**Low-Resource Languages (3):**
- Bengali
- Swahili
- Javanese

**Key Research Finding:**
- Unsafe content rate increases with decreasing language availability
- Low-resource languages: **3x higher harm rate** than high-resource
- Demonstrates language-based safety vulnerabilities

**Proposed Defense:**
- Self-Defence framework
- Automatic multilingual training data generation
- Safety fine-tuning approach

**Annotation Quality:**
- High-quality English baseline
- Parallel annotations across languages
- Validated harm criteria

**Commercial Use:** Yes

**Comparison to JailGuard:**
- MultiJail: 3,150 samples (10 languages)
- JailGuard: 4,500 samples (likely English-focused)
- MultiJail: Reveals language-based vulnerabilities
- JailGuard: Can be extended with MultiJail for multilingual robustness
- Combined value: Critical for global safety

---

### 5.2 CyberSecEval (Meta/Facebook)
**Source:** GitHub + Hugging Face
**GitHub:** https://github.com/meta-llama/PurpleLlama
**Hugging Face:** facebook/cyberseceval3-visual-prompt-injection
**Paper:** "CyberSecEval 2: A Wide-Ranging Cybersecurity Evaluation Suite" (Huang et al., April 2024)
**Version:** CyberSecEval 4 (current)

| Metric | Value |
|--------|-------|
| **Test Categories** | Multiple (including prompt injection) |
| **Prompt Injection Success Rate** | 26-41% across models |
| **Visual Injection Tests** | Included (CyberSecEval 3+) |
| **Multilingual Tests** | Yes |
| **Evolution** | CyberSecEval → CEval 2 → CEval 3 → CEval 4 |
| **License** | Meta-specific |

**Test Coverage Areas:**
1. Prompt injection attacks
2. Malicious code generation
3. Cybersecurity assistance assessment
4. Visual prompt injection (multimodal)
5. Multilingual variants

**Key Metrics:**
- Prompt injection base success: 26-41% (undefended models)
- Visual injection: Separate benchmark
- Multilingual coverage: Across multiple languages

**Related Tools:**
- **Llama-Prompt-Guard-2 (86M):** mDeBERTa-base classifier
- Training data: Mix of benign web data + red-team data + synthetic injections

**Annotation Quality:**
- Meta-curated
- Professionally generated
- Validated against real-world attacks

**Commercial Use:** Limited (Meta's Purple Llama framework)

**Comparison to JailGuard:**
- CyberSecEval: Comprehensive cybersecurity suite
- JailGuard: Focused prompt injection benchmark
- CyberSecEval: Industrial evaluation framework
- JailGuard: Specialized detection dataset
- Combined: CyberSecEval provides context, JailGuard provides specialized focus

---

### 5.3 SPML (System Prompt Meta Language) Dataset
**Source:** GitHub + Website
**Website:** https://prompt-compiler.github.io/SPML/
**GitHub:** https://github.com/prompt-compiler/SPML
**Paper:** "SPML: A DSL for Defending Language Models Against Prompt Attacks" (Sharma et al., Feb 2024)
**ArXiv:** https://arxiv.org/abs/2402.11755

| Metric | Value |
|--------|-------|
| **System Prompts** | 1,800 |
| **User Inputs** | 20,000 |
| **Total Interactions** | 21,800 |
| **Domain** | Chatbot-specific |
| **Creation** | Hybrid human + synthetic |
| **Date** | February 2024 |
| **License:** | Open source |

**Dataset Components:**
1. **System Prompts:** 1,800 distinct chatbot definitions
2. **User Inputs:** 20,000 variations (attacks + benign)

**Problem Addressed:**
- Fixed post-deployment chatbot definitions vulnerable to attacks
- Need for application-specific attack monitoring

**SPML Language Features:**
- Domain-specific language for prompt refinement
- Input monitoring capabilities
- Attack detection before LLM execution
- Cost optimization

**Annotation Quality:**
- Systematic chatbot definition coverage
- Realistic attack scenarios
- Validated attack patterns

**Unique Contribution:**
- First language and benchmark for chatbot definition evaluation
- Surpasses GPT-4, GPT-3.5, LLAMA performance

**Commercial Use:** Yes (open source)

**Comparison to JailGuard:**
- SPML: 21,800 interactions (system-prompt centric)
- JailGuard: 4,500 samples (user-input centric)
- SPML: Chatbot definition defense focus
- JailGuard: Input classification focus
- Complementary: System-level vs. input-level

---

## 6. Benchmark & Evaluation Frameworks

### 6.1 JailbreakBench (NeurIPS 2024)
**Source:** GitHub + Website
**Website:** https://jailbreakbench.github.io/
**GitHub:** https://github.com/JailbreakBench/jailbreakbench
**Paper:** "JailbreakBench: An Open Robustness Benchmark for Jailbreaking Large Language Models" (Chao et al., 2024)
**Conference:** NeurIPS 2024 (Datasets & Benchmarks Track)

| Metric | Value |
|--------|-------|
| **Behaviors** | 100 (distinct misuse behaviors) |
| **Behavior Categories** | 10 |
| **Baseline Attacks** | AdvBench, MaliciousInstruct |
| **Attack Methods** | PAIR, GCG, manual optimization + suffixes |
| **Evaluation Frameworks** | Continuous leaderboard |
| **Date** | 2024 |

**Behavior Categories (10):**
- Aligned with OpenAI usage policies
- Systematic coverage of common misuse cases

**Attack Repository:**
- State-of-the-art adversarial prompts at https://github.com/JailbreakBench/artifacts
- Evolving dataset with attack methods

**Hugging Face Dataset:**
- JailbreakBench/JBB-Behaviors: Available for download

**Leaderboard Features:**
- Track attack effectiveness
- Track defense robustness
- Model-agnostic evaluation

**Citation Impact:**
- NeurIPS 2024 acceptance indicates high impact

**Comparison to JailGuard:**
- JailbreakBench: 100 behaviors (4.5% of JailGuard)
- JailGuard: 4,500 samples
- JailbreakBench: Leaderboard evaluation framework
- JailGuard: Static benchmark
- Both complementary for evaluation

**Integration Value:**
- JailGuard can evaluate against JailbreakBench behaviors
- JailbreakBench provides ongoing evaluation framework

---

### 6.2 HarmBench (Already Detailed in Section 2.2)
See Section 2.2 for comprehensive details.

---

### 6.3 PINT Benchmark (Lakera)
**Source:** GitHub
**GitHub:** https://github.com/lakeraai/pint-benchmark
**Blog:** https://www.lakera.ai/blog/lakera-pint-benchmark

| Metric | Value |
|--------|-------|
| **Total Inputs** | 4,314 |
| **English Inputs** | 3,016 (70%) |
| **Non-English Inputs** | 1,298 (30%) |
| **Data Composition** | Public + proprietary |
| **Special Feature** | Long-document embedded injections |
| **Dataset Availability** | Not public (prevent overfitting) |
| **Date** | 2024 |

**Key Characteristics:**
- Objective evaluation measure for injection detection
- Prevents benchmark overfitting through non-disclosure
- Represents real-world prompt injection distribution
- Subset of injections embedded in longer documents

**Evaluation Methodology:**
- Tests both detection rate (true positives)
- Tests false negative minimization
- Fair comparison (all solutions untrained on dataset)

**Access Model:**
- Jupyter notebook interface (pint-benchmark.ipynb)
- Evaluate detection solutions directly
- No dataset download

**Commercial Use:** Limited (proprietary protection)

**Comparison to JailGuard:**
- PINT: 4,314 samples (private)
- JailGuard: 4,500 samples (public)
- Comparable scale
- PINT's private dataset maintains evaluation integrity
- JailGuard's public nature enables reproducibility

---

### 6.4 RedBench (RedTeamingBench)
**Source:** Hugging Face
**Paper:** "RedBench: A Universal Dataset for Comprehensive Red Teaming of Large Language Models" (2025)
**ArXiv:** https://arxiv.org/abs/2601.03699

| Metric | Value |
|--------|-------|
| **Total Samples** | 29,362 |
| **Source Datasets** | 37 (aggregated) |
| **Risk Categories** | 22 |
| **Domains** | 19 |
| **Prompt Types** | Attack + refusal |
| **Date** | January 2025 |
| **License** | Open source |

**Risk Categories (22):**
- Standardized across all 37 source datasets
- Consistent categorization enabling comparison

**Domain Coverage (19):**
- Broad coverage across diverse application areas

**Unique Feature:**
- Refusal prompts: Tests over-defense behavior
- Benign prompts designed to trigger unnecessary refusals
- Assesses safety boundary appropriateness

**Key Innovation:**
- Standardized taxonomy across disparate datasets
- Enables comparison of datasets with different schemas
- Aggregates state-of-the-art benchmarks

**Annotation Quality:**
- Leverages established datasets
- Unified taxonomy applied
- Baselines established for modern LLMs

**Commercial Use:** Yes (open source)

**Comparison to JailGuard:**
- RedBench: 29,362 samples (6.5x larger)
- RedBench: 22 risk categories + 19 domains
- JailGuard: 4,500 samples with focused taxonomy
- RedBench: Unification/aggregation approach
- JailGuard: Curated specialized dataset
- Value proposition: RedBench for comprehensive benchmarking, JailGuard for focused evaluation

---

### 6.5 LLMail-Inject (IEEE SaTML 2025 Challenge)
**Source:** ArXiv + Competition
**Paper:** "LLMail-Inject: A Dataset from a Realistic Adaptive Prompt Injection Challenge" (2025)
**ArXiv:** https://arxiv.org/abs/2506.09956
**Competition:** IEEE SaTML 2025 (Secure and Trustworthy Machine Learning)
**Timeline:** Dec 9, 2024 - Feb 3, 2025

| Metric | Value |
|--------|-------|
| **Unique Attack Prompts** | 208,095 |
| **Total Submissions** | 370,724 |
| **Participants** | 292 teams |
| **Registered Participants** | 621 |
| **Success Rate** | Variable (adaptive) |
| **Date** | 2025 (competition-based) |

**Challenge Setup:**
- Simulated LLM-based email assistant
- Attacker sends malicious emails
- Goal: Trigger unintended API calls (send_email)
- Detection required to avoid undetected attacks

**Prompts Labeled as Injection:**
- 29,011 prompts triggered send_email API
- Success determined by objective action (not just content)

**Key Characteristic:**
- Real-world scenario: email assistant
- Adaptive attacks reflecting actual attack strategies
- Ground truth: API call logs
- Large-scale community participation

**Dataset Value:**
- Newest dataset (2025)
- Reflects current attack sophistication
- Realistic scenario (email-based)
- High sample count

**License:** Open source (competition dataset)

**Comparison to JailGuard:**
- LLMail-Inject: 208,095 samples (46x larger)
- JailGuard: 4,500 samples
- LLMail-Inject: Email-specific scenario
- JailGuard: General-purpose detection
- LLMail-Inject: Very recent (2025)
- JailGuard: Complementary specialized focus

**Recency:** Most current dataset, reflects 2025 attack sophistication

---

### 6.6 Open-Prompt-Injection Benchmark
**Source:** GitHub
**GitHub:** https://github.com/liu00222/Open-Prompt-Injection

**Key Features:**
- Toolkit for attacks and defenses
- DataSentinel detection mechanism
- PromptLocate localization
- Multiple model support (PaLM2, Mistral, etc.)

**Comparison to JailGuard:**
- Open framework rather than static dataset
- Emphasis on tooling/implementation
- JailGuard: Data-focused

---

## 7. Notable Datasets (Emerging & Specialized)

### 7.1 JailBreakV-28K (Multimodal)
**Source:** Hugging Face
**Hugging Face:** JailbreakV-28K/JailBreakV-28k
**Paper:** "JailBreakV: A Benchmark for Assessing the Robustness of MultiModal Large Language Models against Jailbreak Attacks"

| Metric | Value |
|--------|-------|
| **Total Samples** | 28,000 |
| **Text Jailbreaks** | 20,000 |
| **Image-based MLLMs** | 8,000 |
| **Safety Policies** | 16 |
| **Date** | 2024 |

**Focus:**
- Multimodal LLM vulnerabilities
- Text-image jailbreak pairs
- Policy-based attack categorization

**Comparison to JailGuard:**
- Different modality (multimodal vs. text)
- Complementary for vision-language model safety
- JailGuard: Text-focused, JailBreakV: Multimodal

---

### 7.2 NotInject Benchmark (InjecGuard paper)
**Source:** GitHub + Paper
**GitHub:** https://github.com/SaFoLab-WISC/InjecGuard

| Metric | Value |
|--------|-------|
| **Benign Samples** | 339 |
| **Special Focus** | Trigger word bias testing |
| **Problem Addressed** | Over-defense in guard models |
| **Creation** | Systematic benign + trigger words |
| **Date** | 2024 |

**Key Insight:**
- Guard models suffer from trigger word bias
- False positives on benign text with "ignore", "cancel", etc.
- NotInject measures this vulnerability

**Annotation Quality:**
- Systematic combination of benign + trigger words
- Validates over-defense tendency

**Related Model:** PIGuard (successor to InjecGuard)

**Comparison to JailGuard:**
- NotInject: Specialized over-defense testing
- JailGuard: General detection benchmark
- Complementary: NotInject catches false positive problem

---

### 7.3 Gandalf Dataset (Lakera)
**Source:** Game-based collection
**Website:** Gandalf game (Lakera)

| Metric | Value |
|--------|-------|
| **Collection Method** | Competitive game |
| **Sample Size** | Subset published |
| **Focus** | Direct prompt injections |
| **Accessibility** | Partial release |

**Key Characteristic:**
- Game-based collection (similar to TensorTrust)
- Community-generated adversarial examples

---

### 7.4 Prompt Injection Malignant (Kaggle)
**Source:** Kaggle
**URL:** https://www.kaggle.com/datasets/marycamilainfo/prompt-injection-malignant

**Status:** Community dataset on Kaggle platform
**Availability:** Public download

---

## 8. Comparison Matrix

### Sample Size Comparison

| Dataset | Samples | Category | Scale |
|---------|---------|----------|-------|
| TensorTrust | 681,000 | Extraction/Hijacking | **XXL** |
| RedBench | 29,362 | Aggregated/Unified | **L** |
| LLMail-Inject | 208,095 | Email-specific | **XL** |
| BeaverTails | 333,963+ | Safety alignment | **XL** |
| ALERT | 15,000-45,000 | Red-teaming | **L** |
| JailBreakV-28K | 28,000 | Multimodal | **L** |
| HarmBench | 400-510 behaviors | Semantic/contextual | **M** |
| AdvBench | 520 | Harmful instructions | **M** |
| **JailGuard** | **4,500** | **General detection** | **M-L** |
| CPAD | 10,050 | Chinese-specific | **M** |
| xTRam1 safe-guard | 10,000 | Synthetic | **M** |
| deepset/prompt-injections | 662 | Foundational | **S** |
| SPML | 21,800 | Chatbot-specific | **M-L** |
| BIPIA | 5 tasks | Indirect attacks | **M** |
| Raccoon | 197+ | Extraction | **S** |
| Harelix | 1,174 | Mixed techniques | **S** |
| MultiJail | 3,150 | Multilingual | **S-M** |
| PINT | 4,314 | Private eval | **M** |

### Attack Type Coverage

| Attack Type | Datasets | Best For |
|------------|----------|----------|
| **Direct Injection** | deepset, xTRam1, Harelix, NotInject | Detection/classification |
| **Harmful Instruction** | AdvBench, MaliciousInstruct, HarmBench | Jailbreak evaluation |
| **Indirect Injection** | BIPIA | Context-based attacks |
| **Extraction** | TensorTrust, Raccoon | Prompt recovery |
| **Hijacking** | TensorTrust | Goal manipulation |
| **Social Engineering** | xTRam1, SPML | Defense mechanisms |
| **Context Manipulation** | xTRam1, SPML | Realistic scenarios |
| **Multilingual** | MultiJail, CyberSecEval | Language diversity |
| **Multimodal** | JailBreakV-28K, CyberSecEval-3+ | Vision-language models |
| **Email-specific** | LLMail-Inject | Email assistants |
| **Chatbot-specific** | SPML | Application-specific |
| **Compound** | Harelix, RedBench | Complex attacks |

### Annotation Quality Comparison

| Dataset | Method | Validation | Richness | Quality |
|---------|--------|-----------|----------|---------|
| deepset | Manual | Linguistic | Binary | ⭐⭐⭐⭐ |
| HarmBench | Expert | Semantic | Categorical | ⭐⭐⭐⭐⭐ |
| BeaverTails | Expert | Multi-stage | 14-19 categories | ⭐⭐⭐⭐⭐ |
| CPAD | GPT + filter | Goal-based | 3 dimensions | ⭐⭐⭐⭐ |
| TensorTrust | Game-based | Community | Organic | ⭐⭐⭐⭐ |
| SPML | Hybrid | Chatbot-def | Application | ⭐⭐⭐⭐ |
| xTRam1 | Synthetic | Template | Categorical | ⭐⭐⭐ |
| ALERT | Expert + template | Red-team | 32 micro | ⭐⭐⭐⭐ |
| LLMail-Inject | Game-based | API action | Objective | ⭐⭐⭐⭐⭐ |
| **JailGuard** | **Expert curated** | **Specialized** | **Focused** | **⭐⭐⭐⭐** |

### License & Commercial Use

| Dataset | License | Commercial | Research | Notes |
|---------|---------|-----------|----------|-------|
| deepset | Public | ✅ Yes | ✅ Yes | Permissive |
| HarmBench | MIT | ✅ Yes | ✅ Yes | Open source |
| AdvBench | CC BY-NC | ❌ Limited | ✅ Yes | Non-commercial |
| CPAD | CC BY-SA | ✅ Yes | ✅ Yes | Sharelike |
| BeaverTails | CC BY | ✅ Yes | ✅ Yes | Permissive |
| TensorTrust | Open | ✅ Yes | ✅ Yes | Community-friendly |
| BIPIA | MIT | ✅ Yes | ✅ Yes | Open source |
| Raccoon | (Inferred) | ✅ Yes | ✅ Yes | Public |
| xTRam1 safe-guard | Public | ✅ Yes | ✅ Yes | Open |
| SPML | Open | ✅ Yes | ✅ Yes | Public |
| ALERT | Open | ✅ Yes | ✅ Yes | Public |
| PINT | Limited | ❌ Restricted | ✅ Limited | Proprietary protection |
| RedBench | Open | ✅ Yes | ✅ Yes | Open source |
| LLMail-Inject | Competition | ✅ Yes | ✅ Yes | Public release |
| CyberSecEval | Meta-limited | ⚠️ Limited | ✅ Yes | Purple Llama framework |
| **JailGuard** | **TBD** | **TBD** | **✅ Yes** | **To be determined** |

---

## 9. Recommendations for JailGuard

### 9.1 Positioning within Ecosystem

**JailGuard's Unique Value:**
- **4,500 samples:** Positioned between small (662-1,174) and large (28k+) datasets
- **Curated quality:** Similar annotation depth to HarmBench
- **Focused scope:** Text-based prompt injection (not extraction, not multimodal)
- **Practical benchmark:** Between academic (AdvBench: 520) and industrial (TensorTrust: 681k)

**Optimal Size:**
- Small datasets (662-1,174): Limited coverage, but high quality
- Medium datasets (3,150-10,000): Good balance, practical training
- **JailGuard (4,500):** Sweet spot for specialized evaluation
- Large datasets (28k+): Comprehensive but require significant computational resources

### 9.2 Complementary Dataset Combinations

**For Comprehensive Safety Evaluation:**

**Combination 1: Core Benchmark Suite (Recommended)**
1. JailGuard (4,500 samples) - General detection
2. LLMail-Inject (208,095 samples) - Email-specific, latest
3. BIPIA (5 tasks) - Indirect attacks
4. MultiJail (3,150 samples) - Multilingual validation

*Total: 4,500 + 208,095 + 3,150 = 215,745 samples + indirect*
*Coverage: Direct + indirect + email-specific + multilingual*

**Combination 2: Academic Focused**
1. JailGuard (4,500)
2. HarmBench (510 behaviors → 1,020+ variants)
3. AdvBench (520)
4. CPAD (10,050)

*Total: 4,500 + 10,050 = 14,550 + benchmarks*
*Coverage: General + harmful instructions + Chinese + diverse semantics*

**Combination 3: Defense Training (RLHF/DPO)**
1. JailGuard (4,500) - Detection baseline
2. BeaverTails (333,963+) - Safety alignment
3. ALERT (45,000) - DPO triplets
4. SPML (21,800) - Application-specific

*Total: 404,263 samples*
*Purpose: Training robust defense models*

**Combination 4: Attack Strategy Research**
1. JailGuard (4,500)
2. TensorTrust (681,000) - Extraction/hijacking
3. Raccoon (197+) - Prompt extraction patterns
4. Tensor Trust artifacts (evolving) - State-of-art attacks

*Total: 685,700+*
*Purpose: Understanding attack vectors and evolution*

### 9.3 Dataset Extension Strategies

**Extend JailGuard to 10,000 samples:**
1. Expand existing attack categories by 2.2x
2. Add intermediate complexity examples
3. Include edge cases within each attack type
4. Results in more robust benchmarking

**Extend with Multilingual:**
1. Translate existing 4,500 samples to:
   - Spanish (high-resource)
   - Arabic (medium-resource)
   - Bengali (low-resource)
2. Validate translations against MultiJail patterns
3. Creates 13,500 multilingual samples

**Extend with Indirect Attacks:**
1. Use BIPIA methodology
2. Embed 4,500 samples within:
   - Document contexts (like BIPIA Web QA)
   - Email bodies (like LLMail-Inject)
   - System contexts (like SPML)
3. Creates 13,500 indirect variants

### 9.4 Integration with Existing Tools/Models

**Compatible with:**
1. **ProtectAI models** (deberta-v3) - Fine-tune with JailGuard
2. **Llama-Prompt-Guard-2** - Training alternative
3. **HarmBench evaluation** - Integrate as benchmark
4. **JailbreakBench leaderboard** - Submit evaluations
5. **Giskard scanning** - Input to testing library
6. **OpenAI Evals** - Compatible format

**Contribution Paths:**
1. Hugging Face Datasets - Public release path
2. GitHub + ArXiv - Benchmark/paper release
3. SafetyPrompts.com registry - Catalog entry
4. PINT-style evaluation - Benchmark tool

### 9.5 Gaps in Current Ecosystem

**Underserved Areas:**

| Gap | Opportunity | JailGuard Could Address |
|-----|-------------|------------------------|
| **Real-time attacks** | Only LLMail-Inject covers adaptive adversarial | Could add temporal dimension |
| **Code-based injection** | No Code QA attacks found | Extend to code context |
| **System-prompt specific** | SPML only, limited | More system prompt varieties |
| **Robustness testing** | NotInject alone for false positives | Expand over-defense testing |
| **Few-shot/zero-shot** | No explicit few-shot datasets | Add few-shot variants |
| **Language-specific** | MultiJail (10 langs), but limited injection types | Expand non-English injection types |
| **Model-specific** | Generic datasets, not model-targeted | Create model-specific variants |

### 9.6 Recommended Research Directions

**Using JailGuard as Foundation:**

1. **Cross-dataset Transferability Study**
   - Train on JailGuard, evaluate on: deepset, xTRam1, Harelix
   - Measure transferability across datasets
   - Identify dataset-specific biases

2. **Ensemble Evaluation**
   - Create meta-benchmark combining JailGuard + others
   - Weight by quality/size/recency
   - Improve robustness assessment

3. **Attack Evolution Tracking**
   - Compare JailGuard samples against LLMail-Inject
   - Identify emerging attack patterns
   - Version dataset to track sophistication

4. **Multilingual Validation**
   - Extend JailGuard to multiple languages
   - Validate against MultiJail patterns
   - Identify language-based vulnerabilities

5. **Over-defense Analysis**
   - Use JailGuard benign samples
   - Apply NotInject methodology
   - Measure guard model false positive rates

---

## 10. Dataset Acquisition & Access Methods

### Hugging Face Datasets
```python
from datasets import load_dataset

# Direct injection
datasets.load_dataset("deepset/prompt-injections")
datasets.load_dataset("xTRam1/safe-guard-prompt-injection")
datasets.load_dataset("Harelix/Prompt-Injection-Mixed-Techniques-2024")

# Jailbreak
datasets.load_dataset("walledai/AdvBench")
datasets.load_dataset("walledai/HarmBench")
datasets.load_dataset("JailbreakBench/JBB-Behaviors")

# Multilingual
datasets.load_dataset("JailbreakV-28K/JailBreakV-28k")
```

### GitHub Downloads
```bash
# TensorTrust
git clone https://github.com/HumanCompatibleAI/tensor-trust

# JailbreakBench
git clone https://github.com/JailbreakBench/jailbreakbench

# CPAD
git clone https://github.com/liuchengyuan123/CPAD

# BIPIA
git clone https://github.com/microsoft/BIPIA

# Raccoon
git clone https://github.com/M0gician/RaccoonBench

# RedBench
git clone https://github.com/Babelscape/ALERT
```

### Direct URLs
- **deepset:** https://huggingface.co/datasets/deepset/prompt-injections
- **HarmBench:** https://www.harmbench.org/
- **TensorTrust:** https://tensortrust.ai/
- **JailbreakBench:** https://jailbreakbench.github.io/
- **PINT:** https://github.com/lakeraai/pint-benchmark
- **LLMail-Inject:** https://arxiv.org/abs/2506.09956

---

## 11. Key Papers & Citations

### Foundational Papers

1. **AdvBench** - "Universal and Transferable Adversarial Attacks on Aligned Language Models"
   - Zou et al. (Dec 2023)
   - https://arxiv.org/abs/2312.10286

2. **BeaverTails** - "BeaverTails: Towards Improved Safety Alignment of LLM via a Human-Preference Dataset"
   - Ji et al. (July 2023)
   - NeurIPS 2023 Datasets & Benchmarks Track

3. **HarmBench** - "HarmBench: A Standardized Evaluation Framework for Automated Red Teaming and Robust Refusal"
   - Mazeika et al. (Feb 2024)
   - https://arxiv.org/abs/2402.04249

4. **TensorTrust** - "Tensor Trust: Interpretable Prompt Injection Attacks from an Online Game"
   - Toyer et al. (Nov 2023)
   - NeurIPS 2023

5. **CPAD** - "Goal-Oriented Prompt Attack and Safety Evaluation for LLMs"
   - Liu et al. (Sep 2023)
   - https://arxiv.org/abs/2309.11830

### Recent Papers (2024-2025)

6. **JailbreakBench** - "JailbreakBench: An Open Robustness Benchmark for Jailbreaking Large Language Models"
   - Chao et al. (2024)
   - NeurIPS 2024 Datasets & Benchmarks Track

7. **BIPIA** - "Benchmarking and Defending Against Indirect Prompt Injection Attacks"
   - Li et al. (Dec 2023)
   - https://arxiv.org/abs/2312.14197

8. **Raccoon** - "Raccoon: Prompt Extraction Benchmark of LLM-Integrated Applications"
   - Wang et al. (June 2024)
   - ACL 2024 Findings

9. **ALERT** - "ALERT: A Comprehensive Benchmark for Assessing Large Language Models' Safety through Red Teaming"
   - Tedeschi et al. (April 2024)
   - https://arxiv.org/abs/2404.08676

10. **MultiJail** - "Multilingual Jailbreak Challenges in Large Language Models"
    - Li et al. (Oct 2023)
    - ICLR 2024

11. **SPML** - "SPML: A DSL for Defending Language Models Against Prompt Attacks"
    - Sharma et al. (Feb 2024)
    - https://arxiv.org/abs/2402.11755

12. **CyberSecEval 2** - "CyberSecEval 2: A Wide-Ranging Cybersecurity Evaluation Suite for Large Language Models"
    - Huang et al. (April 2024)
    - https://arxiv.org/abs/2404.13161

13. **InjecGuard/NotInject** - "InjecGuard: Benchmarking and Mitigating Over-defense in Prompt Injection Guardrail Models"
    - (2024)
    - https://arxiv.org/abs/2410.22770

14. **LLMail-Inject** - "LLMail-Inject: A Dataset from a Realistic Adaptive Prompt Injection Challenge"
    - (Jan 2025)
    - https://arxiv.org/abs/2506.09956

15. **RedBench** - "RedBench: A Universal Dataset for Comprehensive Red Teaming of Large Language Models"
    - (Jan 2025)
    - https://arxiv.org/abs/2601.03699

---

## 12. Safety & Licensing Summary

### Fully Open/Permissive
- AdvBench (CC BY-NC, research use)
- BeaverTails (CC BY)
- HarmBench (MIT)
- TensorTrust (Open)
- CPAD (CC BY-SA)
- deepset/prompt-injections (Public)
- ALERT (Open)
- SPML (Open)
- RedBench (Open)
- MultiJail (Open)

### Research-Restricted
- AdvBench (Non-commercial clause)
- PINT (Private, prevents overfitting)

### Industrial/Limited
- CyberSecEval (Meta Purple Llama framework)
- Llama Guard (Meta-specific)

### Public Competitions
- LLMail-Inject (Competition data, publicly released)

---

## 13. Summary Statistics

### Dataset Ecosystem Overview

**Total Indexed Datasets:** 35+
**Total Documented Samples:** 2,500,000+
**Time Span:** Dec 2023 - Jan 2025
**Languages Covered:** 10+
**Categories/Taxonomies:** 100+

### Sample Distribution
- **Size < 1K:** 5 datasets
- **Size 1K-5K:** 9 datasets (includes JailGuard)
- **Size 5K-30K:** 7 datasets
- **Size 30K-100K:** 3 datasets
- **Size > 100K:** 5 datasets

### Attack Type Coverage
- **Direct Injection:** 12 datasets
- **Indirect Injection:** 1 dataset (BIPIA)
- **Jailbreak:** 15+ datasets
- **Extraction:** 3 datasets
- **Multimodal:** 2 datasets
- **Multilingual:** 4 datasets
- **Specialized:** 5+ datasets

### Geographic/Language Diversity
- **English-primary:** 25+ datasets
- **Multilingual:** 4 datasets
- **Chinese-specific:** 2 datasets

---

## 14. JailGuard Positioning Statement

### Size & Scale
JailGuard at 4,500 samples occupies an optimal position in the dataset ecosystem:
- Larger than foundational datasets (deepset: 662, AdvBench: 520)
- Smaller than massive datasets (TensorTrust: 681k, BeaverTails: 333k+)
- **Competitive with curated benchmarks** (HarmBench: 510 behaviors, CPAD: 10k, PINT: 4.3k)

### Quality & Curation
- **Comparable to HarmBench** in expert curation
- **Richer than synthetic datasets** (xTRam1, Harelix)
- **More focused than aggregated datasets** (RedBench)

### Use Cases
JailGuard is optimally suited for:
1. **Evaluation baseline** - Fair comparison across detection models
2. **Fine-tuning source** - Not too large, high quality
3. **Research validation** - Specialized focus, reproducible
4. **Defense benchmarking** - Against general prompt injection

### Complementary Role
- Best combined with: LLMail-Inject (email), BIPIA (indirect), MultiJail (multilingual)
- Alternative to: PINT (smaller, public vs. private)
- Supplement to: AdvBench, HarmBench (for general evaluation)

---

## 15. Appendix: Dataset URLs Reference

### Hugging Face Datasets
| Dataset | URL |
|---------|-----|
| deepset/prompt-injections | https://huggingface.co/datasets/deepset/prompt-injections |
| xTRam1/safe-guard-prompt-injection | https://huggingface.co/datasets/xTRam1/safe-guard-prompt-injection |
| Harelix/Prompt-Injection-Mixed-Techniques-2024 | https://huggingface.co/datasets/Harelix/Prompt-Injection-Mixed-Techniques-2024 |
| walledai/AdvBench | https://huggingface.co/datasets/walledai/AdvBench |
| walledai/HarmBench | https://huggingface.co/datasets/walledai/HarmBench |
| JailbreakBench/JBB-Behaviors | https://huggingface.co/datasets/JailbreakBench/JBB-Behaviors |
| JailbreakV-28K/JailBreakV-28k | https://huggingface.co/datasets/JailbreakV-28K/JailBreakV-28k |
| facebook/cyberseceval3-visual-prompt-injection | https://huggingface.co/datasets/facebook/cyberseceval3-visual-prompt-injection |

### GitHub Repositories
| Dataset | URL |
|---------|-----|
| TensorTrust | https://github.com/HumanCompatibleAI/tensor-trust |
| JailbreakBench | https://github.com/JailbreakBench/jailbreakbench |
| HarmBench | https://github.com/centerforaisafety/HarmBench |
| CPAD | https://github.com/liuchengyuan123/CPAD |
| BeaverTails | https://github.com/PKU-Alignment/beavertails |
| BIPIA | https://github.com/microsoft/BIPIA |
| Raccoon | https://github.com/M0gician/RaccoonBench |
| ALERT | https://github.com/Babelscape/ALERT |
| SPML | https://github.com/prompt-compiler/SPML |
| MultiJail | https://github.com/DAMO-NLP-SG/multilingual-safety-for-LLMs |
| Open-Prompt-Injection | https://github.com/liu00222/Open-Prompt-Injection |
| PINT Benchmark | https://github.com/lakeraai/pint-benchmark |
| Giskard Prompt Injections | https://github.com/Giskard-AI/prompt-injections |
| PurpleLlama | https://github.com/meta-llama/PurpleLlama |
| RedBench | https://github.com/Babelscape/ALERT |

### Websites & Benchmarks
| Resource | URL |
|----------|-----|
| TensorTrust Game | https://tensortrust.ai/ |
| HarmBench Website | https://www.harmbench.org/ |
| JailbreakBench Leaderboard | https://jailbreakbench.github.io/ |
| SafetyPrompts Registry | https://safetyprompts.com/ |
| SPML Website | https://prompt-compiler.github.io/SPML/ |
| CyberSecEval 4 | https://meta-llama.github.io/PurpleLlama/CyberSecEval/ |
| Phare LLM Benchmark | https://phare.giskard.ai/ |

### ArXiv Papers (Direct Links)
| Paper | URL |
|-------|-----|
| AdvBench | https://arxiv.org/abs/2312.10286 |
| BeaverTails | https://arxiv.org/abs/2307.04657 |
| HarmBench | https://arxiv.org/abs/2402.04249 |
| TensorTrust | https://arxiv.org/abs/2311.01011 |
| CPAD | https://arxiv.org/abs/2309.11830 |
| BIPIA | https://arxiv.org/abs/2312.14197 |
| Raccoon | https://arxiv.org/abs/2406.06737 |
| ALERT | https://arxiv.org/abs/2404.08676 |
| SPML | https://arxiv.org/abs/2402.11755 |
| CyberSecEval 2 | https://arxiv.org/abs/2404.13161 |
| InjecGuard | https://arxiv.org/abs/2410.22770 |
| MultiJail | https://arxiv.org/abs/2310.06474 |
| LLMail-Inject | https://arxiv.org/abs/2506.09956 |
| RedBench | https://arxiv.org/abs/2601.03699 |

---

## Sources

This comprehensive catalog was researched through systematic searches across:
- [Hugging Face Datasets](https://huggingface.co/datasets)
- [GitHub topic search](https://github.com/topics/prompt-injection)
- [ArXiv](https://arxiv.org/) (2023-2025 papers)
- [SafetyPrompts.com](https://safetyprompts.com/)
- Academic conference proceedings (NeurIPS, ICLR, ACL, EMNLP)
- Official project websites and repositories
- Research blog posts and documentation

---

**End of Document**

Last updated: 2026-01-16
