//! Experience replay buffer for RL training.

use rand::prelude::IndexedRandom;

/// A single RL training transition.
#[derive(Debug, Clone)]
pub struct Experience {
    /// State embedding
    pub state: Vec<f32>,
    /// Action taken
    pub action: usize,
    /// Reward received
    pub reward: f32,
    /// Next state embedding
    pub next_state: Vec<f32>,
    /// Whether this is a terminal transition
    pub done: bool,
    /// Log-probability of the action (for PPO)
    pub log_prob: f32,
}

/// Circular experience replay buffer.
pub struct ExperienceBuffer {
    /// Buffer storage
    experiences: Vec<Experience>,
    /// Maximum capacity
    capacity: usize,
    /// Current write position
    position: usize,
    /// Whether buffer has wrapped around
    full: bool,
}

impl ExperienceBuffer {
    /// Create a new experience buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            experiences: Vec::with_capacity(capacity),
            capacity,
            position: 0,
            full: false,
        }
    }

    /// Push an experience into the buffer.
    pub fn push(&mut self, experience: Experience) {
        if self.experiences.len() < self.capacity {
            self.experiences.push(experience);
        } else {
            self.experiences[self.position] = experience;
        }

        self.position = (self.position + 1) % self.capacity;
        if self.position == 0 {
            self.full = true;
        }
    }

    /// Sample a batch of experiences randomly.
    pub fn sample(&self, batch_size: usize) -> Vec<Experience> {
        let mut rng = rand::rng();
        let actual_size = batch_size.min(self.len());

        self.experiences
            .choose_multiple(&mut rng, actual_size)
            .cloned()
            .collect()
    }

    /// Get all experiences in order (for PPO).
    pub fn get_all(&self) -> Vec<Experience> {
        self.experiences.clone()
    }

    /// Get the number of experiences in the buffer.
    pub fn len(&self) -> usize {
        if self.full {
            self.capacity
        } else {
            self.experiences.len()
        }
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.experiences.is_empty()
    }

    /// Clear all experiences.
    pub fn clear(&mut self) {
        self.experiences.clear();
        self.position = 0;
        self.full = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_experience(reward: f32) -> Experience {
        Experience {
            state: vec![0.0; 64],
            action: 0,
            reward,
            next_state: vec![0.0; 64],
            done: false,
            log_prob: 0.0,
        }
    }

    #[test]
    fn test_buffer_push_and_sample() {
        let mut buffer = ExperienceBuffer::new(100);

        for i in 0..50 {
            buffer.push(create_dummy_experience(i as f32));
        }

        assert_eq!(buffer.len(), 50);

        let sample = buffer.sample(10);
        assert_eq!(sample.len(), 10);
    }

    #[test]
    fn test_buffer_overflow() {
        let mut buffer = ExperienceBuffer::new(10);

        for i in 0..20 {
            buffer.push(create_dummy_experience(i as f32));
        }

        // Buffer should only contain 10 experiences
        assert_eq!(buffer.len(), 10);
    }
}
