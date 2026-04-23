#!/usr/bin/env sh
# Run the JailGuard benchmark suite and emit a markdown report.
#
# Usage:
#   scripts/bench.sh                  # runs to stdout
#   scripts/bench.sh > bench.md       # captures report
#
# Portable to Linux (x86_64 and aarch64), macOS (Intel and Apple Silicon),
# and ARM Linux inside a Chromebook's Crostini container. POSIX /bin/sh,
# no bash-isms; all tools used (cargo, uname, awk, grep) are available on
# every target.
#
# What it runs:
#   1. cold_start_bench example — reports cold-start + warm-call latency.
#   2. cargo bench --bench detect — Criterion measures single-shot and
#      batch throughput.
#
# Output is a single markdown document with a machine-info block followed
# by the numbers. Paste it into the repo's README or a GitHub issue.

set -eu

cd "$(dirname "$0")/.."

# ---------------------------------------------------------------------------
# Machine metadata
# ---------------------------------------------------------------------------
os_name=$(uname -s)
arch=$(uname -m)
kernel=$(uname -r)
rustc_version=$(rustc --version 2>/dev/null || echo "rustc: not found")

cpu_model="unknown"
case "$os_name" in
  Linux)
    if [ -r /proc/cpuinfo ]; then
      cpu_model=$(awk -F': ' '/^model name/ {print $2; exit}' /proc/cpuinfo)
      if [ -z "$cpu_model" ]; then
        # ARM /proc/cpuinfo has no "model name"; fall back to Hardware or Processor.
        cpu_model=$(awk -F': ' '/^Hardware/ {print $2; exit}' /proc/cpuinfo)
        [ -z "$cpu_model" ] && cpu_model=$(awk -F': ' '/^Processor/ {print $2; exit}' /proc/cpuinfo)
        [ -z "$cpu_model" ] && cpu_model="ARM ($arch)"
      fi
    fi
    ;;
  Darwin)
    cpu_model=$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "Apple Silicon")
    ;;
  *)
    cpu_model="$os_name $arch"
    ;;
esac

cpu_count=$(
  getconf _NPROCESSORS_ONLN 2>/dev/null \
    || sysctl -n hw.ncpu 2>/dev/null \
    || echo "?"
)

# ---------------------------------------------------------------------------
# Report header
# ---------------------------------------------------------------------------
cat <<EOF
# JailGuard benchmark report

- **OS**: $os_name ($kernel)
- **Arch**: $arch
- **CPU**: $cpu_model ($cpu_count logical cores)
- **Toolchain**: $rustc_version
- **Date**: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

## Cold start

\`\`\`
EOF

# Build examples in release mode, then run. We redirect cargo's own build
# output to stderr (via >&2) so the stdout of this script stays clean.
cargo build --release --example cold_start_bench >&2
./target/release/examples/cold_start_bench

cat <<'EOF'
```

Cold start = process launch → first `detect()` return. Model file must
already be in `~/.cache/jailguard/` (or `$JAILGUARD_MODEL_DIR`); delete
that directory before running to include network-download latency.

## Steady-state (Criterion)

```
EOF

# Criterion prints a lot. Keep only the "time:" lines plus the benchmark
# names, which is what readers actually want to see.
cargo bench --bench detect -- --warm-up-time 1 --measurement-time 3 2>&1 \
  | grep -E '^(single_shot|batch_throughput)/|time:|thrpt:' \
  || true

cat <<'EOF'
```

Full Criterion reports (HTML, with distribution plots) land in
`target/criterion/`. Open `target/criterion/report/index.html` to explore.
EOF
