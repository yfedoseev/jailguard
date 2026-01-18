# Phase 2 Implementation Plan - Community Collection

**Status**: 📋 READY FOR PLANNING
**Objective**: Extend dataset by collecting real-world jailbreak attempts from community sources
**Timeline**: 4-6 weeks
**Target**: +4,000-6,000 samples
**Expected Improvement**: +0.4-0.8% accuracy (96.7-97.4% → 97.1-98.2%)

---

## Executive Summary

Phase 2 builds on Phase 1's success by collecting authentic jailbreak attempts from community sources. Rather than generating synthetic examples, Phase 2 focuses on capturing real attack patterns that actual users have discovered, shared, and discussed publicly.

### Key Differences from Phase 1
| Aspect | Phase 1 | Phase 2 |
|--------|---------|---------|
| Data Source | Generated (synthetic + LLM) | Real community sources |
| Attack Authenticity | Created specifically for training | Organic, discovered in the wild |
| Diversity | Controlled, templates-based | Organic, high natural variation |
| Volume | 12,000 samples | +16,000-18,000 total |
| Collection Effort | Low (code-based) | Moderate (scraping + manual review) |
| Label Quality | High (automated validation) | Medium-High (manual review needed) |

---

## Collection Sources Strategy

### Source 1: Reddit r/jailbreak (Estimated: 1,500-2,000 samples)

**Platform**: Reddit - r/jailbreak subreddit
**Rationale**:
- Active community dedicated to discussing jailbreak techniques
- High-quality, peer-reviewed attack discussions
- Real user experiences and attack results
- Public data, no TOS violations

**Collection Strategy**:
1. Use Reddit API (PRAW - Python Reddit API Wrapper)
2. Scrape last 1-2 years of posts (2024-2026)
3. Filter for text posts containing jailbreak attempts
4. Parse comments for attack discussions
5. Extract complete attack prompts from threads

**Targets**:
- Posts with >10 upvotes (community validation)
- Comments with >5 replies (indicates discussion)
- Posts tagged with attack types: prompt-injection, jailbreak, bypass

**Expected Yield**:
- ~3,000-4,000 raw posts
- After filtering: ~1,500-2,000 usable samples (50% pass rate)
- Attack types: RolePlay, JailbreakPatterns, OutputManipulation dominant

**Validation Criteria**:
- Contains clear injection/jailbreak attempt
- >15 characters (meaningful attack)
- <2000 characters (reasonable prompt size)
- No commercial product names (avoid bias)

**Timeline**: 1-1.5 weeks (data collection + processing)

**Implementation**:
```rust
// src/collection/reddit_collector.rs (NEW, ~400 LOC)
pub struct RedditCollector {
    client: RedditClient,  // PRAW wrapper
    start_date: DateTime,
    end_date: DateTime,
}

impl RedditCollector {
    pub async fn collect_from_subreddit(
        &self,
        subreddit: &str,
        keywords: &[&str],
    ) -> Vec<RawSample> {
        // 1. Query posts matching keywords
        // 2. Extract text and comments
        // 3. Parse for attack prompts
        // 4. Validate and score
    }
}
```

---

### Source 2: GitHub Adversarial Repositories (Estimated: 1,000-1,500 samples)

**Platform**: GitHub - Public repositories with adversarial/security focus
**Rationale**:
- Researchers publish adversarial attack examples
- Well-organized, tagged with attack types
- Direct access to source code and papers
- Community-validated through stars/forks

**Target Repositories**:
1. **Jailbreak Collections**:
   - `oobabooga/text-generation-webui` (LLM UI with jailbreak examples)
   - `Plachtaa/VALL-E-X` (includes adversarial prompts)
   - `geekan/MetaGPT` (has attack evaluation)

2. **Adversarial ML Repos**:
   - `adversarial-tools` - Various adversarial attack implementations
   - `fooling-models` - Collection of model fooling techniques
   - `llm-attacks` - Specific LLM attack repositories

3. **Security Research**:
   - Papers with code in `Papers With Code` tagged with "adversarial"
   - GitHub searches: `filename:prompts.txt`, `filename:attacks.json`

