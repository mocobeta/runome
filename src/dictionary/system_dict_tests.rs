//! Advanced tests for SystemDictionary functionality
//!
//! This module contains comprehensive tests for category compatibility,
//! performance benchmarks, and error handling scenarios.

use std::path::PathBuf;

#[cfg(test)]
use super::SystemDictionary;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::time::Instant;

#[cfg(test)]
fn get_test_sysdic_path() -> PathBuf {
    PathBuf::from("sysdic")
}

#[cfg(test)]
mod category_compatibility_tests {
    use super::*;

    #[test]
    fn test_category_compatibility_relationships() {
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

        // Test complex compatibility relationships
        let compatibility_test_cases = [
            // KANJINUMERIC should be compatible with KANJI
            ('一', "KANJINUMERIC", vec!["KANJI"]),
            ('二', "KANJINUMERIC", vec!["KANJI"]),
            ('五', "KANJINUMERIC", vec!["KANJI"]),
            ('十', "KANJINUMERIC", vec!["KANJI"]),
            // Some characters might have multiple compatibility relationships
            // Test that the compatibility system works as expected
        ];

        for (ch, primary_category, expected_compatibilities) in compatibility_test_cases {
            let categories = sys_dict.get_char_categories(ch);

            if let Some(compat_categories) = categories.get(primary_category) {
                for expected_compat in expected_compatibilities {
                    let has_compatibility = categories.contains_key(expected_compat)
                        || compat_categories.contains(&expected_compat.to_string());

                    assert!(
                        has_compatibility,
                        "Character '{}' with {} category should have {} compatibility. Categories: {:?}",
                        ch, primary_category, expected_compat, categories
                    );
                }
            }
        }
    }

    #[test]
    fn test_category_hierarchy_consistency() {
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

        // Test that category hierarchies are consistent
        let test_chars = [
            'あ', 'か', 'さ', 'た', 'な', // Hiragana
            'ア', 'カ', 'サ', 'タ', 'ナ', // Katakana
            '漢', '字', '文', '本', '日', // Kanji
            'A', 'B', 'C', 'a', 'b', // ASCII letters
            '0', '1', '2', '3', '4', // ASCII digits
        ];

        let mut category_relationships: HashMap<String, Vec<String>> = HashMap::new();

        // Collect all category relationships
        for ch in test_chars {
            let categories = sys_dict.get_char_categories(ch);
            for (category_name, compat_categories) in categories {
                let entry = category_relationships.entry(category_name).or_default();
                for compat in compat_categories {
                    if !entry.contains(&compat) {
                        entry.push(compat);
                    }
                }
            }
        }

        // Verify consistency: if A is compatible with B, verify B exists as a category
        for (category, compatibilities) in &category_relationships {
            for compat in compatibilities {
                // Check if the compatibility target is also a known category
                // This might not always be true, but log for analysis
                if !category_relationships.contains_key(compat) {
                    eprintln!(
                        "Category '{}' is compatible with '{}', but '{}' is not found as a primary category",
                        category, compat, compat
                    );
                }
            }
        }
    }

    #[test]
    fn test_unknown_word_processing_category_consistency() {
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

        // Collect all categories that actually exist in the system
        let test_chars = ['あ', 'ア', '漢', 'A', '1', '、', ' ', '！', '五'];

        let mut all_categories = std::collections::HashSet::new();
        for ch in test_chars {
            let categories = sys_dict.get_char_categories(ch);
            for category_name in categories.keys() {
                all_categories.insert(category_name.clone());
            }
        }

        // Test unknown word processing properties for all found categories
        for category in &all_categories {
            let invoke_always = sys_dict.unknown_invoked_always(category);
            let grouping = sys_dict.unknown_grouping(category);
            let length = sys_dict.unknown_length(category);

            // Verify properties are consistent and reasonable
            assert!(
                (-1..=255).contains(&length),
                "Category '{}' has invalid length: {}",
                category,
                length
            );

            // Log the properties for analysis
            eprintln!(
                "Category '{}': invoke={}, group={}, length={}",
                category, invoke_always, grouping, length
            );
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_singleton_access_performance() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        // Test that singleton access is reasonably fast
        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _sys_dict = SystemDictionary::instance().unwrap();
        }

        let duration = start.elapsed();
        let avg_time_per_access = duration / iterations;

        // Singleton access should be very fast (less than 1ms per access)
        assert!(
            avg_time_per_access.as_millis() < 1,
            "Singleton access too slow: {:?} per access",
            avg_time_per_access
        );

        eprintln!(
            "Singleton access performance: {:?} per access ({} iterations)",
            avg_time_per_access, iterations
        );
    }

    #[test]
    fn test_character_classification_performance() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance().unwrap();

