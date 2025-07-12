use crate::error::RunomeError;
use std::path::Path;

pub mod loader;
pub mod types;

pub use types::*;

/// Container for all dictionary resources
pub struct DictionaryResource {
    pub entries: Vec<DictEntry>,
    pub connections: ConnectionMatrix,
    pub char_defs: CharDefinitions,
    pub unknowns: UnknownEntries,
    pub fst_bytes: Vec<u8>,
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
}
