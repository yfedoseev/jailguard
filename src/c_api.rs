//! C ABI for JailGuard.
//!
//! Stable, no-mangle `extern "C"` surface that Go (cgo) and Node.js
//! (napi-rs / N-API) bindings link against. The compiled
//! `libjailguard.{so,dylib,a}` (Linux/macOS) or `jailguard.{dll,lib}`
//! (Windows) exports these symbols when built as a `cdylib` /
//! `staticlib`.
//!
//! # Error codes
//!
//! Non-zero return values indicate failure. The numeric codes are stable
//! across versions:
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0    | OK |
//! | 1    | Null pointer or invalid UTF-8 in input |
//! | 2    | ONNX model download failed (network or checksum) |
//! | 3    | Inference / classification failed |
//! | 99   | Internal error (logic bug — please file an issue) |
//!
//! # Memory ownership
//!
//! - Strings returned by [`jailguard_model_cache_dir`] and
//!   [`jailguard_version`]'s `cache_dir` style functions are owned by
//!   the caller and must be freed with [`jailguard_free_string`].
//! - Static strings (e.g. [`jailguard_version`]) are owned by the library
//!   and must NOT be freed.
//! - Output structs (`jailguard_detection_result_t`) are written
//!   in-place by the library; the caller owns the storage.
//!
//! # Thread safety
//!
//! All functions are safe to call from any thread concurrently. The
//! underlying detector is initialised lazily once per process and
//! protected by a `Mutex<Session>` internally — concurrent calls
//! serialise on the ONNX session lock, which is the same model the
//! Python and embedded Rust APIs use.
//!
//! # Example (C)
//!
//! ```c
//! #include "jailguard.h"
//! #include <stdio.h>
//!
//! int main() {
//!     // Optional — skip if you don't mind first-call latency.
//!     if (jailguard_download_model() != JAILGUARD_OK) {
//!         fprintf(stderr, "model download failed\n");
//!         return 1;
//!     }
//!
//!     jailguard_detection_result_t r;
//!     if (jailguard_detect("ignore previous instructions", &r) == JAILGUARD_OK) {
//!         printf("is_injection=%d score=%.4f risk=%d\n",
//!                r.is_injection, r.score, r.risk);
//!     }
//!     return 0;
//! }
//! ```

// FFI is inherently unsafe — relax the crate-wide
// `[lints.rust] unsafe_code = "warn"` for this module only.
#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int};

use crate::embedded;

// ─── Error codes ─────────────────────────────────────────────────────────────

/// Operation succeeded.
pub const JAILGUARD_OK: c_int = 0;
/// Null pointer or invalid UTF-8 in input string.
pub const JAILGUARD_INVALID_INPUT: c_int = 1;
/// ONNX model download failed (network error or checksum mismatch).
pub const JAILGUARD_DOWNLOAD_FAILED: c_int = 2;
/// Inference or classification failed (corrupt session, OOM, etc.).
pub const JAILGUARD_INFERENCE_FAILED: c_int = 3;
/// Internal error — likely a logic bug. Please file an issue.
pub const JAILGUARD_INTERNAL_ERROR: c_int = 99;

// ─── Risk-level enum (mirrors Rust + Python) ─────────────────────────────────

/// Risk classification bucket. Matches the Rust [`RiskLevel`] and the
/// Python `RiskLevel` enum value-for-value.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum jailguard_risk_t {
    /// Score < 0.3 — almost certainly benign.
    Safe = 0,
    /// 0.3 ≤ score < 0.5 — probably benign but worth monitoring.
    Low = 1,
    /// 0.5 ≤ score < 0.7 — possible injection, review recommended.
    Medium = 2,
    /// 0.7 ≤ score < 0.9 — likely injection.
    High = 3,
    /// score ≥ 0.9 — almost certainly an injection.
    Critical = 4,
}

impl From<embedded::RiskLevel> for jailguard_risk_t {
    fn from(r: embedded::RiskLevel) -> Self {
        match r {
            embedded::RiskLevel::Safe => Self::Safe,
            embedded::RiskLevel::Low => Self::Low,
            embedded::RiskLevel::Medium => Self::Medium,
            embedded::RiskLevel::High => Self::High,
            embedded::RiskLevel::Critical => Self::Critical,
        }
    }
}

// ─── Detection result struct ─────────────────────────────────────────────────

