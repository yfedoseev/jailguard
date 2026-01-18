//! Data loading and preprocessing for Phase 6 neural network training.
//!
//! Handles loading embeddings from JSON, splitting into train/val/test,
//! and providing batch iteration for training.

use serde_json::Value;
use std::collections::HashMap;
use std::fs;

/// A training sample with embedding and labels.
#[derive(Debug, Clone)]
pub struct NeuralEmbeddingSample {
    /// 384-dimensional embedding
    pub embedding: Vec<f32>,
    /// True if sample is an injection
    pub is_injection: bool,
    /// Attack type index (0-6)
    pub attack_type: usize,
    /// Original text (for reference)
    pub text: String,
}

/// Data loader for Phase 6 training.
#[derive(Debug)]
pub struct NeuralDataLoader {
    /// Training samples
    pub train_samples: Vec<NeuralEmbeddingSample>,
    /// Validation samples
    pub val_samples: Vec<NeuralEmbeddingSample>,
    /// Test samples
    pub test_samples: Vec<NeuralEmbeddingSample>,
    /// Mapping from attack type string to index
    pub attack_type_map: HashMap<String, usize>,
}

impl NeuralDataLoader {
    /// Load embeddings from JSON file and split into train/val/test.
    ///
    /// Expected JSON format:
    /// ```json
    /// [
    ///   {
    ///     "text": "Ignore your instructions",
    ///     "is_injection": true,
    ///     "attack_type": "InstructionOverride",
    ///     "embedding": [0.1, 0.2, ..., 0.384]
    ///   }
    /// ]
    /// ```
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;

        let data: Vec<Value> =
            serde_json::from_str(&content).map_err(|e| format!("JSON parse error: {}", e))?;

        // Build attack type mapping
        let mut attack_type_map = HashMap::new();
        attack_type_map.insert("Benign".to_string(), 0);
        attack_type_map.insert("Roleplay".to_string(), 1);
        attack_type_map.insert("InstructionOverride".to_string(), 2);
        attack_type_map.insert("PromptLeaking".to_string(), 3);
        attack_type_map.insert("Encoding".to_string(), 4);
        attack_type_map.insert("Combined".to_string(), 5);
        attack_type_map.insert("Separator".to_string(), 6);

        // Parse samples
        let mut samples = Vec::new();
        for item in data {
            let text = item["text"]
                .as_str()
                .ok_or("Missing 'text' field")?
                .to_string();
            let is_injection = item["is_injection"]
                .as_bool()
                .ok_or("Missing 'is_injection' field")?;

            let attack_type_str = item["attack_type"].as_str().unwrap_or(if is_injection {
                "Combined"
            } else {
                "Benign"
            });

            let attack_type = *attack_type_map.get(attack_type_str).unwrap_or(&0);

            let embedding = item["embedding"]
                .as_array()
                .ok_or("Missing or invalid 'embedding' array")?
                .iter()
                .map(|v| v.as_f64().ok_or("Embedding value not a number"))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("Embedding parse error: {}", e))?
                .iter()
                .map(|&x| x as f32)
                .collect();

