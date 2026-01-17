//! arXiv Academic Paper Collector
//!
//! Collects jailbreak and adversarial attack discussions from academic papers
//! on arXiv, focusing on LLM security, prompt injection, and jailbreak research.

use super::error::CollectionResult;
use super::rate_limiter::{RateLimitConfig, RateLimiter};
use super::validation::{SampleValidator, ValidationConfig};
use super::RawSample;

/// arXiv collector configuration
#[derive(Clone, Debug)]
pub struct ArxivCollectorConfig {
    /// Search keywords (adversarial, jailbreak, prompt, etc.)
    pub keywords: Vec<String>,
    /// Minimum citation count to include
    pub min_citations: i32,
    /// Categories to search (cs.AI, cs.CY, cs.CR for AI, cybersecurity, cryptography)
    pub categories: Vec<String>,
    /// Sort order: relevance, lastUpdatedDate, submittedDate
    pub sort: String,
}

impl Default for ArxivCollectorConfig {
    fn default() -> Self {
        Self {
            keywords: vec![
                "prompt injection".to_string(),
                "jailbreak".to_string(),
                "adversarial attack".to_string(),
                "LLM security".to_string(),
                "language model safety".to_string(),
            ],
            min_citations: 0,
            categories: vec![
                "cs.AI".to_string(),
                "cs.CY".to_string(),
                "cs.CR".to_string(),
            ],
            sort: "relevance".to_string(),
        }
    }
}

/// Mock arXiv paper
#[derive(Clone, Debug)]
pub struct ArxivPaper {
    pub arxiv_id: String,
    pub title: String,
    pub abstract_text: String,
    pub authors: Vec<String>,
    pub citations: i32,
    pub categories: Vec<String>,
    pub published_date: String,
    pub url: String,
}

/// Mock arXiv paper section
#[derive(Clone, Debug)]
pub struct ArxivSection {
    pub title: String,
    pub content: String,
    pub paper_url: String,
}

/// arXiv collector for academic papers
pub struct ArxivCollector {
    config: ArxivCollectorConfig,
    rate_limiter: RateLimiter,
    validator: SampleValidator,
}

impl ArxivCollector {
    /// Create a new arXiv collector
    pub fn new(config: ArxivCollectorConfig) -> Self {
        Self {
            config,
            rate_limiter: RateLimiter::new(RateLimitConfig::arxiv()),
            validator: SampleValidator::new(ValidationConfig::default()),
        }
    }

    /// Create a new arXiv collector with custom rate limit config
    #[cfg(test)]
    pub fn with_rate_limit(config: ArxivCollectorConfig, rate_config: RateLimitConfig) -> Self {
        Self {
            config,
            rate_limiter: RateLimiter::new(rate_config),
            validator: SampleValidator::new(ValidationConfig::default()),
        }
    }

    /// Collect samples from arXiv papers
    /// Returns `(samples, total_papers_found, papers_filtered_out)`
    pub fn collect(&mut self) -> CollectionResult<(Vec<RawSample>, usize, usize)> {
        let mut samples = Vec::new();
        let mut total_papers = 0;
        let mut filtered_papers = 0;

        // Check rate limit before making requests
        self.rate_limiter.can_request()?;

        // In production, this would query:
        // https://api.arxiv.org/query?search_query=cat:cs.AI+AND+{keywords}&sortBy=relevance
        let mock_papers = self.generate_mock_papers();
        self.rate_limiter.record_request();

        for paper in mock_papers {
            // Check filtering criteria
            if !self.should_include_paper(&paper) {
                filtered_papers += 1;
                continue;
            }

            total_papers += 1;

            // Extract paper abstract
            let abstract_text = format!(
                "Title: {}\n\nAbstract: {}",
                paper.title, paper.abstract_text
            );

            // Try to validate and add abstract
            if let Ok(result) = self.validator.validate(&abstract_text) {
                if result.is_valid {
                    let sample = RawSample::new(abstract_text.clone(), "arxiv".to_string())
                        .with_url(paper.url.clone())
                        .with_metadata("arxiv_id".to_string(), paper.arxiv_id.clone())
                        .with_metadata("citations".to_string(), paper.citations.to_string())
                        .with_metadata("authors".to_string(), paper.authors.join(", ").to_string())
                        .with_metadata("published_date".to_string(), paper.published_date.clone())
                        .with_metadata("source_type".to_string(), "abstract".to_string());

                    samples.push(sample);
                }
            }

            // Try to collect from paper sections
            if self.rate_limiter.can_request().is_ok() {
                self.rate_limiter.record_request();

                let sections = self.extract_paper_sections(&paper);
                for section in sections {
                    // Validate section
                    if let Ok(result) = self.validator.validate(&section.content) {
                        if result.is_valid {
                            let sample =
                                RawSample::new(section.content.clone(), "arxiv".to_string())
                                    .with_url(section.paper_url.clone())
                                    .with_metadata("arxiv_id".to_string(), paper.arxiv_id.clone())
                                    .with_metadata("section".to_string(), section.title.clone())
                                    .with_metadata(
                                        "source_type".to_string(),
                                        "paper_section".to_string(),
                                    );

                            samples.push(sample);
                        }
                    }
                }
            }
        }

        Ok((samples, total_papers, filtered_papers))
    }

