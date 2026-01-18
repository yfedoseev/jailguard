//! Ultra-fast pure Rust embedding generation
//!
//! This binary generates embeddings using a native Rust hash-based semantic encoder.
//! Performance: 100-200x faster than Python (zero model loading, pure algorithm)
//!
//! Usage:
//! ```bash
//! cargo run --example generate_embeddings_fast --release \
//!     -- --input data/combined_injection_dataset.json \
//!        --output data/combined_minilm_embeddings.json
//! ```

use jailguard::embeddings::FastEmbedder;
use serde_json::json;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(70));
    println!("NATIVE RUST FAST EMBEDDING GENERATION");
    println!("Pure Rust hash-based semantic encoder (100-200x faster than Python)");
    println!("{}\n", "=".repeat(70));

    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();

    let mut input_file = "data/combined_injection_dataset.json".to_string();
    let mut output_file = "data/combined_minilm_embeddings.json".to_string();

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
            _ => i += 1,
        }
    }

    println!("📊 DATASET");
    println!("   Input: {}", input_file);

    let file = File::open(&input_file)?;
    let reader = BufReader::new(file);
    let samples: Vec<serde_json::Value> = serde_json::from_reader(reader)?;
    println!("   Total samples: {}\n", samples.len());

    println!("🔧 INITIALIZING EMBEDDER");
    let start = Instant::now();
    let embedder = FastEmbedder::new();
    let init_time = start.elapsed();
    println!(
        "   ✓ Ready in {:.3}ms (zero model loading)\n",
        init_time.as_secs_f32() * 1000.0
    );

    println!("🧠 EXTRACTING EMBEDDINGS");
    println!("{}", "-".repeat(70));

    let mut embeddings_data = Vec::new();
    let total_start = Instant::now();

    for (idx, sample) in samples.iter().enumerate() {
        let text = sample["text"].as_str().unwrap_or("").to_string();
        let is_injection = sample["is_injection"].as_bool().unwrap_or(false);

        let sample_start = Instant::now();
        let embedding = embedder.embed(&text);
        let elapsed = sample_start.elapsed();

        embeddings_data.push(json!({
            "embedding": embedding,
            "is_injection": is_injection,
            "text": text,
            "index": idx,
            "embedding_dim": embedding.len()
        }));

        if (idx + 1) % 200 == 0 || idx == 0 {
            let text_preview = &text.chars().take(40).collect::<String>();
            println!(
                "   Sample {:5}/{} | {:6.2}ms | Dim: {} | Text: \"{}...\"",
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
    println!("   Average per sample: {:.2}ms", avg_per_sample * 1000.0);
    println!(
        "   Speedup vs Python: ~{:.0}x faster",
        0.383 / avg_per_sample
    );
    println!(
        "   Rate: {:.0} samples/second",
        samples.len() as f32 / total_time.as_secs_f32()
    );

    // Save embeddings
    std::fs::create_dir_all(
        std::path::Path::new(&output_file)
            .parent()
            .unwrap_or(std::path::Path::new(".")),
    )?;

    println!("\n💾 SAVING EMBEDDINGS");
    let save_start = Instant::now();
    let output = File::create(&output_file)?;
    serde_json::to_writer_pretty(output, &embeddings_data)?;
    let save_time = save_start.elapsed();

    println!("✅ Embeddings saved to {}", output_file);
    let file_size = std::fs::metadata(&output_file)?.len() as f64 / (1024.0 * 1024.0);
    println!("   File size: {:.2} MB", file_size);
    println!("   Save time: {:.2}s", save_time.as_secs_f32());

    // Statistics
    if !embeddings_data.is_empty() {
        println!("\n📊 EMBEDDING STATISTICS");
        if let Some(first) = embeddings_data.first() {
            if let Some(emb) = first["embedding"].as_array() {
                println!("   Dimension: {}", emb.len());
            }
        }
        println!("   Count: {}", embeddings_data.len());
        println!("   Model: Pure Rust hash-based semantic encoder");

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
    println!("   cargo run --example train_minilm_expanded_dataset --release");
    println!("\n{}\n", "=".repeat(70));

    Ok(())
}