**Collection Strategy**:
1. Query GitHub API with targeted searches
2. Clone or download repositories
3. Parse README files for attack descriptions
4. Extract from JSON/YAML data files
5. Extract from Python test files
6. Extract from documentation

**File Patterns to Search**:
- `*.txt` files named: prompts, attacks, jailbreaks, adversarial
- `*.json` files with attack specifications
- `*.md` files with examples in code blocks
- `*.py` test files with attack strings

**Expected Yield**:
- ~500-800 repositories found
- After filtering: ~100-150 quality repositories
- From each: ~10-15 attack examples
- Total: ~1,000-1,500 samples

**Validation Criteria**:
- Repository has >50 stars (community validation)
- Clear attack/jailbreak focus
- Well-documented attacks
- License compatible with research use (MIT/Apache/GPL)

**Timeline**: 1 week (discovery + extraction)

**Implementation**:
```rust
// src/collection/github_collector.rs (NEW, ~350 LOC)
pub struct GitHubCollector {
    api_key: String,
    rate_limiter: RateLimiter,
}

impl GitHubCollector {
    pub async fn collect_from_search(
        &self,
        query: &str,
        file_patterns: &[&str],
    ) -> Vec<RawSample> {
        // 1. Search for repositories
        // 2. Clone and parse files
        // 3. Extract attack examples
        // 4. Parse code blocks and comments
    }

    async fn extract_from_file(&self, path: &Path) -> Vec<String> {
        // Parse JSON/YAML/text for attack examples
    }
}
```

---

### Source 3: Stack Overflow Attack Patterns (Estimated: 800-1,200 samples)

**Platform**: Stack Overflow - Q&A site with security discussions
**Rationale**:
- Real-world attack discussions
- Security professionals discussing techniques
- Questions about prompt injection and LLM security
- Public data with CC-BY-SA license

**Search Strategy**:
1. Query Stack Overflow API for questions tagged:
   - `prompt-injection`
   - `llm-security`
   - `jailbreak`
   - `adversarial`
   - `ai-safety`

2. Extract from question titles and bodies
3. Parse answers for discussions of attacks
4. Extract code examples

**High-Value Question Indicators**:
- >5 answers (multiple perspectives)
- >100 views (popular topic)
- Score >5 (community validation)
- Created after 2023 (recent/relevant)

**Expected Yield**:
- ~2,000-3,000 questions found
- After filtering: ~300-500 quality questions
- From questions + answers: ~800-1,200 attack descriptions

**Validation Criteria**:
- Clearly discusses security/attack
- Contains specific attack example
- From reputable user (>1000 reputation)
- >10 upvotes or 5 answers

**Timeline**: 0.5 week (automated scraping)

**Implementation**:
```rust
// src/collection/stackoverflow_collector.rs (NEW, ~300 LOC)
pub struct StackOverflowCollector {
    api_key: String,
}

impl StackOverflowCollector {
    pub async fn collect_from_tags(&self, tags: &[&str]) -> Vec<RawSample> {
        // 1. Query questions by tags
        // 2. Extract Q&A content
        // 3. Parse for attack examples
        // 4. Validate and score
    }
}
```

---

### Source 4: Academic Papers (Estimated: 500-800 samples)

**Platform**: arXiv, Papers With Code, academic repositories
**Rationale**:
- Peer-reviewed attack examples
- Systematic evaluation of new techniques
- Access to attack datasets used in papers
- Supplementary materials with prompts

**Papers to Mine** (Post-2023):
1. "Universal and Transferable Attacks on Aligned Language Models"
2. "Low-Resource Languages Jailbreaking"
3. "PINT Benchmark" supplementary attacks
4. "xTRam1 Evaluation Dataset"
5. Other prompt injection papers

**Collection Strategy**:
1. Search arXiv for keyword matches
2. Download paper PDFs
3. Extract text from PDFs
4. Search for attack examples in text
5. Download supplementary datasets
6. Parse JSON/CSV attack files

**Expected Yield**:
- ~50-100 relevant papers
- From each: ~5-15 attack examples
- From datasets: Additional 200-400
- Total: ~500-800 samples

**Timeline**: 1 week (paper acquisition + parsing)

