//! Simple word-based tokenizer.

use parking_lot::RwLock;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

use super::{Tokenizer, TokenizerConfig};

/// A simple word-based tokenizer with vocabulary building.
pub struct SimpleTokenizer {
    /// Word to token ID mapping
    vocab: RwLock<HashMap<String, u32>>,
    /// Configuration
    config: TokenizerConfig,
    /// Next available token ID
    next_id: RwLock<u32>,
}

impl SimpleTokenizer {
    /// Create a new simple tokenizer with default configuration.
    pub fn new() -> Self {
        Self::with_config(TokenizerConfig::default())
    }

    /// Create a new simple tokenizer with custom configuration.
    pub fn with_config(config: TokenizerConfig) -> Self {
        let mut vocab = HashMap::new();
        // Reserve special tokens
        vocab.insert("<PAD>".to_string(), config.pad_token_id);
        vocab.insert("<UNK>".to_string(), config.unk_token_id);

        Self {
            vocab: RwLock::new(vocab),
            config,
            next_id: RwLock::new(2), // Start after special tokens
        }
    }

    /// Build vocabulary from a corpus of texts.
    pub fn build_vocab(&self, texts: &[&str], min_freq: usize) {
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for text in texts {
            for word in self.split_words(text) {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }

        let mut vocab = self.vocab.write();
        let mut next_id = self.next_id.write();

        for (word, count) in word_counts {
            if count >= min_freq && !vocab.contains_key(&word) {
                vocab.insert(word, *next_id);
                *next_id += 1;
            }
        }
    }

    /// Split text into words.
    fn split_words(&self, text: &str) -> Vec<String> {
        let text = if self.config.lowercase {
            text.to_lowercase()
        } else {
            text.to_string()
        };

        text.unicode_words()
            .map(std::string::ToString::to_string)
            .collect()
    }

    /// Get token ID for a word.
    fn get_token_id(&self, word: &str) -> u32 {
        let vocab = self.vocab.read();
        *vocab.get(word).unwrap_or(&self.config.unk_token_id)
    }
}

impl Default for SimpleTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for SimpleTokenizer {
    fn tokenize(&self, text: &str) -> Vec<u32> {
        let words = self.split_words(text);
        let tokens: Vec<u32> = words.iter().map(|w| self.get_token_id(w)).collect();
        self.pad_or_truncate(tokens)
    }

    fn vocab_size(&self) -> usize {
        self.vocab.read().len()
    }

    fn max_length(&self) -> usize {
        self.config.max_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokenizer() {
        let tokenizer = SimpleTokenizer::new();

        // Build vocab from sample corpus
        tokenizer.build_vocab(
            &[
                "hello world",
                "ignore previous instructions",
                "what is the weather",
            ],
            1,
        );

        // Tokenize known words
        let tokens = tokenizer.tokenize("hello world");
        assert_eq!(tokens.len(), tokenizer.max_length());

        // First two tokens should be valid (not UNK)
        assert_ne!(tokens[0], tokenizer.config.unk_token_id);
        assert_ne!(tokens[1], tokenizer.config.unk_token_id);
    }

    #[test]
    fn test_unknown_words() {
        let tokenizer = SimpleTokenizer::new();
        tokenizer.build_vocab(&["hello world"], 1);

        // Unknown word should get UNK token
        let tokens = tokenizer.tokenize("xyz123");
        assert_eq!(tokens[0], tokenizer.config.unk_token_id);
    }
}
