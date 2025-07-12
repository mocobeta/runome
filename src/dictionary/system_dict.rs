use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{Dictionary, DictionaryResource, RAMDictionary};
use crate::dictionary::types::DictEntry;
use crate::error::RunomeError;

/// SystemDictionary combines known word lookup with character classification
///
/// This provides the primary interface for Japanese morphological analysis,
/// integrating dictionary lookups for known words with character-based
/// classification for unknown word processing.
pub struct SystemDictionary {
    /// RAM-based dictionary for known word lookup
    ram_dict: RAMDictionary,
}

/// Singleton instance with thread-safe lazy initialization
static SYSTEM_DICT_INSTANCE: Lazy<Arc<Mutex<Option<Arc<SystemDictionary>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

impl SystemDictionary {
    /// Get singleton instance of SystemDictionary
    ///
    /// Returns a shared reference to the singleton SystemDictionary instance,
    /// creating it if it doesn't exist. Uses lazy initialization with thread safety.
    ///
    /// # Returns
    /// * `Ok(Arc<SystemDictionary>)` - Shared reference to singleton instance
    /// * `Err(RunomeError)` - Error if initialization fails
    pub fn instance() -> Result<Arc<SystemDictionary>, RunomeError> {
        let instance_lock =
            SYSTEM_DICT_INSTANCE
                .lock()
                .map_err(|_| RunomeError::SystemDictInitError {
                    reason: "Failed to acquire SystemDictionary lock".to_string(),
                })?;

        if let Some(ref instance) = *instance_lock {
            return Ok(Arc::clone(instance));
        }

        drop(instance_lock);

        // Create new instance using default sysdic path
        let sysdic_path = Path::new("sysdic");
        let new_instance = Arc::new(Self::new(sysdic_path)?);

        let mut instance_lock =
            SYSTEM_DICT_INSTANCE
                .lock()
                .map_err(|_| RunomeError::SystemDictInitError {
                    reason: "Failed to acquire SystemDictionary lock for initialization"
                        .to_string(),
                })?;

        *instance_lock = Some(new_instance.clone());
        Ok(new_instance)
    }

    /// Create new SystemDictionary from sysdic directory
    ///
    /// Loads dictionary data and character definitions from the specified directory.
    /// This is used internally by the singleton pattern.
    ///
    /// # Arguments  
    /// * `sysdic_dir` - Path to directory containing dictionary data
    ///
    /// # Returns
    /// * `Ok(SystemDictionary)` - Successfully created dictionary
    /// * `Err(RunomeError)` - Error if loading fails
    pub fn new(sysdic_dir: &Path) -> Result<Self, RunomeError> {
        // Load dictionary resource
        let resource = DictionaryResource::load(sysdic_dir)?;

        // Create RAMDictionary
        let ram_dict = RAMDictionary::new(resource, sysdic_dir)?;

        Ok(Self { ram_dict })
    }

    /// Look up known words only (delegates to RAMDictionary)
    ///
    /// Performs dictionary lookup for known words using the embedded RAMDictionary.
    /// This does not handle unknown word processing.
    ///
    /// # Arguments
    /// * `surface` - Surface form string to look up
    ///
    /// # Returns  
    /// * `Ok(Vec<&DictEntry>)` - Vector of references to matching dictionary entries
    /// * `Err(RunomeError)` - Error if lookup fails
    pub fn lookup(&self, surface: &str) -> Result<Vec<&DictEntry>, RunomeError> {
        self.ram_dict.lookup(surface)
    }

    /// Get connection cost between part-of-speech IDs
    ///
    /// Delegates to the embedded RAMDictionary to get connection costs
    /// used in lattice-based morphological analysis.
    ///
    /// # Arguments
    /// * `left_id` - Left part-of-speech ID
    /// * `right_id` - Right part-of-speech ID
    ///
    /// # Returns
    /// * `Ok(i16)` - Connection cost
    /// * `Err(RunomeError)` - Error if IDs are invalid
    pub fn get_trans_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError> {
        self.ram_dict.get_trans_cost(left_id, right_id)
    }

