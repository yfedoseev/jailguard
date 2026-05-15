#!/bin/bash
# Extracts release notes for a given version from CHANGELOG.md
# Usage: extract-release-notes.sh <version>
# Outputs:
#   release-title.txt  — "v0.1.0 | Initial release"
#   release-notes.md   — Full body (changelog section + installation footer)

set -euo pipefail

VERSION="$1"
CHANGELOG="CHANGELOG.md"

if [ ! -f "$CHANGELOG" ]; then
  echo "Error: $CHANGELOG not found" >&2
  exit 1
fi

# Extract subtitle from "> ..." blockquote line after version header (if any)
SUBTITLE=$(awk "/^## \[${VERSION}\]/{found=1; next} found && /^>/{gsub(/^> */, \"\"); print; exit}" "$CHANGELOG")

# Build title
if [ -n "$SUBTITLE" ]; then
  echo "v${VERSION} | ${SUBTITLE}" > release-title.txt
else
  echo "v${VERSION}" > release-title.txt
fi

# Extract body: everything between this version's ## and the next ##
awk "/^## \[${VERSION}\]/{flag=1; next} /^## \[/{flag=0} flag" "$CHANGELOG" \
  | sed '/^> /d' \
  | sed '1{/^$/d}' > changelog-section.md

if [ ! -s changelog-section.md ]; then
  echo "Warning: No changelog content found for version ${VERSION}" >&2
  echo "Make sure CHANGELOG.md has a '## [${VERSION}] - YYYY-MM-DD' section." >&2
fi

# Build release body = changelog section + installation footer
cat changelog-section.md > release-notes.md
cat >> release-notes.md << 'FOOTER'

---

### Installation

**Rust (crates.io)**
```bash
cargo add jailguard
```

**Python (PyPI)**
```bash
pip install jailguard
```

**JavaScript / TypeScript (npm)**
```bash
npm install @yfedoseev/jailguard
```

**Go**
```bash
go get github.com/yfedoseev/jailguard/go
go run github.com/yfedoseev/jailguard/go/cmd/install@latest
```

**Elixir (Hex)**
```elixir
def deps do
  [{:jailguard, "~> 0.1.2"}]
end
```

### Platform Support

| Platform | Architecture | Bindings |
|----------|--------------|----------|
| Linux    | x86_64       | Rust, Python wheel, npm prebuild, Go FFI, Hex NIF |
| Linux    | aarch64      | Rust, Python wheel, npm prebuild, Go FFI, Hex NIF |
| macOS    | x86_64       | Rust, Python wheel, npm prebuild, Go FFI, Hex NIF |
| macOS    | aarch64      | Rust, Python wheel, npm prebuild, Go FFI, Hex NIF |
| Windows  | x86_64       | Rust, Python wheel, npm prebuild, Go FFI, Hex NIF |

### Documentation

- [README](https://github.com/yfedoseev/jailguard/blob/main/README.md)
- [API reference](https://docs.rs/jailguard)
- [Benchmarks](https://github.com/yfedoseev/jailguard/blob/main/BENCHMARKS.md)
- [Full changelog](https://github.com/yfedoseev/jailguard/blob/main/CHANGELOG.md)
FOOTER

# Cleanup
rm -f changelog-section.md

echo "Generated release-title.txt and release-notes.md for v${VERSION}"
