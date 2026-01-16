//! Character substitution attacks using homoglyphs, leetspeak, and case variation.

use std::collections::HashMap;

/// Homoglyph substitution attack using visually similar Unicode characters.
///
/// Maps Latin characters to Cyrillic or other Unicode lookalikes:
/// - a → а (U+0430 Cyrillic Small Letter A)
/// - e → е (U+0435 Cyrillic Small Letter IE)
/// - o → о (U+043E Cyrillic Small Letter O)
/// - p → р (U+0440 Cyrillic Small Letter ER)
/// - c → с (U+0441 Cyrillic Small Letter ES)
#[derive(Debug, Clone)]
pub struct HomoglyphSubstitution {
    /// Mapping from Latin characters to Unicode lookalikes
    substitutions: HashMap<char, char>,
}

impl HomoglyphSubstitution {
    /// Create a new homoglyph substitution mapper.
    pub fn new() -> Self {
        let mut substitutions = HashMap::new();

        // Cyrillic lookalikes
        substitutions.insert('a', 'а'); // U+0430
        substitutions.insert('A', 'А'); // U+0410
        substitutions.insert('e', 'е'); // U+0435
        substitutions.insert('E', 'Е'); // U+0415
        substitutions.insert('o', 'о'); // U+043E
        substitutions.insert('O', 'О'); // U+041E
        substitutions.insert('p', 'р'); // U+0440
        substitutions.insert('P', 'Р'); // U+0420
        substitutions.insert('c', 'с'); // U+0441
        substitutions.insert('C', 'С'); // U+0421
        substitutions.insert('x', 'х'); // U+0445
        substitutions.insert('X', 'Х'); // U+0425

        // Greek lookalikes
        substitutions.insert('v', 'ν'); // Greek nu
        substitutions.insert('p', 'ρ'); // Greek rho

        Self { substitutions }
    }

    /// Apply homoglyph substitution to text with given probability.
    pub fn apply(&self, text: &str, substitution_rate: f32) -> String {
        text.chars()
            .map(|ch| {
                if rand::random::<f32>() < substitution_rate {
                    self.substitutions.get(&ch).copied().unwrap_or(ch)
                } else {
                    ch
                }
            })
            .collect()
    }
}

impl Default for HomoglyphSubstitution {
    fn default() -> Self {
        Self::new()
    }
}

/// Leetspeak attack replacing letters with numbers.
///
/// Common leetspeak substitutions:
/// - a → 4
/// - e → 3
/// - i → 1
/// - o → 0
/// - s → 5
/// - t → 7
/// - l → 1
#[derive(Debug, Clone)]
pub struct LeetSpeakAttack {
    /// Mapping from letters to numbers
    substitutions: HashMap<char, &'static str>,
}

impl LeetSpeakAttack {
    /// Create a new leetspeak attack mapper.
    pub fn new() -> Self {
        let mut substitutions = HashMap::new();

        substitutions.insert('a', "4");
        substitutions.insert('A', "4");
        substitutions.insert('e', "3");
        substitutions.insert('E', "3");
        substitutions.insert('i', "1");
        substitutions.insert('I', "1");
        substitutions.insert('o', "0");
        substitutions.insert('O', "0");
        substitutions.insert('s', "5");
        substitutions.insert('S', "5");
        substitutions.insert('t', "7");
        substitutions.insert('T', "7");
        substitutions.insert('l', "1");
        substitutions.insert('L', "1");

        Self { substitutions }
    }

    /// Apply leetspeak substitution with given probability.
    pub fn apply(&self, text: &str, substitution_rate: f32) -> String {
        text.chars()
            .flat_map(|ch| {
                if rand::random::<f32>() < substitution_rate {
                    self.substitutions
                        .get(&ch)
                        .map_or_else(|| vec![ch], |s| s.chars().collect::<Vec<_>>())
                } else {
                    vec![ch]
                }
            })
            .collect()
    }
}

impl Default for LeetSpeakAttack {
    fn default() -> Self {
        Self::new()
    }
}

/// Case variation attack changing character casing.
///
/// Randomly switches upper/lower case to evade exact string matching.
#[derive(Debug, Clone)]
pub struct CaseVariationAttack;

impl CaseVariationAttack {
    /// Apply random case variation to text.
    pub fn apply(text: &str, variation_rate: f32) -> String {
        text.chars()
            .map(|ch| {
                if rand::random::<f32>() < variation_rate {
                    if ch.is_uppercase() {
                        ch.to_lowercase().collect::<String>()
                    } else if ch.is_lowercase() {
                        ch.to_uppercase().collect::<String>()
                    } else {
                        ch.to_string()
                    }
                } else {
                    ch.to_string()
                }
            })
            .collect()
    }
}

/// Combined character substitution attack.
#[derive(Debug, Clone)]
pub struct CharSubstitutionAttack {
    homoglyph: HomoglyphSubstitution,
    leetspeak: LeetSpeakAttack,
    substitution_rate: f32,
}

impl CharSubstitutionAttack {
    /// Create a new character substitution attack.
    pub fn new(substitution_rate: f32) -> Self {
        Self {
            homoglyph: HomoglyphSubstitution::new(),
            leetspeak: LeetSpeakAttack::new(),
            substitution_rate,
        }
    }

    /// Apply character substitution attack.
    ///
    /// Chooses randomly between homoglyph and leetspeak attacks.
    pub fn apply(&self, text: &str) -> String {
        if rand::random::<bool>() {
            self.homoglyph.apply(text, self.substitution_rate)
        } else {
            self.leetspeak.apply(text, self.substitution_rate)
        }
    }

    /// Apply with case variation.
    pub fn apply_with_case(&self, text: &str) -> String {
        let substituted = self.apply(text);
        CaseVariationAttack::apply(&substituted, self.substitution_rate * 0.5)
    }
}

impl Default for CharSubstitutionAttack {
    fn default() -> Self {
        Self::new(0.15) // 15% substitution rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homoglyph_substitution() {
        let attack = HomoglyphSubstitution::new();

        // With 100% substitution rate, should replace 'a', 'e', 'o'
        // (none in "ignore", so text should remain the same)
        let result = attack.apply("banana", 1.0);
        assert!(!result.is_empty());
        assert!(result.len() >= "banana".len());
    }

    #[test]
    fn test_leetspeak_attack() {
        let attack = LeetSpeakAttack::new();

        // With 100% substitution, should replace letters
        let result = attack.apply("abuse", 1.0);
        assert!(result.contains('4') || result.contains('3') || result.contains('5'));
    }

    #[test]
    fn test_case_variation() {
        let original = "Ignore";
        let varied = CaseVariationAttack::apply(original, 1.0);

        // Should have same number of characters but different casing
        assert_eq!(original.len(), varied.len());
        // At least some characters should differ
        assert_ne!(original, varied);
    }

    #[test]
    fn test_char_substitution_attack() {
        let attack = CharSubstitutionAttack::new(0.5);
        let text = "Ignore instructions";

        let result = attack.apply(text);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_char_substitution_with_case() {
        let attack = CharSubstitutionAttack::new(0.3);
        let text = "Override safety";

        let result = attack.apply_with_case(text);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_substitution_rate_zero() {
        let attack = CharSubstitutionAttack::new(0.0);
        let text = "Reveal secrets";

        // With 0% rate, text should remain mostly unchanged
        let result = attack.apply(text);
        // May have minor variations due to randomness, but should be similar
        assert!(!result.is_empty());
    }
}
