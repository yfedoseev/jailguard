# Multilingual dataset assets — preserved for future use

The multilingual dataset infrastructure built on 2026-05-02 is **preserved but not used** by the current production model. Production reverted to English-only with `all-MiniLM-L6-v2` because `paraphrase-multilingual-MiniLM-L12-v2` produced embeddings that degraded English injection detection from 99.34% to 72%.

This file links the multilingual assets so a future multilingual effort can pick up where this one left off, without re-discovering the sources or re-generating the synthetic injection data.

## Where the data lives

### In this repo (`~/Projects/jailguard`)

| Path | Size | Contents |
|------|------|----------|
| `scripts/download_and_combine_datasets.py` | 65 KB, ~700 lines | Downloads + normalises 20 HF datasets across 50 languages. Includes `cap_per_language_imbalance(max_ratio=2)` and `filter_language_minority_only()` to prevent language-as-proxy shortcuts. |
| `data/synthetic/` | ~5,800 samples | Synthetic injection prompts in 17 languages (300–420/lang): AR, BN, DE, ES, FI, FR, HI, IT, JV, KO, RU, SR, SW, TH, TL, VI, ZH |
| `data/combined_v4.json` | 120 MB | 200k rows, 50/50 balanced, ~50 languages — output of the script above |
| `data/combined_v4_embeddings.json` | 1.0 GB | L12 embeddings for `combined_v4.json` |
| `data/combined_v4_embeddings_en.json` | 784 MB | English-only filter of the above (135,717 samples) |
| `models/neural_binary_v4_adam.json` | 1.4 MB | MLP trained on 200k multilingual L12 — 89.96% in-domain, 66.63% English OOD |
| `src/embedded.rs.dual-api.bak` | 12 KB | Backup of the dual-model `detect()` / `detect_multilingual()` API — drop-in if a working multilingual model becomes available |
| `src/lib.rs.dual-api.bak` | — | Matching dual re-export |

### In sister repo (`~/projects/jailguard_dataset`)

| Path | Notes |
|------|-------|
| `pipeline/pipeline.rs` | Rust 3-phase pipeline (download → normalise → train). Ships the `SOURCES` const with 17 sources. Source of truth for the original v3 English dataset. |
| `data/pipeline_dataset.json` | 79,626 samples, 45% injection — the dataset that produced v3's 99.34% pipeline OOD with L6. |
| `data/pipeline_embeddings.json` | L6 embeddings of the above (Apr 30). |
| `data/external/shalyhinpavel_eval.json` | 147-sample hard-negative holdout. |
| `data/external/j1n2.json` | 5,000-sample OOD benchmark. |

## HuggingFace sources used by the multilingual pipeline

Pulled by `scripts/download_and_combine_datasets.py`:

```
Anthropic/hh-rlhf                          # benign English assistant data
CohereForAI/aya_dataset                    # multilingual benign (70+ languages)
DAMO-NLP-SG/MultiJail                      # multilingual jailbreaks
DuoGuard/duoguard-seed-data                # prompt-injection seeds
JailbreakBench/JBB-Behaviors               # behavioural jailbreaks
JailbreakV-28K/JailBreakV-28k              # 28k jailbreak prompts
Lemhf14/EasyJailbreak_Datasets             # jailbreak attempts
OpenAssistant/oasst1                       # benign multilingual conversations
PKU-Alignment/BeaverTails                  # benign harmful queries (NOT injection)
TrustAIRLab/in-the-wild-jailbreak-prompts  # ⚠ has 247 mislabels, see below
databricks/databricks-dolly-15k            # benign English instructions
deepset/prompt-injections                  # canonical injection set
google-research-datasets/tydiqa            # multilingual QA (benign)
jackhhao/jailbreak-classification          # jailbreak labels
lmsys/toxic-chat                           # real LMSYS jailbreaks
reshabhs/SPML_Chatbot_Prompt_Injection     # SPML prompts (97% injection)
rubend18/ChatGPT-Jailbreak-Prompts         # ChatGPT-targeted jailbreaks
tatsu-lab/alpaca                           # benign English instructions
walledai/AyaRedTeaming                     # ⚠ harmful but NOT injection (relabel benign)
yanismiraoui/prompt_injections             # ⚠ broken language tags ('unknown'), drop
```

## Known dataset issues to watch for

These were uncovered the hard way on 2026-05-02:

1. **TrustAIRLab `in-the-wild`**: 247 samples containing the literal phrase "Please ignore all previous instructions" are labeled benign. They are real injection. Re-label or drop before training.
2. **AyaRedTeaming**: 7,419 samples are harmful direct queries (e.g. "how do I make a bomb"), not prompt injection. Treat as benign or drop. Already relabeled in `combined_v4.json`.
3. **yanismiraoui_multilingual**: Language codes in the dataset don't round-trip through `normalize_lang()` so all 3,013 samples land in `language='unknown'` and are 100% injection. Either fix the language mapping or drop the source. Already dropped in latest script.
4. **Per-language imbalance**: Several small languages (JV 96%, IT 88%, VI/SW/ZH/KO/TH 67–75%) are heavily injection-skewed. The model learns "embedding region for these languages" → injection, falsely flagging English text that lands nearby. Mitigation: `cap_per_language_imbalance(max_ratio=2)` + `filter_language_minority_only(min_injection_rate=0.1)` already in script.
5. **0%-injection languages (~40)**: JA, TE, ID, PT, TR, NL, SV, FA, PL, etc. only appear as benign — model learns those language regions are always safe. Fix: generate synthetic injection per language, or drop the source.

## Why the multilingual model was shelved

`paraphrase-multilingual-MiniLM-L12-v2` is trained on cross-lingual paraphrase pairs. Its embedding geometry is optimised for "two texts mean the same thing across languages" — not for the fine-grained instruction-following intent that injection detection needs.

`all-MiniLM-L6-v2` is trained on diverse semantic similarity tasks (NLI, STS, QA pairs). Its embeddings preserve the structural cues injection detectors rely on. Same MLP architecture trained on the same data drops from 99.34% to 72.01% OOD pipeline accuracy when the embedder switches L6 → L12.

The honest path to multilingual without sacrificing English is **two embedders + two MLPs + a language router**, not one shared multilingual embedder. That's a real project (560 MB ONNX downloads, language detection, dual model lifecycle) and was deferred. Backup of the dual-model API is in `src/embedded.rs.dual-api.bak`.

## Reproducing the v4 multilingual training

```bash
# 1. Regenerate dataset with current fixes (yanismiraoui dropped, 2:1 cap, filter)
.venv/bin/python3 scripts/download_and_combine_datasets.py \
    --output data/combined_v5.json --skip-fuzzy

# 2. Embed with L12 (will take ~65 min on the M-series Mac, thermal-throttled)
cargo run --example onnx_embedding_generation --features full --release -- \
    --input data/combined_v5.json \
    --output data/combined_v5_embeddings.json \
    --model-dir ~/.cache/jailguard/ \
    --batch-size 64

# 3. Train (Adam + injection_weight=2.5 — the train_neural_binary example
#    was patched to default to Adam on 2026-05-02; v4 SGD weights collapsed)
cargo run --example train_neural_binary --features full --release -- \
    --data data/combined_v5_embeddings.json \
    --output models/neural_binary_multilingual_v5.json
```

Expect ~75 min training. Validate with both score test (any standalone bare-directive prompts) AND the OOD benchmark in `~/projects/jailguard_dataset` before declaring it production.
