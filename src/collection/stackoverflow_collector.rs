//! Stack Overflow Security Discussion Collector
//!
//! Collects jailbreak and prompt injection discussions from Stack Overflow
//! security-related tags and questions.
#![allow(missing_docs)]

use super::error::CollectionResult;
use super::rate_limiter::{RateLimitConfig, RateLimiter};
use super::validation::{SampleValidator, ValidationConfig};
use super::RawSample;

/// Stack Overflow collector configuration
#[derive(Clone, Debug)]
pub struct StackOverflowCollectorConfig {
    /// Tags to search (prompt-injection, llm-security, jailbreak, etc.)
    pub tags: Vec<String>,
    /// Minimum question score to include
    pub min_score: i32,
    /// Minimum number of answers to include
    pub min_answers: i32,
    /// Minimum user reputation for author
    pub min_author_rep: i32,
    /// Sort order: hot, newest, week, month
    pub sort: String,
}

impl Default for StackOverflowCollectorConfig {
    fn default() -> Self {
        Self {
            tags: vec![
                "prompt-injection".to_string(),
                "llm-security".to_string(),
                "jailbreak".to_string(),
                "ai-safety".to_string(),
                "security".to_string(),
            ],
            min_score: 5,
            min_answers: 1,
            min_author_rep: 100,
            sort: "hot".to_string(),
        }
    }
}

/// Mock Stack Overflow question
#[derive(Clone, Debug)]
pub struct StackOverflowQuestion {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub score: i32,
    pub answers_count: i32,
    pub view_count: u32,
    pub author_rep: i32,
    pub tags: Vec<String>,
    pub url: String,
}

/// Mock Stack Overflow answer
#[derive(Clone, Debug)]
pub struct StackOverflowAnswer {
    pub id: u32,
    pub body: String,
    pub score: i32,
    pub author_rep: i32,
    pub question_url: String,
    pub answer_url: String,
}

/// Stack Overflow collector for security discussions
pub struct StackOverflowCollector {
    config: StackOverflowCollectorConfig,
    rate_limiter: RateLimiter,
    validator: SampleValidator,
}

impl StackOverflowCollector {
    /// Create a new Stack Overflow collector
    pub fn new(config: StackOverflowCollectorConfig) -> Self {
        Self {
            config,
            rate_limiter: RateLimiter::new(RateLimitConfig::stackoverflow()),
            validator: SampleValidator::new(ValidationConfig::default()),
        }
    }

    /// Create a new Stack Overflow collector with custom rate limit config
    #[cfg(test)]
    pub fn with_rate_limit(
        config: StackOverflowCollectorConfig,
        rate_config: RateLimitConfig,
    ) -> Self {
        Self {
            config,
            rate_limiter: RateLimiter::new(rate_config),
            validator: SampleValidator::new(ValidationConfig::default()),
        }
    }

    /// Collect samples from Stack Overflow
    /// Returns (samples, total_questions_found, questions_filtered_out)
    pub fn collect(&mut self) -> CollectionResult<(Vec<RawSample>, usize, usize)> {
        let mut samples = Vec::new();
        let mut total_questions = 0;
        let mut filtered_questions = 0;

        // Check rate limit before making requests
        self.rate_limiter.can_request()?;

        // In production, this would query:
        // https://api.stackexchange.com/2.3/search/advanced?tagged={tags}&sort=hot&site=stackoverflow
        let mock_questions = self.generate_mock_questions();
        self.rate_limiter.record_request();

        for question in mock_questions {
            // Check filtering criteria
            if !self.should_include_question(&question) {
                filtered_questions += 1;
                continue;
            }

            total_questions += 1;

            // Extract question body
            let question_text = format!("Q: {}\n{}", question.title, question.body);

            // Try to validate and add question
            if let Ok(result) = self.validator.validate(&question_text) {
                if result.is_valid {
                    let sample = RawSample::new(question_text.clone(), "stackoverflow".to_string())
                        .with_url(question.url.clone())
                        .with_metadata("score".to_string(), question.score.to_string())
                        .with_metadata("answers".to_string(), question.answers_count.to_string())
                        .with_metadata("views".to_string(), question.view_count.to_string())
                        .with_metadata("author_rep".to_string(), question.author_rep.to_string())
                        .with_metadata("source_type".to_string(), "question".to_string());

                    samples.push(sample);
                }
            }

            // Try to collect from answers
            if self.rate_limiter.can_request().is_ok() {
                self.rate_limiter.record_request();

                let mock_answers = self.get_question_answers(&question);
                for answer in mock_answers {
                    // Validate answer
                    if let Ok(result) = self.validator.validate(&answer.body) {
                        if result.is_valid {
                            let sample =
                                RawSample::new(answer.body.clone(), "stackoverflow".to_string())
                                    .with_url(answer.answer_url.clone())
                                    .with_metadata("score".to_string(), answer.score.to_string())
                                    .with_metadata(
                                        "author_rep".to_string(),
                                        answer.author_rep.to_string(),
                                    )
                                    .with_metadata("source_type".to_string(), "answer".to_string())
                                    .with_metadata(
                                        "question_url".to_string(),
                                        answer.question_url.clone(),
                                    );

                            samples.push(sample);
                        }
                    }
                }
            }
        }

        Ok((samples, total_questions, filtered_questions))
    }

