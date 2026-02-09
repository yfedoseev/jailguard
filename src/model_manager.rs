//! ONNX model download and cache management.
//!
//! The ONNX model (all-MiniLM-L6-v2, ~90 MB) is too large to embed in the binary.
//! This module handles downloading it to a local cache on first use.
//!
//! Cache location (in order of priority):
//! 1. `JAILGUARD_MODEL_DIR` environment variable
//! 2. `~/.cache/jailguard/`
//!
//! # Production Usage
//!
//! Pre-download the model at build/deploy time:
//! ```rust,no_run
//! jailguard::ensure_model().expect("Failed to download ONNX model");
//! ```

use std::path::PathBuf;

use crate::error::Error;

const ONNX_URL: &str = "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx";
const ONNX_FILENAME: &str = "all-MiniLM-L6-v2.onnx";

/// Expected file size in bytes (~90 MB). Used for progress reporting.
const EXPECTED_SIZE: u64 = 90_000_000;

/// Return the cache directory for ONNX model files.
///
/// Checks `JAILGUARD_MODEL_DIR` env var first, then falls back to `~/.cache/jailguard/`.
fn cache_dir() -> Result<PathBuf, Error> {
    if let Ok(dir) = std::env::var("JAILGUARD_MODEL_DIR") {
        return Ok(PathBuf::from(dir));
    }

    let home = std::env::var("HOME")
        .map_err(|_| Error::Config("HOME environment variable not set".into()))?;
    Ok(PathBuf::from(home).join(".cache").join("jailguard"))
}

/// Ensure the ONNX model is available locally, downloading it if necessary.
///
/// Returns the path to the ONNX model file on disk.
///
/// This is safe to call multiple times — it's a no-op if the model already exists.
///
/// # Errors
///
/// Returns an error if:
/// - The cache directory cannot be created
/// - The download fails (network error, HTTP error)
/// - The downloaded file cannot be written to disk
#[allow(clippy::print_stderr)]
pub fn ensure_model() -> Result<PathBuf, Error> {
    let dir = cache_dir()?;
    let model_path = dir.join(ONNX_FILENAME);

    if model_path.exists() {
        return Ok(model_path);
    }

    std::fs::create_dir_all(&dir).map_err(|e| {
        Error::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to create cache dir {}: {}", dir.display(), e),
        ))
    })?;

    eprintln!(
        "jailguard: downloading ONNX model (~90 MB) to {} ...",
        model_path.display()
    );

    let resp = ureq::get(ONNX_URL)
        .call()
        .map_err(|e| Error::Model(format!("Failed to download ONNX model: {e}")))?;

    let content_length = resp
        .header("content-length")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(EXPECTED_SIZE);

    // Download to a temp file first, then rename (atomic-ish)
    let tmp_path = dir.join(format!("{ONNX_FILENAME}.download"));
    let mut file = std::fs::File::create(&tmp_path)?;

    let mut reader = resp.into_reader();
    let mut buf = [0u8; 64 * 1024];
    let mut downloaded: u64 = 0;
    let mut last_pct: u64 = 0;

    loop {
        let n = std::io::Read::read(&mut reader, &mut buf)?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buf[..n])?;
        downloaded += n as u64;

        let pct = (downloaded * 100) / content_length;
        if pct >= last_pct + 10 {
            eprintln!("jailguard: downloaded {pct}%");
            last_pct = pct;
        }
    }

    drop(file);
    std::fs::rename(&tmp_path, &model_path)?;

    eprintln!(
        "jailguard: ONNX model ready ({:.1} MB)",
        downloaded as f64 / 1_000_000.0
    );

    Ok(model_path)
}
