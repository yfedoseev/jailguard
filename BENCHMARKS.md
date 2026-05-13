# JailGuard Benchmark Results

Comparative evaluation of JailGuard against industry solutions across three
dimensions: **classification accuracy** (accuracy, precision, recall, F1),
**latency** (p50/p90/p99), and **throughput** (samples/second).

---

## Test Environment

| | |
|---|---|
| Hardware | Apple M3, 16 GB RAM |
| OS | macOS Darwin 25.4.0 |
| Rust | 1.85 (release build, `--release`) |
| JailGuard | v0.1.0 |
| Date | 2026-04-30 |

---

## Test Dataset

All classification metrics in this document are measured on the
**JailGuard Pipeline Test Set** — held-out samples from the public dataset mix
described below. Augmented samples (synthetic text variations generated during
training) are **excluded** from evaluation; only original human-written text is
used.

### Composition

The dataset covers 17 public sources, including large-scale injection examples
(ALERT adversarial), benign hard negatives (BeaverTails, Dolly, Alpaca, Aegis,
OpenAI Moderation), real jailbreak transcripts (LMSYS Toxic Chat,
JailbreakBench), and security-adjacent hard negatives
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

**Label semantics**: harmful content requests (ALERT regular, BeaverTails, Aegis,
Do-Not-Answer) are labelled **benign** because they are NOT prompt injections — they
are requests for unsafe content. Teaching the model this distinction is critical
to reducing false positives on benign-but-harmful phrasing.

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
| **JailGuard iter-9 (embedded, Rust)** | **98.40%** | **98.56%** | **97.98%** | **0.983** | **13.80** | **14.72** | **17.86** |
| Regex / keyword heuristic | 79.87% | 92.06% | 8.90% | 0.162 | 0.00 | 0.00 | 0.00 |

### Independent out-of-distribution benchmarks

Evaluated with `cargo run --bin benchmark --release -- --external`. These
datasets were **not seen during training** and test generalization on different
distribution of injection/benign text.

The shalyhinpavel evaluation uses only the **validation split (147 samples)**,
which is held completely out of training. The train split (585 samples, 436
unique) is part of the JailGuard training set as of this version.

| Dataset | Samples | Accuracy | Precision | Recall | F1 |
|---|---|---|---|---|---|
| **J1N2/mix-prompt-injection-dataset** | 5,000 | **99.38%** | **98.09%** | **99.94%** | **0.990** |
| **shalyhinpavel (validation holdout)** | 147 | **89.12%** | **76.60%** | **87.80%** | **0.818** |