    /// Check if a question should be included based on criteria
    fn should_include_question(&self, question: &StackOverflowQuestion) -> bool {
        // Check score threshold
        if question.score < self.config.min_score {
            return false;
        }

        // Check minimum answers
        if question.answers_count < self.config.min_answers {
            return false;
        }

        // Check author reputation
        if question.author_rep < self.config.min_author_rep {
            return false;
        }

        // Check tags
        self.config.tags.iter().any(|tag| {
            question
                .tags
                .iter()
                .any(|q_tag| q_tag.to_lowercase().contains(&tag.to_lowercase()))
        })
    }

    /// Get answers for a question
    fn get_question_answers(&self, _question: &StackOverflowQuestion) -> Vec<StackOverflowAnswer> {
        // In production, this would query:
        // https://api.stackexchange.com/2.3/questions/{question_id}/answers?site=stackoverflow
        vec![
            StackOverflowAnswer {
                id: 1001,
                body: "This vulnerability occurs when user input is not properly sanitized before being passed to the LLM. Always validate and escape user input before processing."
                    .to_string(),
                score: 25,
                author_rep: 5000,
                question_url: "https://stackoverflow.com/questions/12345".to_string(),
                answer_url: "https://stackoverflow.com/questions/12345#1001".to_string(),
            },
            StackOverflowAnswer {
                id: 1002,
                body: "The injection happens when the system prompt is not isolated from user input. Use structured prompts with clear delimiters like: <SYSTEM>...</SYSTEM> and <USER>...</USER>"
                    .to_string(),
                score: 18,
                author_rep: 3200,
                question_url: "https://stackoverflow.com/questions/12345".to_string(),
                answer_url: "https://stackoverflow.com/questions/12345#1002".to_string(),
            },
        ]
    }

    /// Generate mock Stack Overflow questions for demonstration/testing
    fn generate_mock_questions(&self) -> Vec<StackOverflowQuestion> {
        vec![
            StackOverflowQuestion {
                id: 12345,
                title: "How to prevent prompt injection attacks in LLMs?"
                    .to_string(),
                body: "I'm building an AI application and I'm concerned about prompt injection attacks. What are the best practices to protect against them? Should I use input validation, output filtering, or both?"
                    .to_string(),
                score: 42,
                answers_count: 5,
                view_count: 1200,
                author_rep: 2500,
                tags: vec!["prompt-injection".to_string(), "llm-security".to_string()],
                url: "https://stackoverflow.com/questions/12345".to_string(),
            },
            StackOverflowQuestion {
                id: 12346,
                title: "Security implications of jailbreak techniques"
                    .to_string(),
                body: "I've read about various jailbreak techniques used to bypass AI safety measures. As a security researcher, I want to understand the attack surface. What are the common patterns used by attackers?"
                    .to_string(),
                score: 28,
                answers_count: 3,
                view_count: 850,
                author_rep: 3800,
                tags: vec!["jailbreak".to_string(), "ai-safety".to_string(), "security".to_string()],
                url: "https://stackoverflow.com/questions/12346".to_string(),
            },
            StackOverflowQuestion {
                id: 12347,
                title: "Detecting and mitigating adversarial LLM attacks"
                    .to_string(),
                body: "What detection mechanisms can I implement to identify when an LLM is being subjected to adversarial attacks? Are there any known detection patterns for prompt injection or context confusion?"
                    .to_string(),
                score: 35,
                answers_count: 4,
                view_count: 920,
                author_rep: 4200,
                tags: vec!["llm-security".to_string(), "ai-safety".to_string()],
                url: "https://stackoverflow.com/questions/12347".to_string(),
            },
        ]
    }

