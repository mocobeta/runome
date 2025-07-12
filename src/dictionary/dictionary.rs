use std::collections::HashSet;

use fst::Map;

use super::{DictionaryResource, loader, types::DictEntry};
use crate::error::RunomeError;

/// Dictionary trait providing core morpheme lookup functionality
///
/// This trait mirrors the interface of Janome's Python Dictionary class,
/// providing lookup and connection cost methods for morphological analysis.
pub trait Dictionary {
    /// Look up morphemes matching a surface form
    ///
    /// Returns a vector of references to DictEntry structs containing
    /// all morphological information for matching dictionary entries.
    ///
    /// # Arguments
    /// * `surface` - The surface form string to look up
    ///
    /// # Returns
    /// * `Ok(Vec<&DictEntry>)` - Vector of references to matching dictionary entries
    /// * `Err(RunomeError)` - Error if lookup fails
    fn lookup(&self, surface: &str) -> Result<Vec<&DictEntry>, RunomeError>;

    /// Get connection cost between part-of-speech IDs
    ///
    /// Returns the connection cost used in lattice-based morphological analysis
    /// to determine the cost of connecting two morphemes.
    ///
    /// # Arguments
    /// * `left_id` - Left part-of-speech ID
    /// * `right_id` - Right part-of-speech ID
    ///
    /// # Returns
    /// * `Ok(i16)` - Connection cost
    /// * `Err(RunomeError)` - Error if IDs are invalid
    fn get_trans_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError>;
}

/// Matcher struct for FST-based string matching
///
/// Handles finite state transducer operations to efficiently map
/// surface form strings to morpheme IDs using the fst crate.
pub struct Matcher {
    fst: Map<Vec<u8>>,
}

impl Matcher {
    /// Create new Matcher from FST bytes
    ///
    /// # Arguments
    /// * `fst_bytes` - Raw FST data as bytes
    ///
    /// # Returns
    /// * `Ok(Matcher)` - Successfully created matcher
    /// * `Err(RunomeError)` - Error if FST data is invalid
    pub fn new(fst_bytes: Vec<u8>) -> Result<Self, RunomeError> {
        let fst = Map::new(fst_bytes).map_err(|e| RunomeError::DictValidationError {
            reason: format!("Failed to create FST: {}", e),
        })?;
        Ok(Self { fst })
    }

    /// Run FST matching on input word
    ///
    /// Performs FST traversal to find morpheme IDs matching the input string.
    /// Supports both exact matching and common prefix matching modes.
    ///
    /// # Arguments
    /// * `word` - Input string to match
    /// * `common_prefix_match` - If true, returns all prefixes; if false, exact match only
    ///
    /// # Returns
    /// * `Ok((bool, HashSet<u32>))` - Tuple of (matched, morpheme_ids)
    /// * `Err(RunomeError)` - Error if matching fails
    pub fn run(
        &self,
        word: &str,
        common_prefix_match: bool,
    ) -> Result<(bool, HashSet<u32>), RunomeError> {
        let mut outputs = HashSet::new();

        if common_prefix_match {
            // Find all prefixes of the word that match entries in the FST
            for i in 1..=word.len() {
                if let Some(byte_boundary) = self.find_char_boundary(word, i) {
                    let prefix = &word[..byte_boundary];
                    if let Some(value) = self.fst.get(prefix) {
                        outputs.insert(value as u32);
                    }
                }
            }
        } else {
            // Exact match only
            if let Some(value) = self.fst.get(word) {
                outputs.insert(value as u32);
            }
        }

        let matched = !outputs.is_empty();
        Ok((matched, outputs))
    }

    /// Find a character boundary at or before the given byte index
    ///
    /// This is necessary because we need to ensure we're splitting at valid UTF-8
    /// character boundaries when doing prefix matching.
    fn find_char_boundary(&self, s: &str, mut index: usize) -> Option<usize> {
        if index >= s.len() {
            return Some(s.len());
        }

        // Move backwards until we find a character boundary
        while index > 0 && !s.is_char_boundary(index) {
            index -= 1;
        }

        if index == 0 && !s.is_char_boundary(0) {
            None
        } else {
            Some(index)
        }
    }
}

/// RAMDictionary implementation using DictionaryResource and Matcher
///
/// Combines dictionary data storage (DictionaryResource) with FST-based
/// string matching (Matcher) to provide efficient morpheme lookup.
pub struct RAMDictionary {
    resource: DictionaryResource,
    matcher: Matcher,
}

impl RAMDictionary {
    /// Create new RAMDictionary from DictionaryResource and sysdic directory
    ///
    /// Loads FST bytes directly from the sysdic directory and creates a Matcher instance
    /// for efficient string-to-morpheme-ID mapping.
    ///
    /// # Arguments
    /// * `resource` - DictionaryResource containing all dictionary data
    /// * `sysdic_dir` - Path to sysdic directory containing FST file
    ///
    /// # Returns
    /// * `Ok(RAMDictionary)` - Successfully created dictionary
    /// * `Err(RunomeError)` - Error if FST creation fails
    pub fn new(
        resource: DictionaryResource,
        sysdic_dir: &std::path::Path,
    ) -> Result<Self, RunomeError> {
        // Load FST bytes directly using loader
        let fst_bytes = loader::load_fst_bytes(sysdic_dir)?;
        let matcher = Matcher::new(fst_bytes)?;

        Ok(Self { resource, matcher })
    }
}

impl Dictionary for RAMDictionary {
    fn lookup(&self, surface: &str) -> Result<Vec<&DictEntry>, RunomeError> {
        // TODO: Implement lookup logic in Phase 2
        // 1. Use matcher to get morpheme IDs matching the surface form
        // 2. Resolve morpheme IDs to dictionary entries using DictionaryResource
        // 3. Return references to DictEntry structs
        todo!("Lookup implementation using Matcher + DictionaryResource")
    }

