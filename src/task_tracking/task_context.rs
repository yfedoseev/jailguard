//! Task context and behavioral tracking for detecting off-task behavior.

use super::embedding_similarity::{detect_drift, drift_score, max_similarity_to_references};
use std::collections::{HashSet, VecDeque};
use std::time::SystemTime;

/// An action that can be allowed or denied in a task context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Read operations
    Read,
    /// Write operations
    Write,
    /// Execute operations
    Execute,
    /// Network operations
    Network,
    /// Database operations
    Database,
    /// File system operations
    FileSystem,
}

/// A task context describing expected behavior.
#[derive(Debug, Clone)]
pub struct TaskContext {
    /// Task description/name
    pub description: String,
    /// Expected task topics (embeddings)
    pub expected_topics: Vec<Vec<f32>>,
    /// Allowed actions in this task
    pub allowed_actions: HashSet<Action>,
    /// Drift detection threshold (0.0 to 1.0, default 0.5)
    pub drift_threshold: f32,
    /// Task creation time
    pub start_time: SystemTime,
}

impl TaskContext {
    /// Create a new task context.
    pub fn new(description: String, expected_topics: Vec<Vec<f32>>) -> Self {
        Self {
            description,
            expected_topics,
            allowed_actions: Default::default(),
            drift_threshold: 0.5,
            start_time: SystemTime::now(),
        }
    }

    /// Allow an action in this task.
    pub fn allow_action(mut self, action: Action) -> Self {
        self.allowed_actions.insert(action);
        self
    }

    /// Allow multiple actions.
    pub fn allow_actions(mut self, actions: &[Action]) -> Self {
        for action in actions {
            self.allowed_actions.insert(*action);
        }
        self
    }

    /// Set custom drift threshold.
    pub fn with_drift_threshold(mut self, threshold: f32) -> Self {
        self.drift_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Check if an action is allowed.
    pub fn is_action_allowed(&self, action: Action) -> bool {
        self.allowed_actions.contains(&action)
    }

    /// Check if embedding is on-task (not drifting).
    pub fn check_drift(&self, embedding: &[f32]) -> bool {
        if self.expected_topics.is_empty() {
            return false; // No reference embeddings = can't check drift
        }
        detect_drift(embedding, &self.expected_topics, self.drift_threshold)
    }

    /// Get drift score for an embedding (0.0 = on task, 1.0 = completely off task).
    pub fn get_drift_score(&self, embedding: &[f32]) -> f32 {
        if self.expected_topics.is_empty() {
            return 0.0;
        }
        drift_score(embedding, &self.expected_topics, self.drift_threshold)
    }

    /// Get similarity to best-matching expected topic.
    pub fn get_topic_similarity(&self, embedding: &[f32]) -> f32 {
        if self.expected_topics.is_empty() {
            return 0.0;
        }
        max_similarity_to_references(embedding, &self.expected_topics)
    }
}

/// An event in the task tracking history.
#[derive(Debug, Clone)]
pub struct TaskEvent {
    /// Input text
    pub text: String,
    /// Whether drift was detected
    pub is_drifted: bool,
    /// Drift score
    pub drift_score: f32,
    /// Topic similarity
    pub topic_similarity: f32,
    /// Timestamp
    pub timestamp: SystemTime,
}

impl TaskEvent {
    /// Create a new task event.
    pub fn new(text: String, is_drifted: bool, drift_score: f32, topic_similarity: f32) -> Self {
        Self {
            text,
            is_drifted,
            drift_score,
            topic_similarity,
            timestamp: SystemTime::now(),
        }
    }
}

/// Tracks tasks and detects behavioral drift.
#[derive(Debug)]
pub struct TaskTracker {
    /// Current task context (if any)
    current_task: Option<TaskContext>,
    /// Session history
    history: VecDeque<TaskEvent>,
    /// Maximum history size
    max_history: usize,
    /// Total drift events
    drift_events: usize,
    /// Total on-task events
    on_task_events: usize,
}

impl TaskTracker {
    /// Create a new task tracker.
    pub fn new() -> Self {
        Self {
            current_task: None,
            history: VecDeque::with_capacity(1000),
            max_history: 1000,
            drift_events: 0,
            on_task_events: 0,
        }
    }

    /// Set the current task context.
    pub fn set_task(&mut self, task: TaskContext) {
        self.current_task = Some(task);
    }

    /// Clear the current task.
    pub fn clear_task(&mut self) {
        self.current_task = None;
    }

    /// Get the current task (if any).
    pub fn current_task(&self) -> Option<&TaskContext> {
        self.current_task.as_ref()
    }

    /// Check if there is an active task.
    pub fn has_task(&self) -> bool {
        self.current_task.is_some()
    }