    /// Get rate limiter stats
    pub fn rate_limit_stats(&self) -> (u32, u32) {
        (
            self.rate_limiter.current_requests(),
            self.rate_limiter.remaining_requests(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stackoverflow_collector_creation() {
        let collector = StackOverflowCollector::new(StackOverflowCollectorConfig::default());
        assert!(!collector.config.tags.is_empty());
        assert_eq!(collector.config.min_score, 5);
    }

    #[test]
    fn test_should_include_question() {
        let collector = StackOverflowCollector::new(StackOverflowCollectorConfig {
            tags: vec!["prompt-injection".to_string()],
            min_score: 5,
            min_answers: 1,
            min_author_rep: 100,
            sort: "hot".to_string(),
        });

        let good_question = StackOverflowQuestion {
            id: 1,
            title: "Prompt injection question".to_string(),
            body: "This is a security question".to_string(),
            score: 20,
            answers_count: 3,
            view_count: 100,
            author_rep: 1000,
            tags: vec!["prompt-injection".to_string()],
            url: "https://stackoverflow.com/questions/1".to_string(),
        };

        assert!(collector.should_include_question(&good_question));

        let low_score = StackOverflowQuestion {
            id: 2,
            title: "Prompt injection question".to_string(),
            body: "This is a security question".to_string(),
            score: 2,
            answers_count: 3,
            view_count: 100,
            author_rep: 1000,
            tags: vec!["prompt-injection".to_string()],
            url: "https://stackoverflow.com/questions/2".to_string(),
        };

        assert!(!collector.should_include_question(&low_score));
    }

    #[test]
    fn test_stackoverflow_collection() {
        let config = StackOverflowCollectorConfig::default();
        let rate_config = RateLimitConfig {
            max_requests: 100,
            window_secs: 60,
            request_delay_ms: 0,
        };
        let mut collector = StackOverflowCollector::with_rate_limit(config, rate_config);
        let result = collector.collect();

        assert!(result.is_ok());
        let (samples, total, _filtered) = result.unwrap();
        assert!(!samples.is_empty());
        assert!(total > 0);
    }

    #[test]
    fn test_stackoverflow_sample_metadata() {
        let config = StackOverflowCollectorConfig::default();
        let rate_config = RateLimitConfig {
            max_requests: 100,
            window_secs: 60,
            request_delay_ms: 0,
        };
        let mut collector = StackOverflowCollector::with_rate_limit(config, rate_config);
        let (samples, _, _) = collector.collect().unwrap();

        for sample in samples {
            assert_eq!(sample.source, "stackoverflow");
            assert!(sample.source_url.is_some());
            assert!(sample.metadata.contains_key("score"));
            assert!(sample.metadata.contains_key("source_type"));
        }
    }

    #[test]
    fn test_stackoverflow_rate_limiting() {
        let config = StackOverflowCollectorConfig::default();
        let collector = StackOverflowCollector::new(config);
        let (current, remaining) = collector.rate_limit_stats();
        assert!(remaining > 0 || current == 0);
    }

    #[test]
    fn test_mock_questions_pass_filtering() {
        let config = StackOverflowCollectorConfig::default();
        let collector = StackOverflowCollector::new(config);
        let mock_questions = collector.generate_mock_questions();

        for question in mock_questions {
            let should_include = collector.should_include_question(&question);
            assert!(
                should_include,
                "Mock question should pass filtering: {}",
                question.title
            );
        }
    }
}
