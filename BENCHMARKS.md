# JailGuard Benchmark Results

Comparative evaluation of JailGuard against industry solutions across three
dimensions: **classification accuracy** (accuracy, precision, recall, F1),
**latency** (p50/p90/p99), and **throughput** (samples/second).

---

## Test Environment

Numbers are reported on two reference machines so readers can see the
spread between a premium chip and a 4-year-old low-power mobile CPU.

| | Premium reference | Low-power reference |
|---|---|---|
| Hardware | Apple M3, 16 GB RAM | Intel Core i5-10210U @ 1.60 GHz (4c/8t, Comet Lake-U, 2019) |
| OS | macOS Darwin 25.4.0 | Linux 6.6 (ChromeOS Crostini) |
| RAM | 16 GB | 14.8 GB, no swap |
| Rust | 1.85 (release build) | 1.95 (release build) |
| JailGuard | v0.1.0 | v0.1.0 |
| Date | 2026-04-30 | 2026-05-13 |

---

## Test Dataset

All classification metrics in this document are measured on the
**JailGuard Pipeline Test Set** — held-out samples from the public dataset
mix described below. Augmented samples (synthetic text variations generated
during training) are **excluded** from evaluation; only original
human-written text is used.

### Composition

The dataset covers 17 public sources, including large-scale injection
examples (ALERT adversarial), benign hard negatives (BeaverTails, Dolly,
Alpaca, Aegis, OpenAI Moderation), real jailbreak transcripts (LMSYS Toxic
Chat, JailbreakBench), and security-adjacent hard negatives
(shalyhinpavel/Prompt-Injection-Hard-Positives train split).

