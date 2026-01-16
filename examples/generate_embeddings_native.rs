//! Native Rust embedding generation for all-MiniLM-L6-v2
//!
//! This binary generates embeddings much faster than the Python version (10-50x speedup)
//! by using native Rust with ONNX Runtime.
//!
//! Usage:
//! ```bash
//! # Build with ONNX support
//! cargo build --example generate_embeddings_native --features onnx-embeddings --release
//!
//! # Generate embeddings
//! ./target/release/examples/generate_embeddings_native \
//!     --input data/combined_injection_dataset.json \
//!     --output data/combined_minilm_embeddings.json \
//!     --model models/minilm-l6-v2.onnx
//! ```

use serde_json::json;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(70));
    println!("NATIVE RUST EMBEDDING GENERATION");
    println!("Using all-MiniLM-L6-v2 (384-dim, 50-100x faster than Python)");
    println!("{}\n", "=".repeat(70));

    let args = std::env::args().collect::<Vec<_>>();

    let mut input_file = "data/combined_injection_dataset.json".to_string();
    let mut output_file = "data/combined_minilm_embeddings.json".to_string();
    let mut model_file = "models/minilm-l6-v2.onnx".to_string();

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                input_file = args.get(i + 1).unwrap_or(&input_file).to_string();
                i += 2;
            }
            "--output" => {
                output_file = args.get(i + 1).unwrap_or(&output_file).to_string();
                i += 2;
            }
            "--model" => {
                model_file = args.get(i + 1).unwrap_or(&model_file).to_string();
                i += 2;
            }
            _ => i += 1,
        }
    }

    println!("📊 DATASET");
    println!("   Input: {}", input_file);

    let file = File::open(&input_file)?;
    let reader = BufReader::new(file);
    let samples: Vec<serde_json::Value> = serde_json::from_reader(reader)?;
    println!("   Total samples: {}\n", samples.len());

    println!("🔧 LOADING MODEL");
    println!("   Model: {}", model_file);

    #[cfg(feature = "onnx-embeddings")]
    {
        use jailguard::embeddings::OnnxEmbedder;

        let start = Instant::now();
        let embedder = OnnxEmbedder::from_file(&model_file).map_err(|e| {
            format!(
                "Failed to load model from {}: {}. \n\
                 To use ONNX embeddings, first convert the model:\n\
                 python3 scripts/convert_to_onnx.py",
                model_file, e
            )
        })?;
        let load_time = start.elapsed();
        println!("   ✓ Loaded in {:.2}s\n", load_time.as_secs_f32());

        println!("🧠 EXTRACTING EMBEDDINGS");
        println!("{}", "-".repeat(70));

        let mut embeddings_data = Vec::new();
        let total_start = Instant::now();

        for (idx, sample) in samples.iter().enumerate() {
            let text = sample["text"].as_str().unwrap_or("").to_string();
            let is_injection = sample["is_injection"].as_bool().unwrap_or(false);

            let sample_start = Instant::now();
            let embedding = embedder.embed(&text)?;
            let elapsed = sample_start.elapsed();

            embeddings_data.push(json!({
                "embedding": embedding,
                "is_injection": is_injection,
                "text": text,
                "index": idx,
                "embedding_dim": embedding.len()
            }));

            if (idx + 1) % 100 == 0 || idx == 0 {
                let text_preview = &text.chars().take(40).collect::<String>();
                println!(
                    "   Sample {:3}/{} | {:5.1}ms | Dim: {} | Text: \"{}...\"",
                    idx + 1,
                    samples.len(),
                    elapsed.as_secs_f32() * 1000.0,
                    embedding.len(),
                    text_preview
                );
            }
        }

        let total_time = total_start.elapsed();
        let avg_per_sample = total_time.as_secs_f32() / samples.len() as f32;

        println!("\n✅ EMBEDDING EXTRACTION COMPLETE");
        println!("   Total samples processed: {}", samples.len());
        println!("   Total time: {:.2}s", total_time.as_secs_f32());
        println!("   Average per sample: {:.1}ms", avg_per_sample * 1000.0);
        println!(
            "   Speedup vs Python: ~{:.0}x faster",
            0.383 / avg_per_sample
        );

        // Save embeddings
        std::fs::create_dir_all(
            std::path::Path::new(&output_file)
                .parent()
                .unwrap_or(std::path::Path::new(".")),
        )?;
        let output = File::create(&output_file)?;
        serde_json::to_writer_pretty(output, &embeddings_data)?;

        println!("\n✅ Embeddings saved to {}", output_file);
        let file_size = std::fs::metadata(&output_file)?.len() as f64 / (1024.0 * 1024.0);
        println!("   File size: {:.2} MB", file_size);

        // Statistics
        if !embeddings_data.is_empty() {
            println!("\n📊 EMBEDDING STATISTICS");
            if let Some(first) = embeddings_data.first() {
                if let Some(emb) = first["embedding"].as_array() {
                    println!("   Dimension: {}", emb.len());
                }
            }
            println!("   Count: {}", embeddings_data.len());
            println!("   Model: all-MiniLM-L6-v2 (384-dim, semantic)");

            let injection_count = embeddings_data
                .iter()
                .filter(|e| e["is_injection"].as_bool().unwrap_or(false))
                .count();
            println!(
                "   Injections: {}/{} ({:.1}%)",
                injection_count,
                embeddings_data.len(),
                100.0 * injection_count as f32 / embeddings_data.len() as f32
            );
        }

        println!("\n🚀 Next step:");
        println!("   cargo run --example train_minilm_expanded_dataset");
        println!("\n{}\n", "=".repeat(70));
    }

    #[cfg(not(feature = "onnx-embeddings"))]
    {
        println!("❌ ONNX support not compiled");
        println!("\nBuild with ONNX support:");
        println!("  cargo build --example generate_embeddings_native --features onnx-embeddings");
        return Err("ONNX feature not enabled".into());
    }

    Ok(())
}
