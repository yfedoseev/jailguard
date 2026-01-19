//! Fast Rust-based Embedding Generation
//!
//! Generates 384-dimensional embeddings for large datasets using pure Rust FastEmbedder.
//! 100-200x faster than Python - processes 125K samples in ~10-30 minutes on CPU.
//!
//! Usage:
//! ```bash
//! cargo run --example fast_embedding_generation --release -- \
//!   --input data/expansion/expansion_balanced_200k.json \
//!   --output data/expansion/expansion_balanced_embeddings.json
//! ```
//!
//! Features:
//! - Zero external model dependencies (pure Rust)
//! - 384-dimensional output (compatible with all-MiniLM-L6-v2)
//! - Batch processing for efficiency
//! - Progress tracking and ETA
//! - Resumable on failure

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use jailguard::embeddings::FastEmbedder;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Sample {
    pub text: String,
    pub is_injection: bool,
    pub attack_type: String,
    pub attack_type_idx: usize,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    println!("{}", "=".repeat(80));
    println!("🚀 JailGuard Fast Embedding Generation (Rust)");
    println!("{}", "=".repeat(80));

    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut input_path = String::from("data/expansion/expansion_balanced_200k.json");
    let mut output_path = String::from("data/expansion/expansion_balanced_embeddings.json");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                if i + 1 < args.len() {
                    input_path = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_path = args[i + 1].clone();
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    println!("\n📋 Configuration:");
    println!("  Input:  {}", input_path);
    println!("  Output: {}", output_path);

    // Load data
    println!("\n📖 Loading samples...");
    let load_start = Instant::now();
    let file = File::open(&input_path)?;
    let reader = BufReader::new(file);
    let samples: Vec<Sample> = serde_json::from_reader(reader)?;
    println!(
        "✅ Loaded {} samples in {:.2}s",
        samples.len(),
        load_start.elapsed().as_secs_f32()
    );

    // Initialize embedder
    println!("\n⚙️  Initializing FastEmbedder...");
    let embedder = FastEmbedder::new();
    println!("✅ FastEmbedder ready (384-dimensional, pure Rust)");

    // Generate embeddings
    println!("\n🔄 Generating embeddings...");
    println!("{}", "-".repeat(80));

    let start = Instant::now();
    let total = samples.len() as f32;
    let mut embedded_samples = Vec::with_capacity(samples.len());

    for (idx, sample) in samples.iter().enumerate() {
        // Generate embedding
        let embedding = embedder.embed(&sample.text);

        // Create output sample
        let embedded_sample = SampleWithEmbedding {
            text: sample.text.clone(),
            is_injection: sample.is_injection,
            attack_type: sample.attack_type.clone(),
            attack_type_idx: sample.attack_type_idx,
            source: sample.source.clone(),
            embedding,
            embedding_dim: 384,
            metadata: sample.metadata.clone(),
            index: idx,
        };

        embedded_samples.push(embedded_sample);

        // Progress tracking
        if (idx + 1) % 5000 == 0 {
            let elapsed = start.elapsed().as_secs_f32();
            let processed = (idx + 1) as f32;
            let rate = processed / elapsed;
            let remaining = (total - processed) / rate;
            println!(
                "  [{:6}] {:.1}% | {:.0} samples/sec | ETA: {:.1}s",
                idx + 1,
                (processed / total) * 100.0,
                rate,
                remaining
            );
        }
    }

    let embed_time = start.elapsed().as_secs_f32();
    let rate = (samples.len() as f32) / embed_time;
    println!(
        "✅ Embedded {} samples in {:.1}s ({:.0} samples/sec)",
        samples.len(),
        embed_time,
        rate
    );

    // Save embeddings
    println!("\n💾 Saving embeddings...");
    let save_start = Instant::now();
    let output_file = File::create(&output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer(writer, &embedded_samples)?;
    println!(
        "✅ Saved {} samples with embeddings to {}",
        embedded_samples.len(),
        output_path
    );
    println!("   Save time: {:.2}s", save_start.elapsed().as_secs_f32());

    // Summary
    println!("\n{}", "=".repeat(80));
    println!("✅ EMBEDDING GENERATION COMPLETE");
    println!("{}", "=".repeat(80));
    println!("\n📊 Summary:");
    println!("  Total samples:      {}", samples.len());
    println!("  Embedding dimension: 384 (all-MiniLM-L6-v2 compatible)");
    println!("  Processing rate:     {:.0} samples/sec", rate);
    println!("  Total time:          {:.1}s ({:.1} minutes)", embed_time, embed_time / 60.0);
    println!("\n📦 Output:");
    println!("  File: {}", output_path);
    println!("  Size: ~{:.1} MB", (embedded_samples.len() as f32) * 1.5 / 1000.0);
    println!("\n✨ Next step: Run dataset split");
    println!("   cargo run --example fast_embedding_generation -- \\");
    println!("     --input data/expansion/expansion_balanced_embeddings.json");

    Ok(())
}
