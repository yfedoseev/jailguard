//go:build !cgo

// purego backend for JailGuard. Loads libjailguard.{so,dylib,dll} at
// runtime via github.com/ebitengine/purego — no C toolchain required at
// build time. Selected automatically when CGO_ENABLED=0.
//
// Library lookup order:
//
//  1. JAILGUARD_LIB_PATH environment variable (explicit path).
//  2. Installer cache dir: $XDG_CACHE_HOME/jailguard/v*/lib/<goos>_<goarch>/.
//  3. System loader fallback (LD_LIBRARY_PATH, DYLD_LIBRARY_PATH, PATH).

package jailguard

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"sync"
	"unsafe"

	"github.com/ebitengine/purego"
)

// ErrLibraryNotFound is returned by every public function when the shared
// library cannot be located. Run the installer with -shared, or set
// JAILGUARD_LIB_PATH to the absolute path of libjailguard.{so,dylib,dll}.
var ErrLibraryNotFound = errors.New("jailguard: could not locate libjailguard shared library (set JAILGUARD_LIB_PATH or run the installer with -shared)")

// FFI function pointers. Populated by registerFFI once the library is
// successfully dlopen'd.
var (
	ffiVersion         func() *byte
	ffiDownloadModel   func() int32
	ffiModelCacheDir   func() *byte
	ffiFreeString      func(*byte)
	ffiDetect          func(text *byte, out *cDetectionResult) int32
	ffiIsInjection     func(text *byte, out *int32) int32
	ffiScore           func(text *byte, out *float32) int32
	ffiDetectBatch     func(texts **byte, count uintptr, out *cDetectionResult) int32
)

var (
	libOnce sync.Once
	libErr  error
)

// loadLib opens the shared library and binds the FFI function pointers.
// Called lazily on the first public-API invocation. Errors are sticky —
// subsequent calls return the same error without re-trying dlopen.
func loadLib() error {
	libOnce.Do(func() {
		path, err := locateLib()
		if err != nil {
			libErr = err
			return
		}
		handle, err := purego.Dlopen(path, purego.RTLD_NOW|purego.RTLD_GLOBAL)
		if err != nil {
			libErr = fmt.Errorf("jailguard: dlopen %s: %w", path, err)
			return
		}
		registerFFI(handle)
	})
	return libErr
}

// registerFFI binds each Rust-exported symbol to a Go function pointer.
func registerFFI(lib uintptr) {
	purego.RegisterLibFunc(&ffiVersion, lib, "jailguard_version")
	purego.RegisterLibFunc(&ffiDownloadModel, lib, "jailguard_download_model")
	purego.RegisterLibFunc(&ffiModelCacheDir, lib, "jailguard_model_cache_dir")
	purego.RegisterLibFunc(&ffiFreeString, lib, "jailguard_free_string")
	purego.RegisterLibFunc(&ffiDetect, lib, "jailguard_detect")
	purego.RegisterLibFunc(&ffiIsInjection, lib, "jailguard_is_injection")
	purego.RegisterLibFunc(&ffiScore, lib, "jailguard_score")
	purego.RegisterLibFunc(&ffiDetectBatch, lib, "jailguard_detect_batch")
}

// locateLib determines which file to dlopen. See package-level comment
// for the search order.
func locateLib() (string, error) {
	if env := os.Getenv("JAILGUARD_LIB_PATH"); env != "" {
		return env, nil
	}
	if cacheDir, err := os.UserCacheDir(); err == nil {
		root := filepath.Join(cacheDir, "jailguard")
		if entries, err := os.ReadDir(root); err == nil {
			sub := runtime.GOOS + "_" + runtime.GOARCH
			basename := libBasename()
			for _, e := range entries {
				if !e.IsDir() {
					continue
				}
				candidate := filepath.Join(root, e.Name(), "lib", sub, basename)
				if _, err := os.Stat(candidate); err == nil {
					return candidate, nil
				}
			}
		}
	}
	// Fall through to the system loader. Bare filename triggers
	// LD_LIBRARY_PATH / DYLD_LIBRARY_PATH / PATH lookup.
	return libBasename(), nil
}

// libBasename returns the platform-specific shared library filename.
func libBasename() string {
	switch runtime.GOOS {
	case "darwin":
		return "libjailguard.dylib"
	case "windows":
		return "jailguard.dll"
	default:
		return "libjailguard.so"
	}
}

