//go:build cgo

// CGo backend for JailGuard. Links against libjailguard via the C ABI
// declared in include/jailguard.h.
//
// Build configuration sources, in order:
//
//  1. The `jailguard_dev` build tag (see cgo_dev.go) — uses workspace
//     target/ paths. For monorepo development.
//  2. CGO_CFLAGS and CGO_LDFLAGS env vars — set these to point at the
//     installer's cache dir. The installer (`go run …/cmd/install`)
//     prints them ready to paste.
//
// For a CGo-free build, set CGO_ENABLED=0 and the purego backend in
// jailguard_purego.go is selected automatically.

package jailguard

/*
#include <stdlib.h>
#include "jailguard.h"
*/
import "C"

import (
	"runtime"
	"unsafe"
)

// Version returns the JailGuard library version string.
func Version() string {
	return C.GoString(C.jailguard_version())
}

// DownloadModel pre-fetches the ONNX embedding model (~90 MB) into the
// cache directory. Idempotent — safe to call multiple times. Calling
// this at startup avoids first-detection latency.
func DownloadModel() error {
	return errorFromCode(int32(C.jailguard_download_model()))
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

// Detect classifies a single string. Returns an error if the underlying
// inference fails or the input is not valid UTF-8.
func Detect(text string) (Result, error) {
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))

	var out C.jailguard_detection_result_t
	rc := C.jailguard_detect(cText, &out)
	if err := errorFromCode(int32(rc)); err != nil {
		return Result{}, err
	}
	return cDetectionResult{
		IsInjection: int32(out.is_injection),
		Score:       float32(out.score),
		Confidence:  float32(out.confidence),
		Risk:        int32(out.risk),
	}.toResult(), nil
}

// IsInjection returns true if the input is classified as a prompt injection.
// Equivalent to Detect(text).IsInjection but slightly cheaper because no
// Result struct is constructed.
func IsInjection(text string) (bool, error) {
	cText := C.CString(text)
	defer C.free(unsafe.Pointer(cText))

	var out C.int
	rc := C.jailguard_is_injection(cText, &out)
	if err := errorFromCode(int32(rc)); err != nil {
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
	if err := errorFromCode(int32(rc)); err != nil {
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

	cTexts := make([]*C.char, count)
	for i, s := range texts {
		cTexts[i] = C.CString(s)
	}
	defer func() {
		for _, p := range cTexts {
			C.free(unsafe.Pointer(p))
		}
	}()

	cArr := (**C.char)(unsafe.Pointer(&cTexts[0]))

	out := make([]C.jailguard_detection_result_t, count)
	rc := C.jailguard_detect_batch(cArr, C.size_t(count), &out[0])
	if err := errorFromCode(int32(rc)); err != nil {
		return nil, err
	}

	results := make([]Result, count)
	for i := range out {
		results[i] = cDetectionResult{
			IsInjection: int32(out[i].is_injection),
			Score:       float32(out[i].score),
			Confidence:  float32(out[i].confidence),
			Risk:        int32(out[i].risk),
		}.toResult()
	}
	runtime.KeepAlive(cTexts)
	return results, nil
}
