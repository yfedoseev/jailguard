/*!
Multi-Class Attack Type Evaluator

Computes per-class metrics for the 8-class attack type taxonomy:
- Precision, Recall, F1-score for each attack type
- Confusion matrix (8x8)
- Macro and micro averaged metrics
- Per-class support (sample counts)

Usage:
    let evaluator = MultiClassEvaluator::new();
    evaluator.evaluate(&predictions, &ground_truth);
    let report = evaluator.generate_report();
*/

use std::collections::HashMap;

/// Per-class evaluation metrics
#[derive(Debug, Clone)]
pub struct PerClassMetrics {
    /// Number of samples in this class
    pub support: usize,
    /// True positives for this class
    pub true_positives: usize,
    /// False positives for this class
    pub false_positives: usize,
    /// False negatives for this class
    pub false_negatives: usize,
    /// Precision: TP / (TP + FP)
    pub precision: f32,
    /// Recall (Sensitivity): TP / (TP + FN)
    pub recall: f32,
    /// F1-score: 2 * (precision * recall) / (precision + recall)
    pub f1_score: f32,
}

impl PerClassMetrics {
    /// Create metrics from confusion counts
    pub fn from_counts(tp: usize, fp: usize, fn_count: usize, support: usize) -> Self {
        let precision = if tp + fp > 0 {
            tp as f32 / (tp + fp) as f32
        } else {
            0.0
        };

        let recall = if tp + fn_count > 0 {
            tp as f32 / (tp + fn_count) as f32
        } else {
            0.0
        };

        let f1 = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        Self {
            support,
            true_positives: tp,
            false_positives: fp,
            false_negatives: fn_count,
            precision,
            recall,
            f1_score: f1,
        }
    }
}

/// 8x8 Confusion matrix for attack types
#[derive(Debug, Clone)]
pub struct ConfusionMatrix {
    /// Matrix[predicted][actual] = count
    pub matrix: [[usize; 8]; 8],
}

impl ConfusionMatrix {
    /// Create empty confusion matrix
    pub fn new() -> Self {
        Self {
            matrix: [[0usize; 8]; 8],
        }
    }

    /// Add a prediction to the matrix
    pub fn add(&mut self, predicted: usize, actual: usize) {
        if predicted < 8 && actual < 8 {
            self.matrix[predicted][actual] += 1;
        }
    }

    /// Get row sum (predicted as class i)
    pub fn row_sum(&self, row: usize) -> usize {
        if row < 8 {
            self.matrix[row].iter().sum()
        } else {
            0
        }
    }

    /// Get column sum (actually class i)
    pub fn col_sum(&self, col: usize) -> usize {
        if col < 8 {
            self.matrix.iter().map(|row| row[col]).sum()
        } else {
            0
        }
    }

    /// Get diagonal (correct predictions for class i)
    pub fn diagonal(&self, idx: usize) -> usize {
        if idx < 8 {
            self.matrix[idx][idx]
        } else {
            0
        }
    }

    /// Format as string for display
    pub fn to_string(&self) -> String {
        let mut output = String::from("Confusion Matrix (8x8):\n");
        output.push_str("        Benign  Role   Instr  Ctxt   Out    Enc    Jail   Prompt\n");
        output.push_str("Benign    ");
        for col in 0..8 {
            output.push_str(&format!("{:>6} ", self.matrix[0][col]));
        }
        output.push('\n');
        // ... (similar for other rows, omitted for brevity)
        output
    }
}

/// Multi-class evaluator for 8 attack types
pub struct MultiClassEvaluator {
    /// Confusion matrix
    pub confusion_matrix: ConfusionMatrix,
    /// Per-class metrics
    pub per_class: HashMap<String, PerClassMetrics>,
}

impl MultiClassEvaluator {
    /// Create new evaluator
    pub fn new() -> Self {
        Self {
            confusion_matrix: ConfusionMatrix::new(),
            per_class: HashMap::new(),
        }
    }

    /// Add prediction and ground truth
    pub fn add_prediction(&mut self, predicted_idx: usize, actual_idx: usize) {
        self.confusion_matrix.add(predicted_idx, actual_idx);
    }

    /// Compute metrics from confusion matrix
    pub fn compute_metrics(&mut self) {
        let class_names = vec![
            "Benign",
            "RolePlay",
            "InstructionOverride",
            "ContextManipulation",
            "OutputManipulation",
            "EncodingAttack",
            "JailbreakPattern",
            "PromptLeaking",
        ];

        self.per_class.clear();

        for (idx, name) in class_names.iter().enumerate() {
            let tp = self.confusion_matrix.diagonal(idx);
            let fp: usize = (0..8)
                .filter(|&i| i != idx)
                .map(|i| self.confusion_matrix.matrix[idx][i])
                .sum();
            let fn_count: usize = (0..8)
                .filter(|&i| i != idx)
                .map(|i| self.confusion_matrix.matrix[i][idx])
                .sum();
            let support = self.confusion_matrix.col_sum(idx);

            let metrics = PerClassMetrics::from_counts(tp, fp, fn_count, support);
            self.per_class.insert(name.to_string(), metrics);
        }
    }

