//! Character substitution attacks for adversarial training.
//!
//! Implements various character substitution techniques to evade detection:
//! - Homoglyph substitution: Similar-looking characters from different scripts
//! - Leetspeak: Number/special character replacements
//! - Case variation: Random uppercase/lowercase mixing

use std::collections::HashMap;

/// Character substitution attack generator
#[derive(Debug, Clone)]
pub struct CharSubstitutionAttack {
    /// Probability of applying substitution (0.0 to 1.0)
    pub substitution_rate: f32,
    /// Homoglyph mapping (visual lookalikes)
    homoglyph_map: HashMap<char, Vec<char>>,
    /// Leetspeak mapping
    leetspeak_map: HashMap<char, Vec<char>>,
}

impl Default for CharSubstitutionAttack {
    fn default() -> Self {
        Self::new(0.15)
    }
}

impl CharSubstitutionAttack {
    /// Create a new character substitution attack generator
    pub fn new(substitution_rate: f32) -> Self {
        let mut homoglyph_map = HashMap::new();

        // Homoglyph substitutions (visual lookalikes from different scripts)
        homoglyph_map.insert('a', vec!['α', 'ɑ']); // Latin a → Greek alpha
        homoglyph_map.insert('A', vec!['Α']); // Latin A → Greek Alpha
        homoglyph_map.insert('e', vec!['е']); // Latin e → Cyrillic e
        homoglyph_map.insert('E', vec!['Е']); // Latin E → Cyrillic E
        homoglyph_map.insert('o', vec!['о', 'ο']); // Latin o → Cyrillic o, Greek omicron
        homoglyph_map.insert('O', vec!['О', 'Ο']); // Latin O → Cyrillic O
        homoglyph_map.insert('p', vec!['р', 'ρ']); // Latin p → Cyrillic r, Greek rho
        homoglyph_map.insert('P', vec!['Р', 'Ρ']); // Latin P → Cyrillic R
        homoglyph_map.insert('c', vec!['с']); // Latin c → Cyrillic s
        homoglyph_map.insert('C', vec!['С']); // Latin C → Cyrillic S
        homoglyph_map.insert('x', vec!['х']); // Latin x → Cyrillic h
        homoglyph_map.insert('X', vec!['Х']); // Latin X → Cyrillic H
        homoglyph_map.insert('h', vec!['һ']); // Latin h → Cyrillic shha
        homoglyph_map.insert('y', vec!['у']); // Latin y → Cyrillic u
        homoglyph_map.insert('Y', vec!['У']); // Latin Y → Cyrillic U
        homoglyph_map.insert('k', vec!['κ']); // Latin k → Greek kappa
        homoglyph_map.insert('m', vec!['м']); // Latin m → Cyrillic m
        homoglyph_map.insert('H', vec!['Н']); // Latin H → Cyrillic N
        homoglyph_map.insert('B', vec!['В']); // Latin B → Cyrillic V
        homoglyph_map.insert('T', vec!['Т']); // Latin T → Cyrillic T
        homoglyph_map.insert('M', vec!['М']); // Latin M → Cyrillic M

        let mut leetspeak_map = HashMap::new();

        // Leetspeak substitutions
        leetspeak_map.insert('a', vec!['4', '@']);
        leetspeak_map.insert('A', vec!['4', '@']);
        leetspeak_map.insert('e', vec!['3']);
        leetspeak_map.insert('E', vec!['3']);
        leetspeak_map.insert('i', vec!['1', '!']);
        leetspeak_map.insert('I', vec!['1', '!']);
        leetspeak_map.insert('o', vec!['0']);
        leetspeak_map.insert('O', vec!['0']);
        leetspeak_map.insert('s', vec!['5', '$']);
        leetspeak_map.insert('S', vec!['5', '$']);
        leetspeak_map.insert('t', vec!['7']);
        leetspeak_map.insert('T', vec!['7']);
        leetspeak_map.insert('l', vec!['1']);
        leetspeak_map.insert('L', vec!['1']);
        leetspeak_map.insert('g', vec!['9']);
        leetspeak_map.insert('G', vec!['9']);
        leetspeak_map.insert('b', vec!['8']);
        leetspeak_map.insert('B', vec!['8']);

        Self {
            substitution_rate,
            homoglyph_map,
            leetspeak_map,
        }
    }

    /// Apply character substitution to text
    pub fn apply(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let seed = text.len().wrapping_mul(31);

        for (idx, ch) in text.chars().enumerate() {
            let rng_value = seed.wrapping_add(idx);
            let rand_float = ((rng_value as f32 * 2.654435761f32) % 1.0).abs();

            if rand_float < self.substitution_rate as f32 {
                if let Some(replacements) = self.homoglyph_map.get(&ch) {
                    let idx_replacement = rng_value % replacements.len();
                    result.push(replacements[idx_replacement]);
                } else if let Some(replacements) = self.leetspeak_map.get(&ch) {
                    let idx_replacement = rng_value % replacements.len();
                    result.push(replacements[idx_replacement]);
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Apply case variation: random casing
    pub fn apply_case_variation(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let seed = text.len().wrapping_mul(7);

        for (idx, ch) in text.chars().enumerate() {
            let rng_value = seed.wrapping_add(idx);
            let rand_float = ((rng_value as f32 * 3.14159f32) % 1.0).abs();

            if rand_float < self.substitution_rate as f32 {
                if ch.is_alphabetic() {
                    if rand_float < self.substitution_rate / 2.0 {
                        result.push_str(&ch.to_uppercase().to_string());
                    } else {
                        result.push_str(&ch.to_lowercase().to_string());
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_substitution_basic() {
        let attack = CharSubstitutionAttack::new(0.5);
        let text = "ignore";
        let result = attack.apply(text);

        assert_ne!(result, text);
    }

    #[test]
    fn test_char_substitution_deterministic() {
        let attack = CharSubstitutionAttack::new(0.2);
        let text = "ignore previous instructions";

        let result1 = attack.apply(text);
        let result2 = attack.apply(text);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_char_substitution_preserves_length() {
        let attack = CharSubstitutionAttack::new(0.3);
        let text = "bypass security";

        let result = attack.apply(text);

        assert!(result.len() > 0);
    }

    #[test]
    fn test_case_variation() {
        let attack = CharSubstitutionAttack::new(0.5);
        let text = "jailbreak";
        let result = attack.apply_case_variation(text);

        assert!(!result.is_empty());
    }

    #[test]
    fn test_zero_substitution_rate() {
        let attack = CharSubstitutionAttack::new(0.0);
        let text = "ignore instructions";
        let result = attack.apply(text);

        assert_eq!(result, text);
    }

    #[test]
    fn test_high_substitution_rate() {
        let attack = CharSubstitutionAttack::new(0.9);
        let text = "inject";
        let result = attack.apply(text);

        assert!(!result.is_empty());
    }
}