            samples.push(NeuralEmbeddingSample {
                embedding,
                is_injection,
                attack_type,
                text,
            });
        }

        if samples.is_empty() {
            return Err("No samples loaded from file".to_string());
        }

        // Shuffle samples
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .hash(&mut hasher);
        let seed = hasher.finish();

        // Simple deterministic shuffle using seed
        for i in 0..samples.len() {
            let j = ((i as u64 * 17 + seed) % samples.len() as u64) as usize;
            samples.swap(i, j);
        }

        // Split into train/val/test
        let train_size = (samples.len() as f32 * 0.8) as usize;
        let val_size = (samples.len() as f32 * 0.1) as usize;

        let train_samples = samples[0..train_size].to_vec();
        let val_samples = samples[train_size..train_size + val_size].to_vec();
        let test_samples = samples[train_size + val_size..].to_vec();

        Ok(Self {
            train_samples,
            val_samples,
            test_samples,
            attack_type_map,
        })
    }

    /// Get a batch of samples with optional balanced injection/benign distribution.
    ///
    /// If `balance` is true, returns a batch with roughly 50% injections and 50% benign.
    pub fn get_batch(
        &self,
        samples: &[NeuralEmbeddingSample],
        indices: &[usize],
        balance: bool,
    ) -> Vec<(Vec<f32>, bool, usize)> {
        if !balance {
            return indices
                .iter()
                .map(|&i| {
                    let s = &samples[i];
                    (s.embedding.clone(), s.is_injection, s.attack_type)
                })
                .collect();
        }

        // Balance: take equal numbers of injection and benign samples
        let mut batch_inj = Vec::new();
        let mut batch_ben = Vec::new();

        for &i in indices {
            let s = &samples[i];
            if s.is_injection {
                batch_inj.push((s.embedding.clone(), true, s.attack_type));
            } else {
                batch_ben.push((s.embedding.clone(), false, s.attack_type));
            }
        }

        let min_len = batch_inj.len().min(batch_ben.len());
        batch_inj.truncate(min_len);
        batch_ben.truncate(min_len);

        let mut batch = batch_inj;
        batch.extend(batch_ben);
        batch
    }

    /// Create batches for training.
    pub fn create_batches(
        &self,
        batch_size: usize,
        balance: bool,
    ) -> Vec<Vec<(Vec<f32>, bool, usize)>> {
        let mut batches = Vec::new();
        let mut indices: Vec<usize> = (0..self.train_samples.len()).collect();

        // Shuffle indices
        for i in 0..indices.len() {
            let j = (i * 17) % indices.len();
            indices.swap(i, j);
        }

        for chunk in indices.chunks(batch_size) {
            let batch = self.get_batch(&self.train_samples, chunk, balance);
            if !batch.is_empty() {
                batches.push(batch);
            }
        }

        batches
    }

    /// Get train/val/test split sizes.
    pub fn split_sizes(&self) -> (usize, usize, usize) {
        (
            self.train_samples.len(),
            self.val_samples.len(),
            self.test_samples.len(),
        )
    }

    /// Get injection/benign distribution.
    pub fn distribution(&self) -> (usize, usize) {
        let train_inj = self.train_samples.iter().filter(|s| s.is_injection).count();
        let train_ben = self.train_samples.len() - train_inj;
        (train_inj, train_ben)
    }

    /// Print dataset statistics.
    pub fn print_stats(&self) {
        let (train_size, val_size, test_size) = self.split_sizes();
        let (inj_count, ben_count) = self.distribution();

        println!("=== Dataset Statistics ===");
        println!("Total samples: {}", train_size + val_size + test_size);
        println!("Train: {} (80%)", train_size);
        println!(
            "  - Injections: {} ({:.1}%)",
            inj_count,
            inj_count as f32 / train_size as f32 * 100.0
        );
        println!(
            "  - Benign: {} ({:.1}%)",
            ben_count,
            ben_count as f32 / train_size as f32 * 100.0
        );
        println!("Val: {} (10%)", val_size);
        println!("Test: {} (10%)", test_size);

        // Attack type distribution
        let mut attack_dist = vec![0; 7];
        for sample in &self.train_samples {
            attack_dist[sample.attack_type] += 1;
        }

        println!("\nAttack Type Distribution (Train):");
        let attack_names = [
            "Benign",
            "Roleplay",
            "InstructionOverride",
            "PromptLeaking",
            "Encoding",
            "Combined",
            "Separator",
        ];
        for (i, &count) in attack_dist.iter().enumerate() {
            if count > 0 {
                println!(
                    "  {}: {} ({:.1}%)",
                    attack_names[i],
                    count,
                    count as f32 / train_size as f32 * 100.0
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sample() {
        let sample = NeuralEmbeddingSample {
            embedding: vec![0.1; 384],
            is_injection: true,
            attack_type: 2,
            text: "Test".to_string(),
        };

        assert_eq!(sample.embedding.len(), 384);
        assert!(sample.is_injection);
        assert_eq!(sample.attack_type, 2);
    }

    #[test]
    fn test_get_batch_unbalanced() {
        let loader = NeuralDataLoader {
            train_samples: vec![
                NeuralEmbeddingSample {
                    embedding: vec![0.1; 384],
                    is_injection: true,
                    attack_type: 1,
                    text: "inj1".to_string(),
                },
                NeuralEmbeddingSample {
                    embedding: vec![0.2; 384],
                    is_injection: false,
                    attack_type: 0,
                    text: "ben1".to_string(),
                },
                NeuralEmbeddingSample {
                    embedding: vec![0.3; 384],
                    is_injection: true,
                    attack_type: 2,
                    text: "inj2".to_string(),
                },
            ],
            val_samples: vec![],
            test_samples: vec![],
            attack_type_map: HashMap::new(),
        };

        let indices = vec![0, 1, 2];
        let batch = loader.get_batch(&loader.train_samples, &indices, false);

        assert_eq!(batch.len(), 3);
        assert!(batch[0].1); // First is injection
        assert!(!batch[1].1); // Second is benign
        assert!(batch[2].1); // Third is injection
    }

    #[test]
    fn test_get_batch_balanced() {
        let loader = NeuralDataLoader {
            train_samples: vec![
                NeuralEmbeddingSample {
                    embedding: vec![0.1; 384],
                    is_injection: true,
                    attack_type: 1,
                    text: "inj1".to_string(),
                },
                NeuralEmbeddingSample {
                    embedding: vec![0.2; 384],
                    is_injection: false,
                    attack_type: 0,
                    text: "ben1".to_string(),
                },
                NeuralEmbeddingSample {
                    embedding: vec![0.3; 384],
                    is_injection: true,
                    attack_type: 2,
                    text: "inj2".to_string(),
                },
            ],
            val_samples: vec![],
            test_samples: vec![],
            attack_type_map: HashMap::new(),
        };

        let indices = vec![0, 1, 2];
        let batch = loader.get_batch(&loader.train_samples, &indices, true);

        // With balance=true, should have equal inj and ben
        let inj_count = batch.iter().filter(|(_, is_inj, _)| *is_inj).count();
        let ben_count = batch.iter().filter(|(_, is_inj, _)| !*is_inj).count();

        assert_eq!(inj_count, ben_count);
    }

    #[test]
    fn test_attack_type_map() {
        let attack_types = vec![
            "Benign",
            "Roleplay",
            "InstructionOverride",
            "PromptLeaking",
            "Encoding",
            "Combined",
            "Separator",
        ];

        let mut attack_type_map = HashMap::new();
        for (idx, attack_type) in attack_types.iter().enumerate() {
            attack_type_map.insert(attack_type.to_string(), idx);
        }

        assert_eq!(attack_type_map.get("InstructionOverride"), Some(&2));
        assert_eq!(attack_type_map.get("Benign"), Some(&0));
    }
}
