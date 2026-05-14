# `lib/` — populated on demand

This directory is **intentionally empty in source control**. The native
FFI libraries (~5 MB each) are downloaded by the installer to the user
cache directory on first use, not committed alongside the Go sources.

## Why

Committing the binaries would bloat the Go module download by ~25 MB
(five platforms × one library each, doubled if shared variants are also
shipped). `go get` would pay that cost for every consumer regardless of
which platform they actually need.

## How libraries get here

Running

```sh
go run github.com/yfedoseev/jailguard/go/cmd/install@latest
```

downloads `jailguard-go-ffi-<platform>.tar.gz` from the
[GitHub Releases page](https://github.com/yfedoseev/jailguard/releases),
verifies its SHA-256, and extracts to `~/.cache/jailguard/v<version>/`
(or the platform equivalent — see `os.UserCacheDir()`).

After extraction the layout under the cache directory is:

```
lib/
  linux_amd64/   libjailguard.a    (or libjailguard.so for --shared)
  linux_arm64/   libjailguard.a
  darwin_amd64/  libjailguard.a    (or libjailguard.dylib)
  darwin_arm64/  libjailguard.a
  windows_amd64/ libjailguard.a    (or jailguard.dll)
include/
  jailguard.h
```

The installer prints the exact `CGO_*` (or `JAILGUARD_LIB_PATH`) env
vars to export for the chosen platform.

## Inside the monorepo

Building this package from inside `~/projects/jailguard/` does not need
the installer. Use the `jailguard_dev` build tag — `cgo_dev.go` resolves
the library path to the workspace `target/release/` directory:

```sh
cargo build --release        # repo root
cd go && go build -tags jailguard_dev ./...
```
