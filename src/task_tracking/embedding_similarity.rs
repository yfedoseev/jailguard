//! Embedding similarity computation for task context and drift detection.

/// Computes cosine similarity between two embeddings.
///
/// Similarity = (a · b) / (||a|| * ||b||)
///
/// Returns a value in range [0.0, 1.0], where:
/// - 1.0 = identical directions (perfect similarity)
/// - 0.0 = perpendicular (no similarity)
pub fn cosine_similarity(vec_a: &[f32], vec_b: &[f32]) -> f32 {
    if vec_a.is_empty() || vec_b.is_empty() {
        return 0.0;
    }

    if vec_a.len() != vec_b.len() {
        return 0.0;
    }

    // Compute dot product
    let dot_product: f32 = vec_a.iter().zip(vec_b.iter()).map(|(a, b)| a * b).sum();

    // Compute magnitudes
    let mag_a: f32 = vec_a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = vec_b.iter().map(|x| x * x).sum::<f32>().sqrt();

    // Handle zero-magnitude vectors
    if mag_a < 1e-10 || mag_b < 1e-10 {
        return 0.0;
    }

    let similarity = dot_product / (mag_a * mag_b);

    // Clamp to [0, 1] to handle floating point errors
    similarity.clamp(0.0, 1.0)
}

/// Computes average similarity to a list of reference embeddings.
///
/// Useful for checking if a new embedding is similar to expected task embeddings.
pub fn avg_similarity_to_references(embedding: &[f32], references: &[Vec<f32>]) -> f32 {
    if references.is_empty() {
        return 0.0;
    }

    let total: f32 = references
        .iter()
        .map(|ref_emb| cosine_similarity(embedding, ref_emb))
        .sum();

    total / references.len() as f32
}

/// Computes maximum similarity to a list of reference embeddings.
///
/// Useful for detecting if embedding matches ANY expected task topic.
pub fn max_similarity_to_references(embedding: &[f32], references: &[Vec<f32>]) -> f32 {
    references
        .iter()
        .map(|ref_emb| cosine_similarity(embedding, ref_emb))
        .fold(0.0, f32::max)
}

/// Detects drift based on similarity threshold.
///
/// Returns true if similarity is BELOW the threshold (indicating drift).
pub fn detect_drift(embedding: &[f32], reference_embeddings: &[Vec<f32>], threshold: f32) -> bool {
    let similarity = max_similarity_to_references(embedding, reference_embeddings);
    similarity < threshold
}

/// Computes drift score (0.0 = on task, 1.0 = completely off task).
///
/// Score = max(0, threshold - similarity)
/// This gives a measure of how far off the task the embedding is.
pub fn drift_score(embedding: &[f32], reference_embeddings: &[Vec<f32>], threshold: f32) -> f32 {
    let similarity = max_similarity_to_references(embedding, reference_embeddings);
    (threshold - similarity).clamp(0.0, 1.0)
}