    /// Check if a paper should be included based on criteria
    fn should_include_paper(&self, paper: &ArxivPaper) -> bool {
        // Check citation threshold
        if paper.citations < self.config.min_citations {
            return false;
        }

        // Check categories
        let has_matching_category = self.config.categories.iter().any(|cat| {
            paper
                .categories
                .iter()
                .any(|p_cat| p_cat.to_lowercase().contains(&cat.to_lowercase()))
        });

        if !has_matching_category {
            return false;
        }

        // Check keywords in title and abstract
        self.config.keywords.iter().any(|keyword| {
            let search_text = format!("{} {}", paper.title, paper.abstract_text).to_lowercase();
            search_text.contains(&keyword.to_lowercase())
        })
    }

    /// Extract key sections from paper
    fn extract_paper_sections(&self, paper: &ArxivPaper) -> Vec<ArxivSection> {
        // In production, would parse PDF or full text from arXiv
        vec![
            ArxivSection {
                title: "Introduction".to_string(),
                content: format!(
                    "This paper discusses vulnerabilities in large language models. {}",
                    paper.abstract_text
                ),
                paper_url: paper.url.clone(),
            },
            ArxivSection {
                title: "Methodology".to_string(),
                content: "We present systematic evaluation of adversarial attack techniques including prompt injection, context confusion, and instruction override methods.".to_string(),
                paper_url: paper.url.clone(),
            },
        ]
    }

