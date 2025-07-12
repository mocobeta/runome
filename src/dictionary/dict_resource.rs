use crate::error::RunomeError;
use std::path::Path;

use super::{loader, types::*};

/// Container for all dictionary resources
pub struct DictionaryResource {
    entries: Vec<DictEntry>,
    connections: ConnectionMatrix,
    char_defs: CharDefinitions,
    unknowns: UnknownEntries,
    fst_bytes: Vec<u8>,
}

impl DictionaryResource {
    /// Load all dictionary components from sysdic directory
    pub fn load(sysdic_dir: &Path) -> Result<Self, RunomeError> {
        loader::validate_sysdic_directory(sysdic_dir)?;

        let entries = loader::load_entries(sysdic_dir)?;
        let connections = loader::load_connections(sysdic_dir)?;
        let char_defs = loader::load_char_definitions(sysdic_dir)?;
        let unknowns = loader::load_unknown_entries(sysdic_dir)?;
        let fst_bytes = loader::load_fst_bytes(sysdic_dir)?;

        Ok(Self {
            entries,
            connections,
            char_defs,
            unknowns,
            fst_bytes,
        })
    }

    /// Load and validate all dictionary components from sysdic directory
    pub fn load_and_validate(sysdic_dir: &Path) -> Result<Self, RunomeError> {
        let resource = Self::load(sysdic_dir)?;
        resource.validate()?;
        Ok(resource)
    }

    /// Validate the integrity of loaded dictionary data
    pub fn validate(&self) -> Result<(), RunomeError> {
        // Validate entries have reasonable values
        if self.entries.is_empty() {
            return Err(RunomeError::DictValidationError {
                reason: "Dictionary entries are empty".to_string(),
            });
        }

        // Validate connection matrix dimensions
        if self.connections.is_empty() {
            return Err(RunomeError::DictValidationError {
                reason: "Connection matrix is empty".to_string(),
            });
        }

        // Check that all rows in connection matrix have same length
        let first_row_len = self.connections[0].len();
        for (i, row) in self.connections.iter().enumerate() {
            if row.len() != first_row_len {
                return Err(RunomeError::DictValidationError {
                    reason: format!(
                        "Connection matrix row {} has inconsistent length: {} vs expected {}",
                        i,
                        row.len(),
                        first_row_len
                    ),
                });
            }
        }

        // Validate character definitions
        if self.char_defs.categories.is_empty() {
            return Err(RunomeError::DictValidationError {
                reason: "Character categories are empty".to_string(),
            });
        }

        if self.char_defs.code_ranges.is_empty() {
            return Err(RunomeError::DictValidationError {
                reason: "Character code ranges are empty".to_string(),
            });
        }

        // Validate that all code ranges reference existing categories
        for range in &self.char_defs.code_ranges {
            if !self.char_defs.categories.contains_key(&range.category) {
                return Err(RunomeError::DictValidationError {
                    reason: format!(
                        "Code range references non-existent category: {}",
                        range.category
                    ),
                });
            }
        }

        // Validate FST bytes are not empty
        if self.fst_bytes.is_empty() {
            return Err(RunomeError::DictValidationError {
                reason: "FST bytes are empty".to_string(),
            });
        }

        // Validate entry IDs are within reasonable bounds for connection matrix
        let max_id = (self.connections.len() - 1) as u16;
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.left_id > max_id {
                return Err(RunomeError::DictValidationError {
                    reason: format!(
                        "Entry {} has left_id {} exceeding connection matrix bounds (max: {})",
                        i, entry.left_id, max_id
                    ),
                });
            }
            if entry.right_id > max_id {
                return Err(RunomeError::DictValidationError {
                    reason: format!(
                        "Entry {} has right_id {} exceeding connection matrix bounds (max: {})",
                        i, entry.right_id, max_id
                    ),
                });
            }
        }