    /// Get character categories for a given character
    ///
    /// Returns all character categories that match the given character,
    /// including both primary and compatible categories.
    ///
    /// # Arguments
    /// * `ch` - Character to classify
    ///
    /// # Returns
    /// HashMap mapping category names to compatible category lists
    pub fn get_char_categories(&self, ch: char) -> HashMap<String, Vec<String>> {
        self.ram_dict.get_resource().get_char_categories(ch)
    }

    /// Check if unknown word processing should always be invoked for category
    ///
    /// # Arguments
    /// * `category` - Character category name
    ///
    /// # Returns
    /// True if unknown word processing should always be invoked
    pub fn unknown_invoked_always(&self, category: &str) -> bool {
        self.ram_dict
            .get_resource()
            .unknown_invoked_always(category)
    }

    /// Check if characters of this category should be grouped together
    ///
    /// # Arguments  
    /// * `category` - Character category name
    ///
    /// # Returns
    /// True if consecutive characters should be grouped
    pub fn unknown_grouping(&self, category: &str) -> bool {
        self.ram_dict.get_resource().unknown_grouping(category)
    }

    /// Get length constraint for unknown words of this category
    ///
    /// # Arguments
    /// * `category` - Character category name  
    ///
    /// # Returns
    /// Length constraint (-1 = no limit, positive = max length)
    pub fn unknown_length(&self, category: &str) -> i32 {
        self.ram_dict.get_resource().unknown_length(category)
    }
}

/// Implement Dictionary trait through delegation to RAMDictionary
impl Dictionary for SystemDictionary {
    fn lookup(&self, surface: &str) -> Result<Vec<&DictEntry>, RunomeError> {
        self.lookup(surface)
    }

    fn get_trans_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError> {
        self.get_trans_cost(left_id, right_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_sysdic_path() -> PathBuf {
        PathBuf::from("sysdic")
    }

    #[test]
    fn test_system_dictionary_creation() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict_result = SystemDictionary::new(&sysdic_path);
        assert!(
            sys_dict_result.is_ok(),
            "Failed to create SystemDictionary: {:?}",
            sys_dict_result.err()
        );
    }

    #[test]
    fn test_singleton_consistency() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let instance1 = SystemDictionary::instance();
        let instance2 = SystemDictionary::instance();

        assert!(instance1.is_ok(), "First instance creation should succeed");
        assert!(instance2.is_ok(), "Second instance creation should succeed");

        // Should be the same instance
        let inst1 = instance1.unwrap();
        let inst2 = instance2.unwrap();
        assert!(Arc::ptr_eq(&inst1, &inst2), "Instances should be the same");
    }

    #[test]
    fn test_lookup_delegation() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");

        let sys_dict = sys_dict.unwrap();
        let lookup_result = sys_dict.lookup("東京");
        assert!(lookup_result.is_ok(), "Lookup should not fail");

