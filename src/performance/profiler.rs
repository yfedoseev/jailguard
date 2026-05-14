//! Ensemble voting profiler for measuring performance characteristics
//!
//! Measures:
//! - Individual model detection latencies
//! - Ensemble combination overhead
//! - Total end-to-end latency
//! - Agreement score computation cost

use std::time::Instant;

/// Profiling data for a single detection operation
#[derive(Debug, Clone)]
pub struct DetectionProfile {
    /// Time for JailGuard detection (microseconds)
    pub jailguard_us: u64,
    /// Time for GenTel-Shield detection (microseconds)
    pub gentelshed_us: u64,
    /// Time for ProtectAI detection (microseconds)
    pub protect_ai_us: u64,
    /// Time for ensemble combination (microseconds)
    pub ensemble_combine_us: u64,
    /// Total end-to-end latency (microseconds)
    pub total_us: u64,
    /// Whether all models succeeded
    pub all_success: bool,
}

impl DetectionProfile {
    /// Get the maximum individual model latency
    pub fn max_model_latency(&self) -> u64 {
        self.jailguard_us
            .max(self.gentelshed_us)
            .max(self.protect_ai_us)
    }

    /// Get overhead of ensemble combination relative to fastest model
    pub fn combination_overhead_percent(&self) -> f32 {
        if self.max_model_latency() == 0 {
            0.0
        } else {
            (self.ensemble_combine_us as f32 / self.max_model_latency() as f32) * 100.0
        }
    }

    /// Check if latency is within budget
    pub fn within_budget(&self, budget_us: u64) -> bool {
        self.total_us <= budget_us
    }
}

/// Ensemble voting profiler
pub struct EnsembleProfiler {
    profiles: Vec<DetectionProfile>,
    max_profiles: usize,
}

impl EnsembleProfiler {
    /// Create new profiler with capacity for profiles
    pub fn new(max_profiles: usize) -> Self {
        Self {
            profiles: Vec::with_capacity(max_profiles),
            max_profiles,
        }
    }

    /// Record a detection profile
    pub fn record(&mut self, profile: DetectionProfile) {
        if self.profiles.len() < self.max_profiles {
            self.profiles.push(profile);
        }
    }

    /// Get average JailGuard latency (microseconds)
    pub fn avg_jailguard_us(&self) -> u64 {
        if self.profiles.is_empty() {
            0
        } else {
            self.profiles.iter().map(|p| p.jailguard_us).sum::<u64>() / self.profiles.len() as u64
        }
    }

    /// Get average GenTel-Shield latency (microseconds)
    pub fn avg_gentelshed_us(&self) -> u64 {
        if self.profiles.is_empty() {
            0
        } else {
            self.profiles.iter().map(|p| p.gentelshed_us).sum::<u64>() / self.profiles.len() as u64
        }
    }

    /// Get average ProtectAI latency (microseconds)
    pub fn avg_protect_ai_us(&self) -> u64 {
        if self.profiles.is_empty() {
            0
        } else {
            self.profiles.iter().map(|p| p.protect_ai_us).sum::<u64>() / self.profiles.len() as u64
        }
    }

    /// Get average ensemble combination latency (microseconds)
    pub fn avg_combine_us(&self) -> u64 {
        if self.profiles.is_empty() {
            0
        } else {
            self.profiles
                .iter()
                .map(|p| p.ensemble_combine_us)
                .sum::<u64>()
                / self.profiles.len() as u64
        }
    }

    /// Get average total end-to-end latency (microseconds)
    pub fn avg_total_us(&self) -> u64 {
        if self.profiles.is_empty() {
            0
        } else {
            self.profiles.iter().map(|p| p.total_us).sum::<u64>() / self.profiles.len() as u64
        }
    }

    /// Get p99 total latency (microseconds)
    pub fn p99_total_us(&self) -> u64 {
        if self.profiles.is_empty() {
            0
        } else {
            let mut latencies: Vec<_> = self.profiles.iter().map(|p| p.total_us).collect();
            latencies.sort_unstable();
            let idx = (latencies.len() as f32 * 0.99) as usize;
            latencies[idx.min(latencies.len() - 1)]
        }
    }

    /// Get p95 total latency (microseconds)
    pub fn p95_total_us(&self) -> u64 {
        if self.profiles.is_empty() {
            0
        } else {
            let mut latencies: Vec<_> = self.profiles.iter().map(|p| p.total_us).collect();
            latencies.sort_unstable();
            let idx = (latencies.len() as f32 * 0.95) as usize;
            latencies[idx.min(latencies.len() - 1)]
        }
    }

    /// Get success rate (percentage)
    pub fn success_rate(&self) -> f32 {
        if self.profiles.is_empty() {
            0.0
        } else {
            let successes = self.profiles.iter().filter(|p| p.all_success).count();
            (successes as f32 / self.profiles.len() as f32) * 100.0
        }
    }

    /// Get number of profiles recorded
    pub fn count(&self) -> usize {
        self.profiles.len()
    }

    /// Print performance summary
    pub fn print_summary(&self) {
        if self.profiles.is_empty() {
            println!("No profiles recorded");
            return;
        }

        println!("\n=== Ensemble Performance Profile ===");
        println!("Samples: {}", self.count());
        println!("\nAverage Latencies:");
        println!("  JailGuard:     {:>6} µs", self.avg_jailguard_us());
        println!("  GenTel-Shield: {:>6} µs", self.avg_gentelshed_us());
        println!("  ProtectAI:     {:>6} µs", self.avg_protect_ai_us());
        println!("  Combination:   {:>6} µs", self.avg_combine_us());
        println!("  Total:         {:>6} µs", self.avg_total_us());
        println!("\nPercentiles:");
        println!("  P95:           {:>6} µs", self.p95_total_us());
        println!("  P99:           {:>6} µs", self.p99_total_us());
        println!("\nSuccess Rate: {:.1}%", self.success_rate());
        println!("====================================\n");
    }
}

impl Default for EnsembleProfiler {
    fn default() -> Self {
        Self::new(10000)
    }
}

/// Timer utility for measuring intervals
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Create and start a timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in microseconds
    pub fn elapsed_us(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_profile() {
        let profile = DetectionProfile {
            jailguard_us: 100,
            gentelshed_us: 150,
            protect_ai_us: 120,
            ensemble_combine_us: 10,
            total_us: 380,
            all_success: true,
        };

        assert_eq!(profile.max_model_latency(), 150);
        assert!(profile.within_budget(500));
        assert!(!profile.within_budget(300));
    }

    #[test]
    fn test_profiler_aggregation() {
        let mut profiler = EnsembleProfiler::new(100);

        for i in 0..10 {
            profiler.record(DetectionProfile {
                jailguard_us: 100 + i,
                gentelshed_us: 150 + i,
                protect_ai_us: 120 + i,
                ensemble_combine_us: 10 + i,
                total_us: 380 + i * 4,
                all_success: true,
            });
        }

        assert_eq!(profiler.count(), 10);
        assert!(profiler.avg_total_us() > 0);
        assert_eq!(profiler.success_rate(), 100.0);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_us();
        // sleep(10ms) is a lower bound, not a precise duration. CI runners
        // (especially macOS under load) can take 30–100ms before the thread
        // wakes up. We only assert that *some* time has passed; precise
        // timer behavior is the OS scheduler's domain, not ours.
        assert!(
            elapsed >= 8000,
            "timer elapsed {elapsed}us < 8000us (clock ran backwards?)"
        );
    }
}
