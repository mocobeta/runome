# SystemDictionary Implementation Plan

## Overview
Create a SystemDictionary implementation that serves as the primary interface for Japanese morphological analysis in the Runome project. The SystemDictionary combines known word dictionary lookup capabilities with character classification functionality, following the same design patterns as Janome's Python implementation.

## Architecture Design

### Module Structure
```
src/dictionary/
├── mod.rs                    # Module exports including SystemDictionary
├── types.rs                  # Existing unified types
├── dict_resource.rs          # Existing DictionaryResource
├── loader.rs                 # Existing data loading functions
├── dict.rs                  # Existing Dictionary trait + RAMDictionary + Matcher
└── system_dict.rs           # New: SystemDictionary implementation
```

### Core Components

#### 1. Character Classification System (`system_dict.rs`)

**CharacterCategory**: Character behavior definitions
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterCategory {
    pub invoke: bool,    // INVOKE flag - always invoke unknown word processing
    pub group: bool,     // GROUP flag - group consecutive characters
    pub length: i32,     // LENGTH - maximum/minimum character length (-1 = no limit)
}
```

**CodePointRange**: Unicode ranges for character classification
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CodePointRange {
    pub from: char,
    pub to: char,
    pub category: String,
    pub compat_categories: Vec<String>,  // Compatible categories
}
```

**CharDefinitions**: Complete character definition data
```rust
#[derive(Debug, Clone)]
pub struct CharDefinitions {
    pub categories: HashMap<String, CharacterCategory>,
    pub code_ranges: Vec<CodePointRange>,
}
```

#### 2. SystemDictionary Implementation

**Core Structure**:
```rust
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

pub struct SystemDictionary {
    ram_dict: RAMDictionary,
    char_defs: CharDefinitions,
}

// Singleton instance with thread-safe lazy initialization
static SYSTEM_DICT_INSTANCE: Lazy<Arc<Mutex<Option<SystemDictionary>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));
```

**Public Interface**:
```rust
impl SystemDictionary {
    /// Get singleton instance of SystemDictionary
    pub fn instance() -> Result<Arc<SystemDictionary>, RunomeError>;
    
    /// Create new SystemDictionary from sysdic directory
    pub fn new(sysdic_dir: &Path) -> Result<Self, RunomeError>;
    
    /// Look up known words only (delegates to RAMDictionary)
    pub fn lookup(&self, surface: &str) -> Result<Vec<&DictEntry>, RunomeError>;
    
    /// Get connection cost between part-of-speech IDs
    pub fn get_trans_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError>;
    
    /// Get character categories for a given character
    pub fn get_char_categories(&self, ch: char) -> HashMap<String, Vec<String>>;
    
    /// Check if unknown word processing should always be invoked for category
    pub fn unknown_invoked_always(&self, category: &str) -> bool;
    
    /// Check if characters of this category should be grouped together
    pub fn unknown_grouping(&self, category: &str) -> bool;
    
    /// Get length constraint for unknown words of this category
    pub fn unknown_length(&self, category: &str) -> i32;
}
```

## Implementation Details

### Character Classification Logic

**Character Category Lookup**:
```rust
impl SystemDictionary {
    pub fn get_char_categories(&self, ch: char) -> HashMap<String, Vec<String>> {
        let mut result = HashMap::new();
        
        // Find all matching code point ranges for this character
        for range in &self.char_defs.code_ranges {
            if ch >= range.from && ch <= range.to {
                result.insert(range.category.clone(), range.compat_categories.clone());
            }
        }
        
        // Default category if no matches found
        if result.is_empty() {
            result.insert("DEFAULT".to_string(), Vec::new());
        }
        
        result
    }
}
```

### Singleton Pattern Implementation

**Thread-Safe Lazy Initialization**:
```rust
impl SystemDictionary {
    pub fn instance() -> Result<Arc<SystemDictionary>, RunomeError> {
        let instance_lock = SYSTEM_DICT_INSTANCE.lock()
            .map_err(|_| RunomeError::DictValidationError {
                reason: "Failed to acquire SystemDictionary lock".to_string(),
            })?;
        
        if let Some(ref instance) = *instance_lock {
            return Ok(Arc::clone(instance));
        }
        
        drop(instance_lock);
        
        // Create new instance
        let sysdic_path = Path::new("sysdic");
        let new_instance = Arc::new(Self::new(sysdic_path)?);
        
        let mut instance_lock = SYSTEM_DICT_INSTANCE.lock()
            .map_err(|_| RunomeError::DictValidationError {
                reason: "Failed to acquire SystemDictionary lock for initialization".to_string(),
            })?;
        
        *instance_lock = Some(Arc::clone(&new_instance));
        Ok(new_instance)
    }
}
```

### Error Handling Strategy

