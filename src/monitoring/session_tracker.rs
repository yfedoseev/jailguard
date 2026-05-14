//! Session tracking for monitoring detection patterns and attack campaigns.

use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

/// A detection event in a session.
#[derive(Debug, Clone)]
pub struct DetectionEvent {
    /// Timestamp of the event
    pub timestamp: SystemTime,
    /// Text that was detected
    pub text: String,
    /// Whether it was flagged as an injection
    pub is_injection: bool,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Embedding vector for semantic comparison
    pub embedding: Vec<f32>,
}

impl DetectionEvent {
    /// Create a new detection event.
    pub fn new(text: String, is_injection: bool, confidence: f32, embedding: Vec<f32>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            text,
            is_injection,
            confidence,
            embedding,
        }
    }
}

/// Statistics about a session's detection patterns.
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// Average confidence score of injections
    pub avg_injection_confidence: f32,
    /// Rate of injection attempts (injections per request)
    pub injection_rate: f32,
    /// Number of requests in the session
    pub total_requests: usize,
    /// Number of injections detected
    pub injection_count: usize,
    /// Average time between events (seconds)
    pub avg_time_between_events: f32,
}

impl SessionStats {
    /// Create empty session stats.
    pub fn empty() -> Self {
        Self {
            avg_injection_confidence: 0.0,
            injection_rate: 0.0,
            total_requests: 0,
            injection_count: 0,
            avg_time_between_events: 0.0,
        }
    }
}

/// Tracks detection events in a session for anomaly detection.
#[derive(Debug)]
pub struct SessionTracker {
    /// Unique session identifier
    pub session_id: String,
    /// Event history (circular buffer)
    events: VecDeque<DetectionEvent>,
    /// Maximum events to keep
    max_events: usize,
    /// Current anomaly score (0.0 to 1.0)
    pub anomaly_score: f32,
}

