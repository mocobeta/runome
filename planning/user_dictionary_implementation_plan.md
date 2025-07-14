# UserDictionary Implementation Plan

## Overview
Implement a Rust counterpart to Janome's UserDictionary that supports both IPADIC and simplified CSV formats, provides FST-based lookup functionality, and integrates seamlessly with the existing Runome tokenizer architecture. The implementation will properly handle multiple morpheme IDs for a single surface form using the same pattern as the existing system dictionary.

## Analysis of Existing FST Implementation

### Key Insights from `build_fst()` in `build.rs`:
1. **Surface Form Grouping**: Multiple entries with the same surface form are grouped together using `HashMap<String, Vec<u32>>`
2. **Separate Morpheme Index**: FST stores simple index IDs (u64), while actual morpheme IDs are stored in a separate `Vec<Vec<u32>>` structure
3. **Index ID Mapping**: Each unique surface form gets one index ID that maps to a vector of morpheme IDs
4. **Efficient Lookup**: The `lookup_morpheme_ids()` method in `Matcher` resolves index IDs to morpheme ID vectors

### Implementation Pattern:
```rust
// Group entries by surface form
let mut surface_groups: HashMap<String, Vec<u32>> = HashMap::new();
for (id, entry) in entries.iter().enumerate() {
    surface_groups.entry(entry.surface.clone()).or_default().push(id as u32);
}

// Create separate morpheme index
let mut morpheme_index: Vec<Vec<u32>> = Vec::new();
let surface_to_index: Vec<(String, u64)> = surface_groups
    .iter()
    .map(|(surface, ids)| {
        let index_id = morpheme_index.len() as u64;
        morpheme_index.push(ids.clone());
        (surface.clone(), index_id)
    })
    .collect();
```

## Architecture Design

### Core Components

#### 1. Reuse Existing DictEntry Struct
Instead of creating a separate UserDictEntry struct, we'll reuse the existing `DictEntry` from `src/dictionary/types.rs`:

```rust
pub struct DictEntry {
    pub surface: String,
    pub left_id: u16,
    pub right_id: u16,
    pub cost: i16,
    pub part_of_speech: String,
    pub inflection_type: String,    // Maps to infl_type in CSV
    pub inflection_form: String,    // Maps to infl_form in CSV
    pub base_form: String,
    pub reading: String,
    pub phonetic: String,
    pub morph_id: usize,           // Set to entry index for user dictionary
}
```

#### 2. UserDictionary Struct
```rust
pub struct UserDictionary {
    entries: Vec<DictEntry>,        // Reuse existing DictEntry struct
    morpheme_index: Vec<Vec<u32>>,  // Maps FST index IDs to morpheme ID vectors
    matcher: Matcher,
    connections: Arc<Vec<Vec<i16>>>, // Reference to system dictionary connections
}
```

#### 3. CSV Format Support
```rust
pub enum UserDictFormat {
    Ipadic,
    Simpledic,
}
```

### File Structure
```
src/dictionary/
├── mod.rs                    # Updated exports
├── user_dict.rs              # New: UserDictionary implementation
└── user_dict_tests.rs        # New: Comprehensive tests
```

## Implementation Details

### 1. CSV Parsing Implementation

#### IPADIC Format Parser (13 fields):
```rust
fn parse_ipadic_line(line: &str, morph_id: usize) -> Result<DictEntry, RunomeError> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() != 13 {
        return Err(RunomeError::UserDictError { 
            reason: format!("Expected 13 fields, got {}", fields.len()) 
        });
    }
    
    Ok(DictEntry {
        surface: fields[0].to_string(),
        left_id: fields[1].parse::<u16>()?,
        right_id: fields[2].parse::<u16>()?,
        cost: fields[3].parse::<i16>()?,
        part_of_speech: format!("{},{},{},{}", fields[4], fields[5], fields[6], fields[7]),
        inflection_type: fields[8].to_string(),
        inflection_form: fields[9].to_string(),
        base_form: fields[10].to_string(),
        reading: fields[11].to_string(),
        phonetic: fields[12].to_string(),
        morph_id,
    })
}
```

#### Simplified Format Parser (3 fields):
```rust
fn parse_simpledic_line(line: &str, morph_id: usize) -> Result<DictEntry, RunomeError> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() != 3 {
        return Err(RunomeError::UserDictError { 
            reason: format!("Expected 3 fields, got {}", fields.len()) 
        });
    }
    
    let surface = fields[0].to_string();
    let pos_major = fields[1].to_string();
    let reading = fields[2].to_string();
    
    Ok(DictEntry {
        surface: surface.clone(),
        left_id: 0,
        right_id: 0,
        cost: -100000,
        part_of_speech: format!("{},*,*,*", pos_major),
        inflection_type: "*".to_string(),
        inflection_form: "*".to_string(),
        base_form: surface,
        reading: reading.clone(),
        phonetic: reading,
        morph_id,
    })
}
```

