# Makefile for JailGuard
#
# Common developer tasks for the Rust crate and language bindings.
# Run `make help` for a categorised list.

.PHONY: help \
        check check-all check-rust check-py check-js check-go \
        build build-release build-py build-go build-js build-wasm \
        test test-rust test-py test-js test-go test-doc \
        bench bench-build score-test \
        lint lint-rust lint-rust-full lint-py lint-py-fix lint-js \
        fmt fmt-rust fmt-py fmt-toml fmt-check \
        coverage docs ci-local clean dev install

# ---------------------------------------------------------------------------
# Default
# ---------------------------------------------------------------------------

help:
	@echo "JailGuard — developer Makefile"
	@echo ""
	@echo "Quick:"
	@echo "  make ci-local         - Run everything CI runs (slow but thorough)"
	@echo "  make check            - Fast pre-commit gate (fmt + clippy + test)"
	@echo "  make score-test       - 7-prompt smoke test against the embedded model"
	@echo ""
	@echo "Build:"
	@echo "  make build            - cargo build (default features)"
	@echo "  make build-release    - cargo build --release --all-targets"
	@echo "  make build-py         - maturin build --release (Python wheel)"
	@echo "  make build-go         - cgo build of the Go bindings (Phase 3)"
	@echo "  make build-js         - napi-rs build of the Node bindings (Phase 4)"
	@echo "  make build-wasm       - wasm-pack build for browsers (Phase 4)"
	@echo ""
	@echo "Test:"
	@echo "  make test             - Rust unit + integration + doc tests"
	@echo "  make test-rust        - Rust tests only (default features)"
	@echo "  make test-py          - Python pytest suite (Phase 2)"
	@echo "  make test-js          - Node + WASM tests (Phase 4)"
	@echo "  make test-go          - Go tests (Phase 3)"
	@echo "  make test-doc         - Rust doc tests"
	@echo ""
	@echo "Quality:"
	@echo "  make lint             - cargo clippy (default features, -D warnings)"
	@echo "  make lint-rust-full   - cargo clippy --features full"
	@echo "  make lint-py          - ruff check Python sources (Phase 2)"
	@echo "  make lint-js          - biome check JS sources (Phase 4)"
	@echo "  make fmt              - cargo fmt + ruff format + taplo fmt"
	@echo "  make fmt-check        - all formatters in --check mode"
	@echo ""
	@echo "Bench / Coverage / Docs:"
	@echo "  make bench            - cargo bench (criterion)"
	@echo "  make bench-build      - cargo bench --no-run (compile-only)"
	@echo "  make coverage         - cargo llvm-cov with HTML report"
	@echo "  make docs             - cargo doc --no-deps --features full"
	@echo ""
	@echo "Aggregate:"
	@echo "  make check-rust       - fmt-check + lint + test-rust + test-doc"
	@echo "  make check-py         - fmt-py-check + lint-py + test-py"
	@echo "  make check-all        - check-rust + check-py + check-js + check-go"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean            - remove build artifacts (target/, dist/, *.so, *.pyd)"

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

build:
	cargo build --locked

build-release:
	cargo build --release --locked --all-targets

build-py:
	maturin build --release

build-py-dev:
	maturin develop --features python

build-go:
	@echo "Phase 3 placeholder — go/ directory not yet created"
	@exit 1

# ---------------------------------------------------------------------------
# C ABI — header regen + smoke test
# ---------------------------------------------------------------------------

# Regenerate include/jailguard.h from src/c_api.rs.
# Requires `cargo install cbindgen`.
regen-c-header:
	@command -v cbindgen >/dev/null || (echo "Install cbindgen: cargo install cbindgen"; exit 1)
	cbindgen --crate jailguard --config cbindgen.toml --output include/jailguard.h
	@echo "Wrote include/jailguard.h ($$(wc -l < include/jailguard.h) lines)"

# Compile a tiny C program against the cdylib to verify the ABI is sane.
test-c-api: build-release
	@cc tests/c/smoke.c -I include -L target/release -ljailguard \
	    -o target/release/c_api_smoke
	@echo "Running C API smoke test..."
	@DYLD_LIBRARY_PATH=target/release LD_LIBRARY_PATH=target/release \
	    target/release/c_api_smoke
	@echo "✓ C API smoke test passed"

