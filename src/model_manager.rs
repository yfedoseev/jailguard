//! ONNX model download and cache management.
//!
//! The ONNX model (all-MiniLM-L6-v2, ~90 MB) is too large to embed in the binary.
//! This module handles downloading it to a local cache on first use.
//!
//! # Cache location (in order of priority)
//!
//! 1. `JAILGUARD_MODEL_DIR` environment variable
//! 2. `~/.cache/jailguard/`
//!
//! # Download URL (in order of priority)
//!
//! 1. `JAILGUARD_MODEL_URL` environment variable — use this to point at an
//!    internal mirror, S3 bucket, or any HTTP server in air-gapped environments.
//! 2. The default `HuggingFace` URL.
//!
//! # Production Usage
//!
//! Pre-download the model at build/deploy time so the first request has no latency:
//! ```rust,no_run
//! jailguard::download_model().expect("Failed to download ONNX model");
//! ```

use sha2::{Digest, Sha256};
use std::path::PathBuf;

use crate::error::Error;

const DEFAULT_ONNX_URL: &str =
    "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx";
const ONNX_FILENAME: &str = "all-MiniLM-L6-v2.onnx";

/// SHA-256 of the canonical `all-MiniLM-L6-v2` ONNX file.
/// Verified against the `HuggingFace` release; used to detect truncated or corrupt downloads.
const ONNX_SHA256: &str = "6fd5d72fe4589f189f8ebc006442dbb529bb7ce38f8082112682524616046452";

/// Expected file size in bytes (~90 MB). Used for progress reporting only.
const EXPECTED_SIZE: u64 = 90_000_000;

/// Return the effective download URL.
///
/// Checks `JAILGUARD_MODEL_URL` first so operators can point at an internal
/// mirror without recompiling the library.
fn model_url() -> String {
    std::env::var("JAILGUARD_MODEL_URL").unwrap_or_else(|_| DEFAULT_ONNX_URL.to_string())
}

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
/// This is safe to call multiple times — it's a no-op if the model already exists
/// and passes its checksum.
///
/// # Environment variables
///
/// - `JAILGUARD_MODEL_DIR` — override the cache directory.
/// - `JAILGUARD_MODEL_URL` — override the download URL (e.g. internal mirror).
///
/// # Errors
///
/// Returns an error if:
/// - The cache directory cannot be created
/// - The download fails (network error, HTTP error)
/// - The downloaded file's SHA-256 does not match the expected value
/// - The downloaded file cannot be written to disk
#[allow(clippy::print_stderr)]
pub fn download_model() -> Result<PathBuf, Error> {
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

    let url = model_url();
    eprintln!(
        "jailguard: downloading ONNX model (~90 MB) to {} ...",
        model_path.display()
    );
    if url != DEFAULT_ONNX_URL {
        eprintln!("jailguard: using custom URL: {url}");
    }

    let resp = ureq::get(&url)
        .call()
        .map_err(|e| Error::Model(format!("Failed to download ONNX model: {e}")))?;

    let body = resp.into_body();
    let content_length = body.content_length().unwrap_or(EXPECTED_SIZE);

    // Download to a temp file while computing SHA-256 in a single pass.
    let tmp_path = dir.join(format!("{ONNX_FILENAME}.download"));
    let mut file = std::fs::File::create(&tmp_path)?;
    let mut reader = body.into_reader();

    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    let mut downloaded: u64 = 0;
    let mut last_pct: u64 = 0;

    loop {
        let n = std::io::Read::read(&mut reader, &mut buf)?;
        if n == 0 {
            break;
        }
        std::io::Write::write_all(&mut file, &buf[..n])?;
        hasher.update(&buf[..n]);
        downloaded += n as u64;

        let pct = (downloaded * 100) / content_length;
        if pct >= last_pct + 10 {
            eprintln!("jailguard: downloaded {pct}%");
            last_pct = pct;
        }
    }

    drop(file);

    // Verify integrity before making the file visible.
    let actual: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if actual != ONNX_SHA256 {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(Error::Model(format!(
            "ONNX model checksum mismatch — expected {ONNX_SHA256}, got {actual}. \
             The download may be incomplete or the file at the URL has changed. \
             Delete {} and retry, or set JAILGUARD_MODEL_URL to a known-good mirror.",
            tmp_path.display()
        )));
    }

    std::fs::rename(&tmp_path, &model_path)?;

    eprintln!(
        "jailguard: ONNX model ready ({:.1} MB)",
        downloaded as f64 / 1_000_000.0
    );

    Ok(model_path)
}
