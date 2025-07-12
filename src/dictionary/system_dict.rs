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
}
