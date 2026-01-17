/// Deduplication framework for removing semantic duplicates from augmented datasets
///
/// Uses embedding-based similarity to identify and remove near-duplicate samples,
/// keeping only diverse representatives of each semantic cluster.
use std::collections::HashMap;

/// Represents a deduplicated sample group (cluster of similar samples)
#[derive(Clone, Debug)]
pub struct SampleCluster {
    /// Canonical sample (representative kept)
    pub canonical: SampleWithEmbedding,
    /// Similar samples removed (deduplicated)
    pub duplicates: Vec<SampleWithEmbedding>,
    /// Average similarity within cluster
    pub avg_similarity: f32,
}

/// Sample with precomputed embedding
#[derive(Clone, Debug)]
pub struct SampleWithEmbedding {
    pub text: String,
    pub embedding: Vec<f32>,
    pub is_injection: bool,
    pub attack_type: Option<String>,
    pub source: String, // "original", "synthetic", "llm_augmented"
}

/// Configuration for deduplication
#[derive(Clone, Debug)]
pub struct DeduplicationConfig {
    /// Cosine similarity threshold for considering samples as duplicates (0.0-1.0)
    pub similarity_threshold: f32,
    /// Method to select canonical sample: "first", "longest", "`highest_confidence`"
    pub canonical_selection: CanonicalSelectionMethod,
    /// Enable logging of deduplication statistics
    pub verbose: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CanonicalSelectionMethod {
    First,
    Longest,
    HighestConfidence,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.92, // Keep samples with <92% similarity
            canonical_selection: CanonicalSelectionMethod::Longest,
            verbose: true,
        }
    }
}

/// Deduplication statistics
#[derive(Clone, Debug, Default)]
pub struct DeduplicationStats {
    pub total_input: usize,
    pub total_clusters: usize,
    pub total_removed: usize,
    pub total_kept: usize,
    pub removal_rate: f32,
    pub avg_cluster_size: f32,
}

/// Deduplicator using cosine similarity clustering
pub struct Deduplicator {
    config: DeduplicationConfig,
}

impl Deduplicator {
    /// Create a new deduplicator
    pub fn new(config: DeduplicationConfig) -> Self {
        Self { config }
    }