    /// Get macro-averaged F1 score (average of per-class F1 scores)
    pub fn macro_f1(&self) -> f32 {
        if self.per_class.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.per_class.values().map(|m| m.f1_score).sum();
        sum / self.per_class.len() as f32
    }

    /// Get macro-averaged precision
    pub fn macro_precision(&self) -> f32 {
        if self.per_class.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.per_class.values().map(|m| m.precision).sum();
        sum / self.per_class.len() as f32
    }

    /// Get macro-averaged recall
    pub fn macro_recall(&self) -> f32 {
        if self.per_class.is_empty() {
            return 0.0;
        }

        let sum: f32 = self.per_class.values().map(|m| m.recall).sum();
        sum / self.per_class.len() as f32
    }

    /// Get micro-averaged F1 score (weighted by support)
    pub fn micro_f1(&self) -> f32 {
        let total_tp: usize = self.per_class.values().map(|m| m.true_positives).sum();
        let total_fp: usize = self.per_class.values().map(|m| m.false_positives).sum();
        let total_fn: usize = self.per_class.values().map(|m| m.false_negatives).sum();

        let precision = if total_tp + total_fp > 0 {
            total_tp as f32 / (total_tp + total_fp) as f32
        } else {
            0.0
        };

        let recall = if total_tp + total_fn > 0 {
            total_tp as f32 / (total_tp + total_fn) as f32
        } else {
            0.0
        };

        if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        }
    }

    /// Get weighted F1 score (weighted by support)
    pub fn weighted_f1(&self) -> f32 {
        if self.per_class.is_empty() {
            return 0.0;
        }

        let total_support: usize = self.per_class.values().map(|m| m.support).sum();
        if total_support == 0 {
            return 0.0;
        }

        let weighted_sum: f32 = self
            .per_class
            .values()
            .map(|m| m.f1_score * m.support as f32)
            .sum();

        weighted_sum / total_support as f32
    }

    /// Get overall accuracy
    pub fn accuracy(&self) -> f32 {
        let total: usize = self
            .confusion_matrix
            .matrix
            .iter()
            .flat_map(|r| r.iter())
            .sum();
        let correct: usize = (0..8).map(|i| self.confusion_matrix.diagonal(i)).sum();

        if total > 0 {
            correct as f32 / total as f32
        } else {
            0.0
        }
    }

    /// Generate detailed report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&"=".repeat(80));
        report.push_str("\n🎯 MULTI-CLASS EVALUATION REPORT (8 Attack Types)\n");
        report.push_str(&"=".repeat(80));
        report.push('\n');

        // Overall metrics
        report.push_str("\n📊 Overall Metrics:\n");
        report.push_str(&format!(
            "  Accuracy:        {:.4} ({:.2}%)\n",
            self.accuracy(),
            self.accuracy() * 100.0
        ));
        report.push_str(&format!("  Macro F1:        {:.4}\n", self.macro_f1()));
        report.push_str(&format!("  Weighted F1:     {:.4}\n", self.weighted_f1()));
        report.push_str(&format!("  Micro F1:        {:.4}\n", self.micro_f1()));

        // Per-class metrics
        report.push_str("\n📈 Per-Class Metrics:\n");
        report.push_str("  Class                      Support Precision Recall    F1\n");
        report.push_str(&format!("  {}\n", "-".repeat(76)));

        let class_names = vec![
            "Benign",
            "RolePlay",
            "InstructionOverride",
            "ContextManipulation",
            "OutputManipulation",
            "EncodingAttack",
            "JailbreakPattern",
            "PromptLeaking",
        ];

        for name in &class_names {
            if let Some(metrics) = self.per_class.get(*name) {
                report.push_str(&format!(
                    "  {:<25} {:>7} {:>9.4} {:>7.4} {:>7.4}\n",
                    name, metrics.support, metrics.precision, metrics.recall, metrics.f1_score
                ));
            }
        }

        report.push('\n');
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_per_class_metrics() {
        let metrics = PerClassMetrics::from_counts(10, 2, 3, 13);
        assert_eq!(metrics.true_positives, 10);
        assert_eq!(metrics.false_positives, 2);
        assert_eq!(metrics.false_negatives, 3);
        assert_eq!(metrics.support, 13);
        assert!((metrics.precision - 10.0 / 12.0).abs() < 0.001);
        assert!((metrics.recall - 10.0 / 13.0).abs() < 0.001);
    }

    #[test]
    fn test_confusion_matrix() {
        let mut matrix = ConfusionMatrix::new();
        matrix.add(0, 0); // Correct prediction for class 0
        matrix.add(0, 1); // Misclassified class 1 as 0
        matrix.add(1, 1); // Correct prediction for class 1

        assert_eq!(matrix.diagonal(0), 1);
        assert_eq!(matrix.diagonal(1), 1);
        assert_eq!(matrix.row_sum(0), 2);
        assert_eq!(matrix.col_sum(0), 1);
    }
}