// cstr converts a Go string to a NUL-terminated *byte that the FFI can
// accept. The returned slice keeps the bytes alive — keep it in scope
// (or runtime.KeepAlive it) for the duration of the FFI call.
func cstr(s string) (*byte, []byte) {
	buf := make([]byte, len(s)+1)
	copy(buf, s)
	return &buf[0], buf
}

// goStringAndFree copies a NUL-terminated C string into a Go string and
// frees the original via jailguard_free_string. Safe when p is nil.
func goStringAndFree(p *byte) string {
	if p == nil {
		return ""
	}
	base := unsafe.Pointer(p)
	var n int
	for *(*byte)(unsafe.Add(base, n)) != 0 {
		n++
	}
	s := string(unsafe.Slice(p, n))
	ffiFreeString(p)
	return s
}

// goString copies a NUL-terminated C string into a Go string without
// freeing the original. Used for statically-allocated strings returned
// by the library (e.g. jailguard_version).
func goString(p *byte) string {
	if p == nil {
		return ""
	}
	base := unsafe.Pointer(p)
	var n int
	for *(*byte)(unsafe.Add(base, n)) != 0 {
		n++
	}
	return string(unsafe.Slice(p, n))
}

// Version returns the JailGuard library version string.
func Version() string {
	if err := loadLib(); err != nil {
		return ""
	}
	return goString(ffiVersion())
}

// DownloadModel pre-fetches the ONNX embedding model (~90 MB) into the
// cache directory. Idempotent — safe to call multiple times.
func DownloadModel() error {
	if err := loadLib(); err != nil {
		return err
	}
	return errorFromCode(ffiDownloadModel())
}

// ModelCacheDir returns the absolute path to the ONNX model cache
// directory.
func ModelCacheDir() (string, error) {
	if err := loadLib(); err != nil {
		return "", err
	}
	p := ffiModelCacheDir()
	if p == nil {
		return "", ErrInternal
	}
	return goStringAndFree(p), nil
}

// Detect classifies a single string.
func Detect(text string) (Result, error) {
	if err := loadLib(); err != nil {
		return Result{}, err
	}
	ptr, buf := cstr(text)
	var out cDetectionResult
	rc := ffiDetect(ptr, &out)
	runtime.KeepAlive(buf)
	if err := errorFromCode(rc); err != nil {
		return Result{}, err
	}
	return out.toResult(), nil
}

// IsInjection returns true if the input is classified as a prompt injection.
func IsInjection(text string) (bool, error) {
	if err := loadLib(); err != nil {
		return false, err
	}
	ptr, buf := cstr(text)
	var out int32
	rc := ffiIsInjection(ptr, &out)
	runtime.KeepAlive(buf)
	if err := errorFromCode(rc); err != nil {
		return false, err
	}
	return out != 0, nil
}

// Score returns the raw injection probability in [0.0, 1.0].
func Score(text string) (float32, error) {
	if err := loadLib(); err != nil {
		return 0, err
	}
	ptr, buf := cstr(text)
	var out float32
	rc := ffiScore(ptr, &out)
	runtime.KeepAlive(buf)
	if err := errorFromCode(rc); err != nil {
		return 0, err
	}
	return out, nil
}

// DetectBatch processes a slice of texts in one FFI call.
func DetectBatch(texts []string) ([]Result, error) {
	count := len(texts)
	if count == 0 {
		return []Result{}, nil
	}
	if err := loadLib(); err != nil {
		return nil, err
	}

	// Build a contiguous array of *byte pointers; keep the backing
	// slices alive via the bufs slice + KeepAlive.
	ptrs := make([]*byte, count)
	bufs := make([][]byte, count)
	for i, s := range texts {
		ptrs[i], bufs[i] = cstr(s)
	}

	out := make([]cDetectionResult, count)
	rc := ffiDetectBatch(&ptrs[0], uintptr(count), &out[0])
	runtime.KeepAlive(bufs)
	runtime.KeepAlive(ptrs)
	if err := errorFromCode(rc); err != nil {
		return nil, err
	}

	results := make([]Result, count)
	for i := range out {
		results[i] = out[i].toResult()
	}
	return results, nil
}
