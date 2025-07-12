# Dictionary Resource Loader Implementation Plan

## Overview
Create a comprehensive dictionary resource loader that can read and deserialize all dictionary components built by the DictionaryBuilder. This implementation focuses solely on data loading without FST lookup functionality.

## Architecture Design

### Module Structure
```
src/
├── lib.rs                    # Export DictionaryBuilder and DictionaryResource
├── dict_builder/             # Build-time system
│   ├── mod.rs                # DictionaryBuilder struct
│   └── build.rs              # Build logic
└── dictionary/               # Runtime system with shared types
    ├── mod.rs                # DictionaryResource struct and exports
    ├── types.rs              # Unified types for both build and runtime
    └── loader.rs             # Binary file loading functions
```

### Core Components

#### 1. Unified Types (`dictionary/types.rs`)
Move all existing types from `dict_builder/types.rs`:
```rust
// Core dictionary entry
pub struct DictEntry { surface, left_id, right_id, cost, pos fields... }

// Character definitions
pub struct CharCategory { invoke, group, length }
pub struct CodePointRange { from, to, category, compat_categories }
pub struct CharDefinitions { categories, code_ranges }

// Unknown word handling
pub struct UnknownEntry { left_id, right_id, cost, part_of_speech }

// Type aliases
pub type ConnectionMatrix = Vec<Vec<i16>>;
pub type UnknownEntries = HashMap<String, Vec<UnknownEntry>>;
```

#### 2. Dictionary Resource Container (`dictionary/mod.rs`)
```rust
pub struct DictionaryResource {
    pub entries: Vec<DictEntry>,
    pub connections: ConnectionMatrix,
    pub char_defs: CharDefinitions,
    pub unknowns: UnknownEntries,
    pub fst_bytes: Vec<u8>,  // Raw FST data for future use
}

impl DictionaryResource {
    pub fn load(sysdic_dir: &Path) -> Result<Self, RunomeError>;
    pub fn get_entries(&self) -> &[DictEntry];
    pub fn get_connection_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError>;
    pub fn get_char_category(&self, ch: char) -> Option<&CharCategory>;
    pub fn get_unknown_entries(&self, category: &str) -> Option<&[UnknownEntry]>;
}
```

#### 3. Data Loader Functions (`dictionary/loader.rs`)
```rust
// Load individual components
pub fn load_entries(sysdic_dir: &Path) -> Result<Vec<DictEntry>, RunomeError>;
pub fn load_connections(sysdic_dir: &Path) -> Result<ConnectionMatrix, RunomeError>;
pub fn load_char_definitions(sysdic_dir: &Path) -> Result<CharDefinitions, RunomeError>;
pub fn load_unknown_entries(sysdic_dir: &Path) -> Result<UnknownEntries, RunomeError>;
pub fn load_fst_bytes(sysdic_dir: &Path) -> Result<Vec<u8>, RunomeError>;

// Validation helpers
fn validate_sysdic_directory(path: &Path) -> Result<(), RunomeError>;
fn validate_file_exists(path: &Path, filename: &str) -> Result<PathBuf, RunomeError>;
```

### File Loading Strategy

#### Binary File Format
Each component is stored as bincode-serialized binary:
- `entries.bin`: Vec<DictEntry>
- `connections.bin`: ConnectionMatrix (Vec<Vec<i16>>)
- `char_defs.bin`: CharDefinitions
- `unknowns.bin`: UnknownEntries (HashMap)
- `dic.fst`: Raw FST bytes (load as Vec<u8>)

#### Error Handling
Uses the unified `RunomeError` enum defined in `src/error.rs`:
```rust
#[derive(Debug, thiserror::Error)]
pub enum RunomeError {
    // Dictionary loading errors
    #[error("Dictionary directory not found: {path}")]
    DictDirectoryNotFound { path: String },
    
    #[error("Required dictionary file missing: {filename}")]
    DictFileMissing { filename: String },
    
    #[error("Failed to deserialize dictionary {component}: {source}")]
    DictDeserializationError { component: String, source: bincode::Error },
    
    #[error("Invalid connection matrix access: left_id={left_id}, right_id={right_id}")]
    InvalidConnectionId { left_id: u16, right_id: u16 },
    
    #[error("Dictionary validation failed: {reason}")]
    DictValidationError { reason: String },
    
    // General IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    // Future: Add variants for other components (FST, tokenizer, etc.)
}
```

### Unified Error Strategy

The implementation uses a single `RunomeError` enum to handle all error scenarios across the entire project:

- **Consistency**: All public APIs return `Result<T, RunomeError>` 
- **Extensibility**: Dictionary-specific variants use `Dict` prefix to avoid conflicts
- **Future-proof**: Room for FST, tokenizer, and other component errors
- **Rich context**: Structured error data for debugging

### Implementation Steps

#### Phase 0: Error Module Setup  
1. Create `src/error.rs` with `RunomeError` enum
2. Export from `lib.rs`
3. Add `thiserror` dependency to `Cargo.toml`

#### Phase 1: Type Migration and Module Setup
1. Create `src/dictionary/` directory structure
2. Move types from `dict_builder/types.rs` to `dictionary/types.rs`
3. Update imports in dict_builder modules
4. Create basic module structure with exports

#### Phase 2: Core Data Loading
1. Implement `loader.rs` with individual loading functions
2. Add file validation and error handling
3. Create DictionaryResource struct with load() method
4. Add basic accessor methods

#### Phase 3: Integration and Testing
1. Update dict_builder to use new type location
2. Export DictionaryResource from lib.rs
3. Add comprehensive error handling
4. Create validation for loaded data integrity

### Data Access Interface

#### Connection Cost Access
```rust
impl DictionaryResource {
    pub fn get_connection_cost(&self, left_id: u16, right_id: u16) -> Result<i16, RunomeError> {
        self.connections
            .get(left_id as usize)
            .and_then(|row| row.get(right_id as usize))
            .copied()
            .ok_or(RunomeError::InvalidConnectionId { left_id, right_id })
    }
}
```

#### Character Category Lookup
```rust
impl DictionaryResource {
    pub fn get_char_category(&self, ch: char) -> Option<&CharCategory> {
        // Find the code point range that contains this character
        for range in &self.char_defs.code_ranges {
            if ch >= range.from && ch <= range.to {
                return self.char_defs.categories.get(&range.category);
            }
        }
        None
    }
}
```

### Key Design Decisions

**Eager Loading**: Load all components at once for simplicity and predictable performance

**Raw FST Storage**: Store FST as bytes without parsing to keep scope focused

**Unified Types**: Single type definition source in dictionary module

**Comprehensive Validation**: Validate directory structure and file presence

**Rich Error Types**: Detailed error information for debugging

**Immutable Data**: All loaded data is read-only for thread safety

This design creates a robust foundation for dictionary data access while keeping FST operations out of scope for this implementation phase.