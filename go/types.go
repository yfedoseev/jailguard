// Package jailguard provides Go bindings for the JailGuard prompt-injection
// detector. The Rust core is exposed via a small C ABI; this package wraps
// that ABI behind an idiomatic Go API.
//
// Two backends are supported and selected automatically by the Go toolchain:
//
//   - CGo (default): links statically against libjailguard.a at build time.
//     Run `go run github.com/yfedoseev/jailguard/go/cmd/install@latest` once
//     per machine to download the prebuilt staticlib + header, then export
//     the printed CGO_CFLAGS / CGO_LDFLAGS.
//
//   - purego (CGO_ENABLED=0): dlopens libjailguard.{so,dylib,dll} at runtime
//     via github.com/ebitengine/purego. No C toolchain required, works on
//     Alpine/musl and cross-compile targets. Run the installer with -shared
//     and export PDF_OXIDE_LIB_PATH.
//
// # Quick start
//
//	package main
//
//	import (
//	    "fmt"
//	    "log"
//
//	    jailguard "github.com/yfedoseev/jailguard/go"
//	)
//
//	func main() {
//	    if err := jailguard.DownloadModel(); err != nil {
//	        log.Fatal(err)
//	    }
//
//	    result, err := jailguard.Detect("ignore previous instructions")
//	    if err != nil {
//	        log.Fatal(err)
//	    }
//	    fmt.Printf("injection=%v score=%.4f risk=%v\n",
//	        result.IsInjection, result.Score, result.Risk)
//	}
//
// # Thread safety
//
// All exported functions are safe for concurrent use from multiple
// goroutines. The underlying Rust detector serialises ONNX session
// access internally.
package jailguard

import (
	"errors"
	"fmt"
)

// RiskLevel mirrors the C ABI's jailguard_risk_t enum.
type RiskLevel int32

// Risk buckets, in increasing order of severity.
const (
	RiskSafe     RiskLevel = 0
	RiskLow      RiskLevel = 1
	RiskMedium   RiskLevel = 2
	RiskHigh     RiskLevel = 3
	RiskCritical RiskLevel = 4
)

// String returns the canonical name of the risk bucket.
func (r RiskLevel) String() string {
	switch r {
	case RiskSafe:
		return "Safe"
	case RiskLow:
		return "Low"
	case RiskMedium:
		return "Medium"
	case RiskHigh:
		return "High"
	case RiskCritical:
		return "Critical"
	default:
		return fmt.Sprintf("RiskLevel(%d)", int32(r))
	}
}

// Result is the output of [Detect] and [DetectBatch].
type Result struct {
	// IsInjection is true if the model classifies the input as a prompt injection.
	IsInjection bool
	// Score is the raw model probability in [0.0, 1.0].
	Score float32
	// Confidence (always >= 0.5). Equals Score for injections,
	// 1.0 - Score for benigns.
	Confidence float32
	// Risk bucket derived from Score.
	Risk RiskLevel
}

// Sentinel errors mapped from the C ABI error code table.
//
// The numeric codes are stable across versions:
//
//	0  → nil (OK)
//	1  → ErrInvalidInput
//	2  → ErrDownloadFailed
//	3  → ErrInferenceFailed
//	99 → ErrInternal
var (
	ErrInvalidInput    = errors.New("jailguard: invalid input (null pointer or non-UTF-8)")
	ErrDownloadFailed  = errors.New("jailguard: ONNX model download failed")
	ErrInferenceFailed = errors.New("jailguard: inference / classification failed")
	ErrInternal        = errors.New("jailguard: internal error")
)

// errorFromCode maps a numeric C ABI error code to a Go sentinel error.
// Returns nil for code 0 (OK).
func errorFromCode(code int32) error {
	switch code {
	case 0:
		return nil
	case 1:
		return ErrInvalidInput
	case 2:
		return ErrDownloadFailed
	case 3:
		return ErrInferenceFailed
	case 99:
		return ErrInternal
	default:
		return fmt.Errorf("jailguard: unknown error code %d", code)
	}
}

// cDetectionResult mirrors the C struct jailguard_detection_result_t.
// Layout: int32 is_injection, float32 score, float32 confidence, int32 risk.
// 16 bytes total, naturally aligned. Shared between CGo and purego backends.
type cDetectionResult struct {
	IsInjection int32
	Score       float32
	Confidence  float32
	Risk        int32
}

// toResult converts the C struct to the public Result type.
func (c cDetectionResult) toResult() Result {
	return Result{
		IsInjection: c.IsInjection != 0,
		Score:       c.Score,
		Confidence:  c.Confidence,
		Risk:        RiskLevel(c.Risk),
	}
}