    /// Compute cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for (x, y) in a.iter().zip(b.iter()) {
            dot_product += x * y;
            norm_a += x * x;
            norm_b += y * y;
        }

        norm_a = norm_a.sqrt();
        norm_b = norm_b.sqrt();

        if norm_a > 0.0 && norm_b > 0.0 {
            dot_product / (norm_a * norm_b)
        } else {
            0.0
        }
    }

    /// Deduplicate samples by clustering similar embeddings
    pub fn deduplicate(
        &self,
        samples: Vec<SampleWithEmbedding>,
    ) -> (
        Vec<SampleWithEmbedding>,
        Vec<SampleCluster>,
        DeduplicationStats,
    ) {
        if samples.is_empty() {
            return (
                vec![],
                vec![],
                DeduplicationStats {
                    total_input: 0,
                    total_clusters: 0,
                    total_removed: 0,
                    total_kept: 0,
                    removal_rate: 0.0,
                    avg_cluster_size: 0.0,
                },
            );
        }

        let mut clusters: HashMap<usize, SampleCluster> = HashMap::new();
        let mut processed = vec![false; samples.len()];

        // Greedy clustering: each sample is either a canonical or added to a cluster
        for i in 0..samples.len() {
            if processed[i] {
                continue;
            }

            let mut cluster = SampleCluster {
                canonical: samples[i].clone(),
                duplicates: vec![],
                avg_similarity: 1.0,
            };

            let mut similarities = vec![];

            // Find all similar samples
            for j in (i + 1)..samples.len() {
                if processed[j] {
                    continue;
                }

                let similarity =
                    Self::cosine_similarity(&samples[i].embedding, &samples[j].embedding);

                if similarity >= self.config.similarity_threshold {
                    similarities.push(similarity);
                    cluster.duplicates.push(samples[j].clone());
                    processed[j] = true;
                }
            }

            // Calculate average similarity in cluster
            if !similarities.is_empty() {
                let sum: f32 = similarities.iter().sum();
                cluster.avg_similarity = sum / similarities.len() as f32;
            }

            processed[i] = true;
            clusters.insert(i, cluster);
        }

        // Build result
        let mut kept_samples = vec![];
        let mut result_clusters = vec![];

        for (_, cluster) in clusters {
            kept_samples.push(cluster.canonical.clone());
            result_clusters.push(cluster);
        }

        let total_removed = samples.len() - kept_samples.len();
        let removal_rate = if samples.is_empty() {
            0.0
        } else {
            (total_removed as f32) / (samples.len() as f32)
        };

        let avg_cluster_size = if result_clusters.is_empty() {
            0.0
        } else {
            let total_duplicates: usize = result_clusters.iter().map(|c| c.duplicates.len()).sum();
            (1.0 + total_duplicates as f32) / (result_clusters.len() as f32)
        };

        let stats = DeduplicationStats {
            total_input: samples.len(),
            total_clusters: result_clusters.len(),
            total_removed,
            total_kept: kept_samples.len(),
            removal_rate,
            avg_cluster_size,
        };

        if self.config.verbose {
            println!("Deduplication Results:");
            println!("  Input samples: {}", stats.total_input);
            println!("  Clusters: {}", stats.total_clusters);
            println!(
                "  Kept: {} ({:.1}%)",
                stats.total_kept,
                (stats.total_kept as f32 / stats.total_input as f32) * 100.0
            );
            println!(
                "  Removed: {} ({:.1}%)",
                stats.total_removed,
                stats.removal_rate * 100.0
            );
            println!("  Avg cluster size: {:.2}", stats.avg_cluster_size);
            println!(
                "  Similarity threshold: {}",
                self.config.similarity_threshold
            );
        }

        (kept_samples, result_clusters, stats)
    }

    /// Deduplicate and keep only most representative samples
    pub fn deduplicate_and_select(
        &self,
        samples: Vec<SampleWithEmbedding>,
        target_count: Option<usize>,
    ) -> (Vec<SampleWithEmbedding>, DeduplicationStats) {
        let (kept, _clusters, stats) = self.deduplicate(samples);

        let final_samples = if let Some(target) = target_count {
            if kept.len() > target {
                // Select most diverse subset by spacing in embedding space
                Self::select_diverse_subset(&kept, target)
            } else {
                kept
            }
        } else {
            kept
        };

        (final_samples, stats)
    }

    /// Select diverse subset of samples using max-min distance criterion
    fn select_diverse_subset(
        samples: &[SampleWithEmbedding],
        target_count: usize,
    ) -> Vec<SampleWithEmbedding> {
        if samples.len() <= target_count {
            return samples.to_vec();
        }

        let mut selected = vec![samples[0].clone()];
        let mut remaining: Vec<usize> = (1..samples.len()).collect();

        while selected.len() < target_count && !remaining.is_empty() {
            // Find sample in remaining that's most dissimilar to selected set
            let mut best_idx = 0;
            let mut best_min_distance = -1.0;

            for (idx, &sample_idx) in remaining.iter().enumerate() {
                let mut min_distance: f32 = 2.0; // Max possible for cosine distance

                for selected_sample in &selected {
                    let similarity = Self::cosine_similarity(
                        &samples[sample_idx].embedding,
                        &selected_sample.embedding,
                    );
                    let distance = 1.0 - similarity;
                    min_distance = min_distance.min(distance);
                }

                if min_distance > best_min_distance {
                    best_min_distance = min_distance;
                    best_idx = idx;
                }
            }

            selected.push(samples[remaining[best_idx]].clone());
            remaining.remove(best_idx);
        }

        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sample(text: &str, embedding: Vec<f32>) -> SampleWithEmbedding {
        SampleWithEmbedding {
            text: text.to_string(),
            embedding,
            is_injection: true,
            attack_type: Some("test".to_string()),
            source: "test".to_string(),
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let v3 = vec![0.0, 1.0, 0.0];

        assert!((Deduplicator::cosine_similarity(&v1, &v2) - 1.0).abs() < 0.01);
        assert!((Deduplicator::cosine_similarity(&v1, &v3)).abs() < 0.01);
    }

    #[test]
    fn test_deduplication_clustering() {
        let config = DeduplicationConfig {
            similarity_threshold: 0.9,
            canonical_selection: CanonicalSelectionMethod::Longest,
            verbose: false,
        };

        let deduplicator = Deduplicator::new(config);

        // Create samples with known similarity
        let samples = vec![
            create_test_sample("Ignore instructions", vec![1.0, 0.0, 0.0]),
            create_test_sample("Disregard previous", vec![0.95, 0.05, 0.0]), // Very similar
            create_test_sample("Help with task", vec![0.0, 1.0, 0.0]),       // Dissimilar
        ];

        let (kept, _clusters, stats) = deduplicator.deduplicate(samples);

        assert_eq!(kept.len(), 2); // Should keep 2 representative samples
        assert_eq!(stats.total_removed, 1);
    }

    #[test]
    fn test_diverse_subset_selection() {
        // Create embedding space with known structure
        let samples = vec![
            create_test_sample("Sample 1", vec![1.0, 0.0, 0.0, 0.0]),
            create_test_sample("Sample 2", vec![0.9, 0.1, 0.0, 0.0]), // Similar to 1
            create_test_sample("Sample 3", vec![0.0, 1.0, 0.0, 0.0]), // Orthogonal
            create_test_sample("Sample 4", vec![0.0, 0.0, 1.0, 0.0]), // Orthogonal to all
        ];

        let selected = Deduplicator::select_diverse_subset(&samples, 2);

        assert_eq!(selected.len(), 2);
        // Should select samples that maximize diversity
    }

    #[test]
    fn test_empty_deduplication() {
        let config = DeduplicationConfig::default();
        let deduplicator = Deduplicator::new(config);

        let (kept, _clusters, stats) = deduplicator.deduplicate(vec![]);

        assert_eq!(kept.len(), 0);
        assert_eq!(stats.total_input, 0);
        assert_eq!(stats.total_removed, 0);
    }
}