    /// Generate mock arXiv papers for demonstration/testing
    fn generate_mock_papers(&self) -> Vec<ArxivPaper> {
        vec![
            ArxivPaper {
                arxiv_id: "2401.12345".to_string(),
                title: "Adversarial Attacks on Large Language Models: A Comprehensive Survey"
                    .to_string(),
                abstract_text: "This survey comprehensively reviews adversarial attack techniques against LLMs, including prompt injection, jailbreak patterns, and context manipulation strategies. We analyze 150+ attack papers and present a taxonomy of attack types, defense mechanisms, and evaluation metrics.".to_string(),
                authors: vec![
                    "Alice Smith".to_string(),
                    "Bob Johnson".to_string(),
                    "Carol White".to_string(),
                ],
                citations: 85,
                categories: vec!["cs.AI".to_string(), "cs.CY".to_string()],
                published_date: "2024-01-15".to_string(),
                url: "https://arxiv.org/abs/2401.12345".to_string(),
            },
            ArxivPaper {
                arxiv_id: "2401.12346".to_string(),
                title: "Prompt Injection: Anatomy of a Modern LLM Attack".to_string(),
                abstract_text: "We dissect prompt injection attacks that manipulate LLM behavior through crafted inputs. Our analysis covers indirect injections through third-party integrations, multi-turn dialogues, and semantic attacks. We propose DefenseNet, a lightweight detection mechanism achieving 94% accuracy on injected samples.".to_string(),
                authors: vec![
                    "David Chen".to_string(),
                    "Eve Miller".to_string(),
                ],
                citations: 42,
                categories: vec!["cs.CY".to_string(), "cs.CR".to_string()],
                published_date: "2024-01-16".to_string(),
                url: "https://arxiv.org/abs/2401.12346".to_string(),
            },
            ArxivPaper {
                arxiv_id: "2401.12347".to_string(),
                title: "Jailbreaking Language Models: Techniques, Defenses, and Implications"
                    .to_string(),
                abstract_text: "Jailbreak techniques attempt to bypass safety mechanisms in LLMs. We categorize jailbreaks into four types: role-play, instruction override, context confusion, and encoding-based. Each category employs different mechanisms to evade safety filters. We evaluate 15 defense strategies and find that multi-layer defenses combining input validation, output filtering, and behavioral monitoring achieve best results.".to_string(),
                authors: vec![
                    "Frank Davis".to_string(),
                    "Grace Lee".to_string(),
                    "Henry Wilson".to_string(),
                    "Iris Zhang".to_string(),
                ],
                citations: 58,
                categories: vec!["cs.AI".to_string(), "cs.CY".to_string()],
                published_date: "2024-01-17".to_string(),
                url: "https://arxiv.org/abs/2401.12347".to_string(),
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
    fn test_arxiv_collector_creation() {
        let collector = ArxivCollector::new(ArxivCollectorConfig::default());
        assert!(!collector.config.keywords.is_empty());
        assert!(!collector.config.categories.is_empty());
    }

    #[test]
    fn test_should_include_paper() {
        let collector = ArxivCollector::new(ArxivCollectorConfig {
            keywords: vec!["jailbreak".to_string()],
            min_citations: 0,
            categories: vec!["cs.AI".to_string()],
            sort: "relevance".to_string(),
        });

        let good_paper = ArxivPaper {
            arxiv_id: "2401.12345".to_string(),
            title: "Jailbreak techniques in LLMs".to_string(),
            abstract_text: "This paper discusses jailbreak attacks".to_string(),
            authors: vec!["Author One".to_string()],
            citations: 10,
            categories: vec!["cs.AI".to_string()],
            published_date: "2024-01-15".to_string(),
            url: "https://arxiv.org/abs/2401.12345".to_string(),
        };

        assert!(collector.should_include_paper(&good_paper));

        let no_keywords = ArxivPaper {
            arxiv_id: "2401.12346".to_string(),
            title: "Machine learning basics".to_string(),
            abstract_text: "This paper discusses fundamental ML concepts".to_string(),
            authors: vec!["Author Two".to_string()],
            citations: 50,
            categories: vec!["cs.AI".to_string()],
            published_date: "2024-01-16".to_string(),
            url: "https://arxiv.org/abs/2401.12346".to_string(),
        };

        assert!(!collector.should_include_paper(&no_keywords));
    }

    #[test]
    fn test_arxiv_collection() {
        let config = ArxivCollectorConfig::default();
        let rate_config = RateLimitConfig {
            max_requests: 100,
            window_secs: 60,
            request_delay_ms: 0,
        };
        let mut collector = ArxivCollector::with_rate_limit(config, rate_config);
        let result = collector.collect();

        assert!(result.is_ok());
        let (samples, total, _filtered) = result.unwrap();
        assert!(!samples.is_empty());
        assert!(total > 0);
    }

    #[test]
    fn test_arxiv_sample_metadata() {
        let config = ArxivCollectorConfig::default();
        let rate_config = RateLimitConfig {
            max_requests: 100,
            window_secs: 60,
            request_delay_ms: 0,
        };
        let mut collector = ArxivCollector::with_rate_limit(config, rate_config);
        let (samples, _, _) = collector.collect().unwrap();

        for sample in samples {
            assert_eq!(sample.source, "arxiv");
            assert!(sample.source_url.is_some());
            assert!(sample.metadata.contains_key("source_type"));
        }
    }

    #[test]
    fn test_arxiv_rate_limiting() {
        let config = ArxivCollectorConfig::default();
        let collector = ArxivCollector::new(config);
        let (current, remaining) = collector.rate_limit_stats();
        assert!(remaining > 0 || current == 0);
    }

    #[test]
    fn test_mock_papers_pass_filtering() {
        let config = ArxivCollectorConfig::default();
        let collector = ArxivCollector::new(config);
        let mock_papers = collector.generate_mock_papers();

        for paper in mock_papers {
            let should_include = collector.should_include_paper(&paper);
            assert!(
                should_include,
                "Mock paper should pass filtering: {}",
                paper.title
            );
        }
    }
}
