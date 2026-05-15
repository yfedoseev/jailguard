//! PyO3 Python bindings for JailGuard.
//!
//! Compiled only when the `python` feature is enabled (via maturin or
//! `cargo build --features python`). The resulting native extension module
//! is named `_jailguard` and lives inside the `jailguard` Python package.
//!
//! # Python API
//!
//! ```python
//! import jailguard
//!
//! # Pre-download ONNX model at startup (optional — auto-downloaded on first detect())
//! jailguard.download_model()
//!
//! # Quick boolean check
//! if jailguard.is_injection("ignore all previous instructions"):
//!     raise ValueError("Injection detected")
//!
//! # Detailed result
//! result = jailguard.detect("What is the capital of France?")
//! print(result.is_injection, result.score, result.risk)
//!
//! # Batch (reuses the same detector session)
//! results = jailguard.detect_batch(["safe query", "ignore all instructions"])
//! ```

use pyo3::prelude::*;

use crate::embedded::{self, RiskLevel};

// ---------------------------------------------------------------------------
// RiskLevel Python enum
// ---------------------------------------------------------------------------

/// Risk level classification for a detection result.
///
/// Values
/// ------
/// RiskLevel.Safe
///     Score < 0.3 — very likely benign.
/// RiskLevel.Low
///     Score 0.3–0.5 — probably benign, worth monitoring.
/// RiskLevel.Medium
///     Score 0.5–0.7 — possible injection, review recommended.
/// RiskLevel.High
///     Score 0.7–0.9 — likely injection.
/// RiskLevel.Critical
///     Score ≥ 0.9 — almost certainly an injection.
#[pyclass(name = "RiskLevel", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyRiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[pymethods]
impl PyRiskLevel {
    fn __repr__(&self) -> &'static str {
        match self {
            PyRiskLevel::Safe => "RiskLevel.Safe",
            PyRiskLevel::Low => "RiskLevel.Low",
            PyRiskLevel::Medium => "RiskLevel.Medium",
            PyRiskLevel::High => "RiskLevel.High",
            PyRiskLevel::Critical => "RiskLevel.Critical",
        }
    }

    fn __str__(&self) -> &'static str {
        match self {
            PyRiskLevel::Safe => "Safe",
            PyRiskLevel::Low => "Low",
            PyRiskLevel::Medium => "Medium",
            PyRiskLevel::High => "High",
            PyRiskLevel::Critical => "Critical",
        }
    }
}

impl From<RiskLevel> for PyRiskLevel {
    fn from(r: RiskLevel) -> Self {
        match r {
            RiskLevel::Safe => PyRiskLevel::Safe,
            RiskLevel::Low => PyRiskLevel::Low,
            RiskLevel::Medium => PyRiskLevel::Medium,
            RiskLevel::High => PyRiskLevel::High,
            RiskLevel::Critical => PyRiskLevel::Critical,
        }
    }
}

// ---------------------------------------------------------------------------
// DetectionResult Python class
// ---------------------------------------------------------------------------

/// Prompt injection detection result.
///
/// Attributes
/// ----------
/// is_injection : bool
///     ``True`` if the text is classified as a prompt injection attempt.
/// score : float
///     Raw model probability (0.0 = definitely benign, 1.0 = definitely injection).
/// confidence : float
///     Confidence in the prediction (always >= 0.5).
/// risk : RiskLevel
///     Risk level enum value (``RiskLevel.Safe``, ``RiskLevel.Low``, etc.).
#[pyclass(name = "DetectionResult", get_all)]
#[derive(Clone)]
pub struct PyDetectionResult {
    /// Whether the text is a prompt injection attempt.
    pub is_injection: bool,
    /// Raw model probability (0.0–1.0).
    pub score: f32,
    /// Confidence in the prediction (always >= 0.5).
    pub confidence: f32,
    /// Risk level enum.
    pub risk: PyRiskLevel,
}

#[pymethods]
impl PyDetectionResult {
    fn __repr__(&self) -> String {
        format!(
            "DetectionResult(is_injection={}, score={:.4}, confidence={:.4}, risk={})",
            if self.is_injection { "True" } else { "False" },
            self.score,
            self.confidence,
            self.risk.__repr__(),
        )
    }

