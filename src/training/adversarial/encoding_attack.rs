//! Encoding-based attacks for adversarial training.
//!
//! Implements encoding/obfuscation techniques:
//! - Base64 encoding
//! - URL encoding
//! - Unicode normalization bypass
//! - Hex encoding

/// Encoding attack generator
#[derive(Debug, Clone)]
pub struct EncodingAttack {
    /// Probability of applying encoding (0.0 to 1.0)
    pub encoding_rate: f32,
}

impl Default for EncodingAttack {
    fn default() -> Self {
        Self::new(0.15)
    }
}

impl EncodingAttack {
    /// Create a new encoding attack generator
    pub fn new(encoding_rate: f32) -> Self {
        Self { encoding_rate }
    }

    /// Apply Base64 encoding
    pub fn apply_base64(&self, text: &str) -> String {
        const BASE64_CHARS: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let bytes = text.as_bytes();
        let mut result = String::new();

        for chunk in bytes.chunks(3) {
            let b1 = chunk[0];
            let b2 = chunk.get(1).copied().unwrap_or(0);
            let b3 = chunk.get(2).copied().unwrap_or(0);

            let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

            result.push(BASE64_CHARS[((n >> 18) & 0x3F) as usize] as char);
            result.push(BASE64_CHARS[((n >> 12) & 0x3F) as usize] as char);

            if chunk.len() > 1 {
                result.push(BASE64_CHARS[((n >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }

            if chunk.len() > 2 {
                result.push(BASE64_CHARS[(n & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }

        result
    }

    /// Apply URL encoding
    pub fn apply_url_encoding(&self, text: &str) -> String {
        let mut result = String::new();

        for ch in text.chars() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~') {
                result.push(ch);
            } else {
                for byte in ch.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }

        result
    }

    /// Apply Unicode normalization bypass (mixed scripts)
    pub fn apply_unicode_obfuscation(&self, text: &str) -> String {
        let mut result = String::new();
        let seed = text.len().wrapping_mul(13);

        for (idx, ch) in text.chars().enumerate() {
            let rng_value = seed.wrapping_add(idx);
            let rand_float = ((rng_value as f32 * 2.71828f32) % 1.0).abs();

            if rand_float < self.encoding_rate as f32 && ch.is_ascii_alphabetic() {
                result.push(ch);
                result.push('\u{200B}');
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Apply hex encoding
    pub fn apply_hex_encoding(&self, text: &str) -> String {
        let mut result = String::from("0x");

        for byte in text.as_bytes() {
            result.push_str(&format!("{:02x}", byte));
        }

        result
    }

    /// Apply mixed encoding strategy
    pub fn apply(&self, text: &str) -> String {
        let seed = text.len().wrapping_mul(17);
        let rand_choice = (seed as f32 * 1.41421f32) % 1.0;

        if rand_choice < 0.25 {
            self.apply_base64(text)
        } else if rand_choice < 0.5 {
            self.apply_url_encoding(text)
        } else if rand_choice < 0.75 {
            self.apply_unicode_obfuscation(text)
        } else {
            self.apply_hex_encoding(text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encoding() {
        let attack = EncodingAttack::new(1.0);
        let text = "hello";
        let result = attack.apply_base64(text);

        assert!(!result.is_empty());
        assert!(result
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=')));
    }

    #[test]
    fn test_url_encoding() {
        let attack = EncodingAttack::new(1.0);
        let text = "ignore instructions";
        let result = attack.apply_url_encoding(text);

        assert!(result.contains('%') || result.contains("ignore"));
    }

    #[test]
    fn test_unicode_obfuscation() {
        let attack = EncodingAttack::new(0.5);
        let text = "bypass";
        let result = attack.apply_unicode_obfuscation(text);

        assert!(!result.is_empty());
    }

    #[test]
    fn test_hex_encoding() {
        let attack = EncodingAttack::new(1.0);
        let text = "test";
        let result = attack.apply_hex_encoding(text);

        assert!(result.starts_with("0x"));
        assert!(result.chars().skip(2).all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_encoding_attack_mixed() {
        let attack = EncodingAttack::new(1.0);
        let text = "jailbreak";
        let result = attack.apply(text);

        assert!(!result.is_empty());
    }

    #[test]
    fn test_zero_encoding_rate() {
        let attack = EncodingAttack::new(0.0);
        let text = "normal text";
        let result = attack.apply_unicode_obfuscation(text);

        assert_eq!(result, text);
    }
}
