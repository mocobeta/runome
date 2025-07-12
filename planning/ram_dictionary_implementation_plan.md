# RAMDictionary Implementation Plan

## Overview
Create a Dictionary trait and RAMDictionary implementation that provides the core API for morpheme lookups using FST and the existing DictionaryResource infrastructure. This implementation focuses on the `lookup()` and `get_trans_cost()` methods, with a Matcher struct handling FST traversal.

## Architecture Design

### Module Structure
```
src/dictionary/
├── mod.rs                    # Module exports
├── types.rs                  # Existing unified types
├── dict_resource.rs          # Existing DictionaryResource
├── loader.rs                 # Existing data loading functions
└── dictionary.rs             # New: Dictionary trait, Matcher struct, and RAMDictionary implementation
```

## Core Components

### 1. All Components in Single File (`src/dictionary/dictionary.rs`)

**Dictionary Trait Interface**:
```rust
use crate::error::RunomeError;
use super::types::DictEntry;

pub trait Dictionary {
    /// Look up morphemes matching a surface form
    /// Returns: Vec<&DictEntry>
    fn lookup(&self, surface: &str) -> Result<Vec<&DictEntry>, RunomeError>;
    
    /// Get connection cost between part-of-speech IDs
    fn get_trans_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError>;
}
```

**Matcher Struct**:
```rust
use fst::Map;
use std::collections::HashSet;
use crate::error::RunomeError;

pub struct Matcher {
    fst: Map<Vec<u8>>,
}

impl Matcher {
    /// Create new Matcher from FST bytes
    pub fn new(fst_bytes: Vec<u8>) -> Result<Self, RunomeError> {
        let fst = Map::new(fst_bytes)
            .map_err(|e| RunomeError::DictValidationError {
                reason: format!("Failed to create FST: {}", e),
            })?;
        Ok(Self { fst })
    }
    
    /// Run FST matching on input word
    /// Returns: (matched: bool, outputs: HashSet<u32>)
    pub fn run(&self, word: &str, common_prefix_match: bool) -> Result<(bool, HashSet<u32>), RunomeError> {
        // Implementation will use fst crate's automaton functionality
    }
}
```

**RAMDictionary Implementation**:
```rust
use super::{DictionaryResource};
use crate::error::RunomeError;

pub struct RAMDictionary {
    resource: DictionaryResource,
    matcher: Matcher,
}

impl RAMDictionary {
    /// Create new RAMDictionary from DictionaryResource
    pub fn new(resource: DictionaryResource) -> Result<Self, RunomeError> {
        // Extract FST bytes and create matcher
        let fst_bytes = resource.fst_bytes.clone();
        let matcher = Matcher::new(fst_bytes)?;
        
        Ok(Self { resource, matcher })
    }
}

impl Dictionary for RAMDictionary {
    fn lookup(&self, surface: &str) -> Result<Vec<&DictEntry>, RunomeError> {
        // 1. Use matcher to get morpheme IDs
        // 2. Resolve morpheme IDs to dictionary entries
        // 3. Return references to DictEntry structs
    }
    
    fn get_trans_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError> {
        // Delegate to DictionaryResource
        self.resource.get_connection_cost(left_id, right_id)
    }
}
```

**Design Rationale**:
- All components in single file for simpler organization
- Dictionary trait mirrors Janome's Python Dictionary abstract class interface
- `lookup()` returns references to `DictEntry` structs for efficiency and direct access to all morphological data
- Matcher struct handles FST operations and string→ID mapping
- RAMDictionary combines DictionaryResource and Matcher functionality
- Uses existing `RunomeError` for consistent error handling

## Implementation Details

### Lookup Method Implementation

**Algorithm Flow**:
1. **FST Matching**: Use `Matcher::run()` to get morpheme IDs matching the surface form
2. **Entry Resolution**: For each morpheme ID, look up the corresponding `DictEntry`
3. **Reference Collection**: Return references to the `DictEntry` structs

**Implementation Approach**:
```rust
fn lookup(&self, surface: &str) -> Result<Vec<&DictEntry>, RunomeError> {
    // Get morpheme IDs from FST
    let (matched, morpheme_ids) = self.matcher.run(surface, true)?;
    
    if !matched {
        return Ok(Vec::new());
    }
    
    let mut results = Vec::new();
    let entries = self.resource.get_entries();
    
    for morpheme_id in morpheme_ids {
        if let Some(entry) = entries.get(morpheme_id as usize) {
            results.push(entry);
        }
    }
    
    Ok(results)
}
```

### Error Handling Strategy

