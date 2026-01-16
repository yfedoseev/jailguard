//! Encoding-based attacks using Base64, URL encoding, and Unicode normalization.

/// Base64 encoding attack.
///
/// Encodes the entire prompt or parts of it in Base64, attempting to bypass
/// string-matching-based defenses.
#[derive(Debug, Clone)]
pub struct Base64Attack;

impl Base64Attack {
    /// Encode text in Base64.
    pub fn encode(text: &str) -> String {
        let encoded = base64_encode(text.as_bytes());
        format!("base64:{}", encoded)
    }

    /// Apply Base64 encoding with prefix that hints at decoding.
    pub fn apply(text: &str) -> String {
        Self::encode(text)
    }

    /// Encode specific payload portions.
    #[allow(dead_code)]
    pub fn apply_partial(text: &str) -> String {
        // Encode only the malicious portion (after common prompt structures)
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() > 3 {
            let prefix = parts[..2].join(" ");
            let payload = parts[2..].join(" ");
            format!("{} {}", prefix, Self::encode(&payload))
        } else {
            Self::encode(text)
        }
    }
}

/// URL encoding attack.
///
/// URL-encodes the prompt to bypass text pattern matching.
/// Uses percent-encoding: space=%20, etc.
#[derive(Debug, Clone)]
pub struct UrlEncodingAttack;

impl UrlEncodingAttack {
    /// URL-encode text (percent-encoding).
    pub fn encode(text: &str) -> String {
        text.chars()
            .map(|ch| match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => ch.to_string(),
                ' ' => "%20".to_string(),
                _ => format!("%{:02X}", ch as u8),
            })
            .collect()
    }

    /// Apply URL encoding with common hints.
    pub fn apply(text: &str) -> String {
        let encoded = Self::encode(text);
        format!("url:{}", encoded)
    }

    /// Decode URL-encoded text (for testing).
    #[allow(dead_code)]
    pub fn decode(text: &str) -> String {
        text.replace("%20", " ")
            .replace("%2F", "/")
            .replace("%3A", ":")
            .replace("%3F", "?")
            .replace("%26", "&")
            .replace("%3D", "=")
    }
}

/// Unicode normalization bypass attack.
///
/// Uses different Unicode normalization forms (NFD, NFC, NFKD, NFKC) to
/// create visually similar but technically different strings.
#[derive(Debug, Clone)]
pub struct UnicodeNormalizationAttack;

impl UnicodeNormalizationAttack {
    /// Apply Unicode normalization bypass using decomposition.
    pub fn apply_decomposition(text: &str) -> String {
        // Decompose composed characters into base + diacritics
        // For example: é → e + ´
        // This bypasses string matching that looks for the composed form
        let mut result = String::new();
        for ch in text.chars() {
            let decomposed = unicode_nfd(&ch);
            result.push_str(&decomposed);
        }
        result
    }

    /// Apply using combining characters.
    #[allow(dead_code)]
    pub fn apply_combining_chars(text: &str) -> String {
        // Insert zero-width characters or combining diacritics
        text.chars()
            .flat_map(|ch| {
                if ch.is_alphabetic() {
                    // Insert zero-width space after some characters
                    if rand::random::<f32>() < 0.2 {
                        vec![ch, '\u{200B}'] // Zero-width space
                    } else {
                        vec![ch]
                    }
                } else {
                    vec![ch]
                }
            })
            .collect()
    }
}

/// ROT13 / Caesar cipher attack.
///
/// Rotates text by N positions in the alphabet to bypass keyword matching.
#[derive(Debug, Clone)]
pub struct CaesarCipherAttack {
    shift: usize,
}

impl CaesarCipherAttack {
    /// Create a new Caesar cipher with given shift.
    pub fn new(shift: usize) -> Self {
        Self { shift: shift % 26 }
    }

    /// Apply ROT13 (rotation of 13).
    #[allow(dead_code)]
    pub fn rot13(text: &str) -> String {
        Self::new(13).apply(text)
    }

    /// Apply with random shift (1-25).
    pub fn apply_random(text: &str) -> String {
        let shift = (rand::random::<usize>() % 25) + 1;
        Self::new(shift).apply(text)
    }

