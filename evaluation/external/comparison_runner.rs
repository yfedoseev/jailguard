//! Aggregates prediction JSONLs from JailGuard + any number of baselines
//! and prints a markdown comparison table.
//!
//! Expected input: every `*.jsonl` file under `data/external/` that
//! matches the schema documented in `evaluation/external/README.md`.
//! File names are treated as `<dataset>_<model>.jsonl`, e.g.
//! `pint_jailguard.jsonl`, `pint_deberta.jsonl`, `agentdojo_rebuff.jsonl`.
//!
//! Output: a markdown table on stdout with four metrics per
//! (model × dataset) pair: accuracy, F1, false-positive rate at 95%
//! recall, median latency.
//!
//! Scaffolding only — see the note at the top of `pint_runner.rs`.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use serde::Deserialize;

const DIR: &str = "data/external";

#[derive(Debug, Deserialize)]
struct Prediction {
    #[allow(dead_code)]
    id: String,
    label: u8,
    pred: u8,
    score: f32,
    latency_ms: f64,
    model: String,
}

#[derive(Default)]
struct Stats {
    tp: u32,
    fp: u32,
    tn: u32,
    fn_: u32,
    scores_pos: Vec<f32>,
    scores_neg: Vec<f32>,
    latencies: Vec<f64>,
}

impl Stats {
    fn push(&mut self, p: &Prediction) {
        match (p.label, p.pred) {
            (1, 1) => self.tp += 1,
            (0, 1) => self.fp += 1,
            (0, 0) => self.tn += 1,
            (1, 0) => self.fn_ += 1,
            _ => {}
        }
        if p.label == 1 {
            self.scores_pos.push(p.score);
        } else {
            self.scores_neg.push(p.score);
        }
        self.latencies.push(p.latency_ms);
    }

    fn accuracy(&self) -> f64 {
        let correct = f64::from(self.tp + self.tn);
        let total = f64::from(self.tp + self.fp + self.tn + self.fn_);
        if total == 0.0 {
            0.0
        } else {
            correct / total
        }
    }

    fn f1(&self) -> f64 {
        let tp = f64::from(self.tp);
        let precision_denom = f64::from(self.tp + self.fp);
        let recall_denom = f64::from(self.tp + self.fn_);
        if precision_denom == 0.0 || recall_denom == 0.0 {
            return 0.0;
        }
        let precision = tp / precision_denom;
        let recall = tp / recall_denom;
        if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        }
    }

    /// False-positive rate at 95% recall — sweep the score threshold
    /// downward until 95% of positives are caught, then report how many
    /// negatives cross that threshold.
    fn fpr_at_recall(&self, target_recall: f64) -> f64 {
        if self.scores_pos.is_empty() || self.scores_neg.is_empty() {
            return 0.0;
        }
        let mut pos = self.scores_pos.clone();
        pos.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // idx such that (pos.len() - idx) / pos.len() >= target_recall.
        let keep = ((pos.len() as f64) * target_recall).ceil() as usize;
        let idx = pos.len().saturating_sub(keep);
        let threshold = pos[idx];
        let fp = self
            .scores_neg
            .iter()
            .filter(|&&s| s >= threshold)
            .count();
        fp as f64 / self.scores_neg.len() as f64
    }

    fn median_latency(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let mut v = self.latencies.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        v[v.len() / 2]
    }
}

fn main() -> std::io::Result<()> {
    // Key: (dataset, model) → Stats
    let mut table: BTreeMap<(String, String), Stats> = BTreeMap::new();

    for entry in fs::read_dir(DIR)? {
        let entry = entry?;
        let path: PathBuf = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };
        // Expect "<dataset>_<model>.jsonl"; skip inputs that haven't been
        // annotated with a model suffix (e.g. the raw `pint.jsonl`).
        let (dataset, _sep) = match stem.find('_') {
            Some(i) => (stem[..i].to_string(), i),
            None => continue,
        };

        let file = fs::File::open(&path)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let p: Prediction = match serde_json::from_str(&line) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let key = (dataset.clone(), p.model.clone());
            table.entry(key).or_default().push(&p);
        }
    }

    if table.is_empty() {
        eprintln!(
            "no prediction files under {DIR}/. Run the per-dataset runners first; \
             see evaluation/external/README.md."
        );
        std::process::exit(1);
    }

    println!("| Dataset | Model | Accuracy | F1 | FPR@0.95 recall | Median latency (ms) |");
    println!("|---------|-------|----------|----|-----------------|---------------------|");
    for ((dataset, model), s) in &table {
        println!(
            "| {} | {} | {:.4} | {:.4} | {:.4} | {:.2} |",
            dataset,
            model,
            s.accuracy(),
            s.f1(),
            s.fpr_at_recall(0.95),
            s.median_latency(),
        );
    }
    Ok(())
}
