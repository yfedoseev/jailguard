# Open-Source Readiness Gaps

Audit date: 2026-04-22. Last updated: 2026-04-22 (Phase A landed).
Target: public crates.io release + credible "this is great" story for
reviewers.

## TL;DR

Packaging is ready. Benchmark story is **scaffolded, not yet executed**.
The internal 99.07% accuracy number is trained *and* inferred with real
ONNX (verified in `src/embedded.rs:30,98,109,144` — no hash-based
fallback), so the figure is methodologically sound. It just has not been
reproduced on any independent public benchmark and has no head-to-head
comparison with existing detectors.

Phase A (all code-only work) is done — benches compile, CI covers ARM,
external-eval harness is in place. Phase B (running the benches on
Mac/Chromebook and executing PINT/AgentDojo/baseline comparisons) is
next and needs physical hardware runs.

## What's solid (do not re-do)

- `LICENSE` (MIT OR Apache-2.0 dual), `CHANGELOG.md`, `CONTRIBUTING.md`,
  `CODE_OF_CONDUCT.md`, `SECURITY.md`.
- `Cargo.toml` metadata: description, repository, keywords, categories,
  license, readme, documentation, `rust-version = "1.85"`, correct `include`
  list that ships only the embedded-detector surface.
- CI on Linux/macOS/Windows x86_64: fmt, taplo, typos, clippy (strict on
  default features, informational on `full`), MSRV, cargo-shear,
  cargo-hack powerset, docs with `-D warnings`, llvm-cov, semver-checks,
  cargo-deny, cargo-audit.
- Internal accuracy: 99.07% / F1 0.9908 on 20K held-out split of a 200K
  composite mix; README already disclaims this as non-independent.
- Runtime fidelity: embedded inference uses the same ONNX model
  (all-MiniLM-L6-v2) as training — not a hashed surrogate.

## Gaps ranked by credibility impact

### 1. No head-to-head comparison (blocks "SOTA" claims)

`docs/paper/PAPER_PLAN.md` lists these as Required and unchecked:

- [ ] vs. PromptGuard (Meta)
- [ ] vs. Rebuff (rule-based)
- [ ] vs. protectai/deberta-v3-base-prompt-injection
- [ ] Ablation: embedding model, architecture depth, dataset size

A reviewer's first question is "how does this compare to X?" and we cannot
answer. This is the single highest-impact gap.

### 2. No independent public benchmark executed

README mentions PINT (Lakera) and AgentDojo as "planned." No loaders, no
results. The 99% number will be read as overfitting to our own 200K mix
until PINT/AgentDojo numbers ship.

### 3. No performance benchmark harness — **RESOLVED (harness only)**

Harness landed 2026-04-22. Numbers from running it are still pending.

- `benches/detect.rs` — Criterion bench for single-shot latency
  (`is_injection`, `detect`, `score`, long-input) + batch throughput at
  n = 1, 8, 32, 128. Compiles with `-D warnings`. Run with
  `cargo bench --bench detect`.
- `examples/cold_start_bench.rs` — process-startup cost (ONNX session
  init + first inference), which Criterion can't measure because of
  `once_cell`. Run with `cargo run --release --example cold_start_bench`.
- `scripts/bench.sh` — POSIX portable wrapper that captures
  OS/arch/CPU/kernel/rustc and emits a single markdown report. Tested
  with `sh -n`; same script on Linux x86_64, Linux aarch64, macOS Intel
  + Apple Silicon, Chromebook Crostini.
- Peak RSS / binary-size-over-time still not tracked.

### 4. No ARM coverage in CI — **RESOLVED**

`.github/workflows/ci.yml` test matrix extended 2026-04-22 to:
`ubuntu-latest`, `ubuntu-24.04-arm`, `macos-latest` (Apple Silicon as of
the 2025+ runner alias), `macos-13` (Intel), `windows-latest`. Covers
Linux x86_64/aarch64, macOS Intel/Apple Silicon, Windows x86_64.

### 5. Indirect prompt injection is not a dedicated output class

Already disclosed in `CHANGELOG.md`. Flagging here so it doesn't get lost:
the current AttackType enum (`src/detection/result.rs`) does not have a
dedicated "IndirectInjection" variant; such attacks fall under
ContextManipulation or JailbreakPattern depending on content. Fine for
0.1.x, worth a 0.2 design discussion.

### 6. No reproducibility container / pinned dataset hashes