        // Should return same results as direct RAMDictionary lookup
        let entries = lookup_result.unwrap();
        if !entries.is_empty() {
            for entry in entries {
                assert!(
                    !entry.part_of_speech.is_empty(),
                    "Part of speech should not be empty"
                );
            }
        }
    }

    #[test]
    fn test_get_trans_cost_delegation() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");

        let sys_dict = sys_dict.unwrap();
        let cost_result = sys_dict.get_trans_cost(0, 0);
        assert!(cost_result.is_ok(), "get_trans_cost should work");
    }

    #[test]
    fn test_character_classification() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");

        let sys_dict = sys_dict.unwrap();

        // Test character classification with real implementation
        let categories = sys_dict.get_char_categories('は');
        assert!(
            !categories.is_empty(),
            "Should have categories for hiragana"
        );

        // Test that some expected categories are present (or DEFAULT if not found)
        let has_hiragana_or_default =
            categories.contains_key("HIRAGANA") || categories.contains_key("DEFAULT");
        assert!(
            has_hiragana_or_default,
            "Should have HIRAGANA category or DEFAULT fallback"
        );

        // Test unknown word processing flags - behavior depends on actual character definitions
        // Just verify methods work without error
        let _ = sys_dict.unknown_invoked_always("HIRAGANA");
        let _ = sys_dict.unknown_grouping("HIRAGANA");
        let length = sys_dict.unknown_length("HIRAGANA");
        assert!(length >= -1, "Length should be valid (-1 or positive)");
    }

    #[test]
    fn test_character_classification_comprehensive() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");
        let sys_dict = sys_dict.unwrap();

        // Test major Japanese character types
        let japanese_test_cases = [
            // Hiragana
            ('あ', "HIRAGANA", "basic hiragana"),
            ('か', "HIRAGANA", "hiragana ka"),
            ('ひ', "HIRAGANA", "hiragana hi"),
            ('ん', "HIRAGANA", "hiragana n"),
            // Katakana
            ('ア', "KATAKANA", "basic katakana"),
            ('カ', "KATAKANA", "katakana ka"),
            ('ヒ', "KATAKANA", "katakana hi"),
            ('ン', "KATAKANA", "katakana n"),
            // Kanji
            ('漢', "KANJI", "kanji character"),
            ('字', "KANJI", "ji character"),
            ('日', "KANJI", "nichi character"),
            ('本', "KANJI", "hon character"),
        ];

        for (ch, expected_category, description) in japanese_test_cases {
            let categories = sys_dict.get_char_categories(ch);
            assert!(
                !categories.is_empty(),
                "Character '{}' ({}) should have categories",
                ch,
                description
            );

            // Should have expected category or DEFAULT
            let has_expected_or_default =
                categories.contains_key(expected_category) || categories.contains_key("DEFAULT");
            assert!(
                has_expected_or_default,
                "Character '{}' ({}) should have {} category or DEFAULT fallback. Got: {:?}",
                ch,
                description,
                expected_category,
                categories.keys().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_character_classification_ascii_and_symbols() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");
        let sys_dict = sys_dict.unwrap();

        // Test ASCII and symbol characters
        let ascii_symbol_test_cases = [
            // ASCII letters
            ('A', vec!["ALPHA", "DEFAULT"], "ASCII uppercase A"),
            ('z', vec!["ALPHA", "DEFAULT"], "ASCII lowercase z"),
            // ASCII digits
            ('0', vec!["NUMERIC", "DEFAULT"], "ASCII digit 0"),
            ('9', vec!["NUMERIC", "DEFAULT"], "ASCII digit 9"),
            // Common symbols
            ('!', vec!["SYMBOL", "DEFAULT"], "exclamation mark"),
            ('?', vec!["SYMBOL", "DEFAULT"], "question mark"),
            (' ', vec!["SPACE", "DEFAULT"], "space character"),
            // Japanese punctuation
            ('、', vec!["SYMBOL", "DEFAULT"], "Japanese comma"),
            ('。', vec!["SYMBOL", "DEFAULT"], "Japanese period"),
        ];

        for (ch, possible_categories, description) in ascii_symbol_test_cases {
            let categories = sys_dict.get_char_categories(ch);
            assert!(
                !categories.is_empty(),
                "Character '{}' ({}) should have categories",
                ch,
                description
            );

            // Should have at least one of the possible categories
            let has_valid_category = possible_categories
                .iter()
                .any(|cat| categories.contains_key(*cat));
            assert!(
                has_valid_category,
                "Character '{}' ({}) should have one of {:?}. Got: {:?}",
                ch,
                description,
                possible_categories,
                categories.keys().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_character_classification_numeric_variants() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");
        let sys_dict = sys_dict.unwrap();

        // Test different types of numeric characters
        let numeric_test_cases = [
            // ASCII digits
            ('1', vec!["NUMERIC", "DEFAULT"], "ASCII digit 1"),
            ('5', vec!["NUMERIC", "DEFAULT"], "ASCII digit 5"),
            // Full-width digits
            ('１', vec!["NUMERIC", "DEFAULT"], "full-width digit 1"),
            ('５', vec!["NUMERIC", "DEFAULT"], "full-width digit 5"),
            // Japanese numerals (these might be KANJI + KANJINUMERIC)
            (
                '一',
                vec!["KANJI", "KANJINUMERIC", "DEFAULT"],
                "kanji numeral 1",
            ),
            (
                '五',
                vec!["KANJI", "KANJINUMERIC", "DEFAULT"],
                "kanji numeral 5",
            ),
            (
                '十',
                vec!["KANJI", "KANJINUMERIC", "DEFAULT"],
                "kanji numeral 10",
            ),
        ];

        for (ch, possible_categories, description) in numeric_test_cases {
            let categories = sys_dict.get_char_categories(ch);
            assert!(
                !categories.is_empty(),
                "Character '{}' ({}) should have categories",
                ch,
                description
            );

            // Should have at least one of the possible categories
            let has_valid_category = possible_categories
                .iter()
                .any(|cat| categories.contains_key(*cat));
            assert!(
                has_valid_category,
                "Character '{}' ({}) should have one of {:?}. Got: {:?}",
                ch,
                description,
                possible_categories,
                categories.keys().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_character_classification_kanji_numeric_compatibility() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");
        let sys_dict = sys_dict.unwrap();

        // Test that KANJINUMERIC characters also have KANJI compatibility
        let kanji_numeric_chars = ['一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];

        for ch in kanji_numeric_chars {
            let categories = sys_dict.get_char_categories(ch);

            // If character has KANJINUMERIC category, it should also be compatible with KANJI
            if categories.contains_key("KANJINUMERIC") {
                let kanjinumeric_compat = categories.get("KANJINUMERIC").unwrap();
                assert!(
                    categories.contains_key("KANJI")
                        || kanjinumeric_compat.contains(&"KANJI".to_string()),
                    "Character '{}' with KANJINUMERIC should also have KANJI category or compatibility. Categories: {:?}",
                    ch,
                    categories
                );
            }
        }
    }

    #[test]
    fn test_unknown_word_processing_properties() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");
        let sys_dict = sys_dict.unwrap();

        // Test unknown word processing properties for common categories
        let categories_to_test = [
            "HIRAGANA",
            "KATAKANA",
            "KANJI",
            "KANJINUMERIC",
            "ALPHA",
            "NUMERIC",
            "SYMBOL",
            "DEFAULT",
        ];

        for category in categories_to_test {
            // Test that all methods work without panicking
            let invoke_always = sys_dict.unknown_invoked_always(category);
            let grouping = sys_dict.unknown_grouping(category);
            let length = sys_dict.unknown_length(category);

            // Verify return values are reasonable
            assert!(
                length >= -1 && length <= 255,
                "Length for category '{}' should be between -1 and 255, got: {}",
                category,
                length
            );

            // Log the properties for debugging (will only show if test fails)
            if invoke_always || grouping || length != -1 {
                eprintln!(
                    "Category '{}': invoke_always={}, grouping={}, length={}",
                    category, invoke_always, grouping, length
                );
            }
        }
    }

    #[test]
    fn test_character_classification_consistency() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");
        let sys_dict = sys_dict.unwrap();

        // Test that multiple calls return consistent results
        let test_chars = ['あ', 'ア', '漢', 'A', '1', '、'];

        for ch in test_chars {
            let first_result = sys_dict.get_char_categories(ch);
            let second_result = sys_dict.get_char_categories(ch);
            let third_result = sys_dict.get_char_categories(ch);

            assert_eq!(
                first_result, second_result,
                "Character '{}' classification should be consistent between calls",
                ch
            );
            assert_eq!(
                second_result, third_result,
                "Character '{}' classification should be consistent across multiple calls",
                ch
            );
        }
    }

    #[test]
    fn test_character_classification_edge_cases() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");
        let sys_dict = sys_dict.unwrap();

        // Test boundary characters and edge cases
        let edge_case_chars = [
            // Unicode boundary characters
            ('\u{0000}', "null character"),
            ('\u{007F}', "DEL character"),
            ('\u{0080}', "first extended ASCII"),
            ('\u{00FF}', "last extended ASCII"),
            // Hiragana boundaries
            ('\u{3041}', "first hiragana (small a)"),
            ('\u{3096}', "last hiragana"),
            // Katakana boundaries
            ('\u{30A1}', "first katakana (small a)"),
            ('\u{30F6}', "last katakana"),
            // CJK boundaries
            ('\u{4E00}', "first CJK ideograph"),
            ('\u{9FFF}', "last CJK ideograph"),
            // Unusual but valid characters
            ('\u{3000}', "ideographic space"),
            ('\u{FEFF}', "zero-width no-break space"),
            ('\u{200B}', "zero-width space"),
            // Full-width variants
            ('Ａ', "full-width A"),
            ('０', "full-width 0"),
            ('！', "full-width exclamation"),
        ];

        for (ch, description) in edge_case_chars {
            let categories = sys_dict.get_char_categories(ch);

            // All characters should get at least some classification (even if DEFAULT)
            assert!(
                !categories.is_empty(),
                "Edge case character '{}' (U+{:04X}) ({}) should have at least one category",
                ch,
                ch as u32,
                description
            );

            // Verify that all category names are non-empty strings
            for (category_name, compat_categories) in &categories {
                assert!(
                    !category_name.is_empty(),
                    "Category name should not be empty for character '{}' ({})",
                    ch,
                    description
                );

                // Verify compatibility categories are also valid
                for compat in compat_categories {
                    assert!(
                        !compat.is_empty(),
                        "Compatibility category should not be empty for character '{}' ({})",
                        ch,
                        description
                    );
                }
            }
        }
    }

    #[test]
    fn test_character_classification_rare_unicode() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");
        let sys_dict = sys_dict.unwrap();

        // Test rare but potentially encountered Unicode characters
        let rare_unicode_chars = [
            // Extended punctuation
            ('\u{2026}', "horizontal ellipsis"),
            ('\u{2014}', "em dash"),
            ('\u{2013}', "en dash"),
            // Mathematical symbols
            ('\u{221E}', "infinity symbol"),
            ('\u{2260}', "not equal to"),
            ('\u{00B1}', "plus-minus sign"),
            // Currency symbols
            ('\u{00A5}', "yen sign"),
            ('\u{20AC}', "euro sign"),
            ('\u{0024}', "dollar sign"),
            // Arrows
            ('\u{2190}', "leftwards arrow"),
            ('\u{2192}', "rightwards arrow"),
            // Emoji (basic)
            ('\u{1F600}', "grinning face emoji"),
            ('\u{1F44D}', "thumbs up emoji"),
        ];

        for (ch, description) in rare_unicode_chars {
            let categories = sys_dict.get_char_categories(ch);

            // Should handle rare characters gracefully
            assert!(
                !categories.is_empty(),
                "Rare Unicode character '{}' (U+{:04X}) ({}) should be classified",
                ch,
                ch as u32,
                description
            );

            // Most rare characters should fall back to DEFAULT if not specifically categorized
            if !categories.keys().any(|k| k != "DEFAULT") {
                assert!(
                    categories.contains_key("DEFAULT"),
                    "Rare character '{}' ({}) should at least have DEFAULT category",
                    ch,
                    description
                );
            }
        }
    }

    #[test]
    fn test_character_classification_combining_characters() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");
        let sys_dict = sys_dict.unwrap();

        // Test combining characters (accents, diacritics)
        let combining_chars = [
            ('\u{0300}', "combining grave accent"),
            ('\u{0301}', "combining acute accent"),
            ('\u{0302}', "combining circumflex accent"),
            ('\u{0303}', "combining tilde"),
            ('\u{0304}', "combining macron"),
            ('\u{0308}', "combining diaeresis"),
            ('\u{030A}', "combining ring above"),
        ];

        for (ch, description) in combining_chars {
            let categories = sys_dict.get_char_categories(ch);

            // Combining characters should be classified (likely as DEFAULT or special category)
            assert!(
                !categories.is_empty(),
                "Combining character '{}' (U+{:04X}) ({}) should be classified",
                ch,
                ch as u32,
                description
            );
        }
    }

    #[test]
    fn test_character_classification_surrogate_handling() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance();
        assert!(sys_dict.is_ok(), "SystemDictionary creation should succeed");
        let sys_dict = sys_dict.unwrap();

        // Test characters that require surrogate pairs in UTF-16
        // These are valid Unicode scalar values that Rust char can represent
        let high_unicode_chars = [
            ('\u{10000}', "first supplementary plane character"),
            ('\u{1F300}', "cyclone emoji"),
            ('\u{1F680}', "rocket emoji"),
            ('\u{20000}', "CJK extension B character"),
        ];

        for (ch, description) in high_unicode_chars {
            let categories = sys_dict.get_char_categories(ch);

            // High Unicode characters should be handled gracefully
            assert!(
                !categories.is_empty(),
                "High Unicode character '{}' (U+{:04X}) ({}) should be classified",
                ch,
                ch as u32,
                description
            );
        }
    }
}