build-js:
	@echo "Phase 4 placeholder — js/ directory not yet created"
	@exit 1

build-wasm:
	@echo "Phase 4 placeholder — wasm-pack not yet wired"
	@exit 1

# ---------------------------------------------------------------------------
# Test
# ---------------------------------------------------------------------------

test: test-rust test-doc

test-rust:
	cargo test --release --locked

test-doc:
	cargo test --release --locked --doc

test-py:
	pytest python/tests/ -v --tb=short

test-js:
	@echo "Phase 4 placeholder — JS test suite not yet wired"
	@exit 1

test-go:
	@echo "Phase 3 placeholder — Go tests not yet wired"
	@exit 1

# ---------------------------------------------------------------------------
# Bench / smoke
# ---------------------------------------------------------------------------

bench:
	cargo bench --features full

bench-build:
	cargo bench --no-run --features full

score-test:
	cargo run --release --example score_test --features full

# ---------------------------------------------------------------------------
# Lint / fmt
# ---------------------------------------------------------------------------

lint: lint-rust

lint-rust:
	cargo clippy --release --all-targets --locked -- -D warnings

# Informational only — the `full` research surface still has known clippy
# warnings (see CI's `clippy (all features, informational)` job). Don't
# promote to error.
lint-rust-full:
	cargo clippy --release --all-targets --locked --features full

lint-py:
	@command -v ruff >/dev/null || (echo "Install ruff: pip install ruff"; exit 1)
	ruff check .

lint-py-fix:
	ruff check --fix .

lint-js:
	@echo "Phase 4 placeholder — biome not yet wired"
	@exit 1

fmt: fmt-rust fmt-py fmt-toml

fmt-rust:
	cargo fmt --all

fmt-py:
	@command -v ruff >/dev/null && ruff format . || echo "(ruff not installed — skipping)"

fmt-toml:
	@command -v taplo >/dev/null && taplo fmt || echo "(taplo not installed — skipping)"

fmt-check:
	cargo fmt --all --check
	@command -v ruff >/dev/null && ruff format --check . || echo "(ruff not installed — skipping)"
	@command -v taplo >/dev/null && taplo fmt --check || echo "(taplo not installed — skipping)"

# ---------------------------------------------------------------------------
# Coverage / docs
# ---------------------------------------------------------------------------

coverage:
	cargo llvm-cov --html --features full
	@echo "Coverage report: target/llvm-cov/html/index.html"

docs:
	cargo doc --no-deps --features full

# ---------------------------------------------------------------------------
# Aggregate
# ---------------------------------------------------------------------------

check: fmt-check lint test-rust

check-rust: fmt-check lint test-rust test-doc

check-py:
	@command -v ruff >/dev/null || (echo "Install ruff: pip install ruff"; exit 1)
	ruff format --check .
	ruff check .

check-js:
	@echo "Phase 4 placeholder"
	@exit 1

check-go:
	@echo "Phase 3 placeholder"
	@exit 1

check-all: check-rust
	@echo ""
	@echo "Run check-py / check-go / check-js individually as bindings come online"

# Mirror what GitHub Actions does, locally. Slow.
# `lint-rust-full` runs but doesn't fail the run — same as CI's
# `clippy (all features, informational)` job.
ci-local: fmt-check lint test-rust test-doc bench-build docs
	@echo ""
	@echo "Running informational full-feature clippy..."
	-@$(MAKE) lint-rust-full
	@echo ""
	@echo "✓ CI-equivalent checks all passed"

# ---------------------------------------------------------------------------
# Misc
# ---------------------------------------------------------------------------

dev:
	maturin develop --features python

install:
	maturin develop --release --features python

clean:
	cargo clean
	rm -rf dist/ wheels/ target/ build/
	rm -rf python/jailguard/*.so python/jailguard/*.pyd python/jailguard/__pycache__
	rm -rf tests/__pycache__ .pytest_cache htmlcov/ .coverage
	rm -rf js/node_modules js/dist js/build js/lib
	rm -rf go/*.so go/*.dll