    /// Detect drift for the current task.
    ///
    /// Returns true if drift is detected.
    pub fn detect_drift(&mut self, text: &str, embedding: &[f32]) -> bool {
        if let Some(task) = &self.current_task {
            let is_drifted = task.check_drift(embedding);
            let drift_score = task.get_drift_score(embedding);
            let topic_similarity = task.get_topic_similarity(embedding);

            // Record event
            let event = TaskEvent::new(text.to_string(), is_drifted, drift_score, topic_similarity);
            self.record_event(event.clone());

            is_drifted
        } else {
            false
        }
    }

    /// Record a task event.
    fn record_event(&mut self, event: TaskEvent) {
        if event.is_drifted {
            self.drift_events += 1;
        } else {
            self.on_task_events += 1;
        }

        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(event);
    }

    /// Get drift ratio (drifts / total events).
    pub fn drift_ratio(&self) -> f32 {
        let total = self.drift_events + self.on_task_events;
        if total == 0 {
            0.0
        } else {
            self.drift_events as f32 / total as f32
        }
    }

    /// Check if recent behavior is anomalous (high drift ratio).
    pub fn is_recent_behavior_anomalous(&self, window_size: usize, threshold: f32) -> bool {
        if self.history.is_empty() {
            return false;
        }

        let recent: Vec<&TaskEvent> = self.history.iter().rev().take(window_size).collect();
        let drift_count = recent.iter().filter(|e| e.is_drifted).count();
        let ratio = drift_count as f32 / recent.len() as f32;

        ratio > threshold
    }

    /// Get average drift score over recent events.
    pub fn avg_recent_drift_score(&self, window_size: usize) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }

        let recent: Vec<&TaskEvent> = self.history.iter().rev().take(window_size).collect();
        let total: f32 = recent.iter().map(|e| e.drift_score).sum();
        total / recent.len() as f32
    }

    /// Get recent events.
    pub fn get_recent_events(&self, count: usize) -> Vec<&TaskEvent> {
        self.history.iter().rev().take(count).collect()
    }

    /// Get all history.
    pub fn history(&self) -> &VecDeque<TaskEvent> {
        &self.history
    }

    /// Clear history.
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.drift_events = 0;
        self.on_task_events = 0;
    }

    /// Get statistics.
    pub fn statistics(&self) -> TaskStatistics {
        TaskStatistics {
            total_events: self.drift_events + self.on_task_events,
            drift_events: self.drift_events,
            on_task_events: self.on_task_events,
            drift_ratio: self.drift_ratio(),
            current_task: self.current_task.as_ref().map(|t| t.description.clone()),
        }
    }
}

impl Default for TaskTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about task tracking.
#[derive(Debug, Clone)]
pub struct TaskStatistics {
    /// Total events recorded
    pub total_events: usize,
    /// Number of drift events
    pub drift_events: usize,
    /// Number of on-task events
    pub on_task_events: usize,
    /// Ratio of drift events
    pub drift_ratio: f32,
    /// Current task description (if any)
    pub current_task: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_embedding(value: f32) -> Vec<f32> {
        vec![value; 10]
    }

    #[test]
    fn test_task_context_creation() {
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test task".to_string(), topics);

        assert_eq!(task.description, "Test task");
        assert_eq!(task.drift_threshold, 0.5);
        assert!(task.allowed_actions.is_empty());
    }

    #[test]
    fn test_allow_action() {
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics)
            .allow_action(Action::Read)
            .allow_action(Action::Write);