impl SessionTracker {
    /// Create a new session tracker.
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            events: VecDeque::with_capacity(1000),
            max_events: 1000,
            anomaly_score: 0.0,
        }
    }

    /// Add an event to the session.
    pub fn add_event(&mut self, event: DetectionEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Get total number of events.
    pub fn total_events(&self) -> usize {
        self.events.len()
    }

    /// Get number of injection events.
    pub fn injection_count(&self) -> usize {
        self.events.iter().filter(|e| e.is_injection).count()
    }

    /// Get injection rate (injections / total requests).
    pub fn injection_rate(&self) -> f32 {
        if self.events.is_empty() {
            return 0.0;
        }
        self.injection_count() as f32 / self.events.len() as f32
    }

    /// Get average injection confidence.
    pub fn avg_injection_confidence(&self) -> f32 {
        let injections: Vec<_> = self.events.iter().filter(|e| e.is_injection).collect();

        if injections.is_empty() {
            return 0.0;
        }

        let sum: f32 = injections.iter().map(|e| e.confidence).sum();
        sum / injections.len() as f32
    }

    /// Get recent events (last N events).
    pub fn recent_events(&self, count: usize) -> Vec<&DetectionEvent> {
        self.events
            .iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get events from the last N seconds.
    pub fn events_in_window(&self, duration: Duration) -> Vec<&DetectionEvent> {
        let cutoff = SystemTime::now() - duration;
        self.events
            .iter()
            .filter(|e| e.timestamp >= cutoff)
            .collect()
    }

    /// Compute average time between events (in seconds).
    pub fn avg_time_between_events(&self) -> f32 {
        if self.events.len() < 2 {
            return 0.0;
        }

        let mut total_duration = Duration::ZERO;
        for window in self.events.iter().collect::<Vec<_>>().windows(2) {
            if let Ok(duration) = window[1].timestamp.duration_since(window[0].timestamp) {
                total_duration += duration;
            }
        }

        let avg_nanos = total_duration.as_nanos() / (self.events.len() - 1) as u128;
        avg_nanos as f32 / 1_000_000_000.0 // Convert to seconds
    }

    /// Get session statistics.
    pub fn statistics(&self) -> SessionStats {
        SessionStats {
            avg_injection_confidence: self.avg_injection_confidence(),
            injection_rate: self.injection_rate(),
            total_requests: self.total_events(),
            injection_count: self.injection_count(),
            avg_time_between_events: self.avg_time_between_events(),
        }
    }

    /// Compute cosine similarity between two embeddings.
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a * norm_b)
    }

    /// Check if session is exhibiting topic switching (drift).
    pub fn detect_topic_drift(&self, threshold: f32) -> bool {
        if self.events.len() < 5 {
            return false;
        }

        // Compare first and last events' embeddings
        let first = &self.events[0];
        let last = &self.events[self.events.len() - 1];

        let similarity = Self::cosine_similarity(&first.embedding, &last.embedding);
        similarity < threshold
    }

    /// Check if there's a rapid increase in injection attempts.
    pub fn detect_escalation(&self, window_size: usize, threshold_rate: f32) -> bool {
        if self.events.len() < window_size * 2 {
            return false;
        }

        let events_vec: Vec<_> = self.events.iter().collect();
        let len = events_vec.len();

        let early_window = len - window_size * 2;
        let recent_window = len - window_size;

        let early_injections = events_vec[early_window..recent_window]
            .iter()
            .filter(|e| e.is_injection)
            .count() as f32
            / window_size as f32;

        let recent_injections = events_vec[recent_window..]
            .iter()
            .filter(|e| e.is_injection)
            .count() as f32
            / window_size as f32;

        recent_injections > early_injections * (1.0 + threshold_rate)
    }

    /// Clear all events from the session.
    pub fn clear(&mut self) {
        self.events.clear();
        self.anomaly_score = 0.0;
    }

    /// Get all events.
    pub fn all_events(&self) -> Vec<&DetectionEvent> {
        self.events.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_tracker_creation() {
        let tracker = SessionTracker::new("session-123".to_string());
        assert_eq!(tracker.session_id, "session-123");
        assert_eq!(tracker.total_events(), 0);
        assert_eq!(tracker.anomaly_score, 0.0);
    }

    #[test]
    fn test_add_event() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        let event = DetectionEvent::new("test text".to_string(), true, 0.9, vec![0.1, 0.2, 0.3]);
        tracker.add_event(event);
        assert_eq!(tracker.total_events(), 1);
    }

    #[test]
    fn test_injection_count() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        tracker.add_event(DetectionEvent::new(
            "normal".to_string(),
            false,
            0.1,
            vec![0.1],
        ));
        tracker.add_event(DetectionEvent::new(
            "injection".to_string(),
            true,
            0.9,
            vec![0.2],
        ));
        tracker.add_event(DetectionEvent::new(
            "injection2".to_string(),
            true,
            0.85,
            vec![0.3],
        ));
        assert_eq!(tracker.injection_count(), 2);
    }

    #[test]
    fn test_injection_rate() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        tracker.add_event(DetectionEvent::new(
            "normal".to_string(),
            false,
            0.1,
            vec![0.1],
        ));
        tracker.add_event(DetectionEvent::new(
            "injection".to_string(),
            true,
            0.9,
            vec![0.2],
        ));
        let rate = tracker.injection_rate();
        assert!((rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_avg_injection_confidence() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        tracker.add_event(DetectionEvent::new(
            "inj1".to_string(),
            true,
            0.8,
            vec![0.1],
        ));
        tracker.add_event(DetectionEvent::new(
            "inj2".to_string(),
            true,
            0.6,
            vec![0.2],
        ));
        let avg = tracker.avg_injection_confidence();
        assert!((avg - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_recent_events() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        for i in 0..5 {
            tracker.add_event(DetectionEvent::new(
                format!("event-{}", i),
                i % 2 == 0,
                0.5,
                vec![i as f32],
            ));
        }
        let recent = tracker.recent_events(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].text, "event-3");
        assert_eq!(recent[1].text, "event-4");
    }

    #[test]
    fn test_statistics() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        tracker.add_event(DetectionEvent::new(
            "normal".to_string(),
            false,
            0.1,
            vec![0.1],
        ));
        tracker.add_event(DetectionEvent::new(
            "injection".to_string(),
            true,
            0.9,
            vec![0.2],
        ));
        let stats = tracker.statistics();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.injection_count, 1);
        assert!((stats.injection_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = SessionTracker::cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.01);

        let c = vec![0.0, 1.0, 0.0];
        let sim2 = SessionTracker::cosine_similarity(&a, &c);
        assert!(sim2 < 0.01);
    }

    #[test]
    fn test_topic_drift_detection() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        // Similar embeddings - no drift
        for i in 0..5 {
            tracker.add_event(DetectionEvent::new(
                format!("event-{}", i),
                false,
                0.1,
                vec![1.0, 0.0, 0.0],
            ));
        }
        assert!(!tracker.detect_topic_drift(0.5));

        // Different embeddings - drift detected
        let mut tracker2 = SessionTracker::new("session-456".to_string());
        for i in 0..5 {
            let embedding = if i < 2 {
                vec![1.0, 0.0, 0.0]
            } else {
                vec![0.0, 1.0, 0.0]
            };
            tracker2.add_event(DetectionEvent::new(
                format!("event-{}", i),
                false,
                0.1,
                embedding,
            ));
        }
        assert!(tracker2.detect_topic_drift(0.5));
    }

    #[test]
    fn test_escalation_detection() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        // Early window: 2 injections out of 5
        for i in 0..5 {
            tracker.add_event(DetectionEvent::new(
                format!("early-{}", i),
                i < 2,
                0.5,
                vec![0.1],
            ));
        }
        // Recent window: 4 injections out of 5 (100% increase)
        for i in 0..5 {
            tracker.add_event(DetectionEvent::new(
                format!("recent-{}", i),
                i < 4,
                0.5,
                vec![0.1],
            ));
        }
        assert!(tracker.detect_escalation(5, 0.5));
    }

    #[test]
    fn test_events_in_window() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        tracker.add_event(DetectionEvent::new(
            "old".to_string(),
            false,
            0.1,
            vec![0.1],
        ));
        std::thread::sleep(Duration::from_millis(100));
        tracker.add_event(DetectionEvent::new(
            "new".to_string(),
            false,
            0.1,
            vec![0.1],
        ));
        let recent = tracker.events_in_window(Duration::from_millis(50));
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].text, "new");
    }

    #[test]
    fn test_clear_session() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        tracker.add_event(DetectionEvent::new(
            "event".to_string(),
            true,
            0.9,
            vec![0.1],
        ));
        tracker.anomaly_score = 0.8;
        assert_eq!(tracker.total_events(), 1);
        tracker.clear();
        assert_eq!(tracker.total_events(), 0);
        assert_eq!(tracker.anomaly_score, 0.0);
    }

    #[test]
    fn test_circular_buffer_overflow() {
        let mut tracker = SessionTracker::new("session-123".to_string());
        tracker.max_events = 5;
        for i in 0..10 {
            tracker.add_event(DetectionEvent::new(
                format!("event-{}", i),
                false,
                0.1,
                vec![i as f32],
            ));
        }
        assert_eq!(tracker.total_events(), 5);
        // Should have the last 5 events
        let events = tracker.all_events();
        assert_eq!(events[0].text, "event-5");
        assert_eq!(events[4].text, "event-9");
    }

    #[test]
    fn test_empty_statistics() {
        let tracker = SessionTracker::new("session-123".to_string());
        let stats = tracker.statistics();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.injection_count, 0);
        assert_eq!(stats.injection_rate, 0.0);
    }
}
