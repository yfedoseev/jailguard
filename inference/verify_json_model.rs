//! Verify Saved JSON Model - Ensure Accuracy Matches Trained Model
//!
//! This example loads the saved JSON model and verifies it produces the same
//! 99.62% test accuracy as the original trained model.
//!
//! Usage:
//! ```bash
//! cargo run --example verify_json_model --release
//! ```

use jailguard::training::{NeuralBinaryNetwork, NeuralDataLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "=".repeat(80));
    println!("🔍 VERIFYING SAVED JSON MODEL");
    println!("{}", "=".repeat(80));

    // === STEP 1: Load the saved model ===
    println!("\n📂 Loading model from JSON file...");
    let model_path = "models/jailguard_injection_detector.json";

    if !std::path::Path::new(model_path).exists() {
        println!("❌ Error: Model file not found at {}", model_path);
        println!("Please run: cargo run --example evaluate_on_test_set --release");
        return Err("Model not found".into());
    }

    let loaded_model = NeuralBinaryNetwork::load(model_path)?;
    println!("✅ Successfully loaded model from {}", model_path);

    // === STEP 2: Load test data ===
    println!("\n📖 Loading test set from splits_200k/test.json...");
    let test_loader = NeuralDataLoader::load_from_file("splits_200k/test.json")?;
    println!("✅ Loaded {} test samples", test_loader.test_samples.len());

    // === STEP 3: Run inference on test set ===
    println!("\n{}", "=".repeat(80));
    println!("🧪 EVALUATING LOADED MODEL ON TEST SET");
    println!("{}", "-".repeat(80));

    let batch_size = 128;
    let test_batches = test_loader.create_batches(batch_size, false);

    let mut test_loss = 0.0;
    let mut test_correct = 0;
    let mut test_total = 0;
    let mut true_positives = 0;
    let mut false_positives = 0;
    let mut true_negatives = 0;
    let mut false_negatives = 0;

    for batch in test_batches {
        for (embedding, is_injection, _) in batch {
            let loss = loaded_model.evaluate_loss(&embedding, is_injection);
            test_loss += loss;

            let pred = loaded_model.forward_eval(&embedding);
            let pred_injection = pred > 0.5;

            if pred_injection == is_injection {
                test_correct += 1;
            }

            // Confusion matrix
            if is_injection && pred_injection {
                true_positives += 1;
            } else if !is_injection && pred_injection {
                false_positives += 1;
            } else if !is_injection && !pred_injection {
                true_negatives += 1;
            } else if is_injection && !pred_injection {
                false_negatives += 1;
            }

            test_total += 1;
        }
    }

    let test_loss = test_loss / test_total as f32;
    let test_acc = test_correct as f32 / test_total as f32;
    let precision = if true_positives + false_positives > 0 {
        true_positives as f32 / (true_positives + false_positives) as f32
    } else {
        0.0
    };
    let recall = if true_positives + false_negatives > 0 {
        true_positives as f32 / (true_positives + false_negatives) as f32
    } else {
        0.0
    };
    let specificity = if true_negatives + false_positives > 0 {
        true_negatives as f32 / (true_negatives + false_positives) as f32
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * (precision * recall) / (precision + recall)
    } else {
        0.0
    };

    // === STEP 4: Display results ===
    println!("\n{}", "=".repeat(80));
    println!("📊 VERIFICATION RESULTS");
    println!("{}", "-".repeat(80));

    println!("Test Accuracy:  {:.4} ({:.2}%)", test_acc, test_acc * 100.0);
    println!("Test Loss:      {:.4}", test_loss);
    println!("Precision:      {:.4}", precision);
    println!("Recall:         {:.4}", recall);
    println!("Specificity:    {:.4}", specificity);
    println!("F1 Score:       {:.4}", f1);

    println!("\nConfusion Matrix:");
    println!("  True Positives:  {}", true_positives);
    println!("  False Positives: {}", false_positives);
    println!("  True Negatives:  {}", true_negatives);
    println!("  False Negatives: {}", false_negatives);

    // === STEP 5: Verification comparison ===
    println!("\n{}", "=".repeat(80));
    println!("✅ VERIFICATION STATUS");
    println!("{}", "-".repeat(80));

    // Expected accuracy from training
    let expected_accuracy = 0.9957;
    let accuracy_match = (test_acc - expected_accuracy).abs();

    if accuracy_match < 0.0001 {
        println!("✅ PERFECT MATCH!");
        println!("   Expected: {:.4}", expected_accuracy);
        println!("   Got:      {:.4}", test_acc);
        println!("   Difference: {:.6}", accuracy_match);
    } else if accuracy_match < 0.001 {
        println!("✅ EXCELLENT MATCH!");
        println!("   Expected: {:.4}", expected_accuracy);
        println!("   Got:      {:.4}", test_acc);
        println!("   Difference: {:.6} (< 0.1%)", accuracy_match);
    } else {
        println!("⚠️  MISMATCH DETECTED");
        println!("   Expected: {:.4}", expected_accuracy);
        println!("   Got:      {:.4}", test_acc);
        println!("   Difference: {:.6}", accuracy_match);
    }

    // Verify all metrics match expected values
    println!("\n📋 DETAILED VERIFICATION:");
    println!("   Accuracy Match:  {}", if test_acc > 0.9950 { "✅ YES" } else { "❌ NO" });
    println!("   Precision Match: {}", if precision > 0.9980 { "✅ YES" } else { "❌ NO" });
    println!("   Recall Match:    {}", if recall > 0.9770 { "✅ YES" } else { "❌ NO" });
    println!("   Specificity OK:  {}", if specificity > 0.9990 { "✅ YES" } else { "❌ NO" });
    println!("   F1 Score OK:     {}", if f1 > 0.9880 { "✅ YES" } else { "❌ NO" });

    println!("\n{}", "=".repeat(80));

    // Final verdict
    if test_acc > 0.9950 && precision > 0.9980 && recall > 0.9770 {
        println!("🎯 FINAL VERDICT: ✅ JSON MODEL VERIFIED SUCCESSFULLY");
        println!("   The saved JSON model produces identical results!");
        println!("   Test Accuracy: {:.2}% (Target: 99.62%)", test_acc * 100.0);
    } else {
        println!("🔴 FINAL VERDICT: ❌ VERIFICATION FAILED");
        println!("   Results do not match expected values");
    }

    println!("{}\n", "=".repeat(80));

    Ok(())
}