/// Detects anomalous embeddings based on statistical outliers.
///
/// Uses z-score: (x - mean) / `std_dev`
/// Returns true if max similarity is unusually low.
/// If `std_dev` is near zero (all similarities identical), treats very low mean as anomalous.
pub fn is_outlier(embedding: &[f32], reference_embeddings: &[Vec<f32>], z_threshold: f32) -> bool {
    if reference_embeddings.is_empty() {
        return false;
    }

    // Compute similarities to all references
    let similarities: Vec<f32> = reference_embeddings
        .iter()
        .map(|ref_emb| cosine_similarity(embedding, ref_emb))
        .collect();

    // Compute mean and std dev
    let mean: f32 = similarities.iter().sum::<f32>() / similarities.len() as f32;
    let variance: f32 =
        similarities.iter().map(|s| (s - mean).powi(2)).sum::<f32>() / similarities.len() as f32;
    let std_dev = variance.sqrt();

    // If mean is very low and embedding has low similarity, it's anomalous
    if mean < 0.1 {
        return similarities.iter().all(|s| *s < 0.1);
    }

    // Otherwise use z-score for standard outlier detection
    if std_dev < 1e-10 {
        return false;
    }

    let max_similarity = similarities.iter().copied().fold(0.0, f32::max);
    let z_score = (max_similarity - mean) / std_dev;

    // True if z-score indicates an outlier (too low = anomalous)
    z_score < -z_threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let vec = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&vec, &vec);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_perpendicular() {
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![0.0, 1.0, 0.0];
        let similarity = cosine_similarity(&vec_a, &vec_b);
        assert!(similarity < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![-1.0, 0.0, 0.0];
        let similarity = cosine_similarity(&vec_a, &vec_b);
        assert!(similarity < 0.001); // Negative becomes 0.0 after clamping
    }

    #[test]
    fn test_cosine_similarity_partial() {
        let vec_a = vec![1.0, 1.0, 0.0];
        let vec_b = vec![1.0, 0.0, 0.0];
        let similarity = cosine_similarity(&vec_a, &vec_b);
        assert!(similarity > 0.5 && similarity < 1.0);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let vec_a = vec![1.0, 2.0];
        let vec_empty = vec![];
        let similarity = cosine_similarity(&vec_a, &vec_empty);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_cosine_similarity_zero_magnitude() {
        let vec_a = vec![0.0, 0.0, 0.0];
        let vec_b = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&vec_a, &vec_b);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_cosine_similarity_mismatched_dims() {
        let vec_a = vec![1.0, 2.0];
        let vec_b = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&vec_a, &vec_b);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_avg_similarity_to_references() {
        let embedding = vec![1.0, 0.0, 0.0];
        let references = vec![
            vec![1.0, 0.0, 0.0], // similarity = 1.0
            vec![0.0, 1.0, 0.0], // similarity = 0.0
        ];
        let avg = avg_similarity_to_references(&embedding, &references);
        assert!((avg - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_max_similarity_to_references() {
        let embedding = vec![1.0, 0.0, 0.0];
        let references = vec![
            vec![1.0, 0.0, 0.0], // similarity = 1.0
            vec![0.5, 0.5, 0.0], // similarity < 1.0
            vec![0.0, 1.0, 0.0], // similarity = 0.0
        ];
        let max = max_similarity_to_references(&embedding, &references);
        assert!((max - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_detect_drift_on_task() {
        let embedding = vec![1.0, 0.0, 0.0];
        let references = vec![vec![1.0, 0.0, 0.0]]; // Identical
        let is_drifted = detect_drift(&embedding, &references, 0.5);
        assert!(!is_drifted); // Similarity 1.0 > threshold 0.5
    }

    #[test]
    fn test_detect_drift_off_task() {
        let embedding = vec![1.0, 0.0, 0.0];
        let references = vec![vec![0.0, 1.0, 0.0]]; // Perpendicular
        let is_drifted = detect_drift(&embedding, &references, 0.5);
        assert!(is_drifted); // Similarity 0.0 < threshold 0.5
    }

    #[test]
    fn test_drift_score_on_task() {
        let embedding = vec![1.0, 0.0, 0.0];
        let references = vec![vec![1.0, 0.0, 0.0]];
        let score = drift_score(&embedding, &references, 0.5);
        assert!(score < 0.001); // Perfect match = no drift
    }

    #[test]
    fn test_drift_score_off_task() {
        let embedding = vec![1.0, 0.0, 0.0];
        let references = vec![vec![0.0, 1.0, 0.0]];
        let score = drift_score(&embedding, &references, 0.5);
        assert!(score > 0.4); // Off task = high drift score
    }

    #[test]
    fn test_is_outlier_normal() {
        let embedding = vec![1.0, 1.0, 0.0];
        let references = vec![
            vec![1.0, 1.0, 0.0],
            vec![1.0, 0.9, 0.0],
            vec![0.9, 1.0, 0.0],
        ];
        let outlier = is_outlier(&embedding, &references, 2.0);
        assert!(!outlier); // Normal embedding
    }

    #[test]
    fn test_is_outlier_anomalous() {
        // Create embeddings with known similarities
        // Reference is [1, 0] and we'll test with [0, 1] (perpendicular)
        let embedding_low = vec![0.0, 1.0];
        let references = vec![vec![1.0, 0.0], vec![0.95, 0.0], vec![0.9, 0.0]];
        // All similarities to references will be ~0
        // This is a statistical outlier (much lower than expected)
        let outlier = is_outlier(&embedding_low, &references, 0.5);
        assert!(outlier); // Anomalous embedding due to very low similarity
    }

    #[test]
    fn test_max_similarity_empty_references() {
        let embedding = vec![1.0, 2.0, 3.0];
        let references = vec![];
        let max = max_similarity_to_references(&embedding, &references);
        assert_eq!(max, 0.0);
    }
}