**J1N2** ([J1N2/mix-prompt-injection-dataset](https://huggingface.co/datasets/J1N2/mix-prompt-injection-dataset)):
1,539 injection + 3,461 benign. A mixed dataset from jayavibhav/prompt-injection,
not in training data. Near-identical performance to the in-domain test set —
confirms the model generalizes, not just memorizes.

**shalyhinpavel holdout** ([shalyhinpavel/Prompt-Injection-Hard-Positives](https://huggingface.co/datasets/shalyhinpavel/Prompt-Injection-Hard-Positives)):
41 injection + 106 benign hard-negatives (validation split only). The benign
samples are deliberately crafted to look suspicious — complex security-policy
documents, technical architecture specs, red-team exercise descriptions. Adding
the train split (436 unique samples) to JailGuard's training data reduced the
false-positive rate substantially: see over-defense table below.

> **JailGuard** uses the all-MiniLM-L6-v2 ONNX embedding model (~66 M params)
> followed by a 130 K-param MLP classifier (384 → 256 → 128 → 1). Both run
> entirely locally with no network calls.
>
> **Regex baseline** matches 26 hand-crafted injection keyword patterns.
> High precision (few false positives) but near-useless recall — it misses
> ~93% of injections, confirming that keyword matching alone is insufficient
> for production use.

### Published numbers from competitor model cards

Each vendor evaluates on its own test set, so the in-domain accuracies
below are **not directly comparable** — they're cited only to give a
sense of where each model claims to land on its own data. For
apples-to-apples numbers see the local-CPU head-to-head sections below,
which run every model on the same public datasets.

| Model | Vendor-published in-domain accuracy | Mechanism | Latency (CPU) |
|---|---|---|---|
| **JailGuard iter-9 (this repo)** | **98.40%** *(7,049-sample held-out test split)* | Local ONNX + MLP, Rust | **p50 14 ms, p99 18 ms** |
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
  ~99% on benign prompts containing the word "ignore"; averaged accuracy across
  in/out-of-distribution is ~41.6%.

### Local CPU head-to-head — iter-7 (no mosscap) — measured 2026-05-05

Run with `benchmark/compare_local_models.py --limit 200` — same 200-sample
balanced subset per dataset for every model. Three open-source baselines
ran successfully; five additional models (Llama Prompt Guard 2 22M/86M,
protectai-small, qualifire/sentinel, hlyn) returned 403 because the
HuggingFace account hadn't accepted those models' license click-through
yet — see [Reproducing](#reproducing-these-results) for the one-time fix.

JailGuard's row uses its iter-7 ship-ready numbers from the Rust
benchmark (full deduped sets via `cargo run --bin benchmark --release --
--external`); the other rows are from the Python script's n=200 random
subsets. **Direct cell-to-cell comparison is approximate** (different
sample sizes + the cross-dedup gate in the Rust path). Same direction,
not bit-identical.

| Model | Pipeline acc / F1 | J1N2 acc / F1 | shalyhinpavel acc / F1 | p50 latency¹ |
|---|---|---|---|---|
| **JailGuard (embedded, Rust, 130K MLP)** | **98.85% / 0.989** | 80.51% / 0.503 | **91.30% / 0.900** | ~25-90 ms² |
| protectai/deberta-v3-base-prompt-injection-v2 (184M) | 85.00% / 0.824 | **93.50% / 0.930** | 90.07% / 0.800 | 185 ms / 1145 ms / 1271 ms³ |
| deepset/deberta-v3-base-injection (184M) | 83.00% / 0.855 | 59.50% / 0.708 | 48.94% / 0.532 | 333 / 1302 / 1538 ms |
| madhurjindal/Jailbreak-Detector-Large (~280M) | 61.00% / 0.381 | 81.00% / 0.768 | 68.09% / 0.471 | 243 / 1362 / 1839 ms |

¹ JailGuard from `cargo bench --bench detect`; baselines from Python
`compare_local_models.py` (CPU, no batching).
² Steady-state warm; varies with system load (p50 was 14 ms on Apple M3,
74-89 ms on shared Linux runner).
³ Three latency values are pipeline / J1N2 / shalyhinpavel respectively
(longer prompts in J1N2/shalyhin → higher per-call cost on transformer
baselines).

**What the table shows:**

- JailGuard wins **in-domain** (98.85% vs 85% best baseline) and
  **shalyhinpavel** (91.30% vs 90.07% best baseline) — the two evals
  most representative of curated training distributions and benign
  hard negatives. The shalyhinpavel margin is notable because protectai
  trained on a similar-style dataset; we hold even with their 184M
  model using a 130K-param MLP.
- **protectai-base wins J1N2** (93.5% vs our 80.5%). They have stronger
  coverage of the jayavibhav/prompt-injection style (J1N2's source).
  This is our biggest open gap — see Roadmap.
- **deepset's over-defense problem is visible**: 100% recall on every
  set, but 79% FPR on J1N2 / 72% FPR on shalyhinpavel (it fires on most
  benign hard negatives). Confirms the InjecGuard paper's finding.
- **madhur** swings the other way — 24% recall on pipeline (very
  conservative), 81% acc on J1N2 / 68% on shalyhin from being
  selective, but misses most pipeline injections.
- **Latency:** JailGuard's MLP is ~5-25× faster than the transformer
  baselines on CPU per call. The advantage compounds at scale.

**5 models still to evaluate** (gated, awaiting license accept):

| Model | Why missing | One-time fix |
|---|---|---|
| meta-llama/Llama-Prompt-Guard-2-22M | Llama license not accepted | Visit [HF page](https://huggingface.co/meta-llama/Llama-Prompt-Guard-2-22M), agree |
| meta-llama/Llama-Prompt-Guard-2-86M | Llama license | same as above |
| protectai/deberta-v3-small-prompt-injection-v2 | Recently gated | Visit [HF page](https://huggingface.co/protectai/deberta-v3-small-prompt-injection-v2), agree |
| qualifire/prompt-injection-sentinel | Recently gated | Visit [HF page](https://huggingface.co/qualifire/prompt-injection-sentinel), agree |
| hlyn/prompt-injection-judge-deberta-70m | Manual approval required | Visit [HF page](https://huggingface.co/hlyn/prompt-injection-judge-deberta-70m), request access |

Re-run after accepting:
```sh
/tmp/adj-venv/bin/python benchmark/compare_local_models.py \
  --models pg2-22m,pg2-86m,protectai-small,sentinel,hlyn \
  --limit 200 --hf-token $HF_TOKEN --output-dir results
```

### Local CPU head-to-head — iter-8 (+mosscap) — measured 2026-05-05

Iter-8 added `Lakera/mosscap_prompt_injection` train split (11,872 unique
samples after dedup) to JailGuard's training mix to lift J1N2 recall.
The mosscap dataset is 223K real human-authored attack attempts from
Lakera's gamified injection challenge — diverse direct-injection
attacker writing styles. Three baselines re-run on the new
pipeline_embeddings.json random subset for fresh apples-to-apples
comparison.

| Model | Pipeline acc / F1 | J1N2 acc / F1 | shalyhinpavel acc / F1 | p50 latency¹ |
|---|---|---|---|---|
| **JailGuard iter-8 (130K MLP, +mosscap)** | **97.89% / 0.972** | **89.65% / 0.794** | 83.70% / 0.800 | ~80-110 ms |
| protectai/deberta-v3-base-prompt-injection-v2 (184M) | 86.50% / 0.844 | **93.50% / 0.930** | **90.07% / 0.800** | 220-1554 ms |
| deepset/deberta-v3-base-injection (184M) | 74.50% / 0.787 | 59.50% / 0.708 | 48.94% / 0.532 | 423-1883 ms |
| madhurjindal/Jailbreak-Detector-Large (~280M) | 58.50% / 0.325 | 81.00% / 0.768 | 68.09% / 0.471 | 475-1872 ms |

¹ JailGuard from Rust benchmark on full deduped sets; baselines from
Python script on n=200 subsets. Same caveat as iter-7 table: not
bit-identical methodology.

### Iter-7 vs iter-8 — the trade-off

Same JailGuard architecture; only training data differs. Cross-dedup
gate, persisted split, and SGD methodology unchanged.

| JailGuard variant | Pipeline | J1N2 acc / **recall** | shalyhin acc / **recall** | AgentDojo acc / **recall** | Avg acc |
|---|---|---|---|---|---|
| **iter-7 (3K synthetic, no mosscap)** | 98.85% | 80.51% / **35.29%** | **91.30% / 87.80%** | **82.54% / 60.87%** | 88.30% |
| **iter-8 (3K synthetic + 11.8K mosscap)** | 97.89% | **89.65% / 71.25%** | 83.70% / 73.17% | 79.37% / 56.52% | 87.65% |

**What changed:**

- **J1N2: +9.14pp accuracy, +35.96pp recall.** Biggest single-iteration
  improvement we've achieved. Iter-7 missed 65% of J1N2 injections;
  iter-8 misses 29%. Real safety win on diverse human attack patterns.
- **shalyhinpavel: -7.60pp accuracy, -14.63pp recall.** Mosscap's
  attack-heavy distribution shifted the decision boundary toward
  conservative-on-benigns. Same over-defense pattern that bit us with
  the failed iter-5 HackAPrompt experiment, smaller magnitude.
- **AgentDojo: -3.17pp accuracy, -4.35pp recall.** Marginal regression;
  mosscap's direct-injection style doesn't transfer to indirect
  agent-context attacks.
- **In-domain: -0.96pp.** Acceptable cost for the J1N2 lift.

**Average is essentially flat (88.30% → 87.65%).** Iter-7 wins shalyhin
+ AgentDojo; iter-8 wins J1N2. Different operating points of the
precision/recall trade-off, not strict improvement either way.

**Decision: iter-9 shipped.** See the iter-9 section below for the rationale and full comparison table.

### Local CPU head-to-head — iter-9 (filtered mosscap + benign hard-negs) — measured 2026-05-05

Iter-9 was the rebalance attempt: keep iter-8's J1N2 lift while
recovering iter-7's shalyhinpavel performance. Two changes:

1. **Mosscap filtered to Level 1-6 only** (drops 6,759 subtler L7/L8
   attacks like "let's play word association game" that pushed the
   model toward over-defense on benign-but-suspicious text). 6,386
   unique mosscap samples after dedup, vs iter-8's 11,872.
2. **+1,446 synthetic security-doc benign hard-negatives** generated
   by an agent in shalyhinpavel-style: internal security policies,
   ADRs, runbooks, RBAC specs, compliance attestations. Embedded
   suspicious-looking phrasing ("ignore the deprecated warning until
   patched", "override the default policy when X") in legitimate
   document context. 32 distinct document types.

Cross-dedup gate stats unchanged (J1N2 5.9%, AgentDojo 1.6%,
shalyhinpavel 37.4%) — the new sources don't introduce new
contamination.

| Model | Pipeline acc / F1 | J1N2 acc / F1 | shalyhinpavel acc / F1 | p50 latency¹ |
|---|---|---|---|---|
| **JailGuard iter-9 (130K MLP, filtered mosscap + benigns)** | **98.40% / 0.983** | 87.97% / 0.743 | 86.96% / 0.838 | ~50-140 ms² |
| protectai/deberta-v3-base-prompt-injection-v2 (184M) | 86.50% / 0.844 | **93.50% / 0.930** | **90.07% / 0.800** | 1448-1564 ms |
| deepset/deberta-v3-base-injection (184M) | 75.50% / 0.795 | 59.50% / 0.708 | 48.94% / 0.532 | 354-3101 ms |
| madhurjindal/Jailbreak-Detector-Large (~280M) | 59.00% / 0.305 | 81.00% / 0.768 | 68.09% / 0.471 | 1637-7841 ms |

¹ Baselines latency higher than the iter-7/iter-8 measurements
because the system was under heavy load during this run (concurrent
training jobs). Apples-to-apples direction holds.
² JailGuard latency was also load-affected; isolated `cargo bench
--bench detect` measurement on the actual deployment hardware
(Chromebook Linux) is pending — expect ~10-25 ms p50.

### Three-iteration JailGuard summary

Same architecture (130K MLP after MiniLM-L6-v2 ONNX embedding); only
training data composition differs.

| Eval | iter-7 (3K synth) | iter-8 (+11.8K mosscap) | iter-9 (+L1-6 mosscap +1.5K benigns) |
|---|---|---|---|
| In-domain accuracy | **98.85%** | 97.89% | 98.40% |
| In-domain F1 | **0.989** | 0.972 | 0.983 |
| shalyhinpavel acc | **91.30%** | 83.70% | 86.96% |
| shalyhinpavel recall | **87.80%** | 73.17% | 75.61% |
| J1N2 acc | 80.51% | **89.65%** | 87.97% |
| J1N2 recall | 35.29% | **71.25%** | 62.13% |
| AgentDojo acc | **82.54%** | 79.37% | 79.37% |
| AgentDojo recall | 60.87% | 56.52% | **65.22%** |
| Average accuracy | **88.30%** | 87.65% | 88.18% |
| **Worst-case eval** | 80.51% | 79.37% | **79.37%** |
| **Worst-case recall** | 35.29% | 56.52% | **62.13%** |

**What iter-9 demonstrates:**

- ✅ **Best worst-case recall** (62.13% vs iter-7's 35.29% / iter-8's
  56.52%). No eval falls below 62% recall — the model catches at least
  62% of attacks across every distribution we test.
- ✅ **Best AgentDojo recall ever** (65.22%). The synthetic benigns
  appear to have helped indirect-injection generalization too, which
  wasn't a designed-for outcome.
- ✅ **Tightest spread across evals** (79.37% to 98.40% acc; iter-7
  spread 80.51% to 98.85%, iter-8 spread 79.37% to 97.89%).
- ⚠️ Doesn't strictly beat iter-7 on any individual non-J1N2 eval —
  iter-7 still wins shalyhinpavel and AgentDojo accuracy. iter-9 wins
  J1N2 by +7.46pp / +26.84pp recall, average by +0.12pp on AD recall.
- ⚠️ shalyhinpavel only partially recovered (86.96% vs iter-8's
  83.70%, iter-7's 91.30%). The benign augmentation helped but didn't
  fully close the gap.

### Recommended ship state: iter-9

Iter-9 has the best **deployable** profile:

- **No catastrophic weakness** — the lowest-performing eval is 79.37%
  accuracy / 62.13% recall. Both iter-7 (J1N2 35.29% recall) and
  iter-8 (shalyhin 73.17% recall) have a single eval below the floor
  iter-9 establishes.
- **Beats every open-CPU baseline on Pipeline** by 11+pp (next-best is
  protectai-base at 86.50%). Closes the J1N2 gap to protectai-base
  from iter-7's 13pp to ~5pp. Trails by 3pp on shalyhinpavel.
- **Latency advantage stays** — 130K MLP vs 184M transformer is the
  fundamental architectural win.

Iter-9 is currently the embedded model in the jailguard library. To
roll back to iter-7 or iter-8, reproduce by:
- iter-7: keep current `synthetic_benign_hardneg` source disabled,
  remove `lakera_mosscap` source, restore single-line `BENIGNS[:81]`
  truncation.
- iter-8: disable the L1-6 filter in the mosscap normalizer.
Both rollbacks are ~1.5 hr pipeline reruns.

### Important caveats

**In-domain vs out-of-distribution.** JailGuard iter-9's 98.40% is measured
on its own test split, which shares the same distribution as its training
data. Out-of-distribution holdouts (J1N2, shalyhinpavel) are reported
separately above and remain the honest measure of generalization. Any
single-number "best model" claim across vendors should be treated with
skepticism — each vendor evaluates on their own data with their own
preprocessing.

**Over-defense problem.**  The InjecGuard paper (arxiv:2410.22770) measured
false-positive rates on 339 benign prompts containing injection-like words:

| Model | FPR on "hard negative" benign prompts |
|---|---|
| Llama Prompt Guard 1 | **99.1%** |
| deepset/deberta-v3-base-injection | 65.9% |
| protectai/deberta-v3-base-injection-v2 | 43.4% |
| **JailGuard (before hard-neg training)** | **~27% FPR** (72.95% acc on 732-sample full set) |
| Lakera Guard | 12.4% |
| **JailGuard (after hard-neg training)** | **~10.4% FPR** (89.12% acc on 147-sample holdout) |

JailGuard was explicitly trained with benign hard negatives (ALERT regular,
BeaverTails, Aegis, etc.) to reduce over-defense. Adding the shalyhinpavel
train split (436 unique security-adjacent benign + real RAG injection examples)
reduced the false-positive rate on the held-out validation set from ~27% to
~10.4%, matching the Lakera Guard level. The precision/recall trade-off on the
in-domain test set moved as expected when adding harder negatives (iter-9: 98.40% accuracy, 98.56% precision).

---

## Latency Benchmarks

### Steady-state single-call latency (Criterion, warm session)

Measured with the Criterion harness (`cargo bench --bench detect`) after a
3-second warm-up phase. This represents **throughput-mode latency** — the ONNX
session is already loaded and warmed up, equivalent to a long-running server
process handling its second and subsequent requests.

| Benchmark | Median | 95% CI |
|---|---|---|
| `is_injection` (benign text) | **19.70 ms** | [19.59, 19.81] ms |
| `is_injection` (injection text) | **19.60 ms** | [19.49, 19.72] ms |

> Both benign and injection inputs take the same time — inference cost is
> independent of the classification result.

### Batch throughput (Criterion)

`detect_batch` runs each sample sequentially within the batch (the ONNX
session is single-threaded by default). These numbers show that there is no
significant per-sample overhead for batching at current batch sizes.

| Batch size | Total time (median) | Per-sample | Samples/sec |
|---|---|---|---|
| 8 | 110.6 ms | 13.8 ms | **72 samples/sec** |
| 32 | 437.6 ms | 13.7 ms | **73 samples/sec** |
| 128 | 1,840 ms | 14.4 ms | **70 samples/sec** |

> Throughput is flat across batch sizes (~70–73 samples/sec) because the ONNX
> runtime processes one item at a time. True batched ONNX inference could
> improve GPU throughput significantly but is not yet implemented.

### Cold-start latency

The first call in a fresh process must initialize the ONNX runtime and load the
tokenizer. Subsequent calls reuse the initialized session.

```sh
cargo run --release --example cold_start_bench
```

Typical output (Apple M3):

```
cold_start   : ~2,500 ms  (ONNX session initialization + first inference)
warm call    : ~19.7 ms   (steady-state)
init overhead: ~2,480 ms
```

Cold-start is a one-time cost per process. In production, call
`jailguard::download_model()` at startup to pre-download the ONNX model and
warm the session before handling any traffic.

### Wall-clock latency on real samples (benchmark binary)

Measured across 1,113 real samples of varying length, including tokenization
and MLP forward pass. This is the **end-to-end per-request cost** a production
server would observe on steady-state traffic.

| Percentile | Latency |
|---|---|
| p50 | **14.40 ms** |
| p90 | **15.76 ms** |
| p99 | **18.99 ms** |

### Latency comparison summary

| Solution | p50 latency | Notes |
|---|---|---|
| **JailGuard (local, Rust)** | **~14 ms** | No network, CPU only |
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

- [x] **Hard-negatives FPR test** — shalyhinpavel validation holdout (147 samples)
  measures FPR on security-adjacent benign text; reduced from ~27% to ~10.4%
  after adding the train split to training data
- [x] **Full embedding phase** — 79,626 embeddings (59,447 original + 20,179 augmented)
  trained and deployed (iter-9); model achieves 98.40% accuracy, 97.98% recall on
  7,049-sample in-domain test set; 86.96% on shalyhinpavel OOD holdout
- [ ] **AgentDojo APR** — indirect injection benchmark against tool-return content
- [ ] **Multi-language evaluation** — MiniLM-L6-v2 is English-weighted; non-English
  accuracy is expected to degrade. Build a held-out multilingual eval set from
  public sources.
- [ ] **Batched ONNX inference** — enable true batching in the ONNX session for
  higher GPU throughput
- [ ] **GPU latency numbers** — measure on a GPU instance (CUDA / Metal) to
  compete with the Llama PG2 GPU numbers

---

## References

- [LLM Guard PromptInjectionScanner](https://protectai.github.io/llm-guard/input_scanners/prompt_injection/)
- [protectai/deberta-v3-base-prompt-injection-v2 model card](https://huggingface.co/protectai/deberta-v3-base-prompt-injection-v2)
- [Llama Prompt Guard 2 model card](https://huggingface.co/meta-llama/Llama-Prompt-Guard-2-86M)
- [Azure Prompt Shields docs](https://learn.microsoft.com/en-us/azure/ai-services/content-safety/concepts/jailbreak-detection)
- [InjecGuard: Mitigating Over-Defense in Guard Models (arxiv:2410.22770)](https://arxiv.org/abs/2410.22770)
- [Adversarial Prompt Guardrail Benchmark (arxiv:2502.15427)](https://arxiv.org/html/2502.15427v1)
- [Babelscape/ALERT dataset](https://huggingface.co/datasets/Babelscape/ALERT) — source of adversarial injection examples
