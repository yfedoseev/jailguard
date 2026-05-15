use jailguard::{DetectionOutput, RiskLevel};
use rustler::{Atom, NifStruct, NifUnitEnum};

mod atoms {
    rustler::atoms! {
        ok,
        error,
        download_failed,
        internal,
        safe,
        low,
        medium,
        high,
        critical,
    }
}

#[derive(NifUnitEnum)]
pub enum NifRisk {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl From<RiskLevel> for NifRisk {
    fn from(value: RiskLevel) -> Self {
        match value {
            RiskLevel::Safe => NifRisk::Safe,
            RiskLevel::Low => NifRisk::Low,
            RiskLevel::Medium => NifRisk::Medium,
            RiskLevel::High => NifRisk::High,
            RiskLevel::Critical => NifRisk::Critical,
        }
    }
}

#[derive(NifStruct)]
#[module = "JailGuard.Result"]
pub struct ElixirDetectionResult {
    pub is_injection: bool,
    pub score: f64,
    pub confidence: f64,
    pub risk: NifRisk,
}

impl From<DetectionOutput> for ElixirDetectionResult {
    fn from(value: DetectionOutput) -> Self {
        ElixirDetectionResult {
            is_injection: value.is_injection,
            score: value.score as f64,
            confidence: value.confidence as f64,
            risk: value.risk.into(),
        }
    }
}

#[rustler::nif]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// Rustler encodes `Result<T, E>` as `{:ok, T} | {:error, E}`, which double-
// wraps when T is itself an atom or already a tagged tuple. `NifResult<T>`
// (which is `Result<T, rustler::Error>`) bypasses the Ok wrapping — the
// success-path T is encoded directly — and `rustler::Error::Term(box)`
// makes the error path return the boxed term verbatim. We use that to
// produce idiomatic Elixir shapes:
//   download_model:    :ok | {:error, :download_failed}
//   model_cache_dir:   {:ok, path} | {:error, :internal}

#[rustler::nif(schedule = "DirtyIo")]
fn download_model() -> rustler::NifResult<Atom> {
    match jailguard::download_model() {
        Ok(_) => Ok(atoms::ok()),
        Err(_) => Err(rustler::Error::Term(Box::new((
            atoms::error(),
            atoms::download_failed(),
        )))),
    }
}

#[rustler::nif]
fn model_cache_dir() -> rustler::NifResult<(Atom, String)> {
    match jailguard::model_cache_dir() {
        Ok(path) => Ok((atoms::ok(), path)),
        Err(_) => Err(rustler::Error::Term(Box::new((
            atoms::error(),
            atoms::internal(),
        )))),
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
fn detect(text: String) -> (Atom, ElixirDetectionResult) {
    (atoms::ok(), jailguard::detect(&text).into())
}

#[rustler::nif(schedule = "DirtyCpu")]
fn is_injection(text: String) -> (Atom, bool) {
    (atoms::ok(), jailguard::is_injection(&text))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn score(text: String) -> (Atom, f64) {
    (atoms::ok(), jailguard::score(&text) as f64)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn detect_batch(texts: Vec<String>) -> (Atom, Vec<ElixirDetectionResult>) {
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let outputs = jailguard::detect_batch(&refs);
    (atoms::ok(), outputs.into_iter().map(Into::into).collect())
}

rustler::init!("Elixir.JailGuard.Native");