**Implementation**:
```rust
// src/collection/arxiv_collector.rs (NEW, ~300 LOC)
pub struct ArxivCollector {
    pdf_parser: PdfExtractor,
}

impl ArxivCollector {
    pub async fn collect_from_query(
        &self,
        query: &str,
        years: (u32, u32),
    ) -> Vec<RawSample> {
        // 1. Search arXiv API
        // 2. Download PDFs
        // 3. Extract text and tables
        // 4. Parse attack examples
    }
}
```

---

### Source 5: Manual Community Contributions (Estimated: 500-700 samples)

**Platform**: GitHub Issues, Email, Community Forum
**Rationale**:
- Direct submissions from security researchers
- Novel attacks not yet published
- Community engagement
- Helps identify blind spots

**Collection Method**:
1. Create GitHub Issue template for submission
2. Request community samples
3. Email security researchers
4. Reddit post requesting submissions
5. Post in LLM community Discord/Slack

**Submission Format**:
```json
{
  "text": "The actual attack prompt here...",
  "attack_type": "RolePlay|InstructionOverride|...",
  "source": "discovered_where",
  "context": "brief description of attack",
  "contributed_by": "username/email"
}
```

**Timeline**: Ongoing (1 week for initial push)

---

## Phase 2 Implementation Roadmap

### Week 1: Infrastructure Setup

**Tasks**:
1. **API Access Setup**
   - Reddit API (PRAW) - Set up developer app
   - GitHub API - Generate personal token
   - Stack Overflow API - Get API key
   - arXiv API - Configure access (no key needed)

2. **Dependencies Addition**
   ```toml
   [dependencies]
   reqwest = { version = "0.11", features = ["json"] }
   tokio = { version = "1", features = ["full"] }
   praw = "0.1"  # Reddit API wrapper (hypothetical)
   octocat = "0.1"  # GitHub API wrapper
   pdf-extract = "0.1"
   chrono = "0.4"
   serde_json = "1.0"
   ```

3. **Error Handling**
   ```rust
   // src/collection/error.rs (NEW, ~100 LOC)
   pub enum CollectionError {
       ApiError(String),
       RateLimitExceeded,
       ParseError(String),
       ValidationError(String),
       NetworkError(String),
   }
   ```

4. **Validation Framework**
   ```rust
   // src/collection/validation.rs (NEW, ~200 LOC)
   pub struct SampleValidator {
       min_length: usize,
       max_length: usize,
       min_uniqueness: f32,  // 0.85 - avoid duplicates from Phase 1
       forbidden_patterns: Vec<String>,
   }
   ```

**Files Created**: 5 infrastructure files (~500 LOC total)
**Deliverable**: All APIs accessible, collectors ready for implementation

---

### Week 2: Reddit & GitHub Collectors

**Tasks**:
1. **Implement Reddit Collector**
   - Subreddit query logic
   - Post filtering
   - Comment extraction
   - Attack pattern parsing
   - Quality scoring

2. **Implement GitHub Collector**
   - Repository search
   - File extraction
   - Code block parsing
   - Attack example detection

3. **Rate Limiting & Retry Logic**
   - Implement exponential backoff
   - Cache responses
   - Error recovery

**Files Created**:
- `src/collection/reddit_collector.rs` (~400 LOC)
- `src/collection/github_collector.rs` (~350 LOC)
- `src/collection/rate_limiter.rs` (~150 LOC)

**Tests**:
- Mock API responses for testing
- Parsing correctness verification
- Rate limiting validation

**Deliverable**: Collect 2,500-3,500 raw samples

**Estimated Output**:
- Reddit: 1,500-2,000 samples
- GitHub: 1,000-1,500 samples
- Total: 2,500-3,500 raw

---

### Week 3: Stack Overflow, arXiv, & Manual

**Tasks**:
1. **Implement Stack Overflow Collector**
   - Tag-based query
   - Question/answer parsing
   - Score filtering

2. **Implement arXiv Collector**
   - Paper search and download
   - PDF parsing
   - Supplementary dataset extraction

3. **Set up Manual Contribution Pipeline**
   - GitHub issue template
   - Validation framework
   - Attribution tracking

