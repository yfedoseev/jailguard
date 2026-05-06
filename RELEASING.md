# Releasing JailGuard

JailGuard ships four artifacts from a single tag:

1. **Rust crate** → [crates.io/crates/jailguard](https://crates.io/crates/jailguard)
2. **Python wheel** (`jailguard`) → [pypi.org/project/jailguard](https://pypi.org/project/jailguard/)
3. **Node.js package** (`@jailguard/jailguard`) → [npm](https://www.npmjs.com/package/@jailguard/jailguard)
4. **Go module** (`github.com/yfedoseev/jailguard/go`) → [pkg.go.dev](https://pkg.go.dev/github.com/yfedoseev/jailguard/go)

All four are produced by `.github/workflows/release.yml` from a single
`vX.Y.Z` tag push.

## Prerequisites

- **Repository secrets** configured in GitHub Settings → Secrets and variables → Actions:
  - `CARGO_REGISTRY_TOKEN` — crates.io API token
  - `NPM_TOKEN` — npm automation token (for non-OIDC fallback; provenance via OIDC is preferred)
- **PyPI trusted publisher** configured at https://pypi.org/manage/account/publishing/
  - Workflow: `release.yml`
  - Environment: leave blank
- **npm provenance** — handled automatically when `id-token: write` is set on the publish job (already configured)
- **Branch protection** on `main` — every CI workflow's `*-success` aggregate job must be required
- All bindings tested locally: `make ci-local && make test-py && make test-go && make test-js`

## Versioning

We use [SemVer](https://semver.org/). Pre-1.0.0 means MINOR bumps may break
the public API; PATCH is bug-fix-only.

The version must be identical across:

- `Cargo.toml` → `[package].version`
- `pyproject.toml` → `[project].version`
- `js/package.json` → `version`
- `go/` (no manifest — uses git tag `go/vX.Y.Z`)

The pre-flight job in `release.yml` enforces this. A mismatch fails the
release before any publish step runs.

## Step-by-step

```bash
# 1. Bump versions in all manifests + add CHANGELOG entry.
sed -i '' 's/^version = "0.1.0"/version = "0.1.1"/' Cargo.toml
sed -i '' 's/^version = "0.1.0"/version = "0.1.1"/' pyproject.toml
node -e "const p=require('./js/package.json'); p.version='0.1.1'; require('fs').writeFileSync('./js/package.json', JSON.stringify(p, null, 2))"
$EDITOR CHANGELOG.md   # add a "## [0.1.1] - 2026-MM-DD" section

# 2. Verify locally — same commands CI will run.
make ci-local
make test-py
make test-go
make test-js
make test-c-api

# 3. Commit + tag.
git add Cargo.toml pyproject.toml js/package.json CHANGELOG.md
git commit -m "release: v0.1.1"
git tag -a v0.1.1 -m "v0.1.1"

# 4. Dry-run the release pipeline first (highly recommended).
gh workflow run release.yml --ref main -f dry-run=true
# Watch:
gh run watch
# This runs the full build matrix but skips publish — confirms artifacts
# build cleanly on every target before exposing them to the world.

# 5. If the dry-run is clean, push tag to trigger the real release.
git push origin main
git push origin v0.1.1

# 6. Watch the release workflow.
gh run watch

# 7. Post-release verification.
cargo install jailguard --version 0.1.1
pip install jailguard==0.1.1
npm view @jailguard/jailguard@0.1.1 version
GOPROXY=https://proxy.golang.org go install github.com/yfedoseev/jailguard/go@v0.1.1
```

## What the release pipeline does

1. **Pre-flight**: validates that all four manifests carry the same version
   and a CHANGELOG entry exists.
2. **Native build matrix** (5 targets): `libjailguard.{so,dylib,dll}` +
   napi addon + `include/jailguard.h` for x86_64/aarch64 Linux, x86_64/aarch64
   macOS, x86_64 Windows. Each target gets a SHA-256 manifest.
3. **Python wheel matrix**: 4 Linux variants (manylinux+musllinux ×
   x86_64+aarch64) + 2 macOS + 2 Windows + sdist = 9 wheels.
4. **Multi-registry publish** (only on tag push, not dry-run):
   - `cargo publish` → crates.io
   - `maturin upload` (via OIDC trusted publisher) → PyPI
   - `npm publish --provenance` → npmjs.com
   - `git tag go/vX.Y.Z` → triggers proxy.golang.org snapshot
5. **GitHub Release**: every artifact attached, release notes auto-generated.

## Troubleshooting

- **"Cargo.toml version does not match tag"**: pre-flight is strict. Bump
  the manifests to match the tag, or rename the tag.
- **Wheel build fails on aarch64 Linux**: maturin's docker image needs
  internet access. Check the runner is using the configured PyO3/maturin-action.
- **PyPI publish "trusted publisher not configured"**: re-check the
  publishing config at pypi.org includes the exact workflow filename
  (`release.yml`) and repo (`yfedoseev/jailguard`).
- **npm provenance fails**: verify `id-token: write` is on the publish job
  AND the repo is under a personal account or org with that permission.
- **Go module not appearing on pkg.go.dev**: proxy.golang.org snapshots
  lazily. Trigger by `curl https://proxy.golang.org/github.com/yfedoseev/jailguard/go/@v/v0.1.1.info`.

## Yanking a release

If you ship a broken artifact:

```bash
cargo yank --version 0.1.1
pip --user uninstall jailguard
# (PyPI doesn't support yanking; release a 0.1.2 quickly)
npm deprecate @jailguard/jailguard@0.1.1 "use 0.1.2 instead"
# Go modules can't be yanked; release a fixed 0.1.2 and update go.sum
```