- No `Dockerfile` or `devcontainer.json`.
- `scripts/download_and_combine_datasets.py` is not hash-pinned to a
  specific HF dataset commit — results are not bitwise reproducible by a
  third party.
- Not a 0.1.0 blocker, but flagged for paper submission.

## Action plan

### Phase A — code-only (no benchmark runs needed) — **DONE 2026-04-22**

Everything in this phase landed in one pass. Validation against the
default surface: `cargo check --all-targets` green, `cargo clippy
--all-targets -- -D warnings` green, `cargo shear` no unused deps,
`cargo publish --dry-run` succeeds (only the expected "not in include"
warnings, matching the existing pattern).

- [x] **A1** `benches/detect.rs` with Criterion — single-shot latency
  for `is_injection` / `detect` / `score`, batch throughput at
  n = 1/8/32/128. Wired into `Cargo.toml` with `[[bench]] name = "detect"
  harness = false`. Cold-start split out into
  `examples/cold_start_bench.rs` (autodiscovered, not in publish
  include) because `once_cell` makes Criterion the wrong tool for
  startup cost. `criterion = "0.5"` added to `[dev-dependencies]`.
- [x] **A2** `scripts/bench.sh` — POSIX `/bin/sh`, no bash-isms. Captures
  OS, arch, kernel, CPU model (handles Linux x86_64 `/proc/cpuinfo`,
  ARM `/proc/cpuinfo` fallbacks, macOS `sysctl`), logical core count,
  rustc version. Runs cold-start example + `cargo bench` and emits a
  single markdown report. Executable, passes `sh -n`.
- [x] **A3** `.github/workflows/ci.yml` test matrix extended to
  `ubuntu-latest`, `ubuntu-24.04-arm`, `macos-latest`, `macos-13`,
  `windows-latest`. Comment in the file documents the arch of each
  runner.
- [x] **A4** `evaluation/external/` scaffolded with:
  - `README.md` — PINT + AgentDojo sources, licensing notes, baseline
    detectors (protectai/deberta-v3-base, Meta PromptGuard, Rebuff),
    reproduction steps, canonical prediction JSONL schema.
  - `pint_runner.rs` and `agentdojo_runner.rs` — load cases, run
    JailGuard, emit prediction JSONL in the documented shape. Schema:
    `{"id","label","pred","score","latency_ms","model"}`.
  - `comparison_runner.rs` — aggregates every `<dataset>_<model>.jsonl`
    under `data/external/` and prints a markdown table with accuracy,
    F1, **FPR@0.95 recall**, median latency.
- [x] **A5** `README.md` — new "Benchmarks" section between "Latency"
  and "Attack categories", linking to `benches/`, `examples/`,
  `scripts/bench.sh`, and `evaluation/external/`. Honestly labels
  external comparison results as pending and points at this file.

### Phase B — needs a benchmark run (owner executes)

Runs to do yourself. Not blocking 0.1.0 release, but needed for the
"this is great" story.

- [ ] **B1** Run `scripts/bench.sh` on Linux x86_64 (current machine),
  macOS (your Mac — `macos-latest` in CI is already Apple Silicon, so
  local Intel numbers are the incremental signal), Chromebook (Crostini
  / ARM Linux). Paste the three markdown tables into README. This is
  the "headline" artifact.
- [ ] **B2** Download PINT and AgentDojo into `data/external/`, wire
  `pint_runner.rs` + `agentdojo_runner.rs` as `[[bin]]` entries with
  `required-features = ["full"]`, run them, paste results into README
  and `CHANGELOG.md`.
- [ ] **B3** Run head-to-head against at least one baseline
  (protectai/deberta-v3-base is the lowest-friction — HF hub download,
  one-liner Python script that writes the same JSONL schema). Run
  `comparison_runner` to emit the final table. Add to README.

### Phase C — paper-grade rigor (later)

- [ ] **C1** Pin HF dataset commits in
  `scripts/download_and_combine_datasets.py`.
- [ ] **C2** Add a `Dockerfile` for bit-reproducible training.
- [ ] **C3** Indirect-injection dedicated AttackType + training data.
- [ ] **C4** Ablations from `docs/paper/PAPER_PLAN.md`.

## Not-doing (explicitly out of scope for 0.1.x)

- GPU inference path. CPU-only is a stated design constraint.
- Multi-language bindings (Python, JS). Rust crate first.
- Streaming / token-level detection. Current API is whole-prompt.