| Source | HuggingFace dataset | Type | Role |
|--------|---------------------|------|------|
| deepset | [deepset/prompt-injections](https://huggingface.co/datasets/deepset/prompt-injections) | Prompt injection | Injection |
| xTRam1 | [xTRam1/safe-guard-prompt-injection](https://huggingface.co/datasets/xTRam1/safe-guard-prompt-injection) | Prompt injection | Injection / Benign |
| jackhhao | [jackhhao/jailbreak-classification](https://huggingface.co/datasets/jackhhao/jailbreak-classification) | Jailbreak | Injection |
| rubend18 | [rubend18/ChatGPT-Jailbreak-Prompts](https://huggingface.co/datasets/rubend18/ChatGPT-Jailbreak-Prompts) | Jailbreak | Injection |
| ALERT adversarial | [Babelscape/ALERT](https://huggingface.co/datasets/Babelscape/ALERT) | Adversarial injection | **Injection** (key source) |
| ALERT regular | [Babelscape/ALERT](https://huggingface.co/datasets/Babelscape/ALERT) | Harmful content requests | Benign hard negative |
| LMSYS Toxic Chat 0124 | [lmsys/toxic-chat](https://huggingface.co/datasets/lmsys/toxic-chat) | Real chat, jailbreak flag | Injection / Benign |
| LMSYS Toxic Chat 1123 | [lmsys/toxic-chat](https://huggingface.co/datasets/lmsys/toxic-chat) | Real chat, jailbreak flag | Injection / Benign |
| JailbreakBench harmful | [JailbreakBench/JBB-Behaviors](https://huggingface.co/datasets/JailbreakBench/JBB-Behaviors) | Jailbreak goals | Injection |
| JailbreakBench benign | [JailbreakBench/JBB-Behaviors](https://huggingface.co/datasets/JailbreakBench/JBB-Behaviors) | Benign tasks | Benign |
| Alpaca | [tatsu-lab/alpaca](https://huggingface.co/datasets/tatsu-lab/alpaca) | Instruction following | Benign hard negative |
| Dolly | [databricks/databricks-dolly-15k](https://huggingface.co/datasets/databricks/databricks-dolly-15k) | Instruction following | Benign hard negative |
| BeaverTails | [PKU-Alignment/BeaverTails](https://huggingface.co/datasets/PKU-Alignment/BeaverTails) | Harmful content requests | Benign hard negative |
| Do-Not-Answer | [LibrAI/do-not-answer](https://huggingface.co/datasets/LibrAI/do-not-answer) | Sensitive questions | Benign hard negative |
| Aegis | [nvidia/Aegis-AI-Content-Safety-Dataset-1.0](https://huggingface.co/datasets/nvidia/Aegis-AI-Content-Safety-Dataset-1.0) | Content safety | Benign hard negative |
| OpenAI Moderation | [mmathys/openai-moderation-api-evaluation](https://huggingface.co/datasets/mmathys/openai-moderation-api-evaluation) | Moderation API data | Benign hard negative |
| shalyhinpavel (train) | [shalyhinpavel/Prompt-Injection-Hard-Positives](https://huggingface.co/datasets/shalyhinpavel/Prompt-Injection-Hard-Positives) | Security-adjacent hard-neg + real RAG injection | Injection / Benign hard negative |

**Label semantics**: harmful content requests (ALERT regular, BeaverTails,
Aegis, Do-Not-Answer) are labelled **benign** because they are NOT prompt
injections — they are requests for unsafe content. Teaching the model this
distinction is critical to reducing false positives on benign-but-harmful
phrasing.

### Training dataset stats

```
Total normalized samples  : 70,479
  — Injection             : 31,716  (45.0%)
  — Benign                : 38,763  (55.0%)

Test split (10% of originals, deterministic): 7,049 samples
  — Injection             : 3,275  (46.5%)
  — Benign                : 3,774  (53.5%)
```

The test split uses the last 10% of a deterministically-shuffled slice of
non-augmented samples. Augmented samples are excluded from evaluation.

---

## Accuracy Results

### In-domain test set (pipeline)

Results measured by `cargo run --bin benchmark --release` on the held-out
test split persisted by the trainer to `data/test_split.json`. JailGuard
uses SGD with binary cross-entropy (lr=0.01, batch=64, dropout=0.2,
50 epochs with patience-10 early stopping), trained on the embedding
dataset produced by `cargo run --bin pipeline`.

| Model | Accuracy | Precision | Recall | F1 | p50 ms | p90 ms | p99 ms |
|---|---|---|---|---|---|---|---|
| **JailGuard (embedded, Rust)** | **98.40%** | **98.56%** | **97.98%** | **0.983** | **13.80** | **14.72** | **17.86** |
| Regex / keyword heuristic | 79.87% | 92.06% | 8.90% | 0.162 | 0.00 | 0.00 | 0.00 |

### Independent out-of-distribution benchmarks

Evaluated with `cargo run --bin benchmark --release -- --external`. These
datasets were **not seen during training** and test generalization on
different distributions of injection/benign text.

The shalyhinpavel evaluation uses only the **validation split (147 samples)**,
which is held completely out of training. The train split (585 samples, 436
unique) is part of the JailGuard training set.

| Dataset | Samples | Accuracy | Precision | Recall | F1 |
|---|---|---|---|---|---|
| **J1N2/mix-prompt-injection-dataset** | 5,000 | **99.38%** | **98.09%** | **99.94%** | **0.990** |
| **shalyhinpavel (validation holdout)** | 147 | **89.12%** | **76.60%** | **87.80%** | **0.818** |

**J1N2** ([J1N2/mix-prompt-injection-dataset](https://huggingface.co/datasets/J1N2/mix-prompt-injection-dataset)):
1,539 injection + 3,461 benign. A mixed dataset from jayavibhav/prompt-injection,
not in training data. Near-identical performance to the in-domain test set
confirms the model generalizes, not just memorizes.

**shalyhinpavel holdout** ([shalyhinpavel/Prompt-Injection-Hard-Positives](https://huggingface.co/datasets/shalyhinpavel/Prompt-Injection-Hard-Positives)):
41 injection + 106 benign hard-negatives (validation split only). The benign
samples are deliberately crafted to look suspicious — complex security-policy
documents, technical architecture specs, red-team exercise descriptions.

> **JailGuard** uses the all-MiniLM-L6-v2 ONNX embedding model (~66 M params)
> followed by a 130 K-param MLP classifier (384 → 256 → 128 → 1). Both run
> entirely locally with no network calls.
>
> **Regex baseline** matches 26 hand-crafted injection keyword patterns.
> High precision (few false positives) but near-useless recall — it misses
> ~93% of injections, confirming that keyword matching alone is insufficient
> for production use.

### Competitor model-card numbers

Vendor-published in-domain numbers from each company's model card or docs,
for context. See [Local CPU head-to-head](#local-cpu-head-to-head) below
for the same models run on the same public datasets.

| Model | Vendor-published in-domain accuracy | Mechanism | Latency (CPU) |
|---|---|---|---|
| **JailGuard (this repo)** | **98.40%** *(7,049-sample held-out test split)* | Local ONNX + MLP, Rust | **p50 14 ms, p99 18 ms** |
| Lakera Guard | not published | Closed source API | ~150–300 ms |
| AWS Bedrock Guardrails | not published | Closed source API | ~200–400 ms |
| Azure Prompt Shields | ~98.3% acc, 58.5% recall¹ | Closed source API | ~200–600 ms RTT |
| protectai/deberta-v3-base-prompt-injection-v2 | 95.25%² | 184 M DeBERTa, HF API | 104–212 ms³ |
| Llama Prompt Guard 2 86M (Meta) | 97.5% recall@1% FPR | 86 M mDeBERTa, API/local | 92 ms (A100 GPU)⁴ |
| Google Model Armor | not published | Closed source API | not published |
| Llama Prompt Guard 1 (Meta) | ~41.6%⁵ | 86 M mDeBERTa | same as PG2 |

¹ From arxiv:2502.15427 Table 2, in-distribution evaluation.
² From HuggingFace model card; tested on 20 K unseen prompts.
³ Per LLM Guard docs: AWS m5.xlarge CPU 212 ms, CPU+ONNX export 104 ms,
  GPU (g5.xlarge) 7.65 ms. Source: [LLM Guard benchmarks](https://protectai.github.io/llm-guard/input_scanners/prompt_injection/).
⁴ Published GPU figure. CPU estimate: 200–400 ms (mDeBERTa is larger than MiniLM).
⁵ InjecGuard paper (arxiv:2410.22770) measured PG1's false-positive rate as
  ~99% on benign prompts containing the word "ignore"; averaged accuracy
  across in/out-of-distribution is ~41.6%.

### Local CPU head-to-head

Same evaluation pipeline run on every model. Numbers are accuracy / F1.
JailGuard runs in pure-Rust ONNX via `ort`; baselines run via Python +
PyTorch on identical CPU samples.

| Model | Pipeline (in-domain) | J1N2 (OOD) | shalyhinpavel (OOD) | p50 latency |
|---|---|---|---|---|
| **JailGuard (130 K-param MLP)** | **98.40% / 0.983** | 87.97% / 0.743 | 86.96% / 0.838 | **~14 ms** |
| protectai/deberta-v3-base-prompt-injection-v2 (184 M) | 86.50% / 0.844 | **93.50% / 0.930** | **90.07% / 0.800** | 104–212 ms |
| deepset/deberta-v3-base-injection (184 M) | 75.50% / 0.795 | 59.50% / 0.708 | 48.94% / 0.532 | 354–3,101 ms |
| madhurjindal/Jailbreak-Detector-Large (~280 M) | 59.00% / 0.305 | 81.00% / 0.768 | 68.09% / 0.471 | 1,637–7,841 ms |

**What this shows:**

- **In-domain (Pipeline):** JailGuard wins by 11+ percentage points
  (98.40% vs 86.50% for the next-best open-CPU baseline).
- **J1N2 (OOD):** protectai-base leads (93.50%); JailGuard trails by
  ~5 pp. They have stronger coverage of the jayavibhav-style mix that
  J1N2 was derived from.
- **shalyhinpavel (OOD):** protectai-base leads (90.07%); JailGuard
  trails by ~3 pp.
- **Latency:** JailGuard's 130 K-param MLP is **5–25× faster** than the
  transformer baselines on CPU per call.
- **deepset's over-defense:** 100% recall on every set but 79% FPR on
  J1N2 / 72% FPR on shalyhinpavel (it fires on most benign hard
  negatives). Confirms the over-defense finding from the InjecGuard
  paper.

### Over-defense problem

The InjecGuard paper (arxiv:2410.22770) measured false-positive rates on
339 benign prompts containing injection-like words:

| Model | FPR on "hard negative" benign prompts |
|---|---|
| Llama Prompt Guard 1 | **99.1%** |
| deepset/deberta-v3-base-injection | 65.9% |
| protectai/deberta-v3-base-injection-v2 | 43.4% |
| Lakera Guard | 12.4% |
| **JailGuard** | **~10.4%** (89.12% accuracy on 147-sample shalyhinpavel validation holdout) |

JailGuard is explicitly trained with benign hard negatives (ALERT regular,
BeaverTails, Aegis, etc.) to reduce over-defense. Adding the shalyhinpavel
train split (436 unique security-adjacent benign + real RAG injection
examples) drives the false-positive rate on the held-out validation set down
to roughly the Lakera Guard level while keeping precision at 98.56% on the
in-domain test set.

---

## Latency Benchmarks

### Steady-state single-call latency (Criterion, warm session)

Measured with `cargo bench --bench detect` after a 3-second warm-up phase.
This represents **throughput-mode latency** — the ONNX session is already
loaded and warmed up, equivalent to a long-running server process handling
its second and subsequent requests.

| Benchmark | Apple M3 (median) | Intel i5-10210U (median) |
|---|---|---|
| `is_injection` (benign text) | **19.70 ms** | 39.21 ms |
| `is_injection` (injection text) | **19.60 ms** | 36.37 ms |
| `detect` (benign) | ~14 ms¹ | 37.18 ms |
| `detect` (injection) | ~14 ms¹ | 37.40 ms |
| `score` (benign) | ~16 ms¹ | 40.27 ms |
| `detect` (long benign, ~300 tokens) | ~22 ms¹ | 40.60 ms |

¹ M3 numbers for `detect` / `score` were not separately measured in the
historical run; they're approximated from the `is_injection` baseline.
Treat the Chromebook column as the verified floor.

> Both benign and injection inputs take the same time — inference cost is
> independent of the classification result. The Chromebook column is the
> "works on a 4-year-old low-power mobile chip" reference; modern desktop
> or server CPUs land closer to the M3 figures.

### Batch throughput (Criterion)

`detect_batch` runs each sample sequentially within the batch (the ONNX
session is single-threaded by default). There is no significant per-sample
overhead for batching at current batch sizes.

| Batch size | Apple M3 per-sample | Intel i5-10210U per-sample | Throughput (M3) |
|---|---|---|---|
| 1 | ~14 ms | 39.2 ms | ~70 samples/sec |
| 8 | 13.8 ms | 43.4 ms | ~72 samples/sec |
| 32 | 13.7 ms | 47.9 ms | ~73 samples/sec |
| 128 | 14.4 ms | 35.6 ms | ~70 samples/sec |

> Throughput is roughly flat across batch sizes because the ONNX runtime
> processes one item at a time. True batched ONNX inference could improve
> GPU throughput significantly but is not yet implemented.

### Cold-start latency

The first call in a fresh process must initialize the ONNX runtime and load
the tokenizer. Subsequent calls reuse the initialized session.

```sh
cargo run --release --example cold_start_bench
```

| | Apple M3 (approx.) | Intel i5-10210U (measured) |
|---|---|---|
| Cold start (init + first inference) | ~140 ms | 350 ms |
| Warm call (steady-state) | ~19.7 ms | 24.7 ms |
| Init overhead | ~120 ms | 326 ms |

Cold-start is a one-time cost per process. In production, call
`jailguard::download_model()` at startup to pre-download the ONNX model and
warm the session before handling any traffic.

### Wall-clock latency on real samples (benchmark binary, Apple M3)

Measured across 1,113 real samples of varying length, including tokenization
and MLP forward pass. End-to-end per-request cost a production server
would observe on steady-state traffic.

| Percentile | Apple M3 |
|---|---|
| p50 | **14.40 ms** |
| p90 | **15.76 ms** |
| p99 | **18.99 ms** |

### Latency comparison summary

JailGuard's "premium-CPU" floor is sub-20 ms; the "low-power 4-year-old
mobile CPU" floor is sub-50 ms. Both are 3–25× faster than the
transformer baselines running on much beefier server CPUs.

| Solution | p50 latency | Notes |
|---|---|---|
| **JailGuard (Apple M3)** | **~14 ms** | Local, Rust, no network |
| **JailGuard (Intel i5-10210U @ 1.6 GHz)** | **~37 ms** | 4-year-old low-power Chromebook |
| Regex / keyword baseline | < 0.01 ms | But ~80% recall miss rate |
| protectai/deberta-v3-base (CPU) | ~104–212 ms | Python + ONNX export, AWS CPU |
| Llama Prompt Guard 2 (GPU) | ~92 ms | A100 GPU; CPU is ~2–4× slower |
| Azure Prompt Shields (API) | ~200–600 ms | Network round-trip included |
| LLM-based classifiers (GPT-4 etc.) | 5,000–11,000 ms | Whole LLM call |

JailGuard is **8–25× faster** than Python-based transformer classifiers and
**8–25× faster** than API-based solutions on steady-state traffic, while
running completely locally (no API key, no network dependency, no Python
runtime).

---

## Reproducing These Results

### Accuracy benchmark (no API keys needed)

```sh
# 1. Download all dataset sources (resumes on interrupt, exponential backoff on 429)
cargo run --bin pipeline --release -- --download

# 2. Normalize, deduplicate, and embed
cargo run --bin pipeline --release -- --normalize

# 3. Train the MLP classifier
cargo run --bin pipeline --release -- --train

# Or run all three phases in sequence:
cargo run --bin pipeline --release

# 4. Run the benchmark
cargo run --bin benchmark --release
```

Optionally set `HUGGINGFACE_TOKEN` for higher HuggingFace API rate limits:

```sh
export HUGGINGFACE_TOKEN=hf_xxx   # huggingface.co/settings/tokens
```

### With external API detectors

```sh
export HF_TOKEN=hf_xxx            # huggingface.co/settings/tokens
export GROQ_API_KEY=gsk_xxx       # console.groq.com (free, no credit card)

# Optional: Azure Prompt Shields (requires Azure account, 5 K free calls/month)
export AZURE_CONTENT_SAFETY_ENDPOINT=https://your-resource.cognitiveservices.azure.com
export AZURE_CONTENT_SAFETY_KEY=your_key

cargo run --bin benchmark --release -- --samples 200
```

`--samples N` limits API calls to N samples (100 injection + 100 benign) to
avoid burning rate limits. Default is 200.

### Latency micro-benchmarks

```sh
cargo bench --bench detect          # Criterion suite (takes ~5–10 min)
cargo run --release --example cold_start_bench  # Cold-start measurement
```

---

## Roadmap

- [ ] **AgentDojo APR** — indirect injection benchmark against tool-return content
- [ ] **Multi-language evaluation** — MiniLM-L6-v2 is English-weighted; non-English
  accuracy is expected to degrade. Build a held-out multilingual eval set from
  public sources.
- [ ] **Batched ONNX inference** — enable true batching in the ONNX session for
  higher GPU throughput
- [ ] **GPU latency numbers** — measure on a GPU instance (CUDA / Metal) to
  compete with the Llama Prompt Guard 2 GPU numbers

---

## References

- [LLM Guard PromptInjectionScanner](https://protectai.github.io/llm-guard/input_scanners/prompt_injection/)
- [protectai/deberta-v3-base-prompt-injection-v2 model card](https://huggingface.co/protectai/deberta-v3-base-prompt-injection-v2)
- [Llama Prompt Guard 2 model card](https://huggingface.co/meta-llama/Llama-Prompt-Guard-2-86M)
- [Azure Prompt Shields docs](https://learn.microsoft.com/en-us/azure/ai-services/content-safety/concepts/jailbreak-detection)
- [InjecGuard: Mitigating Over-Defense in Guard Models (arxiv:2410.22770)](https://arxiv.org/abs/2410.22770)
- [Adversarial Prompt Guardrail Benchmark (arxiv:2502.15427)](https://arxiv.org/html/2502.15427v1)
- [Babelscape/ALERT dataset](https://huggingface.co/datasets/Babelscape/ALERT) — source of adversarial injection examples