**New Error Variants**:
```rust
// In existing RunomeError enum
#[error("Character classification error: {reason}")]
CharClassificationError { reason: String },

#[error("SystemDictionary initialization failed: {reason}")]
SystemDictInitError { reason: String },
```

### Data Loading Integration

**Character Definitions Loading**:
- Reuse existing `DictionaryResource::load()` to get character definitions
- Extract character categories and code point ranges from `char_defs.bin`
- Maintain compatibility with existing bincode serialization format

**Dictionary Integration**:
- Create `RAMDictionary` using existing infrastructure
- Delegate all dictionary operations to the embedded `RAMDictionary`
- Maintain separation of concerns: known word lookup vs character analysis

## Key Design Decisions

### Clear Separation of Responsibilities
- **Known Word Lookup**: Delegated entirely to `RAMDictionary.lookup()`
- **Character Analysis**: Handled by `SystemDictionary` methods
- **Unknown Word Processing**: External to SystemDictionary (caller responsibility)

### Compatibility with Janome
- **Interface Parity**: Same method signatures as Python SystemDictionary
- **Behavior Consistency**: Character categories return same format as Python
- **Data Compatibility**: Uses same character definitions and dictionary data

### Performance Considerations
- **Singleton Pattern**: Single instance shared across application
- **Delegation**: Minimal overhead for dictionary operations
- **Lazy Loading**: CharDefinitions loaded once during initialization
- **Thread Safety**: Safe concurrent access to singleton instance

### Extensibility
- **Trait Compatibility**: Implements Dictionary trait through delegation
- **Modular Design**: CharDefinitions can be enhanced independently
- **Error Handling**: Rich error types for debugging and monitoring

## Implementation Steps

### Phase 1: Character Classification Infrastructure
1. **Define data structures** for CharacterCategory, CodePointRange, CharDefinitions
2. **Update DictionaryResource** to expose character definitions
3. **Implement character lookup logic** with proper Unicode handling

### Phase 2: SystemDictionary Core Implementation
1. **Create SystemDictionary struct** with RAMDictionary integration
2. **Implement delegation methods** for lookup and get_trans_cost
3. **Add character classification methods** with proper error handling

### Phase 3: Singleton Pattern and Thread Safety
1. **Implement singleton pattern** with lazy initialization
2. **Add thread-safe access** using Arc and Mutex
3. **Handle initialization errors** gracefully

### Phase 4: Testing and Validation
1. **Unit tests for character classification** covering all Unicode ranges
2. **Integration tests** comparing behavior with Python SystemDictionary
3. **Performance tests** for singleton access and lookup operations
4. **Error handling tests** for edge cases and invalid inputs

### Phase 5: Module Integration
1. **Update mod.rs exports** to include SystemDictionary
2. **Update lib.rs** to expose SystemDictionary publicly
3. **Add documentation** with usage examples

## Testing Strategy

### Character Classification Tests
```rust
#[test]
fn test_char_categories() {
    let sys_dict = SystemDictionary::instance().unwrap();
    
    assert_eq!(sys_dict.get_char_categories('は'), 
               hashmap!{"HIRAGANA".to_string() => vec![]});
    assert_eq!(sys_dict.get_char_categories('ハ'), 
               hashmap!{"KATAKANA".to_string() => vec![]});
    assert_eq!(sys_dict.get_char_categories('漢'), 
               hashmap!{"KANJI".to_string() => vec![]});
    assert_eq!(sys_dict.get_char_categories('五'), 
               hashmap!{"KANJI".to_string() => vec![], 
                       "KANJINUMERIC".to_string() => vec!["KANJI".to_string()]});
}
```

### Singleton Pattern Tests
```rust
#[test]
fn test_singleton_consistency() {
    let instance1 = SystemDictionary::instance().unwrap();
    let instance2 = SystemDictionary::instance().unwrap();
    
    // Should be the same instance
    assert!(Arc::ptr_eq(&instance1, &instance2));
}
```

### Dictionary Integration Tests
```rust
#[test]
fn test_lookup_delegation() {
    let sys_dict = SystemDictionary::instance().unwrap();
    let entries = sys_dict.lookup("東京").unwrap();
    
    // Should return same results as direct RAMDictionary lookup
    assert!(!entries.is_empty());
    for entry in entries {
        assert!(!entry.surface.is_empty());
        assert!(!entry.part_of_speech.is_empty());
    }
}
```

## Dependencies

### New Dependencies to Add
- `once_cell`: For lazy static initialization
- `parking_lot`: Alternative to std::sync for better performance (optional)

### Existing Dependencies to Leverage
- `fst`: Already used by RAMDictionary/Matcher
- `serde`: For character definitions deserialization
- `bincode`: For loading binary character data
- `thiserror`: For error handling

This implementation plan creates a robust, thread-safe SystemDictionary that maintains compatibility with Janome while leveraging Rust's performance and safety features. The design clearly separates known word lookup from character analysis, providing a clean foundation for Japanese morphological analysis applications.