**Comprehensive Error Coverage**:
- **FST Creation Errors**: Handle malformed FST data
- **Index Out of Bounds**: Validate morpheme ID access
- **Encoding Issues**: Handle string encoding problems
- **Resource Access**: Leverage existing DictionaryResource error handling

**Error Types to Add**:
```rust
// In existing RunomeError enum
#[error("FST operation failed: {reason}")]
FstError { reason: String },

#[error("Invalid morpheme ID: {morpheme_id}")]
InvalidMorphemeId { morpheme_id: u32 },
```

## Integration with Existing Infrastructure

### Leveraging DictionaryResource

**Reuse Existing Functionality**:
- Entry access via `get_entries()`
- Connection costs via `get_connection_cost()`
- Character categories via `get_char_category()`
- Unknown entries via `get_unknown_entries()`
- Comprehensive validation via `validate()`

**Benefits**:
- No duplication of data access logic
- Consistent error handling and validation
- Proven robustness from existing implementation

### Module Export Strategy

**Updated `src/dictionary/mod.rs`**:
```rust
pub mod dict_resource;
pub mod loader;
pub mod types;
pub mod dictionary;      // New: Contains Dictionary trait, Matcher, and RAMDictionary

pub use dict_resource::DictionaryResource;
pub use dictionary::{Dictionary, Matcher, RAMDictionary};  // New exports from single module
pub use types::*;
```

**Updated `src/lib.rs`**:
```rust
pub use dictionary::{Dictionary, Matcher, RAMDictionary};  // New exports
```

## Testing Strategy

### Comprehensive Test Coverage

**Unit Tests for Matcher**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_matcher_creation() {
        // Test successful Matcher creation from valid FST bytes
    }
    
    #[test]
    fn test_matcher_run_exact_match() {
        // Test exact string matching
    }
    
    #[test]
    fn test_matcher_run_prefix_match() {
        // Test prefix matching mode
    }
    
    #[test]
    fn test_matcher_invalid_fst() {
        // Test error handling for malformed FST data
    }
}
```

**Integration Tests for RAMDictionary**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_ram_dictionary_creation() {
        // Test successful creation from DictionaryResource
    }
    
    #[test]
    fn test_lookup_known_words() {
        // Test lookup of known dictionary entries
    }
    
    #[test]
    fn test_lookup_unknown_words() {
        // Test lookup of words not in dictionary
    }
    
    #[test]
    fn test_get_trans_cost() {
        // Test connection cost retrieval
    }
    
    #[test]
    fn test_lookup_consistency() {
        // Compare results with expected morpheme data
    }
}
```

**Performance Tests**:
- Benchmark lookup performance against large datasets
- Memory usage validation for FST and entry data
- Comparison with Janome Python performance

## Implementation Steps

### Phase 1: Core Infrastructure
1. **Create dictionary.rs file** with Dictionary trait, Matcher struct, and RAMDictionary all in one file
2. **Implement basic structure** with method signatures and constructors

### Phase 2: Core Functionality  
4. **Implement FST matching logic** in `Matcher::run()`
5. **Implement lookup method** using Matcher + DictionaryResource
6. **Implement get_trans_cost()** by delegating to DictionaryResource

### Phase 3: Testing and Integration
7. **Add comprehensive unit tests** for all components
8. **Add integration tests** with real dictionary data
9. **Update module exports** and public API
10. **Performance validation** and optimization

## Key Design Decisions

### Separation of Concerns
- **Matcher**: Handles FST operations and string→ID mapping (within dictionary.rs)
- **RAMDictionary**: Handles entry resolution and implements Dictionary trait (within dictionary.rs)
- **DictionaryResource**: Continues to handle data storage and access (separate module)
- **Single File Organization**: All new dictionary interface components co-located for simplicity

### Performance Considerations
- **Eager Loading**: Load FST once during RAMDictionary creation
- **Direct Entry Access**: Use Vec indexing for O(1) morpheme ID lookup
- **Zero-Copy Lookups**: Return `DictEntry` references to avoid data copying
- **Efficient Memory Usage**: No string or data duplication in lookup results

### Compatibility with Janome
- **Interface Parity**: Dictionary trait matches Python Dictionary class
- **Return Format**: Returns `DictEntry` references providing complete morphological data
- **Behavior Consistency**: FST matching behavior mirrors Janome Matcher
- **Data Access**: Callers get direct access to all entry fields (surface, part_of_speech, reading, etc.)

### Future Extensibility
- **Trait-based Design**: Allows for additional Dictionary implementations
- **Pluggable Matcher**: Matcher can be enhanced with caching/optimization
- **Error Handling**: Rich error types support debugging and monitoring

This implementation creates a robust, efficient dictionary lookup system that integrates seamlessly with the existing Runome infrastructure while providing the core functionality needed for Japanese morphological analysis.