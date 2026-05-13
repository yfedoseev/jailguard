# Scripts

Two small developer utilities. Everything dataset- and benchmark-related
that used to live here is now part of the upstream training pipeline; this
directory only ships things that are useful inside the public crate.

| Script | Purpose |
|--------|---------|
| `bench.sh` | Portable POSIX wrapper that runs `cargo bench --bench detect` plus the `cold_start_bench` example and emits a single markdown report with machine metadata (CPU, arch, kernel, toolchain). Works on Linux x86_64/aarch64, macOS Intel/Apple Silicon, and Chromebook Crostini. |
| `setup-hooks.sh` | Installs the repo's git pre-commit hooks (clippy, fmt, taplo). |

## bench.sh

```sh
scripts/bench.sh                # report to stdout
scripts/bench.sh > bench.md     # capture report
```

Numbers from this script feed the latency/throughput tables in
[`BENCHMARKS.md`](../BENCHMARKS.md). For the accuracy / OOD / head-to-head
evaluations against ProtectAI / Deepset / Bedrock, see that same document
— those are produced by a separate training+evaluation pipeline that is
not redistributed with the crate.

## setup-hooks.sh

```sh
scripts/setup-hooks.sh
```

Installs `.git/hooks/pre-commit`, which runs `cargo fmt --check`,
`cargo clippy -- -D warnings`, and `taplo fmt --check` before each commit.
