// Package jailguard provides Go bindings for the JailGuard prompt-injection
// detector. The Rust core is exposed via a small C ABI (cdylib +
// include/jailguard.h); this package wraps that ABI behind an idiomatic
// Go API.
//
// # Build
//
// Building this package requires a libjailguard cdylib visible to the
// linker. The simplest path:
//
//	cd ../              # repo root
//	cargo build --release
//	cd go
//	CGO_LDFLAGS="-L../target/release -ljailguard" \
//	    CGO_CFLAGS="-I../include" \
//	    go build ./...
//
// The Makefile target `make test-go` automates this for development.
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

/*
#cgo CFLAGS: -I${SRCDIR}/../include
#cgo LDFLAGS: -L${SRCDIR}/../target/release -ljailguard
#include <stdlib.h>
#include "jailguard.h"
*/
import "C"

import (
	"errors"
	"fmt"
	"runtime"
	"unsafe"
)

// RiskLevel mirrors the C ABI's jailguard_risk_t.
type RiskLevel int32

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

// errorFromCode converts a C error code into a Go error.
func errorFromCode(rc C.int) error {
	switch rc {
	case C.JAILGUARD_OK:
		return nil
	case C.JAILGUARD_INVALID_INPUT:
		return ErrInvalidInput
	case C.JAILGUARD_DOWNLOAD_FAILED:
		return ErrDownloadFailed
	case C.JAILGUARD_INFERENCE_FAILED:
		return ErrInferenceFailed
	case C.JAILGUARD_INTERNAL_ERROR:
		return ErrInternal
	default:
		return fmt.Errorf("jailguard: unknown error code %d", rc)
	}
}

// Sentinel errors mapped from the C error code table.
var (
	ErrInvalidInput    = errors.New("jailguard: invalid input (null pointer or non-UTF-8)")
	ErrDownloadFailed  = errors.New("jailguard: ONNX model download failed")
	ErrInferenceFailed = errors.New("jailguard: inference / classification failed")
	ErrInternal        = errors.New("jailguard: internal error")
)

// Version returns the JailGuard library version string.
func Version() string {
	return C.GoString(C.jailguard_version())
}

// DownloadModel pre-fetches the ONNX embedding model (~90 MB) into the
// cache directory. Idempotent — safe to call multiple times. Calling
// this at startup avoids first-detection latency.
func DownloadModel() error {
	return errorFromCode(C.jailguard_download_model())
}

// ModelCacheDir returns the absolute path to the ONNX model cache
// directory. Defaults to ~/.cache/jailguard/; override with the
// JAILGUARD_MODEL_DIR environment variable.
func ModelCacheDir() (string, error) {
	cs := C.jailguard_model_cache_dir()
	if cs == nil {
		return "", ErrInternal
	}
	defer C.jailguard_free_string(cs)
	return C.GoString(cs), nil
}

// resultFromC converts a C struct to the Go Result.
func resultFromC(r C.jailguard_detection_result_t) Result {
	return Result{
		IsInjection: r.is_injection != 0,
		Score:       float32(r.score),
		Confidence:  float32(r.confidence),
		Risk:        RiskLevel(r.risk),
	}
}

// Detect classifies a single string. Returns an error if the underlying
// inference fails or the input is not valid UTF-8.
func Detect(text string) (Result, error) {
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))

	var out C.jailguard_detection_result_t
	rc := C.jailguard_detect(cText, &out)
	if err := errorFromCode(rc); err != nil {
		return Result{}, err
	}
	return resultFromC(out), nil
}

// IsInjection returns true if the input is classified as a prompt injection.
// Equivalent to Detect(text).IsInjection but slightly cheaper because no
// Result struct is constructed.
func IsInjection(text string) (bool, error) {
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))

	var out C.int
	rc := C.jailguard_is_injection(cText, &out)
	if err := errorFromCode(rc); err != nil {
		return false, err
	}
	return out != 0, nil
}

// Score returns the raw injection probability in [0.0, 1.0].
func Score(text string) (float32, error) {
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))

	var out C.float
	rc := C.jailguard_score(cText, &out)
	if err := errorFromCode(rc); err != nil {
		return 0, err
	}
	return float32(out), nil
}

// DetectBatch processes a slice of texts in one call. The returned slice
// has the same length and order as the input. Empty input returns an
// empty slice and no error.
func DetectBatch(texts []string) ([]Result, error) {
	count := len(texts)
	if count == 0 {
		return []Result{}, nil
	}

	// Allocate C-string copies and remember to free each one.
	cTexts := make([]*C.char, count)
	for i, s := range texts {
		cTexts[i] = C.CString(s)
	}
	defer func() {
		for _, p := range cTexts {
			C.free(unsafe.Pointer(p))
		}
	}()

	// jailguard_detect_batch wants a contiguous array of `const char*`.
	// In Go we already have []*C.char which has the right layout —
	// take a pointer to its first element.
	cArr := (**C.char)(unsafe.Pointer(&cTexts[0]))

	out := make([]C.jailguard_detection_result_t, count)
	rc := C.jailguard_detect_batch(cArr, C.size_t(count), &out[0])
	if err := errorFromCode(rc); err != nil {
		return nil, err
	}

	results := make([]Result, count)
	for i := range out {
		results[i] = resultFromC(out[i])
	}
	// Pin cTexts so it isn't garbage collected before C is done.
	runtime.KeepAlive(cTexts)
	return results, nil
}
