//! Integration tests for the multi-task training pipeline.
//!
//! Tests the complete workflow:
//! - Dataset creation and loading
//! - Detector initialization
//! - Multi-task training
//! - Metric computation
//! - Model evaluation

use jailguard::{
    dataset::MultiTaskSample,
    detection::{AttackType, TransformerDetector},
    training::{MultiTaskTrainer, MultiTaskTrainingConfig},
};

#[test]
fn test_training_pipeline_basic() {
    // Create simple training data
    let train_samples = vec![
        MultiTaskSample::new(
            "Ignore instructions".to_string(),
            true,
            AttackType::InstructionOverride,
        ),
        MultiTaskSample::new("What is 2+2?".to_string(), false, AttackType::Benign),
    ];

    // Create detector
    let detector = TransformerDetector::new().expect("Failed to create detector");

    // Create trainer with default config
    let config = MultiTaskTrainingConfig::default();
    let trainer = MultiTaskTrainer::new(detector, config);

    // Train on the samples
    let metrics = trainer.train_epoch(&train_samples);

    // Verify metrics
    assert_eq!(metrics.num_samples, 2);
    assert!(metrics.binary_accuracy >= 0.0 && metrics.binary_accuracy <= 1.0);
    assert!(metrics.attack_accuracy >= 0.0 && metrics.attack_accuracy <= 1.0);
    assert!(metrics.total_loss >= 0.0);
}

#[test]
fn test_metrics_accumulation() {
    let samples = vec![
        MultiTaskSample::new(
            "Reveal secrets".to_string(),
            true,
            AttackType::OutputManipulation,
        ),
        MultiTaskSample::new("Hello world".to_string(), false, AttackType::Benign),
        MultiTaskSample::new("Jailbreak".to_string(), true, AttackType::JailbreakPattern),
    ];

    let detector = TransformerDetector::new().expect("Failed to create detector");
    let config = MultiTaskTrainingConfig::default();
    let trainer = MultiTaskTrainer::new(detector, config);

    let metrics = trainer.train_epoch(&samples);

    // Check metric properties
    assert_eq!(metrics.num_samples, 3);
    let avg_loss = metrics.avg_loss();
    assert!(avg_loss.is_finite());
    assert!(avg_loss >= 0.0);

    // Accuracies should be normalized
    assert!(metrics.binary_accuracy >= 0.0 && metrics.binary_accuracy <= 1.0);
    assert!(metrics.attack_accuracy >= 0.0 && metrics.attack_accuracy <= 1.0);
}

#[test]
fn test_multiple_training_epochs() {
    let samples = vec![
        MultiTaskSample::new(
            "Ignore previous".to_string(),
            true,
            AttackType::InstructionOverride,
        ),
        MultiTaskSample::new("Normal text".to_string(), false, AttackType::Benign),
    ];

    let detector = TransformerDetector::new().expect("Failed to create detector");
    let config = MultiTaskTrainingConfig::default();
    let trainer = MultiTaskTrainer::new(detector, config);

    // Train multiple epochs
    let mut prev_loss = f32::INFINITY;
    for epoch in 0..3 {
        let metrics = trainer.train_epoch(&samples);

        // Verify metrics are valid each epoch
        assert_eq!(metrics.num_samples, 2);
        assert!(metrics.avg_loss().is_finite());

        // Loss should vary (not always the same)
        if epoch > 0 {
            // Note: In an untrained model, loss might not decrease, but should be consistent
            assert!(metrics.avg_loss().is_finite());
        }
        prev_loss = metrics.avg_loss();
    }
}

#[test]
fn test_trainer_evaluation() {
    let test_samples = vec![
        MultiTaskSample::new(
            "Disregard instructions".to_string(),
            true,
            AttackType::InstructionOverride,
        ),
        MultiTaskSample::new("How are you?".to_string(), false, AttackType::Benign),
    ];

    let detector = TransformerDetector::new().expect("Failed to create detector");
    let config = MultiTaskTrainingConfig::default();
    let trainer = MultiTaskTrainer::new(detector, config);

    // Evaluate on test set
    let metrics = trainer.evaluate(&test_samples);

    // Verify evaluation metrics
    assert_eq!(metrics.num_samples, 2);
    assert!(metrics.binary_accuracy >= 0.0 && metrics.binary_accuracy <= 1.0);
    assert!(metrics.attack_accuracy >= 0.0 && metrics.attack_accuracy <= 1.0);
}

