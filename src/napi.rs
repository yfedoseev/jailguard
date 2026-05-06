//! Node.js bindings for JailGuard via napi-rs / N-API.
//!
//! Compiled only when the `napi` feature is enabled. The resulting native
//! module is named `jailguard.node` and is loaded by the JavaScript
//! wrapper at `js/src/native.ts`.

#![allow(clippy::missing_safety_doc)]
// napi-rs macros generate code that uses unsafe under the hood and
// triggers our [lints.rust] unsafe_code = "warn" config. Same scope as
// the c_api module — relax the lint locally.
#![allow(unsafe_code)]
#![allow(non_snake_case, missing_docs, clippy::needless_pass_by_value)]

use napi_derive::napi;

use crate::embedded;

/// Mirror of `jailguard::RiskLevel` exposed as a JS-friendly enum.
///
/// napi-rs derives Clone/Copy automatically — don't add them manually
/// or we get conflicting trait impls (E0119).
#[napi]
pub enum RiskLevel {
    Safe = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl From<embedded::RiskLevel> for RiskLevel {
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

/// Plain-old-data result struct. napi-rs derives JS bindings + TS types.
#[napi(object)]
pub struct DetectionResult {
    pub is_injection: bool,
    pub score: f64,
    pub confidence: f64,
    pub risk: RiskLevel,
}

impl From<embedded::DetectionOutput> for DetectionResult {
    fn from(r: embedded::DetectionOutput) -> Self {
        Self {
            is_injection: r.is_injection,
            score: r.score as f64,
            confidence: r.confidence as f64,
            risk: r.risk.into(),
        }
    }
}

#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[napi]
pub fn download_model() -> napi::Result<()> {
    crate::download_model()
        .map(|_| ())
        .map_err(|e| napi::Error::from_reason(format!("download failed: {e}")))
}

#[napi]
pub fn model_cache_dir() -> napi::Result<String> {
    crate::model_manager::cache_dir_string()
        .map_err(|e| napi::Error::from_reason(format!("cache dir lookup failed: {e}")))
}

#[napi]
pub fn detect(text: String) -> DetectionResult {
    embedded::detect(&text).into()
}

#[napi]
pub fn is_injection(text: String) -> bool {
    embedded::is_injection(&text)
}

#[napi]
pub fn score(text: String) -> f64 {
    embedded::score(&text) as f64
}

#[napi]
pub fn detect_batch(texts: Vec<String>) -> Vec<DetectionResult> {
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    embedded::detect_batch(&refs)
        .into_iter()
        .map(Into::into)
        .collect()
}
