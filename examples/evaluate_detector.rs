/// Evaluate RealDetector on test dataset and compute accuracy metrics
use jailguard::api::RealDetector;
use std::fs;
use std::path::Path;

fn main() {
    println!("\n{}", "=".repeat(80));
    println!("RealDetector Evaluation on Test Dataset");
    println!("{}\n", "=".repeat(80));

    // Load test dataset
    let test_path = Path::new("data/training/splits/test.json");
    if !test_path.exists() {
        eprintln!("❌ Test data not found at {:?}", test_path);
        eprintln!("Run from project root: cargo run --example evaluate_detector");
        std::process::exit(1);
    }

    let test_str = fs::read_to_string(test_path).expect("Failed to read test data");
    let samples: Vec<serde_json::Value> =
        serde_json::from_str(&test_str).expect("Failed to parse test data JSON");

    println!("📊 Dataset Information");
    println!("{}", "-".repeat(80));
    println!("Test samples loaded: {}", samples.len());

    let injection_count = samples
        .iter()
        .filter(|s| s["is_injection"].as_bool().unwrap_or(false))
        .count();
    let benign_count = samples.len() - injection_count;

    println!("  - Injections: {}", injection_count);
    println!("  - Benign: {}\n", benign_count);

    // Initialize detector
    println!("🚀 Running Detection");
    println!("{}", "-".repeat(80));

    let detector = RealDetector::new();
    let mut results = Vec::new();

    // Run detector on all samples
    for (idx, sample) in samples.iter().enumerate() {
        let text = sample["text"].as_str().unwrap_or("");
        let is_injection = sample["is_injection"].as_bool().unwrap_or(false);

        let detection = detector.detect(text);

        results.push((is_injection, detection.is_injection, detection.confidence));

        let label = if is_injection { "INJ" } else { "BEN" };
        let pred_label = if detection.is_injection { "INJ" } else { "BEN" };
        let status = if is_injection == detection.is_injection {
            "✓"
        } else {
            "✗"
        };

        println!(
            "  {} Sample {:2}: [{}] Predicted: [{}] Conf: {:.2} - {}",
            status,
            idx + 1,
            label,
            pred_label,
            detection.confidence,
            &text[..text.len().min(50)]
        );
    }

    // Compute confusion matrix
    println!("\n{}", "=".repeat(80));
    println!("📈 Evaluation Results");
    println!("{}\n", "=".repeat(80));

    let mut tp = 0; // True Positives: correctly detected injections
    let mut fp = 0; // False Positives: benign marked as injection
    let mut tn = 0; // True Negatives: correctly identified benign
    let mut fn_ = 0; // False Negatives: missed injections

    for (actual, predicted, _confidence) in &results {
        if *actual && *predicted {
            tp += 1;
        } else if !*actual && *predicted {
            fp += 1;
        } else if !*actual && !*predicted {
            tn += 1;
        } else if *actual && !*predicted {
            fn_ += 1;
        }
    }

    let total = results.len();
    let accuracy = (tp + tn) as f32 / total as f32;
    let precision = if tp + fp > 0 {
        tp as f32 / (tp + fp) as f32
    } else {
        0.0
    };
    let recall = if tp + fn_ > 0 {
        tp as f32 / (tp + fn_) as f32
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * (precision * recall) / (precision + recall)
    } else {
        0.0
    };

    // Specificity and sensitivity for binary classification
    let sensitivity = recall; // Same as recall for binary classification
    let specificity = if tn + fp > 0 {
        tn as f32 / (tn + fp) as f32
    } else {
        0.0
    };

    println!("📊 Confusion Matrix:");
    println!("  ┌─────────────────┬──────────────┬──────────────┐");
    println!("  │                 │ Pred: Inject │ Pred: Benign │");
    println!("  ├─────────────────┼──────────────┼──────────────┤");
    println!(
        "  │ Actual: Inject  │      {:2}      │      {:2}      │",
        tp, fn_
    );
    println!(
        "  │ Actual: Benign  │      {:2}      │      {:2}      │",
        fp, tn
    );
    println!("  └─────────────────┴──────────────┴──────────────┘");

    println!("\n📊 Metrics:");
    println!(
        "  Accuracy:   {:.2}% ({}/{})",
        accuracy * 100.0,
        tp + tn,
        total
    );
    println!(
        "  Precision:  {:.2}% (TP / (TP + FP) = {} / {})",
        precision * 100.0,
        tp,
        tp + fp
    );
    println!(
        "  Recall:     {:.2}% (TP / (TP + FN) = {} / {})",
        recall * 100.0,
        tp,
        tp + fn_
    );
    println!(
        "  Sensitivity:{:.2}% (True Positive Rate)",
        sensitivity * 100.0
    );
    println!(
        "  Specificity:{:.2}% (True Negative Rate)",
        specificity * 100.0
    );
    println!("  F1 Score:   {:.2}", f1);

    // Per-class metrics
    println!("\n📊 Per-Class Metrics:");
    if injection_count > 0 {
        println!(
            "  Injection Detection Rate: {:.2}% ({}/{})",
            sensitivity * 100.0,
            tp,
            injection_count
        );
    }
    if benign_count > 0 {
        println!(
            "  Benign Specificity:       {:.2}% ({}/{})",
            specificity * 100.0,
            tn,
            benign_count
        );
    }

    // Confidence analysis
    println!("\n📊 Confidence Distribution:");
    let injection_results: Vec<_> = results.iter().filter(|(actual, _, _)| *actual).collect();
    let benign_results: Vec<_> = results.iter().filter(|(actual, _, _)| !*actual).collect();

    if !injection_results.is_empty() {
        let avg_conf: f32 = injection_results.iter().map(|(_, _, c)| c).sum::<f32>()
            / injection_results.len() as f32;
        println!("  Injection avg confidence: {:.3}", avg_conf);
    }

    if !benign_results.is_empty() {
        let avg_conf: f32 =
            benign_results.iter().map(|(_, _, c)| c).sum::<f32>() / benign_results.len() as f32;
        println!("  Benign avg confidence:    {:.3}", avg_conf);
    }

    println!("\n{}", "=".repeat(80));
    println!("✅ Evaluation Complete!");
    println!("{}\n", "=".repeat(80));
}
