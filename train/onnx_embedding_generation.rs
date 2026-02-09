//! ONNX-based embedding generation using all-MiniLM-L6-v2
//!
//! Generates real 384-dimensional neural embeddings for the dataset.
//! Requires ONNX model + tokenizer.json in models/ directory.
//!
//! Usage:
//!   cargo run --bin onnx_embedding_generation --features onnx --release -- \
//!     --input data/combined.json \
//!     --output data/combined_onnx_embeddings.json \
//!     --model-dir models/

use jailguard::embeddings::OnnxEmbedder;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::time::Instant;

#[derive(Debug, Clone, Deserialize)]
struct Sample {
    pub text: String,
    pub is_injection: bool,
    pub attack_type: String,
    pub attack_type_idx: usize,
    pub source: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
struct SampleWithEmbedding {
    pub text: String,
    pub is_injection: bool,
    pub attack_type: String,
    pub attack_type_idx: usize,
    pub source: String,
    pub embedding: Vec<f32>,
    pub embedding_dim: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub index: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("{}", "=".repeat(70));
    eprintln!("  JailGuard ONNX Embedding Generation (all-MiniLM-L6-v2)");
    eprintln!("{}", "=".repeat(70));

    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut input_path = String::from("data/combined.json");
    let mut output_path = String::from("data/combined_onnx_embeddings.json");
    let mut model_dir = String::from("models/");
    let mut batch_size: usize = 32;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" if i + 1 < args.len() => {
                input_path = args[i + 1].clone();
                i += 2;
            }
            "--output" if i + 1 < args.len() => {
                output_path = args[i + 1].clone();
                i += 2;
            }
            "--model-dir" if i + 1 < args.len() => {
                model_dir = args[i + 1].clone();
                i += 2;
            }
            "--batch-size" if i + 1 < args.len() => {
                batch_size = args[i + 1].parse().unwrap_or(32);
                i += 2;
            }
            _ => i += 1,
        }
    }

    eprintln!("\n  Configuration:");
    eprintln!("    Input:      {input_path}");
    eprintln!("    Output:     {output_path}");
    eprintln!("    Model dir:  {model_dir}");
    eprintln!("    Batch size: {batch_size}");

    // Load dataset
    eprintln!("\n  Loading samples...");
    let load_start = Instant::now();
    let file = File::open(&input_path)?;
    let reader = BufReader::new(file);
    let samples: Vec<Sample> = serde_json::from_reader(reader)?;
    eprintln!(
        "  Loaded {} samples in {:.2}s",
        samples.len(),
        load_start.elapsed().as_secs_f32()
    );

    // Initialize ONNX embedder
    eprintln!("\n  Loading ONNX model...");
    let model_start = Instant::now();
    let embedder = OnnxEmbedder::from_dir(&model_dir)?;
    eprintln!(
        "  Model loaded in {:.2}s",
        model_start.elapsed().as_secs_f32()
    );

    // Generate embeddings in batches
    eprintln!("\n  Generating embeddings...");
    eprintln!("{}", "-".repeat(70));

    let start = Instant::now();
    let total = samples.len();
    let mut embedded_samples = Vec::with_capacity(total);
    let mut processed = 0;

    for chunk_start in (0..total).step_by(batch_size) {
        let chunk_end = (chunk_start + batch_size).min(total);
        let texts: Vec<&str> = samples[chunk_start..chunk_end]
            .iter()
            .map(|s| s.text.as_str())
            .collect();

        let embeddings = embedder.embed_batch(&texts)?;

        for (i, embedding) in embeddings.into_iter().enumerate() {
            let idx = chunk_start + i;
            let sample = &samples[idx];
            embedded_samples.push(SampleWithEmbedding {
                text: sample.text.clone(),
                is_injection: sample.is_injection,
                attack_type: sample.attack_type.clone(),
                attack_type_idx: sample.attack_type_idx,
                source: sample.source.clone(),
                embedding,
                embedding_dim: 384,
                metadata: sample.metadata.clone(),
                index: idx,
            });
        }

        processed = chunk_end;

        if processed % 1000 < batch_size || processed == total {
            let elapsed = start.elapsed().as_secs_f32();
            let rate = processed as f32 / elapsed;
            let remaining = (total - processed) as f32 / rate;
            eprintln!(
                "  [{:6}/{:6}] {:.1}% | {:.1} samples/sec | ETA: {:.1}s",
                processed,
                total,
                (processed as f32 / total as f32) * 100.0,
                rate,
                remaining
            );
        }
    }

    let embed_time = start.elapsed().as_secs_f32();
    let rate = total as f32 / embed_time;
    eprintln!(
        "\n  Embedded {} samples in {:.1}s ({:.1} samples/sec)",
        total, embed_time, rate
    );

    // Save
    eprintln!("\n  Saving embeddings...");
    let save_start = Instant::now();
    let output_file = File::create(&output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer(writer, &embedded_samples)?;

    let file_size = std::fs::metadata(&output_path)?.len() as f64 / (1024.0 * 1024.0);
    eprintln!(
        "  Saved to {} ({:.1} MB) in {:.2}s",
        output_path,
        file_size,
        save_start.elapsed().as_secs_f32()
    );

    // Summary
    eprintln!("\n{}", "=".repeat(70));
    eprintln!("  COMPLETE");
    eprintln!("{}", "=".repeat(70));
    eprintln!("  Samples:    {total}");
    eprintln!("  Dimensions: 384 (all-MiniLM-L6-v2)");
    eprintln!("  Rate:       {rate:.1} samples/sec");
    eprintln!("  Total time: {embed_time:.1}s ({:.1} minutes)", embed_time / 60.0);

    Ok(())
}