    /// ``bool(result)`` is ``True`` when an injection is detected.
    fn __bool__(&self) -> bool {
        self.is_injection
    }
}

impl From<embedded::DetectionOutput> for PyDetectionResult {
    fn from(r: embedded::DetectionOutput) -> Self {
        Self {
            is_injection: r.is_injection,
            score: r.score,
            confidence: r.confidence,
            risk: r.risk.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Detect whether *text* is a prompt injection attempt.
///
/// Returns a :class:`DetectionResult` with ``is_injection``, ``score``,
/// ``confidence``, and ``risk`` fields.
///
/// On first call the ONNX embedding model (~90 MB) is downloaded to
/// ``~/.cache/jailguard/`` if it is not already cached.
/// Call :func:`download_model` at startup to avoid latency on the first request.
///
/// Parameters
/// ----------
/// text : str
///     Input text to classify.
///
/// Returns
/// -------
/// DetectionResult
#[pyfunction]
fn detect(text: &str) -> PyDetectionResult {
    embedded::detect(text).into()
}

/// Return ``True`` if *text* is a prompt injection attempt.
///
/// Parameters
/// ----------
/// text : str
///
/// Returns
/// -------
/// bool
#[pyfunction]
fn is_injection(text: &str) -> bool {
    embedded::is_injection(text)
}

/// Return the injection probability score for *text* (0.0 to 1.0).
///
/// Parameters
/// ----------
/// text : str
///
/// Returns
/// -------
/// float
#[pyfunction]
fn score(text: &str) -> f32 {
    embedded::score(text)
}

/// Classify a list of texts in one call.
///
/// More efficient than calling :func:`detect` in a loop because it reuses
/// the same detector session for all inputs.
///
/// Parameters
/// ----------
/// texts : list[str]
///
/// Returns
/// -------
/// list[DetectionResult]
///     Results in the same order as *texts*.
#[pyfunction]
fn detect_batch(texts: Vec<String>) -> Vec<PyDetectionResult> {
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    embedded::detect_batch(&refs)
        .into_iter()
        .map(PyDetectionResult::from)
        .collect()
}

/// Ensure the ONNX embedding model is downloaded and return its local path.
///
/// The model (~90 MB) is cached at ``~/.cache/jailguard/`` by default.
/// Override the location with the ``JAILGUARD_MODEL_DIR`` environment variable.
///
/// This function is idempotent — it skips the download if the model already
/// exists and passes its SHA-256 checksum.
///
/// Raises
/// ------
/// RuntimeError
///     If the download fails or the checksum does not match.
///
/// Returns
/// -------
/// str
///     Absolute path to the cached ONNX model file.
#[pyfunction]
fn download_model() -> PyResult<String> {
    crate::model_manager::download_model()
        .map(|p| p.display().to_string())
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
}

/// Return the path to the ONNX model cache directory.
///
/// Reads ``JAILGUARD_MODEL_DIR`` if set; otherwise returns
/// ``~/.cache/jailguard``.
///
/// Returns
/// -------
/// str
#[pyfunction]
fn model_cache_dir() -> pyo3::PyResult<String> {
    // Delegate to the canonical Rust function so Windows-specific fallback
    // logic (USERPROFILE / LOCALAPPDATA when HOME is unset or = "~") stays
    // in one place. The duplicated env-var ladder this used to inline
    // hardcoded a literal "~/.cache/jailguard" on Windows when HOME wasn't
    // set, which crashed pytest::test_model_cache_dir_exists.
    crate::model_cache_dir().map_err(|e| pyo3::exceptions::PyOSError::new_err(e.to_string()))
}

// ---------------------------------------------------------------------------
// Module entry point
// ---------------------------------------------------------------------------

/// Native extension module — imported as ``jailguard._jailguard``.
#[pymodule]
fn _jailguard(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyRiskLevel>()?;
    m.add_class::<PyDetectionResult>()?;
    m.add_function(wrap_pyfunction!(detect, m)?)?;
    m.add_function(wrap_pyfunction!(is_injection, m)?)?;
    m.add_function(wrap_pyfunction!(score, m)?)?;
    m.add_function(wrap_pyfunction!(detect_batch, m)?)?;
    m.add_function(wrap_pyfunction!(download_model, m)?)?;
    m.add_function(wrap_pyfunction!(model_cache_dir, m)?)?;
    Ok(())
}