4. **Data Deduplication**
   - Cross-source duplicate detection
   - Similarity-based clustering
   - Keep highest-quality examples

**Files Created**:
- `src/collection/stackoverflow_collector.rs` (~300 LOC)
- `src/collection/arxiv_collector.rs` (~300 LOC)
- `src/collection/manual_submission.rs` (~200 LOC)
- `src/collection/deduplication_cross_source.rs` (~250 LOC)

**Tests**:
- API response parsing
- PDF extraction correctness
- Deduplication accuracy

**Deliverable**: Collect additional 1,300-2,000 samples
**Total After Week 3**: 3,800-5,500 raw samples

---

### Week 4: Labeling & Quality Control

**Tasks**:
1. **Create Labeling Pipeline**
   ```rust
   // src/collection/labeling.rs (NEW, ~300 LOC)
   pub struct LabelingPipeline {
       detector: InjectionDetector,  // Use existing detection
       attack_classifier: AttackTypeClassifier,
       confidence_threshold: f32,
   }
   ```

2. **Automated Label Assignment**
   - Use Phase 1-trained models to suggest labels
   - Confidence scoring
   - Uncertainty flagging for manual review

3. **Manual Review Queue**
   - Flag low-confidence samples
   - Expert review interface
   - Appeal/correction process

