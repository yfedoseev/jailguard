// Convenience directive for `go generate ./...`. In a consumer module,
// run this once to fetch the native FFI library and print the env vars
// required to build with CGo.

//go:generate go run github.com/yfedoseev/jailguard/go/cmd/install@latest

package jailguard