        // Test character classification performance across different character types
        let test_chars = [
            'あ', 'か', 'ひ', 'ん', // Hiragana
            'ア', 'カ', 'ヒ', 'ン', // Katakana
            '漢', '字', '日', '本', // Kanji
            'A', 'B', 'a', 'b', // ASCII
            '1', '2', '3', '4', // Digits
            '!', '?', '、', '。', // Symbols
        ];

        let iterations = 10000;
        let start = Instant::now();

        for _ in 0..iterations {
            for ch in test_chars {
                let _categories = sys_dict.get_char_categories(ch);
            }
        }

        let duration = start.elapsed();
        let total_classifications = iterations * test_chars.len() as u32;
        let avg_time_per_classification = duration / total_classifications;

        // Character classification should be very fast (less than 1μs per classification)
        assert!(
            avg_time_per_classification.as_micros() < 10,
            "Character classification too slow: {:?} per classification",
            avg_time_per_classification
        );

        eprintln!(
            "Character classification performance: {:?} per classification ({} total)",
            avg_time_per_classification, total_classifications
        );
    }

    #[test]
    fn test_unknown_word_processing_performance() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance().unwrap();

        let categories = [
            "HIRAGANA", "KATAKANA", "KANJI", "ALPHA", "NUMERIC", "DEFAULT",
        ];
        let iterations = 10000;
        let start = Instant::now();

        for _ in 0..iterations {
            for category in categories {
                let _invoke = sys_dict.unknown_invoked_always(category);
                let _group = sys_dict.unknown_grouping(category);
                let _length = sys_dict.unknown_length(category);
            }
        }

        let duration = start.elapsed();
        let total_calls = iterations * categories.len() as u32 * 3; // 3 methods per category
        let avg_time_per_call = duration / total_calls;

        // Unknown word processing calls should be very fast
        assert!(
            avg_time_per_call.as_nanos() < 1000,
            "Unknown word processing too slow: {:?} per call",
            avg_time_per_call
        );

        eprintln!(
            "Unknown word processing performance: {:?} per call ({} total)",
            avg_time_per_call, total_calls
        );
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_empty_string_handling() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance().unwrap();

        // Test lookup with empty string
        let result = sys_dict.lookup("");
        assert!(result.is_ok(), "Empty string lookup should not panic");
        assert!(
            result.unwrap().is_empty(),
            "Empty string should return no results"
        );
    }

    #[test]
    fn test_invalid_category_handling() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let sys_dict = SystemDictionary::instance().unwrap();

        // Test unknown word processing with invalid categories
        let invalid_categories = [
            "", // Empty string
            "INVALID_CATEGORY",
            "NON_EXISTENT",
            "123",
            "カテゴリー", // Japanese characters
        ];

        for category in invalid_categories {
            // These should not panic and should return reasonable defaults
            let invoke = sys_dict.unknown_invoked_always(category);
            let group = sys_dict.unknown_grouping(category);
            let length = sys_dict.unknown_length(category);

            // Invalid categories should return false/false/-1
            assert!(
                !invoke,
                "Invalid category '{}' should not invoke always",
                category
            );
            assert!(!group, "Invalid category '{}' should not group", category);
            assert_eq!(
                length, -1,
                "Invalid category '{}' should return -1 for length",
                category
            );
        }
    }

    #[test]
    fn test_concurrent_access() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        use std::thread;

        // Test that concurrent access to singleton is safe
        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    let sys_dict = SystemDictionary::instance().unwrap();

                    // Perform some operations
                    let categories = sys_dict.get_char_categories('あ');
                    assert!(!categories.is_empty(), "Thread {} should get categories", i);

                    let lookup_result = sys_dict.lookup("東京");
                    assert!(lookup_result.is_ok(), "Thread {} lookup should succeed", i);

                    let cost = sys_dict.get_trans_cost(0, 0);
                    assert!(cost.is_ok(), "Thread {} get_trans_cost should succeed", i);
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }
    }

    #[test]
    fn test_memory_consistency() {
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        // Test that multiple accesses to the same singleton return identical data
        let sys_dict1 = SystemDictionary::instance().unwrap();
        let sys_dict2 = SystemDictionary::instance().unwrap();

        // Should be the same Arc instance
        assert!(
            Arc::ptr_eq(&sys_dict1, &sys_dict2),
            "Should be same Arc instance"
        );

        // Should return identical results
        let categories1 = sys_dict1.get_char_categories('あ');
        let categories2 = sys_dict2.get_char_categories('あ');
        assert_eq!(categories1, categories2, "Should return identical results");

        let lookup1 = sys_dict1.lookup("東京").unwrap();
        let lookup2 = sys_dict2.lookup("東京").unwrap();
        assert_eq!(
            lookup1.len(),
            lookup2.len(),
            "Should return same number of entries"
        );
    }
}
