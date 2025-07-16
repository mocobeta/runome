# TokenFilter Implementation Plan for Rust

## Overview
Create a complete Rust implementation of Janome's TokenFilter system using existing types from the Runome codebase. The implementation will provide a trait-based architecture for post-processing tokenization results with full Python compatibility.

## Background Analysis

### Existing Rust Infrastructure
- **Token struct** (`src/tokenizer.rs`): Complete with all fields (surface, part_of_speech, base_form, reading, phonetic, etc.)
- **Error handling** (`src/error.rs`): RunomeError type for consistent error propagation
- **Dependencies**: Unicode handling via standard library, HashMap for counting
- **Test framework**: Cargo test with comprehensive test patterns already established

### Python Janome TokenFilter System Analysis
Based on examination of `janome/janome/tokenfilter.py` and related test files:

#### TokenFilter Base Class
- Abstract base class with single `apply(tokens: Iterator[Token]) -> Iterator[Any]` method
- Callable interface via `__call__` delegation
- Return type flexibility to accommodate different output types

#### Filter Implementations
1. **LowerCaseFilter/UpperCaseFilter**: Case conversion for surface and base_form fields
2. **POSStopFilter/POSKeepFilter**: POS-based filtering with prefix matching
3. **CompoundNounFilter**: Stateful noun combination with field concatenation
4. **ExtractAttributeFilter**: Terminal filter extracting specific attributes as strings
5. **TokenCountFilter**: Terminal filter counting frequencies, returns tuples

#### Integration Points
- Used in `Analyzer` class for sequential filter application
- Memory-efficient iterator-based design
- Support for filter chaining with type transformations

## Implementation Plan

### Phase 1: Core TokenFilter Trait and Module Structure

#### 1.1 Create TokenFilter Module (`src/tokenfilter.rs`)
```rust
use std::collections::HashMap;
use crate::{Token, RunomeError};

/// Core trait for token filtering operations
/// Mirrors Python Janome's TokenFilter abstract base class
pub trait TokenFilter {
    type Output;
    
    /// Apply the filter to a stream of tokens
    /// Returns an iterator over the filtered/transformed output
    fn apply<I>(&self, tokens: I) -> Result<Box<dyn Iterator<Item = Self::Output>>, RunomeError>
    where 
        I: Iterator<Item = Token> + 'static;
}
```

#### 1.2 Error Handling
```rust
// Add to src/error.rs if needed
pub enum RunomeError {
    // ... existing variants
    InvalidAttribute { attribute: String },
    FilterError { message: String },
}
```

### Phase 2: Basic Filter Implementations

#### 2.1 Case Conversion Filters
```rust
/// Converts surface and base_form fields to lowercase
pub struct LowerCaseFilter;

impl TokenFilter for LowerCaseFilter {
    type Output = Token;
    
    fn apply<I>(&self, tokens: I) -> Result<Box<dyn Iterator<Item = Token>>, RunomeError>
    where I: Iterator<Item = Token> + 'static 
    {
        let iter = tokens.map(|mut token| {
            // Modify surface and base_form fields
            // Use to_lowercase() for Unicode-aware conversion
            token
        });
        Ok(Box::new(iter))
    }
}

/// Converts surface and base_form fields to uppercase  
pub struct UpperCaseFilter;
// Similar implementation with to_uppercase()
```

#### 2.2 POS-based Filters
```rust
/// Removes tokens with specified part-of-speech prefixes
pub struct POSStopFilter {
    pos_list: Vec<String>,
}

impl POSStopFilter {
    pub fn new(pos_list: Vec<String>) -> Self {
        Self { pos_list }
    }
}

impl TokenFilter for POSStopFilter {
    type Output = Token;
    
    fn apply<I>(&self, tokens: I) -> Result<Box<dyn Iterator<Item = Token>>, RunomeError>
    where I: Iterator<Item = Token> + 'static 
    {
        let pos_list = self.pos_list.clone();
        let iter = tokens.filter(move |token| {
            !pos_list.iter().any(|pos| token.part_of_speech().starts_with(pos))
        });
        Ok(Box::new(iter))
    }
}

/// Keeps only tokens with specified part-of-speech prefixes
pub struct POSKeepFilter {
    pos_list: Vec<String>,
}
// Similar implementation with inverted logic
```

### Phase 3: Advanced Filter Implementations

#### 3.1 CompoundNounFilter
```rust
/// Combines contiguous noun tokens into compound nouns
pub struct CompoundNounFilter;

impl TokenFilter for CompoundNounFilter {
    type Output = Token;
    
    fn apply<I>(&self, tokens: I) -> Result<Box<dyn Iterator<Item = Token>>, RunomeError>
    where I: Iterator<Item = Token> + 'static 
    {
        // Implement stateful processing to combine contiguous nouns
        // Logic:
        // 1. Collect tokens into groups
        // 2. For each group starting with "名詞", combine into compound
        // 3. Set POS to "名詞,複合,*,*"
        // 4. Concatenate surface, base_form, reading, phonetic fields
        
        Ok(Box::new(CompoundNounIterator::new(tokens)))
    }
}

struct CompoundNounIterator<I> {
    tokens: std::iter::Peekable<I>,
    pending: Option<Token>,
}
// Implementation details for stateful processing
```

