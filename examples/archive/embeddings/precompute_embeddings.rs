/// Pre-compute real transformer embeddings for all dataset samples
/// This extracts 256-dimensional embeddings from the transformer
/// and saves them for fast training
use jailguard::detection::TransformerDetector;
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let separator = "=".repeat(70);

    println!("\n{}", separator);
    println!("PRE-COMPUTING REAL TRANSFORMER EMBEDDINGS");
    println!("Extracting 256-dim vectors from transformer encoder");
    println!("{}\n", separator);

    // Load dataset
    let data_path = Path::new("data/prompt_injections_real.json");
    if !data_path.exists() {
        println!("⚠️  Dataset not found");
        return Ok(());
    }

    let data_str = fs::read_to_string(data_path)?;
    let samples: Vec<serde_json::Value> = serde_json::from_str(&data_str)?;

    println!("📊 DATASET");
    println!("   Total samples: {}\n", samples.len());

    // Initialize transformer
    println!("🔧 Initializing Transformer Detector...");
    let start = Instant::now();
    let detector = TransformerDetector::new()?;
    println!(
        "   ✓ Initialized in {:.2}s\n",
        start.elapsed().as_secs_f32()
    );

    // Pre-compute embeddings
    println!("🧠 EXTRACTING EMBEDDINGS");
    println!("{}", "-".repeat(70));

    let mut embeddings_data = Vec::new();
    let total_start = Instant::now();

    for (idx, sample) in samples.iter().enumerate() {
        let text = sample["text"].as_str().unwrap_or("");
        let is_injection = sample["is_injection"].as_bool().unwrap_or(false);

        let sample_start = Instant::now();
        let result = detector.detect(text);
        let elapsed = sample_start.elapsed();

        // Store embedding with label
        embeddings_data.push(serde_json::json!({
            "embedding": result.embedding,
            "is_injection": is_injection,
            "text": text,
            "index": idx
        }));

        if (idx + 1) % 50 == 0 || idx == 0 {
            println!(
                "   Sample {:3}/{} | {:.1}s | Text: \"{}...\"",
                idx + 1,
                samples.len(),
                elapsed.as_secs_f32(),
                &text[..text.len().min(40)]
            );
        }
    }

    let total_time = total_start.elapsed();

    println!("\n✅ EMBEDDING EXTRACTION COMPLETE");
    println!("   Total samples processed: {}", samples.len());
    println!("   Total time: {:.2}s", total_time.as_secs_f32());
    println!(
        "   Average per sample: {:.2}s",
        total_time.as_secs_f32() / samples.len() as f32
    );

    // Save embeddings
    let output_file = "data/transformer_embeddings.json";
    fs::write(output_file, serde_json::to_string_pretty(&embeddings_data)?)?;

    println!("\n✅ Embeddings saved to {}", output_file);
    println!(
        "   File size: {:.2} MB",
        fs::metadata(output_file)?.len() as f32 / 1_000_000.0
    );

    println!("\n📝 EMBEDDING STATISTICS");
    println!("   Dimension: 256");
    println!("   Count: {}", embeddings_data.len());
    println!("   Format: JSON array with embedding, label, text, index");

    println!("\n🚀 Next step:");
    println!("   cargo run --example train_with_real_embeddings");

    println!("\n{}\n", separator);

    Ok(())
}