        Ok(())
    }

    /// Get all dictionary entries
    pub fn get_entries(&self) -> &[DictEntry] {
        &self.entries
    }

    /// Get connection cost between left and right part-of-speech IDs
    pub fn get_connection_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError> {
        self.connections
            .get(left_id as usize)
            .and_then(|row| row.get(right_id as usize))
            .copied()
            .ok_or(RunomeError::InvalidConnectionId { left_id, right_id })
    }

    /// Get character category for a given character
    pub fn get_char_category(&self, ch: char) -> Option<&CharCategory> {
        for range in &self.char_defs.code_ranges {
            if ch >= range.from && ch <= range.to {
                return self.char_defs.categories.get(&range.category);
            }
        }
        None
    }

    /// Get unknown entries for a specific category
    pub fn get_unknown_entries(&self, category: &str) -> Option<&[UnknownEntry]> {
        self.unknowns.get(category).map(|v| v.as_slice())
    }

    /// Get FST bytes for creating Matcher instances
    pub fn get_fst_bytes(&self) -> &[u8] {
        &self.fst_bytes
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
    fn test_load_dictionary_success() {
        let sysdic_path = get_test_sysdic_path();

        // Skip test if sysdic directory doesn't exist (e.g., in CI)
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let result = DictionaryResource::load(&sysdic_path);
        assert!(result.is_ok(), "Failed to load dictionary");

        let dict = result.unwrap();

        // Verify all components were loaded and are non-empty
        assert!(
            !dict.entries.is_empty(),
            "Dictionary entries should not be empty"
        );
        assert!(
            !dict.connections.is_empty(),
            "Connection matrix should not be empty"
        );
        assert!(
            !dict.char_defs.categories.is_empty(),
            "Character categories should not be empty"
        );
        assert!(
            !dict.char_defs.code_ranges.is_empty(),
            "Character code ranges should not be empty"
        );
        assert!(!dict.fst_bytes.is_empty(), "FST bytes should not be empty");

        println!(
            "Successfully loaded {} dictionary entries",
            dict.entries.len()
        );
        println!(
            "Connection matrix dimensions: {}x{}",
            dict.connections.len(),
            dict.connections.first().map_or(0, |row| row.len())
        );
        println!("Character categories: {}", dict.char_defs.categories.len());
        println!(
            "Character code ranges: {}",
            dict.char_defs.code_ranges.len()
        );
        println!("Unknown entry categories: {}", dict.unknowns.len());
        println!("FST size: {} bytes", dict.fst_bytes.len());
    }

    #[test]
    fn test_load_and_validate_success() {
        let sysdic_path = get_test_sysdic_path();

        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let result = DictionaryResource::load_and_validate(&sysdic_path);
        assert!(result.is_ok(), "Failed to load and validate dictionary.");

        println!("Dictionary loaded and validated successfully");
    }

    #[test]
    fn test_validate_data() {
        let sysdic_path = get_test_sysdic_path();

        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let dict = DictionaryResource::load(&sysdic_path).expect("Failed to load dictionary");
        let validation_result = dict.validate();

        assert!(
            validation_result.is_ok(),
            "Dictionary validation failed: {:?}",
            validation_result
        );
        println!("Dictionary validation passed");
    }

    #[test]
    fn test_get_entries() {
        let sysdic_path = get_test_sysdic_path();

        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let dict = DictionaryResource::load(&sysdic_path).expect("Failed to load dictionary");
        let entries = dict.get_entries();

        assert!(!entries.is_empty(), "Should have dictionary entries");

        // Print first few entries for verification
        for (i, entry) in entries.iter().take(5).enumerate() {
            println!("Entry {}: {} ({})", i, entry.surface, entry.part_of_speech);
        }
    }

    #[test]
    fn test_connection_costs() {
        let sysdic_path = get_test_sysdic_path();

        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let dict = DictionaryResource::load(&sysdic_path).expect("Failed to load dictionary");

        // Test some valid connection costs
        let cost_result = dict.get_connection_cost(0, 0);
        assert!(
            cost_result.is_ok(),
            "Should be able to get connection cost for valid indices"
        );

        let cost = cost_result.unwrap();
        println!("Connection cost (0,0): {}", cost);

        // Test boundary cases
        let max_id = (dict.connections.len() - 1) as u16;
        let boundary_cost = dict.get_connection_cost(max_id, max_id);
        assert!(
            boundary_cost.is_ok(),
            "Should be able to get connection cost for boundary indices"
        );

        // Test invalid indices
        let invalid_cost = dict.get_connection_cost(max_id + 1, 0);
        assert!(invalid_cost.is_err(), "Should fail for invalid indices");
    }

    #[test]
    fn test_char_categories() {
        let sysdic_path = get_test_sysdic_path();

        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let dict = DictionaryResource::load(&sysdic_path).expect("Failed to load dictionary");

        // Test some common characters
        let test_chars = ['あ', 'ア', '漢', 'A', '1', '。'];

        for ch in test_chars {
            let category = dict.get_char_category(ch);
            println!(
                "Character '{}': {:?}",
                ch,
                category
                    .map(|c| format!(
                        "invoke={}, group={}, length={}",
                        c.invoke, c.group, c.length
                    ))
                    .unwrap_or_else(|| "No category".to_string())
            );
        }
    }

    #[test]
    fn test_unknown_entries() {
        let sysdic_path = get_test_sysdic_path();

        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let dict = DictionaryResource::load(&sysdic_path).expect("Failed to load dictionary");

        // Print available unknown entry categories
        for category in dict.unknowns.keys() {
            let entries = dict.get_unknown_entries(category).unwrap();
            println!("Unknown category '{}': {} entries", category, entries.len());
        }

        // Test a non-existent category
        let nonexistent = dict.get_unknown_entries("NONEXISTENT_CATEGORY");
        assert!(
            nonexistent.is_none(),
            "Should return None for non-existent category"
        );
    }

    #[test]
    fn test_load_missing_directory() {
        let nonexistent_dir = PathBuf::from("/definitely/nonexistent/directory");
        let result = DictionaryResource::load(&nonexistent_dir);
        assert!(
            result.is_err(),
            "Should fail when loading non-existent directory"
        );

        if let Err(error) = result {
            match error {
                RunomeError::DictDirectoryNotFound { .. } => {
                    println!("Correctly detected missing directory");
                }
                _ => panic!("Expected DictDirectoryNotFound error, got: {:?}", error),
            }
        }
    }

    #[test]
    fn test_data_consistency() {
        let sysdic_path = get_test_sysdic_path();

        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let dict = DictionaryResource::load(&sysdic_path).expect("Failed to load dictionary");

        // Verify connection matrix is square
        let rows = dict.connections.len();
        for (i, row) in dict.connections.iter().enumerate() {
            assert_eq!(
                row.len(),
                dict.connections[0].len(),
                "Connection matrix row {} has inconsistent length",
                i
            );
        }

        // Verify all entries have valid connection IDs
        let max_id = (rows - 1) as u16;
        for (i, entry) in dict.entries.iter().enumerate() {
            assert!(
                entry.left_id <= max_id,
                "Entry {} has left_id {} exceeding matrix bounds (max: {})",
                i,
                entry.left_id,
                max_id
            );
            assert!(
                entry.right_id <= max_id,
                "Entry {} has right_id {} exceeding matrix bounds (max: {})",
                i,
                entry.right_id,
                max_id
            );
        }

        // Verify character code ranges reference existing categories
        for range in &dict.char_defs.code_ranges {
            assert!(
                dict.char_defs.categories.contains_key(&range.category),
                "Code range references non-existent category: {}",
                range.category
            );
        }

        println!("Data consistency checks passed");
    }
}