### 2. FST Building Implementation (Supporting Multiple Morpheme IDs)

```rust
impl UserDictionary {
    pub fn new(
        csv_path: &Path,
        format: UserDictFormat,
        connections: Arc<Vec<Vec<i16>>>,
    ) -> Result<Self, RunomeError> {
        let entries = Self::load_entries(csv_path, format)?;
        let (matcher, morpheme_index) = Self::build_fst(&entries)?;
        
        Ok(Self {
            entries,
            morpheme_index,
            matcher,
            connections,
        })
    }
    
    fn build_fst(entries: &[DictEntry]) -> Result<(Matcher, Vec<Vec<u32>>), RunomeError> {
        use std::collections::HashMap;
        
        // Group entries by surface form to handle duplicates
        let mut surface_groups: HashMap<String, Vec<u32>> = HashMap::new();
        for (id, entry) in entries.iter().enumerate() {
            surface_groups
                .entry(entry.surface.clone())
                .or_default()
                .push(id as u32);
        }
        
        // Create separate morpheme index for storing multiple morpheme IDs
        let mut morpheme_index: Vec<Vec<u32>> = Vec::new();
        
        // Create surface form to index ID mappings
        let mut surface_to_index: Vec<(String, u64)> = surface_groups
            .iter()
            .map(|(surface, ids)| {
                // Store morpheme IDs in separate index, FST stores only the index ID
                let index_id = morpheme_index.len() as u64;
                morpheme_index.push(ids.clone());
                (surface.clone(), index_id)
            })
            .collect();
        
        // Sort by surface form (required for FST building)
        surface_to_index.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Build FST
        let mut builder = fst::MapBuilder::memory();
        for (surface, index_id) in surface_to_index {
            builder
                .insert(surface.as_bytes(), index_id)
                .map_err(|e| RunomeError::FstBuildError { 
                    reason: format!("Failed to insert '{}': {}", surface, e) 
                })?;
        }
        
        let fst_bytes = builder.into_inner().map_err(|e| RunomeError::FstBuildError { 
            reason: format!("Failed to build FST: {}", e) 
        })?;
        
        let matcher = Matcher::new(fst_bytes)?;
        Ok((matcher, morpheme_index))
    }
}
```

### 3. Dictionary Interface Implementation

```rust
impl Dictionary for UserDictionary {
    fn lookup(&self, surface: &str) -> Result<Vec<&DictEntry>, RunomeError> {
        // Handle empty string case
        if surface.is_empty() {
            return Ok(Vec::new());
        }
        
        // 1. Use matcher to get index IDs matching the surface form
        let (matched, index_ids) = self.matcher.run(surface, true)?;
        
        // 2. If no matches found, return empty vector
        if !matched {
            return Ok(Vec::new());
        }
        
        let mut results = Vec::new();
        
        // 3. For each index ID, look up the morpheme IDs and resolve to entries
        for index_id in index_ids {
            let morpheme_ids = self.lookup_morpheme_ids(index_id);
            
            for morpheme_id in morpheme_ids {
                // Validate morpheme ID is within bounds
                if let Some(entry) = self.entries.get(morpheme_id as usize) {
                    // Entry is already a DictEntry - no conversion needed
                    results.push(entry);
                }
            }
        }
        
        Ok(results)
    }
    
    fn get_trans_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError> {
        // Delegate to system dictionary connections
        if let Some(row) = self.connections.get(left_id as usize) {
            if let Some(cost) = row.get(right_id as usize) {
                Ok(*cost)
            } else {
                Err(RunomeError::InvalidConnectionId { left_id, right_id })
            }
        } else {
            Err(RunomeError::InvalidConnectionId { left_id, right_id })
        }
    }
}

impl UserDictionary {
    /// Decode FST index ID to morpheme IDs using separate morpheme index
    fn lookup_morpheme_ids(&self, index_id: u64) -> Vec<u32> {
        if let Some(morpheme_ids) = self.morpheme_index.get(index_id as usize) {
            morpheme_ids.clone()
        } else {
            Vec::new()
        }
    }
    
    // No conversion method needed - entries are already DictEntry structs
}
```

### 4. Integration with Tokenizer

```rust
// Update Tokenizer constructor to support user dictionary
impl Tokenizer {
    pub fn new(
        user_dict_path: Option<&Path>,
        user_dict_format: Option<UserDictFormat>,
    ) -> Result<Self, RunomeError> {
        let system_dict = SystemDictionary::new(None)?;
        
        let user_dict = if let Some(path) = user_dict_path {
            let format = user_dict_format.unwrap_or(UserDictFormat::Ipadic);
            let connections = system_dict.get_connections();
            Some(UserDictionary::new(path, format, connections)?)
        } else {
            None
        };
        
        Ok(Self {
            system_dict,
            user_dict,
        })
    }
}

// Update tokenize method to check user dictionary first
impl Tokenizer {
    pub fn tokenize(&self, text: &str, wakati: Option<bool>, baseform_unk: Option<bool>) -> TokenizeResultIterator {
        // During lattice building, check user dictionary first, then system dictionary
        // Mark user dictionary entries with NodeType::UserDict
    }
}
```

