/// Fast embedding pre-computation using basic Detector (128-dim)
use jailguard::detection::Detector;
use std::fs;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let separator = "=".repeat(70);

    println!("\n{}", separator);
    println!("FAST EMBEDDING PRE-COMPUTATION");
    println!("Using basic Detector (128-dim embeddings)");
    println!("{}\n", separator);

    // Load dataset
    let data_path = Path::new("data/prompt_injections_real.json");
    if !data_path.exists() {
        println!("⚠️  Dataset not found at {}", data_path.display());
        return Ok(());
    }

    let data_str = fs::read_to_string(data_path)?;
    let samples: Vec<serde_json::Value> = serde_json::from_str(&data_str)?;

    println!("📊 DATASET");
    println!("   Total samples: {}\n", samples.len());

    // Initialize basic detector (faster than transformer detector)
    println!("🔧 Initializing Detector...");
    let start = Instant::now();
    let detector = Detector::new()?;
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

        // Store detection result with label
        embeddings_data.push(serde_json::json!({
            "is_injection": is_injection,
            "prediction": result.is_injection,
            "confidence": result.confidence,
            "text": text,
            "index": idx,
            "accuracy": if result.is_injection == is_injection { 1 } else { 0 }
        }));

        if (idx + 1) % 100 == 0 || idx == 0 {
            println!(
                "   Sample {:3}/{} | {:.3}s | Conf: {:.2} | Text: \"{}...\"",
                idx + 1,
                samples.len(),
                elapsed.as_secs_f32(),
                result.confidence,
                &text[..text.len().min(40)]
            );
        }
    }

    let total_time = total_start.elapsed();

    println!("\n✅ EMBEDDING EXTRACTION COMPLETE");
    println!("   Total samples processed: {}", samples.len());
    println!("   Total time: {:.2}s", total_time.as_secs_f32());
    println!(
        "   Average per sample: {:.3}s",
        total_time.as_secs_f32() / samples.len() as f32
    );
    println!(
        "   Estimated for 1M samples: {:.0} seconds ({:.1} hours)",
        (total_time.as_secs_f32() / samples.len() as f32) * 1_000_000.0,
        (total_time.as_secs_f32() / samples.len() as f32) * 1_000_000.0 / 3600.0
    );

    // Save embeddings
    let output_file = "data/detector_embeddings.json";
    fs::write(output_file, serde_json::to_string_pretty(&embeddings_data)?)?;

    println!("\n✅ Embeddings saved to {}", output_file);
    let file_size = fs::metadata(output_file)?.len() as f32 / (1024.0 * 1024.0);
    println!("   File size: {:.2} MB", file_size);

    // Verify embedding statistics
    if let Some(first) = embeddings_data.first() {
        if let Some(emb) = first["embedding"].as_array() {
            println!("\n📊 EMBEDDING STATISTICS");
            println!("   Dimension: {}", emb.len());
            println!("   Count: {}", embeddings_data.len());
            println!("   Format: JSON array with embedding, label, text, index");
        }
    }

    println!("\n🚀 Next step:");
    println!("   cargo run --example train_with_embeddings");

    println!("\n{}\n", separator);

    Ok(())
}
