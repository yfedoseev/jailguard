//! Adam optimizer implementation for training neural networks.
//!
//! Adam (Adaptive Moment Estimation) combines the advantages of:
//! - AdaGrad: Adaptive learning rates per parameter
//! - RMSprop: Root mean square propagation for stability
//! - Momentum: First-order moments for acceleration
//!
//! Adam update rule (ASCII notation):
//! - m_t = beta1 * m_{t-1} + (1 - beta1) * g_t         (First moment - momentum)
//! - v_t = beta2 * v_{t-1} + (1 - beta2) * g_t^2       (Second moment - variance)
//! - m_hat_t = m_t / (1 - beta1^t)                      (Bias correction)
//! - v_hat_t = v_t / (1 - beta2^t)                      (Bias correction)
//! - theta_t = theta_{t-1} - alpha * m_hat_t / (sqrt(v_hat_t) + epsilon)  (Update)
#![allow(missing_docs)]

/// Adam optimizer configuration
#[derive(Debug, Clone)]
pub struct AdamConfig {
    /// Learning rate (typical: 1e-4 to 1e-3)
    pub learning_rate: f32,
    /// Momentum decay (typical: 0.9)
    pub beta_1: f32,
    /// RMSprop decay (typical: 0.999)
    pub beta_2: f32,
    /// Numerical stability epsilon (typical: 1e-8)
    pub epsilon: f32,
    /// L2 weight decay (typical: 0.0 to 0.01)
    pub weight_decay: f32,
}

impl Default for AdamConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1e-4,
            beta_1: 0.9,
            beta_2: 0.999,
            epsilon: 1e-8,
            weight_decay: 0.0,
        }
    }
}

impl AdamConfig {
    /// Create configuration with custom learning rate
    pub fn with_learning_rate(mut self, lr: f32) -> Self {
        self.learning_rate = lr;
        self
    }

    /// Create configuration with custom beta values
    pub fn with_betas(mut self, beta_1: f32, beta_2: f32) -> Self {
        self.beta_1 = beta_1;
        self.beta_2 = beta_2;
        self
    }

    /// Create configuration with weight decay
    pub fn with_weight_decay(mut self, wd: f32) -> Self {
        self.weight_decay = wd;
        self
    }
}

/// Adam optimizer state
pub struct Adam {
    /// Configuration
    config: AdamConfig,
    /// First moment estimates (momentum)
    m: Vec<f32>,
    /// Second moment estimates (variance)
    v: Vec<f32>,
    /// Timestep counter (for bias correction)
    t: u32,
}

impl Adam {
    /// Create a new Adam optimizer
    pub fn new(config: AdamConfig, param_count: usize) -> Self {
        Self {
            config,
            m: vec![0.0; param_count],
            v: vec![0.0; param_count],
            t: 0,
        }
    }

    /// Create with default config
    pub fn default(param_count: usize) -> Self {
        Self::new(AdamConfig::default(), param_count)
    }

    /// Perform an optimization step
    pub fn step(&mut self, params: &mut [f32], gradients: &[f32]) {
        assert_eq!(params.len(), gradients.len());
        assert_eq!(params.len(), self.m.len());
        assert_eq!(params.len(), self.v.len());

        self.t += 1;

        // Bias correction terms
        let bias_corr_1 = 1.0 - self.config.beta_1.powi(self.t as i32);
        let bias_corr_2 = 1.0 - self.config.beta_2.powi(self.t as i32);

        // Update each parameter
        for i in 0..params.len() {
            let g = gradients[i];

            // Update biased first moment estimate
            self.m[i] = self.config.beta_1 * self.m[i] + (1.0 - self.config.beta_1) * g;

            // Update biased second raw moment estimate
            self.v[i] = self.config.beta_2 * self.v[i] + (1.0 - self.config.beta_2) * g * g;

            // Compute bias-corrected first moment estimate
            let m_hat = self.m[i] / bias_corr_1;

            // Compute bias-corrected second raw moment estimate
            let v_hat = self.v[i] / bias_corr_2;

            // Update parameters
            let step = self.config.learning_rate * m_hat / (v_hat.sqrt() + self.config.epsilon);

            // Apply weight decay (L2 regularization)
            if self.config.weight_decay > 0.0 {
                params[i] -= self.config.weight_decay * self.config.learning_rate * params[i];
            }

            // Apply gradient step
            params[i] -= step;
        }
    }

    /// Set learning rate
    pub fn set_learning_rate(&mut self, lr: f32) {
        self.config.learning_rate = lr;
    }

    /// Get current timestep
    pub fn timestep(&self) -> u32 {
        self.t
    }

    /// Reset optimizer state
    pub fn reset(&mut self) {
        self.m.fill(0.0);
        self.v.fill(0.0);
        self.t = 0;
    }
}

/// Learning rate scheduler for progressive adjustment
pub struct LearningRateScheduler {
    /// Base learning rate
    base_lr: f32,
    /// Schedule type
    schedule_type: ScheduleType,
}

#[derive(Debug, Clone)]
pub enum ScheduleType {
    /// Constant learning rate
    Constant,
    /// Linear warmup then exponential decay
    WarmupExponential { warmup_steps: u32, decay_rate: f32 },
    /// Linear warmup then linear decay
    WarmupLinear { warmup_steps: u32, decay_steps: u32 },
    /// Cosine annealing (warm restarts)
    CosineAnnealing { warmup_steps: u32, total_steps: u32 },
}