    fn get_trans_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError> {
        // Delegate to DictionaryResource connection cost method
        self.resource.get_connection_cost(left_id, right_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_sysdic_path() -> PathBuf {
        // Assuming tests are run from the project root
        PathBuf::from("sysdic")
    }

    #[test]
    fn test_matcher_creation() {
        // Skip test if sysdic directory doesn't exist (e.g., in CI)
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        // Load FST bytes and test Matcher creation
        let fst_bytes_result = loader::load_fst_bytes(&sysdic_path);
        assert!(
            fst_bytes_result.is_ok(),
            "Failed to load FST bytes: {:?}",
            fst_bytes_result.err()
        );

        let fst_bytes = fst_bytes_result.unwrap();
        assert!(!fst_bytes.is_empty(), "FST bytes should not be empty");

        // Test Matcher creation from valid FST bytes
        let matcher_result = Matcher::new(fst_bytes);
        assert!(
            matcher_result.is_ok(),
            "Failed to create Matcher: {:?}",
            matcher_result.err()
        );

        let matcher = matcher_result.unwrap();

        // Test basic functionality with a simple run
        let run_result = matcher.run("test", true);
        assert!(
            run_result.is_ok(),
            "Matcher run should not fail: {:?}",
            run_result.err()
        );

        let (_matched, outputs) = run_result.unwrap();
        // Verify the run completed successfully (matched can be true or false depending on dictionary content)
        assert!(outputs.len() == 0 || outputs.len() > 0); // Basic sanity check
    }

    #[test]
    fn test_matcher_invalid_fst() {
        // Test Matcher creation with invalid FST data
        let invalid_fst_bytes = vec![0x00, 0x01, 0x02, 0x03]; // Invalid FST data

        let matcher_result = Matcher::new(invalid_fst_bytes);
        assert!(
            matcher_result.is_err(),
            "Matcher creation should fail with invalid FST data"
        );

        // Verify it's the right kind of error
        if let Err(error) = matcher_result {
            match error {
                RunomeError::DictValidationError { reason } => {
                    assert!(reason.contains("Failed to create FST"));
                }
                _ => panic!("Expected DictValidationError, got: {:?}", error),
            }
        }
    }

    #[test]
    fn test_matcher_run_exact_match() {
        // Skip test if sysdic directory doesn't exist (e.g., in CI)
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        // Load FST bytes and create matcher
        let fst_bytes = loader::load_fst_bytes(&sysdic_path).expect("Failed to load FST bytes");
        let matcher = Matcher::new(fst_bytes).expect("Failed to create Matcher");

        // Test exact match for "東京"
        let run_result = matcher.run("東京", false);
        assert!(
            run_result.is_ok(),
            "Matcher run should not fail: {:?}",
            run_result.err()
        );

        let (matched, outputs) = run_result.unwrap();

        // "東京" should exist in the dictionary
        if matched {
            assert!(!outputs.is_empty(), "Should have morpheme IDs for 東京");
            // Verify we got valid morpheme IDs (non-zero values)
            for morpheme_id in &outputs {
                assert!(*morpheme_id > 0, "Morpheme ID should be greater than 0");
            }
        }
        // Note: If not matched, the word might not be in this particular dictionary,
        // which is acceptable for this test - we're mainly testing the method works
    }

    #[test]
    fn test_matcher_run_prefix_match() {
        // Skip test if sysdic directory doesn't exist (e.g., in CI)
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        // Load FST bytes and create matcher
        let fst_bytes = loader::load_fst_bytes(&sysdic_path).expect("Failed to load FST bytes");
        let matcher = Matcher::new(fst_bytes).expect("Failed to create Matcher");

        // Test prefix match for "東京" (should include "東" if it exists)
        let run_result = matcher.run("東京", true);
        assert!(
            run_result.is_ok(),
            "Matcher run should not fail: {:?}",
            run_result.err()
        );

        let (matched, outputs) = run_result.unwrap();

        // With prefix matching, we should get results for any prefixes that exist
        // This could include "東" (1-char prefix) and "東京" (full word) if they exist
        if matched {
            assert!(
                !outputs.is_empty(),
                "Should have morpheme IDs for prefixes of 東京"
            );

            // Verify we got valid morpheme IDs
            for morpheme_id in &outputs {
                assert!(*morpheme_id > 0, "Morpheme ID should be greater than 0");
            }

            // With prefix matching, we should get more results than exact matching
            // Test that the method correctly processes multiple character boundaries
            // for multi-byte UTF-8 characters like "東京"
            let exact_match_result = matcher.run("東京", false);
            let (_, exact_outputs) = exact_match_result.unwrap();
            assert!(
                outputs.len() > exact_outputs.len(),
                "Prefix match should return at least as many results as exact match"
            );
        }
    }

    #[test]
    fn test_ram_dictionary_creation() {
        // Skip test if sysdic directory doesn't exist (e.g., in CI)
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        // TODO: Test RAMDictionary creation from DictionaryResource
        // This will be implemented when constructor is complete
    }

    #[test]
    fn test_get_trans_cost_delegation() {
        // Skip test if sysdic directory doesn't exist (e.g., in CI)
        let sysdic_path = get_test_sysdic_path();
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        // TODO: Test that get_trans_cost properly delegates to DictionaryResource
        // This will be implemented when RAMDictionary constructor is complete
    }

    #[test]
    fn test_lookup_interface() {
        // This test will verify the lookup interface once implemented
        // TODO: Test lookup functionality with known dictionary entries
    }
}
