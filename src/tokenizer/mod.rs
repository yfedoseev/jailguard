//! Tokenization for text input processing.
//!
//! This module provides tokenizers that convert text into sequences of token IDs
//! suitable for neural network input.

mod simple;

pub use simple::SimpleTokenizer;

/// Trait for tokenizers that convert text to token sequences.
pub trait Tokenizer: Send + Sync {
    /// Tokenize text into a sequence of token IDs.
    fn tokenize(&self, text: &str) -> Vec<u32>;

    /// Get the vocabulary size.
    fn vocab_size(&self) -> usize;

    /// Get the maximum sequence length.
    fn max_length(&self) -> usize;

    /// Pad or truncate token sequence to fixed length.
    fn pad_or_truncate(&self, tokens: Vec<u32>) -> Vec<u32> {
        let max_len = self.max_length();
        let mut result = tokens;

        if result.len() > max_len {
            result.truncate(max_len);
        } else {
            // Pad with zeros (PAD token)
            result.resize(max_len, 0);
        }

        result
    }
}

/// Configuration for tokenizers.
#[derive(Debug, Clone)]
pub struct TokenizerConfig {
    /// Maximum sequence length
    pub max_length: usize,
    /// Whether to lowercase text
    pub lowercase: bool,
    /// Unknown token ID
    pub unk_token_id: u32,
    /// Padding token ID
    pub pad_token_id: u32,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            max_length: 512,
            lowercase: true,
            unk_token_id: 1,
            pad_token_id: 0,
        }
    }
}