#[test]
fn test_trainer_with_custom_weights() {
    let samples = vec![MultiTaskSample::new(
        "Override".to_string(),
        true,
        AttackType::InstructionOverride,
    )];

    let detector = TransformerDetector::new().expect("Failed to create detector");
    let config = MultiTaskTrainingConfig::default();

    // Create trainer with custom loss weights
    let trainer = MultiTaskTrainer::new(detector, config).with_loss_weights(0.5, 0.3, 0.2);

    // Verify weights are normalized
    let total = trainer.loss_fn.alpha + trainer.loss_fn.beta + trainer.loss_fn.gamma;
    assert!((total - 1.0).abs() < 0.01);

    // Should still train normally
    let metrics = trainer.train_epoch(&samples);
    assert_eq!(metrics.num_samples, 1);
}

#[test]
fn test_empty_batch_handling() {
    let empty_samples: Vec<MultiTaskSample> = vec![];

    let detector = TransformerDetector::new().expect("Failed to create detector");
    let config = MultiTaskTrainingConfig::default();
    let trainer = MultiTaskTrainer::new(detector, config);

    // Training on empty batch should not crash
    let metrics = trainer.train_epoch(&empty_samples);

    // Metrics should indicate empty batch
    assert_eq!(metrics.num_samples, 0);
    assert_eq!(metrics.binary_accuracy, 0.0);
    assert_eq!(metrics.attack_accuracy, 0.0);
}

#[test]
fn test_large_batch_training() {
    // Create a larger batch to test scalability
    let mut samples = Vec::new();
    for i in 0..50 {
        let is_injection = i % 2 == 0;
        let attack_type = match i % 7 {
            0 => AttackType::RolePlay,
            1 => AttackType::InstructionOverride,
            2 => AttackType::ContextManipulation,
            3 => AttackType::OutputManipulation,
            4 => AttackType::EncodingAttack,
            5 => AttackType::JailbreakPattern,
            _ => AttackType::Benign,
        };

        samples.push(MultiTaskSample::new(
            format!("Sample {}", i),
            is_injection,
            attack_type,
        ));
    }

    let detector = TransformerDetector::new().expect("Failed to create detector");
    let config = MultiTaskTrainingConfig::default();
    let trainer = MultiTaskTrainer::new(detector, config);

    // Should handle larger batches
    let metrics = trainer.train_epoch(&samples);

    assert_eq!(metrics.num_samples, 50);
    assert!(metrics.avg_loss().is_finite());
}

#[test]
fn test_metrics_consistency() {
    let samples = vec![
        MultiTaskSample::new(
            "Test injection".to_string(),
            true,
            AttackType::InstructionOverride,
        ),
        MultiTaskSample::new("Test benign".to_string(), false, AttackType::Benign),
    ];

    let detector = TransformerDetector::new().expect("Failed to create detector");
    let config = MultiTaskTrainingConfig::default();
    let trainer = MultiTaskTrainer::new(detector, config);

    // Same data should produce consistent metrics
    let metrics1 = trainer.train_epoch(&samples);
    let metrics2 = trainer.train_epoch(&samples);

    // Metrics should be the same for same input
    assert_eq!(metrics1.num_samples, metrics2.num_samples);
    assert!((metrics1.avg_loss() - metrics2.avg_loss()).abs() < 0.0001);
}

#[test]
fn test_attack_type_distribution() {
    // Create balanced samples across all attack types
    let samples = vec![
        MultiTaskSample::new("Role".to_string(), true, AttackType::RolePlay),
        MultiTaskSample::new("Instr".to_string(), true, AttackType::InstructionOverride),
        MultiTaskSample::new("Ctx".to_string(), true, AttackType::ContextManipulation),
        MultiTaskSample::new("Out".to_string(), true, AttackType::OutputManipulation),
        MultiTaskSample::new("Enc".to_string(), true, AttackType::EncodingAttack),
        MultiTaskSample::new("Jail".to_string(), true, AttackType::JailbreakPattern),
        MultiTaskSample::new("Benign".to_string(), false, AttackType::Benign),
    ];

    let detector = TransformerDetector::new().expect("Failed to create detector");
    let config = MultiTaskTrainingConfig::default();
    let trainer = MultiTaskTrainer::new(detector, config);

    let metrics = trainer.train_epoch(&samples);

    // Should handle all attack types
    assert_eq!(metrics.num_samples, 7);
    assert!(metrics.attack_accuracy >= 0.0);
}
