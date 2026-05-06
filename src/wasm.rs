//! WASM bindings for JailGuard via wasm-bindgen.
//!
//! **Status: alpha / scaffold only.** The `ort` Rust ONNX runtime we use
//! for inference doesn't yet compile to `wasm32-unknown-unknown`. This
//! module exposes the public API surface so callers can integrate
//! against it, but every detection function returns an error explaining
//! the gap.
//!
//! Two paths are tracked for a future release:
//!
//! - **`tract`**: a pure-Rust ONNX runtime that compiles to WASM. Slower
//!   than `ort` on native, but doesn't depend on system libraries.
//! - **ORT-Web**: a JavaScript-side ONNX runtime that the WASM module
//!   would defer to via wasm-bindgen + JS interop.
//!
//! For now, use the Node.js binding (`@jailguard/jailguard`) for
//! production workloads.

#![allow(unsafe_code)] // wasm-bindgen macros generate unsafe code internally
#![allow(missing_docs, non_snake_case, clippy::needless_pass_by_value)]

use wasm_bindgen::prelude::*;

const WASM_NOT_SUPPORTED: &str = "JailGuard WASM binding is alpha — \
    ONNX inference under wasm32-unknown-unknown is not yet implemented. \
    Use the Node.js binding (@jailguard/jailguard) for now. \
    See https://github.com/yfedoseev/jailguard for the tracking issue.";

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen(js_name = "downloadModel")]
pub fn download_model() -> Result<(), JsError> {
    Err(JsError::new(WASM_NOT_SUPPORTED))
}

#[wasm_bindgen(js_name = "modelCacheDir")]
pub fn model_cache_dir() -> Result<String, JsError> {
    Err(JsError::new(WASM_NOT_SUPPORTED))
}

#[wasm_bindgen]
pub fn detect(_text: &str) -> Result<JsValue, JsError> {
    Err(JsError::new(WASM_NOT_SUPPORTED))
}

#[wasm_bindgen(js_name = "isInjection")]
pub fn is_injection(_text: &str) -> Result<bool, JsError> {
    Err(JsError::new(WASM_NOT_SUPPORTED))
}

#[wasm_bindgen]
pub fn score(_text: &str) -> Result<f64, JsError> {
    Err(JsError::new(WASM_NOT_SUPPORTED))
}

#[wasm_bindgen(js_name = "detectBatch")]
pub fn detect_batch(_texts: Vec<JsValue>) -> Result<JsValue, JsError> {
    Err(JsError::new(WASM_NOT_SUPPORTED))
}