impl LearningRateScheduler {
    /// Create a new scheduler
    pub fn new(base_lr: f32, schedule_type: ScheduleType) -> Self {
        Self {
            base_lr,
            schedule_type,
        }
    }

    /// Get learning rate for current step
    pub fn get_learning_rate(&self, step: u32) -> f32 {
        match &self.schedule_type {
            ScheduleType::Constant => self.base_lr,

            ScheduleType::WarmupExponential {
                warmup_steps,
                decay_rate,
            } => {
                if step < *warmup_steps {
                    // Linear warmup
                    self.base_lr * (step as f32) / (*warmup_steps as f32)
                } else {
                    // Exponential decay
                    let decay_steps = (step - warmup_steps) as f32;
                    self.base_lr * decay_rate.powf(decay_steps / 1000.0)
                }
            }

            ScheduleType::WarmupLinear {
                warmup_steps,
                decay_steps,
            } => {
                if step < *warmup_steps {
                    // Linear warmup
                    self.base_lr * (step as f32) / (*warmup_steps as f32)
                } else {
                    // Linear decay
                    let steps_after_warmup = step.saturating_sub(*warmup_steps) as f32;
                    let decay_factor = (steps_after_warmup / *decay_steps as f32).min(1.0);
                    self.base_lr * (1.0 - decay_factor * 0.9) // Decay to 10% of base
                }
            }

            ScheduleType::CosineAnnealing {
                warmup_steps,
                total_steps,
            } => {
                if step < *warmup_steps {
                    // Linear warmup
                    self.base_lr * (step as f32) / (*warmup_steps as f32)
                } else {
                    // Cosine annealing
                    let progress = (step.saturating_sub(*warmup_steps) as f32)
                        / (*total_steps - warmup_steps) as f32;
                    let progress = progress.min(1.0);
                    self.base_lr * 0.5 * (1.0 + (std::f32::consts::PI * progress).cos())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adam_creation() {
        let adam = Adam::new(AdamConfig::default(), 10);
        assert_eq!(adam.timestep(), 0);
        assert_eq!(adam.m.len(), 10);
        assert_eq!(adam.v.len(), 10);
    }

    #[test]
    fn test_adam_step() {
        let mut adam = Adam::new(AdamConfig::default(), 5);
        let mut params = vec![1.0; 5];
        let gradients = vec![0.1; 5];

        adam.step(&mut params, &gradients);

        assert_eq!(adam.timestep(), 1);
        // Parameters should decrease (negative gradient direction)
        assert!(params[0] < 1.0);
    }

    #[test]
    fn test_adam_convergence() {
        let config = AdamConfig::default().with_learning_rate(0.01);
        let mut adam = Adam::new(config, 1);
        let mut params = vec![10.0];
        let gradients = vec![1.0; 1]; // Constant gradient pointing toward 0

        for _ in 0..100 {
            adam.step(&mut params, &gradients);
        }

        // Should have moved toward 0 (since gradient is positive)
        assert!(
            params[0] < 10.0,
            "Parameter should decrease with positive gradient"
        );
    }

    #[test]
    fn test_learning_rate_scheduler_constant() {
        let scheduler = LearningRateScheduler::new(0.001, ScheduleType::Constant);
        assert_eq!(scheduler.get_learning_rate(0), 0.001);
        assert_eq!(scheduler.get_learning_rate(100), 0.001);
        assert_eq!(scheduler.get_learning_rate(10000), 0.001);
    }

    #[test]
    fn test_learning_rate_scheduler_warmup() {
        let scheduler = LearningRateScheduler::new(
            0.001,
            ScheduleType::WarmupLinear {
                warmup_steps: 100,
                decay_steps: 100,
            },
        );

        // During warmup
        let lr_0 = scheduler.get_learning_rate(0);
        let lr_50 = scheduler.get_learning_rate(50);
        let lr_100 = scheduler.get_learning_rate(100);

        assert!(lr_0 < lr_50, "Learning rate should increase during warmup");
        assert!(
            lr_50 < lr_100,
            "Learning rate should increase during warmup"
        );
        assert!(
            lr_100 > 0.0,
            "Learning rate should be positive after warmup"
        );

        // After warmup and decay
        let lr_200 = scheduler.get_learning_rate(200);
        assert!(lr_200 < lr_100, "Learning rate should decay after warmup");
    }

    #[test]
    fn test_cosine_annealing() {
        let scheduler = LearningRateScheduler::new(
            0.001,
            ScheduleType::CosineAnnealing {
                warmup_steps: 100,
                total_steps: 1000,
            },
        );

        let lr_0 = scheduler.get_learning_rate(0);
        let lr_100 = scheduler.get_learning_rate(100);
        let lr_500 = scheduler.get_learning_rate(500);
        let lr_1000 = scheduler.get_learning_rate(1000);

        assert!(lr_0 < lr_100, "LR increases during warmup");
        assert!(lr_100 > lr_500, "LR decreases during annealing");
        assert!(lr_500 > lr_1000.min(0.00005), "LR continues to decrease");
    }

    #[test]
    fn test_adam_config_builder() {
        let config = AdamConfig::default()
            .with_learning_rate(0.01)
            .with_betas(0.95, 0.995)
            .with_weight_decay(0.001);

        assert_eq!(config.learning_rate, 0.01);
        assert_eq!(config.beta_1, 0.95);
        assert_eq!(config.beta_2, 0.995);
        assert_eq!(config.weight_decay, 0.001);
    }
}