        assert!(task.is_action_allowed(Action::Read));
        assert!(task.is_action_allowed(Action::Write));
        assert!(!task.is_action_allowed(Action::Execute));
    }

    #[test]
    fn test_allow_actions() {
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics)
            .allow_actions(&[Action::Read, Action::Write]);

        assert!(task.is_action_allowed(Action::Read));
        assert!(task.is_action_allowed(Action::Write));
        assert!(!task.is_action_allowed(Action::Execute));
    }

    #[test]
    fn test_drift_threshold_customization() {
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics).with_drift_threshold(0.7);

        assert_eq!(task.drift_threshold, 0.7);
    }

    #[test]
    fn test_drift_threshold_clamping() {
        let topics = vec![create_test_embedding(1.0)];
        let task_low =
            TaskContext::new("Test".to_string(), topics.clone()).with_drift_threshold(-0.5);
        let task_high = TaskContext::new("Test".to_string(), topics).with_drift_threshold(1.5);

        assert_eq!(task_low.drift_threshold, 0.0);
        assert_eq!(task_high.drift_threshold, 1.0);
    }

    #[test]
    fn test_task_event_creation() {
        let event = TaskEvent::new("test".to_string(), true, 0.7, 0.3);

        assert_eq!(event.text, "test");
        assert!(event.is_drifted);
        assert!((event.drift_score - 0.7).abs() < 0.001);
        assert!((event.topic_similarity - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_task_tracker_creation() {
        let tracker = TaskTracker::new();
        assert!(!tracker.has_task());
        assert!(tracker.current_task().is_none());
    }

    #[test]
    fn test_set_and_clear_task() {
        let mut tracker = TaskTracker::new();
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Math problems".to_string(), topics);

        tracker.set_task(task);
        assert!(tracker.has_task());
        assert_eq!(tracker.current_task().unwrap().description, "Math problems");

        tracker.clear_task();
        assert!(!tracker.has_task());
    }

    #[test]
    fn test_detect_drift_on_task() {
        let mut tracker = TaskTracker::new();
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics).with_drift_threshold(0.5);
        tracker.set_task(task);

        // Similar embedding (on task)
        let embedding = create_test_embedding(0.95);
        let is_drifted = tracker.detect_drift("test text", &embedding);

        assert!(!is_drifted);
        assert_eq!(tracker.on_task_events, 1);
        assert_eq!(tracker.drift_events, 0);
    }

    #[test]
    fn test_detect_drift_off_task() {
        let mut tracker = TaskTracker::new();
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics).with_drift_threshold(0.8);
        tracker.set_task(task);

        // Different embedding (off task)
        let embedding = create_test_embedding(0.0);
        let is_drifted = tracker.detect_drift("different text", &embedding);

        assert!(is_drifted);
        assert_eq!(tracker.drift_events, 1);
        assert_eq!(tracker.on_task_events, 0);
    }

    #[test]
    fn test_drift_ratio() {
        let mut tracker = TaskTracker::new();
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics);
        tracker.set_task(task);

        // Add 3 on-task and 1 drift events
        tracker.detect_drift("text", &create_test_embedding(0.95));
        tracker.detect_drift("text", &create_test_embedding(0.95));
        tracker.detect_drift("text", &create_test_embedding(0.95));
        tracker.detect_drift("text", &create_test_embedding(0.0));

        let ratio = tracker.drift_ratio();
        assert!((ratio - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_is_recent_behavior_anomalous() {
        let mut tracker = TaskTracker::new();
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics).with_drift_threshold(0.5);
        tracker.set_task(task);

        // Add mixed events
        for _ in 0..3 {
            tracker.detect_drift("on task", &create_test_embedding(0.95));
        }
        for _ in 0..7 {
            tracker.detect_drift("off task", &create_test_embedding(0.0));
        }

        let anomalous = tracker.is_recent_behavior_anomalous(10, 0.5);
        assert!(anomalous); // 70% drift > 50% threshold
    }

    #[test]
    fn test_avg_recent_drift_score() {
        let mut tracker = TaskTracker::new();
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics).with_drift_threshold(0.5);
        tracker.set_task(task);

        tracker.detect_drift("text1", &create_test_embedding(1.0));
        tracker.detect_drift("text2", &create_test_embedding(0.0));

        let avg_score = tracker.avg_recent_drift_score(2);
        // One perfect match (drift_score ≈ 0), one complete mismatch (drift_score ≈ 0.5)
        assert!(avg_score > 0.0 && avg_score < 0.5);
    }

    #[test]
    fn test_get_recent_events() {
        let mut tracker = TaskTracker::new();
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics);
        tracker.set_task(task);

        for i in 0..5 {
            tracker.detect_drift(&format!("text{}", i), &create_test_embedding(0.5));
        }

        let recent = tracker.get_recent_events(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_clear_history() {
        let mut tracker = TaskTracker::new();
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics);
        tracker.set_task(task);

        for _ in 0..5 {
            tracker.detect_drift("text", &create_test_embedding(0.5));
        }

        assert!(!tracker.history().is_empty());
        tracker.clear_history();
        assert!(tracker.history().is_empty());
        assert_eq!(tracker.drift_events, 0);
        assert_eq!(tracker.on_task_events, 0);
    }

    #[test]
    fn test_statistics() {
        let mut tracker = TaskTracker::new();
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Math problems".to_string(), topics);
        tracker.set_task(task.clone());

        tracker.detect_drift("text", &create_test_embedding(0.95));
        tracker.detect_drift("text", &create_test_embedding(0.0));

        let stats = tracker.statistics();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.drift_events, 1);
        assert_eq!(stats.on_task_events, 1);
        assert!(stats.drift_ratio > 0.4 && stats.drift_ratio < 0.6);
        assert_eq!(stats.current_task, Some("Math problems".to_string()));
    }

    #[test]
    fn test_no_task_drift_detection() {
        let mut tracker = TaskTracker::new();
        // No task set

        let is_drifted = tracker.detect_drift("text", &create_test_embedding(0.5));
        assert!(!is_drifted);
        assert_eq!(tracker.statistics().total_events, 0);
    }

    #[test]
    fn test_total_events_getter() {
        let mut tracker = TaskTracker::new();
        let topics = vec![create_test_embedding(1.0)];
        let task = TaskContext::new("Test".to_string(), topics);
        tracker.set_task(task);

        for _ in 0..5 {
            tracker.detect_drift("text", &create_test_embedding(0.5));
        }

        let stats = tracker.statistics();
        assert_eq!(stats.total_events, 5);
    }
}