#### 3.2 Terminal Filters
```rust
/// Extracts specific token attributes as strings (terminal filter)
pub struct ExtractAttributeFilter {
    attribute: String,
}

impl ExtractAttributeFilter {
    pub fn new(attribute: String) -> Result<Self, RunomeError> {
        // Validate attribute name
        match attribute.as_str() {
            "surface" | "part_of_speech" | "infl_type" | "infl_form" | 
            "base_form" | "reading" | "phonetic" => Ok(Self { attribute }),
            _ => Err(RunomeError::InvalidAttribute { attribute }),
        }
    }
}

impl TokenFilter for ExtractAttributeFilter {
    type Output = String;
    
    fn apply<I>(&self, tokens: I) -> Result<Box<dyn Iterator<Item = String>>, RunomeError>
    where I: Iterator<Item = Token> + 'static 
    {
        let attr = self.attribute.clone();
        let iter = tokens.map(move |token| {
            match attr.as_str() {
                "surface" => token.surface().to_string(),
                "part_of_speech" => token.part_of_speech().to_string(),
                "base_form" => token.base_form().to_string(),
                "reading" => token.reading().to_string(),
                "phonetic" => token.phonetic().to_string(),
                _ => String::new(), // Should not happen due to validation
            }
        });
        Ok(Box::new(iter))
    }
}

/// Counts token frequencies (terminal filter)
pub struct TokenCountFilter {
    attribute: String,
    sorted: bool,
}

impl TokenFilter for TokenCountFilter {
    type Output = (String, usize);
    
    fn apply<I>(&self, tokens: I) -> Result<Box<dyn Iterator<Item = (String, usize)>>, RunomeError>
    where I: Iterator<Item = Token> + 'static 
    {
        // Collect all tokens, count frequencies, return iterator over pairs
        // Use HashMap for counting, optionally sort by frequency
        Ok(Box::new(TokenCountIterator::new(tokens, &self.attribute, self.sorted)?))
    }
}
```

### Phase 4: Integration and Testing

#### 4.1 Module Integration
```rust
// Add to src/lib.rs
pub mod tokenfilter;
pub use tokenfilter::{
    TokenFilter, LowerCaseFilter, UpperCaseFilter, 
    POSStopFilter, POSKeepFilter, CompoundNounFilter,
    ExtractAttributeFilter, TokenCountFilter
};
```

#### 4.2 Test Suite Structure
```rust
// src/tokenfilter.rs or separate test file
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Tokenizer;
    
    // Test cases ported from Python:
    // - test_pos_filter_*
    // - test_lower_case_filter
    // - test_upper_case_filter  
    // - test_compound_noun_filter
    // - test_extract_attribute_filter_*
    // - test_token_count_filter_*
    
    #[test]
    fn test_lower_case_filter() {
        // Port from test_tokenfilter.py
    }
    
    #[test]
    fn test_compound_noun_filter() {
        // Test compound noun creation with Japanese text
    }
    
    // ... additional tests
}
```

## Implementation Strategy

### Step-by-Step Execution
1. **Create base trait and module** - Establish core architecture
2. **Implement simple filters** - LowerCase, UpperCase as foundation
3. **Add POS filters** - Build on string matching logic
4. **Implement compound filter** - Most complex stateful processing
5. **Add terminal filters** - Different output types
6. **Comprehensive testing** - Port all Python test cases
7. **Integration testing** - Test with tokenizer and real Japanese text

### Key Design Decisions

#### Memory Efficiency
- Iterator-based design prevents loading all tokens into memory
- Use `Box<dyn Iterator>` for type erasure while maintaining performance
- Clone tokens only when necessary for modifications

#### Error Handling
- Validate filter parameters at construction time
- Use `Result` types for fallible operations
- Provide clear error messages matching Python behavior

#### Python Compatibility
- Match exact field modification behavior
- Preserve token field semantics (surface, base_form, etc.)
- Maintain same POS prefix matching logic
- Handle Unicode correctly for Japanese text

#### Type Safety
- Use associated types for different output types
- Compile-time verification of filter chains where possible
- Runtime validation for dynamic attribute access

### Performance Considerations
- Minimize string allocations in hot paths
- Use efficient iteration patterns
- Consider SIMD for case conversion if needed
- Profile against Python implementation

### Testing Strategy
- Port all existing Python test cases
- Add Rust-specific edge case tests
- Integration tests with real Japanese text
- Performance benchmarks vs Python
- Memory usage validation

## Expected Deliverables

1. **Complete TokenFilter implementation** (`src/tokenfilter.rs`)
   - Core trait and all 7 filter types
   - Proper error handling and validation
   - Full Python API compatibility

2. **Comprehensive test suite**
   - >95% code coverage
   - All Python test cases ported
   - Japanese text processing validation

3. **Documentation**
   - Rustdoc for all public APIs
   - Usage examples for each filter
   - Migration guide from Python

4. **Integration**
   - Module exports in `src/lib.rs`
   - Compatibility with existing tokenizer
   - Future analyzer integration points

5. **Quality assurance**
   - Passes cargo fmt, clippy, and test
   - Performance benchmarks
   - Memory efficiency validation

This implementation will provide a robust, efficient, and Python-compatible token filtering system for the Runome library while leveraging Rust's type safety and performance benefits.