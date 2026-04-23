//! Runs JailGuard over the AgentDojo indirect-injection cases and emits
//! prediction JSONL.
//!
//! AgentDojo is an agent-centric benchmark, so we flatten each case to
//! its attacker-controlled content + expected label. The conversion is
//! documented in `evaluation/external/README.md`.
//!
//! Expected input: `data/external/agentdojo_cases.jsonl`
//!   `{"id": "...", "text": "...", "label": 0 | 1}`
//! Output: `data/external/agentdojo_jailguard.jsonl`
//!
//! Scaffolding only — see the note at the top of `pint_runner.rs`.

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::Instant;

use serde::{Deserialize, Serialize};

const INPUT: &str = "data/external/agentdojo_cases.jsonl";
const OUTPUT: &str = "data/external/agentdojo_jailguard.jsonl";
const MODEL_TAG: &str = concat!("jailguard-", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    text: String,
    label: u8,
}

#[derive(Debug, Serialize)]
struct Prediction<'a> {
    id: &'a str,
    label: u8,
    pred: u8,
    score: f32,
    latency_ms: f64,
    model: &'a str,
}

fn main() -> std::io::Result<()> {
    let input = File::open(INPUT).unwrap_or_else(|e| {
        eprintln!("could not open {INPUT}: {e}. See evaluation/external/README.md for setup.");
        std::process::exit(1);
    });
    let mut output = BufWriter::new(File::create(OUTPUT)?);

    let _ = jailguard::detect("warmup");

    let mut n = 0usize;
    for line in BufReader::new(input).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let case: Case = serde_json::from_str(&line).unwrap_or_else(|e| {
            eprintln!("skipping malformed line: {e}");
            std::process::exit(2);
        });

        let t0 = Instant::now();
        let result = jailguard::detect(&case.text);
        let latency_ms = t0.elapsed().as_secs_f64() * 1000.0;

        let pred = Prediction {
            id: &case.id,
            label: case.label,
            pred: u8::from(result.is_injection),
            score: result.confidence,
            latency_ms,
            model: MODEL_TAG,
        };
        serde_json::to_writer(&mut output, &pred)?;
        output.write_all(b"\n")?;
        n += 1;
    }
    output.flush()?;
    eprintln!("wrote {n} predictions → {OUTPUT}");
    Ok(())
}