4. **Quality Metrics**
   - Inter-rater agreement (Cohen's kappa)
   - Label distribution analysis
   - Confidence calibration

**Files Created**:
- `src/collection/labeling.rs` (~300 LOC)
- `src/collection/quality_metrics.rs` (~250 LOC)
- `examples/review_phase2_samples.rs` (~200 LOC)

**Deliverable**:
- All 4,000-6,000 samples labeled
- Quality metrics reported
- ~200 flagged for manual review

---

### Week 5: Integration & Testing

**Tasks**:
1. **Dataset Integration**
   - Merge Phase 1 (12,000) + Phase 2 (4,000-6,000) = 16,000-18,000 total
   - Re-run deduplication on combined set
   - Final balance ratio calculation

2. **Comprehensive Testing**
   ```rust
   // tests/phase2/test_collection.rs (~400 LOC)
   #[test]
   fn test_reddit_collector() { }

   #[test]
   fn test_github_collector() { }

   #[test]
   fn test_integration_dataset() { }

   #[test]
   fn test_no_duplicates_across_phases() { }

   #[test]
   fn test_label_distribution() { }
   ```

3. **Documentation**
   - Collection methodology
   - Source attribution
   - License compliance
   - Dataset card (Model Card format)

**Files Created**:
- `tests/phase2/test_collection.rs` (~400 LOC)
- `docs/PHASE_2_DATASET_CARD.md` (~400 lines)
- `PHASE_2_COMPLETION_SUMMARY.md` (~500 lines)

**Deliverable**: Phase 2 dataset verified and integrated

---

### Week 6: Evaluation Preparation

**Tasks**:
1. **Create Training Examples**
   ```bash
   examples/train_phase2_extended.rs
   examples/evaluate_phase2_improvement.rs
   ```

2. **Benchmark Preparation**
   - Prepare evaluation harness for accuracy measurement
   - Cross-validation on PINT, xTRam1 benchmarks
   - Comparison with Phase 1 results

3. **Documentation & Release**
   - Roadmap for Phase 3
   - Lessons learned
   - Community contribution guidelines

**Deliverable**: Ready for accuracy evaluation

---

## Phase 2 Dataset Specifications

### Expected Volume Distribution

| Source | Estimated Samples | % of Phase 2 | Attack Types |
|--------|-----------------|--------------|--------------|
| Reddit | 1,500-2,000 | 40% | RolePlay, JailbreakPatterns, OutputManipulation |
| GitHub | 1,000-1,500 | 25% | All types, especially EncodingObfuscation |
| Stack Overflow | 800-1,200 | 20% | InstructionOverride, ContextManipulation |
| arXiv Papers | 500-800 | 10% | All types, academic rigor |
| Manual | 500-700 | 5% | Novel/emerging types |
| **Total** | **4,000-6,000** | **100%** | **Balanced** |

### Expected Attack Type Distribution

```
Original Distribution (Phase 1):
  RolePlay:              18%
  InstructionOverride:   22%
  ContextManipulation:   16%
  OutputManipulation:    17%
  EncodingObfuscation:   14%
  JailbreakPatterns:     11%
  Benign:                2%

Phase 2 Additions (Real-world):
  RolePlay:              25% (more prevalent in community)
  InstructionOverride:   18%
  ContextManipulation:   12%
  OutputManipulation:    20% (good for extraction attacks)
  EncodingObfuscation:   18% (GitHub has many encoding examples)
  JailbreakPatterns:     5%
  Benign:                2%

Combined Distribution (Post-Phase 2):
  RolePlay:              21%
  InstructionOverride:   20%
  ContextManipulation:   14%
  OutputManipulation:    18%
  EncodingObfuscation:   15%
  JailbreakPatterns:     10%
  Benign:                2%
```

### Quality Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Authenticity | 95%+ | Manual review sample |
| Label Accuracy | 90%+ | Expert consensus |
| Novelty vs Phase 1 | 85%+ | Embedding similarity |
| Linguistic Diversity | 80%+ | Token uniqueness |
| Attack Coverage | 95%+ | All 6 types represented |

---

## Phase 2 Success Criteria

### Primary Success
- [ ] Collect 4,000-6,000 new authentic samples
- [ ] Maintain 90%+ label accuracy
- [ ] Zero license/TOS violations
- [ ] All samples validate against detector
- [ ] No personally identifiable information (PII)

### Secondary Success
- [ ] 85%+ novelty vs Phase 1
- [ ] Balanced attack type distribution
- [ ] Documentation complete
- [ ] Community contributions active
- [ ] Deduplication clean (0.1% duplicates max)

### Stretch Goals
- [ ] 6,000+ samples (upper bound)
- [ ] 95%+ label accuracy
- [ ] <0.05% duplicates
- [ ] Achieve 97.1-98.2% accuracy in evaluation
- [ ] Academic paper from collection methodology

---

## Risk Mitigation

### Risk 1: API Rate Limiting
**Probability**: High | **Impact**: Medium
**Mitigation**:
- Implement exponential backoff
- Cache responses
- Stagger collection over time
- Use multiple API keys

### Risk 2: Data Quality Issues
**Probability**: Medium | **Impact**: High
**Mitigation**:
- Manual review of 20% sample
- Automated validation pipeline
- Clear labeling guidelines
- Iterative refinement

### Risk 3: License Compliance
**Probability**: Low | **Impact**: Critical
**Mitigation**:
- Verify license before collection
- Attribute all sources
- Legal review for academic use
- CC-BY-SA compliance for Stack Overflow

### Risk 4: PII Leakage
**Probability**: Medium | **Impact**: High
**Mitigation**:
- Scan for email addresses, phone numbers
- Redact usernames
- Manual review for sensitive data
- Strip metadata

### Risk 5: Duplicate Collection
**Probability**: Medium | **Impact**: Low
**Mitigation**:
- Cross-source deduplication
- Embedding similarity checking
- Canonical form extraction
- Manual spot-checking

---

## Resource Requirements

### Development
- 1 full-time engineer: 6 weeks
- 1 part-time data scientist: 2 weeks (labeling review)
- 1 security reviewer: 1 week (license/compliance)

### Infrastructure
- GitHub API quota: ~60K requests/month (included free)
- Reddit API: ~100 requests/min (free tier sufficient)
- Stack Overflow API: ~10K requests/month (free)
- arXiv: Unlimited (free)
- Storage: ~200MB for 6K samples

### Tools & Services
- PDF parsing library (open source)
- Text embedding model (existing)
- Rate limiting library (open source)
- Annotation tool (open source - Prodigy optional)

---

## File Structure (Phase 2)

```
src/collection/
├── mod.rs                              (NEW)
├── error.rs                            (NEW, ~100 LOC)
├── validation.rs                       (NEW, ~200 LOC)
├── rate_limiter.rs                     (NEW, ~150 LOC)
├── reddit_collector.rs                 (NEW, ~400 LOC)
├── github_collector.rs                 (NEW, ~350 LOC)
├── stackoverflow_collector.rs          (NEW, ~300 LOC)
├── arxiv_collector.rs                  (NEW, ~300 LOC)
├── manual_submission.rs                (NEW, ~200 LOC)
├── deduplication_cross_source.rs       (NEW, ~250 LOC)
├── labeling.rs                         (NEW, ~300 LOC)
└── quality_metrics.rs                  (NEW, ~250 LOC)

tests/phase2/
├── test_reddit_collector.rs            (NEW, ~250 LOC)
├── test_github_collector.rs            (NEW, ~250 LOC)
├── test_integration.rs                 (NEW, ~200 LOC)
└── test_quality_metrics.rs             (NEW, ~200 LOC)

examples/
├── collect_phase2_data.rs              (NEW, ~300 LOC)
├── label_phase2_samples.rs             (NEW, ~250 LOC)
└── evaluate_phase2_improvement.rs      (NEW, ~300 LOC)

docs/
├── PHASE_2_DATASET_CARD.md             (NEW, ~400 lines)
└── COLLECTION_METHODOLOGY.md           (NEW, ~300 lines)
```

---

## Timeline Summary

| Week | Task | Deliverable | Status |
|------|------|-------------|--------|
| Week 1 | Infrastructure, APIs, Validation | 5 new files, APIs ready | Pending |
| Week 2 | Reddit + GitHub Collectors | 2,500-3,500 samples | Pending |
| Week 3 | Stack Overflow, arXiv, Manual | +1,300-2,000 samples | Pending |
| Week 4 | Labeling & Quality Control | All samples labeled, metrics | Pending |
| Week 5 | Integration & Testing | Phase 2 dataset verified | Pending |
| Week 6 | Evaluation Preparation | Ready for accuracy eval | Pending |

**Total**: 6 weeks | **Total LOC**: ~4,500+ | **Total Samples**: 4,000-6,000

---

## Expected Outcome

### Dataset Growth
```
Phase 1 Complete: 12,000 samples
Phase 2 Addition:  +5,000 samples (estimated mid-point)
Phase 2 Total:     17,000 samples (2.67x → 3.78x original)
```

### Accuracy Projection
```
Baseline (95.9%):           4,500 samples
Phase 1 (96.7-97.4%):      12,000 samples (+0.8-1.5%)
Phase 2 (97.1-98.2%):      17,000 samples (+0.4-0.8%)

Conservative estimate:
  Baseline:  95.9%
  +Phase 1:  +0.8% → 96.7%
  +Phase 2:  +0.4% → 97.1%

Optimistic estimate:
  Baseline:  95.9%
  +Phase 1:  +1.5% → 97.4%
  +Phase 2:  +0.8% → 98.2%
```

### Community Impact
- Open-sourced Phase 2 dataset
- Established collection methodology
- Framework for continuous improvement
- Contributors acknowledged and credited
- Research publication opportunity

---

## Next Phase (Phase 3)

Upon Phase 2 completion and evaluation success, Phase 3 will focus on:

1. **Production Data Partnerships** (+5,000-10,000 samples)
   - Anonymized real-world attack attempts
   - Enterprise security logs
   - LLM provider datasets

2. **Multilingual Extension** (+10,000-20,000 samples)
   - Attack prompts in 5-10 languages
   - Cultural variation in jailbreak techniques
   - Cross-lingual robustness

3. **Continuous Collection** (Ongoing)
   - Automated monitoring of new attacks
   - Community feedback loop
   - Quarterly dataset updates

---

## Conclusion

Phase 2 completes the community-focused dataset extension strategy, collecting authentic real-world jailbreak attempts from multiple sources. Combined with Phase 1's synthetic augmentation, this approach provides comprehensive coverage of attack surface while maintaining data authenticity and quality.

**Status**: ✅ PLANNING COMPLETE - Ready for implementation upon approval

---

**Document Status**: 📋 READY FOR PHASE 2 IMPLEMENTATION
**Date**: January 17, 2026
**Next Action**: Begin Week 1 infrastructure setup or request modifications to strategy
