//go:build cgo && jailguard_dev

// Dev-mode CGo linker flags. Used when building this package inside the
// monorepo, after running `cargo build --release` from the repo root.
//
// Activate with: go build -tags jailguard_dev ./...
//
// Non-dev users get CGO_CFLAGS / CGO_LDFLAGS from the installer's printed
// env vars instead, so this file deliberately has no effect outside the
// jailguard_dev build tag.

package jailguard

/*
#cgo CFLAGS: -I${SRCDIR}/../include
#cgo linux,amd64    LDFLAGS: -L${SRCDIR}/../target/release -ljailguard -lm -lpthread -ldl
#cgo linux,arm64    LDFLAGS: -L${SRCDIR}/../target/aarch64-unknown-linux-gnu/release -ljailguard -lm -lpthread -ldl
#cgo darwin,amd64   LDFLAGS: -L${SRCDIR}/../target/release -ljailguard -framework CoreFoundation -framework Security
#cgo darwin,arm64   LDFLAGS: -L${SRCDIR}/../target/release -ljailguard -framework CoreFoundation -framework Security
#cgo windows,amd64  LDFLAGS: -L${SRCDIR}/../target/release -ljailguard -lws2_32 -luserenv
*/
import "C"