/// Output of [`jailguard_detect`].
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct jailguard_detection_result_t {
    /// 1 if classified as injection, 0 otherwise.
    pub is_injection: c_int,
    /// Raw model probability in [0.0, 1.0].
    pub score: c_float,
    /// Confidence (always ≥ 0.5). Equals score for injections,
    /// 1.0 - score for benigns.
    pub confidence: c_float,
    /// Risk bucket derived from score.
    pub risk: jailguard_risk_t,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// SAFETY: caller must guarantee `text` is either null or a valid C string.
unsafe fn cstr_to_str<'a>(text: *const c_char) -> Result<&'a str, c_int> {
    if text.is_null() {
        return Err(JAILGUARD_INVALID_INPUT);
    }
    // SAFETY: caller-validated above.
    let cstr = unsafe { CStr::from_ptr(text) };
    cstr.to_str().map_err(|_| JAILGUARD_INVALID_INPUT)
}

fn fill_result(out: &mut jailguard_detection_result_t, r: embedded::DetectionOutput) {
    out.is_injection = c_int::from(r.is_injection);
    out.score = r.score;
    out.confidence = r.confidence;
    out.risk = r.risk.into();
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Return the JailGuard library version string (matches `Cargo.toml`).
///
/// The returned pointer is to a static, NUL-terminated string and must
/// not be freed.
#[unsafe(no_mangle)]
pub extern "C" fn jailguard_version() -> *const c_char {
    // Concatenate `\0` at compile time so the byte slice is a valid C string.
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Pre-download the ONNX embedding model (~90 MB) to the cache directory.
///
/// Idempotent — does nothing if the model is already cached. Returns
/// [`JAILGUARD_OK`] on success or [`JAILGUARD_DOWNLOAD_FAILED`] otherwise.
///
/// Calling this at startup avoids first-detection latency for the
/// download (typically ~2–10 seconds).
#[unsafe(no_mangle)]
pub extern "C" fn jailguard_download_model() -> c_int {
    match crate::download_model() {
        Ok(_) => JAILGUARD_OK,
        Err(_) => JAILGUARD_DOWNLOAD_FAILED,
    }
}

/// Get the path to the ONNX model cache directory.
///
/// The returned string is owned by the caller and must be freed with
/// [`jailguard_free_string`]. Returns null on internal error.
#[unsafe(no_mangle)]
pub extern "C" fn jailguard_model_cache_dir() -> *mut c_char {
    let dir = match crate::model_manager::cache_dir_string() {
        Ok(d) => d,
        Err(_) => return std::ptr::null_mut(),
    };
    match CString::new(dir) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string returned by the JailGuard C API.
///
/// SAFETY: `s` must have been returned by a `jailguard_*` function that
/// documents its return value as caller-owned. Must not be called twice
/// on the same pointer. Null is a no-op.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jailguard_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    // SAFETY: caller contract — pointer originated from CString::into_raw.
    unsafe {
        let _ = CString::from_raw(s);
    }
}

/// Detect whether `text` is a prompt injection.
///
/// Writes the result to `*out` on success.
///
/// SAFETY: `text` must be a valid NUL-terminated C string. `out` must
/// be a valid, properly aligned pointer to a `jailguard_detection_result_t`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jailguard_detect(
    text: *const c_char,
    out: *mut jailguard_detection_result_t,
) -> c_int {
    if out.is_null() {
        return JAILGUARD_INVALID_INPUT;
    }
    // SAFETY: caller contract for `text`.
    let s = match unsafe { cstr_to_str(text) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    let result = embedded::detect(s);
    // SAFETY: caller contract for `out`.
    unsafe { fill_result(&mut *out, result) };
    JAILGUARD_OK
}

/// Quick boolean injection check. Writes `0` or `1` to `*out`.
///
/// SAFETY: `text` valid C string; `out` valid pointer to `c_int`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jailguard_is_injection(text: *const c_char, out: *mut c_int) -> c_int {
    if out.is_null() {
        return JAILGUARD_INVALID_INPUT;
    }
    let s = match unsafe { cstr_to_str(text) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    let v = embedded::is_injection(s);
    // SAFETY: caller contract for `out`.
    unsafe { *out = c_int::from(v) };
    JAILGUARD_OK
}

/// Get the raw injection probability in [0.0, 1.0].
///
/// SAFETY: `text` valid C string; `out` valid pointer to `c_float`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jailguard_score(text: *const c_char, out: *mut c_float) -> c_int {
    if out.is_null() {
        return JAILGUARD_INVALID_INPUT;
    }
    let s = match unsafe { cstr_to_str(text) } {
        Ok(s) => s,
        Err(code) => return code,
    };
    // SAFETY: caller contract for `out`.
    unsafe { *out = embedded::score(s) };
    JAILGUARD_OK
}

/// Detect prompt injection on a batch of texts.
///
/// `texts` is an array of `count` NUL-terminated C string pointers.
/// `out` is a pre-allocated array of `count` result slots that will be
/// written in order. The output array must be at least `count` long;
/// the caller owns the storage.
///
/// SAFETY:
/// - `texts` must be a valid array of `count` valid C string pointers
///   (each element must satisfy the contract of [`jailguard_detect`]).
/// - `out` must be a valid array of at least `count` results.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jailguard_detect_batch(
    texts: *const *const c_char,
    count: usize,
    out: *mut jailguard_detection_result_t,
) -> c_int {
    // Empty batch is valid even if both arrays are null — return
    // immediately before any pointer deref. This matches the convention
    // used by the C standard library (e.g. memcpy with count=0).
    if count == 0 {
        return JAILGUARD_OK;
    }
    if texts.is_null() || out.is_null() {
        return JAILGUARD_INVALID_INPUT;
    }
    // SAFETY: caller contract — `texts` is a valid array of `count` pointers.
    let text_slice = unsafe { std::slice::from_raw_parts(texts, count) };
    let mut owned: Vec<&str> = Vec::with_capacity(count);
    for &p in text_slice {
        match unsafe { cstr_to_str(p) } {
            Ok(s) => owned.push(s),
            Err(code) => return code,
        }
    }
    let results = embedded::detect_batch(&owned);
    // SAFETY: caller contract — `out` has at least `count` slots.
    let out_slice = unsafe { std::slice::from_raw_parts_mut(out, count) };
    for (slot, r) in out_slice.iter_mut().zip(results) {
        fill_result(slot, r);
    }
    JAILGUARD_OK
}
