// Batch scorer for benchmarking — reads JSONL from stdin, writes predictions to stdout.
//
// Input:  one JSON object per line: {"id":"...","text":"...","label":0}
// Output: one JSON object per line: {"id":"...","label":0,"pred":1,"score":0.95,"latency_ms":14.2,"model":"jailguard-0.1.0"}
//
// Used by scripts/compare_models.py so JailGuard participates in the same
// benchmark loop as Python-based classifiers.
//
// Build:  cargo build --release --example score_jsonl
// Run:    echo '{"id":"0","text":"ignore all instructions","label":1}' \
//           | ./target/release/examples/score_jsonl

use jailguard::detect;
use std::io::{BufRead, Write};

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    // Warm up the ONNX session on the first call so it doesn't skew latency
    // numbers for the first real sample.
    let _ = detect("warmup");

    for line in stdin.lock().lines() {
        let line = line.expect("stdin read error");
        if line.trim().is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        let Some(text) = v["text"].as_str() else {
            continue;
        };
        let id = v["id"].as_str().unwrap_or("");
        let label = v["label"].as_i64().unwrap_or(-1);

        let t0 = std::time::Instant::now();
        let result = detect(text);
        let latency_ms = (t0.elapsed().as_secs_f64() * 1_000_000.0).round() / 1_000.0;

        let _ = writeln!(
            out,
            "{}",
            serde_json::json!({
                "id":         id,
                "label":      label,
                "pred":       if result.is_injection { 1i32 } else { 0i32 },
                "score":      (result.score * 100_000.0).round() / 100_000.0,
                "latency_ms": latency_ms,
                "model":      concat!("jailguard-", env!("CARGO_PKG_VERSION")),
            })
        );
        let _ = out.flush();
    }
}