## Testing Strategy

### 1. Unit Tests for Multiple Morpheme IDs
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_multiple_entries_same_surface() {
        // Test CSV with multiple entries for same surface form
        let csv_content = "\
東京,1288,1288,4569,名詞,固有名詞,地域,一般,*,*,東京,トウキョウ,トウキョウ
東京,1285,1285,4000,名詞,固有名詞,人名,一般,*,*,東京,トウキョウ,トウキョウ";
        
        let entries = parse_csv_content(csv_content, UserDictFormat::Ipadic).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].surface, "東京");
        assert_eq!(entries[1].surface, "東京");
        
        let (matcher, morpheme_index) = UserDictionary::build_fst(&entries).unwrap();
        
        // Both entries should be accessible via the same surface form
        let (matched, index_ids) = matcher.run("東京", true).unwrap();
        assert!(matched);
        assert_eq!(index_ids.len(), 1); // One index ID for the surface form
        
        let morpheme_ids = morpheme_index[index_ids[0] as usize].clone();
        assert_eq!(morpheme_ids.len(), 2); // Two morpheme IDs for this surface
    }
    
    #[test]
    fn test_lookup_multiple_morpheme_ids() {
        // Test that lookup returns all entries for a surface form
        let user_dict = create_test_user_dict_with_duplicates();
        let results = user_dict.lookup("東京").unwrap();
        assert_eq!(results.len(), 2); // Should return both entries
        
        // Verify both entries are different (different POS or other fields)
        assert_ne!(results[0].part_of_speech, results[1].part_of_speech);
    }
}
```

### 2. Integration Tests
```rust
#[test]
fn test_user_dictionary_with_system_dictionary() {
    // Test that user dictionary entries are preferred over system dictionary
    let user_dict = UserDictionary::new(
        Path::new("tests/user_ipadic.csv"),
        UserDictFormat::Ipadic,
        system_connections,
    ).unwrap();
    
    let results = user_dict.lookup("東京スカイツリー").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].node_type, NodeType::UserDict);
}
```

## Error Handling

### New Error Types
```rust
// Add to RunomeError enum
#[error("User dictionary error: {reason}")]
UserDictError { reason: String },

#[error("CSV parsing error at line {line}: {reason}")]
CsvParseError { line: usize, reason: String },

#[error("FST building error: {reason}")]
FstBuildError { reason: String },
```

## Implementation Steps

### Phase 1: Core Structure (Steps 1-3)
1. Create `user_dict.rs` with basic structs
2. Implement CSV parsing for both formats
3. Add comprehensive parsing tests including multiple entries

### Phase 2: FST Integration with Multiple Morpheme ID Support (Steps 4-6)
4. Implement FST building with surface form grouping
5. Create separate morpheme index structure
6. Add FST building tests with duplicate surface forms

### Phase 3: Dictionary Interface (Steps 7-9)
7. Implement Dictionary trait for UserDictionary
8. Add lookup functionality with multiple morpheme ID resolution
9. Implement connection cost delegation

### Phase 4: Tokenizer Integration (Steps 10-12)
10. Update Tokenizer constructor to support user dictionary
11. Add user dictionary lookup to tokenization process
12. Mark entries with NodeType::UserDict

### Phase 5: Testing & Validation (Steps 13-16)
13. Add comprehensive tests for multiple morpheme IDs
14. Integration tests with system dictionary
15. Performance benchmarking
16. Compatibility testing with Janome

## Key Features

### Multiple Morpheme ID Support
- **Surface Form Grouping**: Handles multiple CSV entries with identical surface forms
- **Efficient Storage**: Uses separate morpheme index to avoid FST data duplication
- **Consistent Interface**: Returns all matching entries for a surface form via single lookup
- **Performance**: O(1) morpheme ID resolution after FST lookup

### CSV Format Compatibility
- **IPADIC Format**: Full 13-field support with proper field validation
- **Simplified Format**: 3-field format with sensible defaults
- **Error Handling**: Comprehensive parsing error messages with line numbers

### DictEntry Reuse
- **Unified Data Structure**: Reuses existing `DictEntry` struct instead of creating separate `UserDictEntry`
- **Consistent Interface**: Dictionary lookup returns the same `DictEntry` type for both system and user dictionaries
- **No Conversion Overhead**: Eliminates need for data conversion between user and system dictionary entries
- **Simplified Code**: Reduces code duplication and maintenance burden

This implementation follows the exact same pattern as the existing system dictionary for handling multiple morpheme IDs, ensuring consistency and reliability.