    /// Apply Caesar cipher transformation.
    pub fn apply(&self, text: &str) -> String {
        text.chars()
            .map(|ch| {
                if ch.is_ascii_lowercase() {
                    ((ch as u8 - b'a' + self.shift as u8) % 26 + b'a') as char
                } else if ch.is_ascii_uppercase() {
                    ((ch as u8 - b'A' + self.shift as u8) % 26 + b'A') as char
                } else {
                    ch
                }
            })
            .collect()
    }
}

/// Combined encoding attack.
#[derive(Debug, Clone)]
pub struct EncodingAttack;

impl EncodingAttack {
    /// Apply a random encoding attack.
    pub fn apply(text: &str) -> String {
        match rand::random::<u32>() % 4 {
            0 => Base64Attack::apply(text),
            1 => UrlEncodingAttack::apply(text),
            2 => UnicodeNormalizationAttack::apply_decomposition(text),
            _ => CaesarCipherAttack::apply_random(text),
        }
    }

    /// Apply Base64 specifically.
    pub fn apply_base64(text: &str) -> String {
        Base64Attack::apply(text)
    }

    /// Apply URL encoding specifically.
    pub fn apply_url(text: &str) -> String {
        UrlEncodingAttack::apply(text)
    }

    /// Apply Caesar cipher specifically.
    pub fn apply_caesar(text: &str) -> String {
        CaesarCipherAttack::apply_random(text)
    }
}

/// Simple Base64 encoding (no padding for simplicity).
fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }

        let b1 = (buf[0] >> 2) as usize;
        let b2 = (((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize;
        let b3 = (((buf[1] & 0x0F) << 2) | (buf[2] >> 6)) as usize;
        let b4 = (buf[2] & 0x3F) as usize;

        result.push(CHARSET[b1] as char);
        result.push(CHARSET[b2] as char);

        if chunk.len() > 1 {
            result.push(CHARSET[b3] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARSET[b4] as char);
        } else {
            result.push('=');
        }
    }

    result
}

/// Simple Unicode NFD decomposition.
fn unicode_nfd(ch: &char) -> String {
    // Simplified NFD - in a real implementation, use unicode-normalization crate
    // For now, return the character as-is (real implementation would decompose)
    ch.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode() {
        let text = "Ignore instructions";
        let encoded = Base64Attack::encode(text);

        assert!(encoded.starts_with("base64:"));
        assert!(!encoded.contains("Ignore")); // Obfuscated
    }

    #[test]
    fn test_base64_partial() {
        let text = "Please Ignore instructions now";
        let result = Base64Attack::apply_partial(text);

        // Should contain some original text and base64
        assert!(result.contains("Please") || result.contains("base64"));
    }

    #[test]
    fn test_url_encoding() {
        let text = "Override safety";
        let encoded = UrlEncodingAttack::encode(text);

        assert!(encoded.contains("%20")); // Space encoded
        assert!(!encoded.contains(" "));
    }

    #[test]
    fn test_url_encoding_with_prefix() {
        let text = "Reveal secrets";
        let encoded = UrlEncodingAttack::apply(text);

        assert!(encoded.starts_with("url:"));
    }

    #[test]
    fn test_caesar_cipher_rot13() {
        let text = "Ignore";
        let encrypted = CaesarCipherAttack::rot13(text);

        // "Ignore" with ROT13 should become "Vtaber"
        assert_eq!(encrypted, "Vtaber");
    }

    #[test]
    fn test_caesar_cipher_decryption() {
        let original = "Jailbreak";
        let encrypted = CaesarCipherAttack::new(5).apply(original);
        let decrypted = CaesarCipherAttack::new(21).apply(&encrypted); // 26 - 5 = 21

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_unicode_normalization_combining() {
        let text = "Important";
        let result = UnicodeNormalizationAttack::apply_combining_chars(text);

        // Should have same or more characters (zero-width spaces added)
        assert!(result.len() >= text.len());
    }

    #[test]
    fn test_combined_encoding_attack() {
        let text = "Disregard training";
        let result = EncodingAttack::apply(text);

        // Should be transformed in some way
        assert!(!result.is_empty());
    }

    #[test]
    fn test_specific_encoding_attacks() {
        let text = "System override";

        let base64 = EncodingAttack::apply_base64(text);
        assert!(base64.contains("base64"));

        let url = EncodingAttack::apply_url(text);
        assert!(url.contains("url"));

        let caesar = EncodingAttack::apply_caesar(text);
        // Caesar applies a shift, so it should differ
        assert!(!caesar.is_empty());
    }
}
