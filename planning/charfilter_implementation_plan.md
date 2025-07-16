# CharFilter Implementation Plan for Rust

## Overview
Create a complete Rust implementation of Janome's CharFilter system for text preprocessing operations. The implementation will provide a trait-based architecture for text transformation before tokenization with full Python compatibility.

## Background Analysis

### Existing Rust Infrastructure
- **Error handling** (`src/error.rs`): RunomeError type for consistent error propagation
- **Module system** (`src/lib.rs`): Well-organized module structure with proper exports
- **Unicode handling**: Standard library support for Unicode normalization
- **Test framework**: Cargo test with comprehensive test patterns already established
- **Dependencies**: No regex or Unicode normalization crates currently - will need to add

### Python Janome CharFilter System Analysis
Based on examination of `janome/janome/charfilter.py` and related files:

#### CharFilter Base Class Architecture
```python
class CharFilter(ABC):
    @abstractmethod
    def apply(self, text: str) -> str:
        pass
    
    def __call__(self, text: str) -> str:
        return self.apply(text)
```

**Key characteristics:**
- Simple abstract base class with single `apply()` method
- Callable interface via `__call__` delegation
- String-to-string transformation (unlike TokenFilter which can change output types)
- Stateless design - no persistent state between calls
- Used for text preprocessing before tokenization

#### CharFilter Implementations

1. **RegexReplaceCharFilter**
   - **Purpose**: Replace text patterns using regular expressions
   - **Constructor**: `RegexReplaceCharFilter(pattern: str, replacement: str)`
   - **Behavior**: Compiles regex pattern and applies `re.sub()` replacement
   - **Example**: `RegexReplaceCharFilter('蛇の目', 'janome')` - replaces "蛇の目" with "janome"
   - **Edge cases**: Empty pattern/replacement strings are handled gracefully

2. **UnicodeNormalizeCharFilter**
   - **Purpose**: Unicode normalization for text standardization
   - **Constructor**: `UnicodeNormalizeCharFilter(form='NFKC')`
   - **Supported forms**: 'NFC', 'NFKC', 'NFD', 'NFKD' (default: 'NFKC')
   - **Behavior**: Uses `unicodedata.normalize()` for Unicode normalization
   - **Example**: `'Ｐｙｔｈｏｎ'` → `'Python'` (fullwidth to halfwidth conversion)
   - **Japanese text**: `'ﾒｶﾞﾊﾞｲﾄ'` → `'メガバイト'` (halfwidth katakana to fullwidth)

#### Integration with Analyzer Framework
From `janome/janome/analyzer.py`:
```python
class Analyzer:
    def __init__(self, *, char_filters: List[CharFilter] = [], 
                 tokenizer: Optional[Tokenizer] = None,
                 token_filters: List[TokenFilter] = []):
        self.char_filters = char_filters
        # ...
    
    def analyze(self, text: str) -> Iterator[Any]:
        # Apply CharFilters sequentially
        for cfilter in self.char_filters:
            text = cfilter(text)
        
        # Then tokenize the preprocessed text
        tokens = self.tokenizer.tokenize(text, wakati=False)
        
        # Finally apply TokenFilters
        for tfilter in self.token_filters:
            tokens = tfilter(tokens)
        return tokens
```

**Key integration points:**
- CharFilters are applied **before** tokenization
- Sequential application in list order
- Text transformation is complete before tokenization begins
- No interaction with tokens - pure text preprocessing

#### Usage Patterns from Examples
```python
# Basic usage with Unicode normalization and regex replacement
text = '蛇の目はPure Ｐｙｔｈｏｎな形態素解析器です。'
char_filters = [
    UnicodeNormalizeCharFilter(),  # 'Ｐｙｔｈｏｎ' → 'Python'
    RegexReplaceCharFilter('蛇の目', 'janome')  # '蛇の目' → 'janome'
]

# Integration with full analysis pipeline
a = Analyzer(char_filters=char_filters, tokenizer=tokenizer, token_filters=token_filters)
tokens = a.analyze(text)
```

## Implementation Plan

### Phase 1: Core CharFilter Module Structure

#### 1.1 Create CharFilter Module (`src/charfilter.rs`)
```rust
use crate::RunomeError;
use std::borrow::Cow;

/// Core trait for character filtering operations
/// Mirrors Python Janome's CharFilter abstract base class
pub trait CharFilter {
    /// Apply the filter to input text
    /// Returns the transformed text
    fn apply(&self, text: &str) -> Result<String, RunomeError>;
    
    /// Convenience method for direct calling (mimics Python __call__)
    fn call(&self, text: &str) -> Result<String, RunomeError> {
        self.apply(text)
    }
}
```

#### 1.2 Add Required Dependencies
Update `Cargo.toml`:
```toml
[dependencies]
# ... existing dependencies
regex = "1.10"
unicode-normalization = "0.1"
```

#### 1.3 Error Handling Extensions
Add to `src/error.rs`:
```rust
pub enum RunomeError {
    // ... existing variants
    
    // CharFilter-specific errors
    #[error("Invalid regex pattern: {pattern}")]
    InvalidRegexPattern { 
        pattern: String,
        #[source]
        source: regex::Error,
    },
    
    #[error("Invalid Unicode normalization form: {form}")]
    InvalidNormalizationForm { form: String },
    
    #[error("CharFilter error: {message}")]
    CharFilterError { message: String },
}
```

### Phase 2: CharFilter Implementations

#### 2.1 RegexReplaceCharFilter
```rust
use regex::Regex;

/// Replaces text patterns using regular expressions
/// Mirrors Python's RegexReplaceCharFilter
#[derive(Debug, Clone)]
pub struct RegexReplaceCharFilter {
    pattern: Regex,
    replacement: String,
}

impl RegexReplaceCharFilter {
    /// Create a new RegexReplaceCharFilter
    /// 
    /// # Arguments
    /// * `pattern` - Regular expression pattern string
    /// * `replacement` - Replacement string
    /// 
    /// # Returns
    /// * `Ok(RegexReplaceCharFilter)` if pattern is valid
    /// * `Err(RunomeError)` if pattern is invalid
    pub fn new(pattern: &str, replacement: &str) -> Result<Self, RunomeError> {
        let regex = Regex::new(pattern)
            .map_err(|e| RunomeError::InvalidRegexPattern {
                pattern: pattern.to_string(),
                source: e,
            })?;
        
        Ok(Self {
            pattern: regex,
            replacement: replacement.to_string(),
        })
    }
}

impl CharFilter for RegexReplaceCharFilter {
    fn apply(&self, text: &str) -> Result<String, RunomeError> {
        Ok(self.pattern.replace_all(text, &self.replacement).to_string())
    }
}
```

#### 2.2 UnicodeNormalizeCharFilter
```rust
use unicode_normalization::{UnicodeNormalization, IsNormalized};

/// Unicode normalization for text standardization
/// Mirrors Python's UnicodeNormalizeCharFilter
#[derive(Debug, Clone)]
pub struct UnicodeNormalizeCharFilter {
    form: NormalizationForm,
}

#[derive(Debug, Clone)]
enum NormalizationForm {
    NFC,
    NFKC,
    NFD,
    NFKD,
}

impl UnicodeNormalizeCharFilter {
    /// Create a new UnicodeNormalizeCharFilter
    /// 
    /// # Arguments
    /// * `form` - Normalization form ('NFC', 'NFKC', 'NFD', 'NFKD')
    /// 
    /// # Returns
    /// * `Ok(UnicodeNormalizeCharFilter)` if form is valid
    /// * `Err(RunomeError)` if form is invalid
    pub fn new(form: &str) -> Result<Self, RunomeError> {
        let norm_form = match form {
            "NFC" => NormalizationForm::NFC,
            "NFKC" => NormalizationForm::NFKC,
            "NFD" => NormalizationForm::NFD,
            "NFKD" => NormalizationForm::NFKD,
            _ => return Err(RunomeError::InvalidNormalizationForm {
                form: form.to_string(),
            }),
        };
        
        Ok(Self { form: norm_form })
    }
    
    /// Create with default NFKC normalization
    pub fn default() -> Self {
        Self { form: NormalizationForm::NFKC }
    }
}

impl CharFilter for UnicodeNormalizeCharFilter {
    fn apply(&self, text: &str) -> Result<String, RunomeError> {
        let normalized = match self.form {
            NormalizationForm::NFC => text.nfc().collect::<String>(),
            NormalizationForm::NFKC => text.nfkc().collect::<String>(),
            NormalizationForm::NFD => text.nfd().collect::<String>(),
            NormalizationForm::NFKD => text.nfkd().collect::<String>(),
        };
        Ok(normalized)
    }
}
```

### Phase 3: Integration and Testing

#### 3.1 Module Integration
```rust
// Add to src/lib.rs
pub mod charfilter;

pub use charfilter::{
    CharFilter, RegexReplaceCharFilter, UnicodeNormalizeCharFilter
};
```

#### 3.2 Test Suite Structure
```rust
// src/charfilter.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regex_replace_charfilter_japanese() {
        // Test case from Python: RegexReplaceCharFilter('蛇の目', 'janome')
        let filter = RegexReplaceCharFilter::new("蛇の目", "janome").unwrap();
        let result = filter.apply("蛇の目は形態素解析器です。").unwrap();
        assert_eq!(result, "janomeは形態素解析器です。");
    }
    
    #[test]
    fn test_regex_replace_charfilter_whitespace() {
        // Test case from Python: RegexReplaceCharFilter('\s+', '')
        let filter = RegexReplaceCharFilter::new(r"\s+", "").unwrap();
        let result = filter.apply(" a  b c   d  ").unwrap();
        assert_eq!(result, "abcd");
    }
    
    #[test]
    fn test_regex_replace_charfilter_empty() {
        // Test edge case with empty replacement
        let filter = RegexReplaceCharFilter::new("", "").unwrap();
        let result = filter.apply("abc あいうえお").unwrap();
        assert_eq!(result, "abc あいうえお");
    }
    
    #[test]
    fn test_unicode_normalize_charfilter_default() {
        // Test default NFKC normalization
        let filter = UnicodeNormalizeCharFilter::default();
        
        // Fullwidth to halfwidth conversion
        let result = filter.apply("Ｐｙｔｈｏｎ").unwrap();
        assert_eq!(result, "Python");
        
        // Halfwidth katakana to fullwidth
        let result = filter.apply("ﾒｶﾞﾊﾞｲﾄ").unwrap();
        assert_eq!(result, "メガバイト");
    }
    
    #[test]
    fn test_unicode_normalize_charfilter_forms() {
        // Test different normalization forms
        let nfc = UnicodeNormalizeCharFilter::new("NFC").unwrap();
        let nfkc = UnicodeNormalizeCharFilter::new("NFKC").unwrap();
        let nfd = UnicodeNormalizeCharFilter::new("NFD").unwrap();
        let nfkd = UnicodeNormalizeCharFilter::new("NFKD").unwrap();
        
        // Test with composed/decomposed characters
        let text = "é"; // U+00E9 (composed) vs U+0065 U+0301 (decomposed)
        
        // All should handle the text appropriately
        assert!(nfc.apply(text).is_ok());
        assert!(nfkc.apply(text).is_ok());
        assert!(nfd.apply(text).is_ok());
        assert!(nfkd.apply(text).is_ok());
    }
    
    #[test]
    fn test_unicode_normalize_charfilter_invalid_form() {
        // Test invalid normalization form
        let result = UnicodeNormalizeCharFilter::new("INVALID");
        assert!(result.is_err());
        
        if let Err(RunomeError::InvalidNormalizationForm { form }) = result {
            assert_eq!(form, "INVALID");
        } else {
            panic!("Expected InvalidNormalizationForm error");
        }
    }
    
    #[test]
    fn test_regex_replace_charfilter_invalid_pattern() {
        // Test invalid regex pattern
        let result = RegexReplaceCharFilter::new("[", "replacement");
        assert!(result.is_err());
        
        if let Err(RunomeError::InvalidRegexPattern { pattern, source: _ }) = result {
            assert_eq!(pattern, "[");
        } else {
            panic!("Expected InvalidRegexPattern error");
        }
    }
    
    #[test]
    fn test_charfilter_call_method() {
        // Test the convenience call method
        let filter = UnicodeNormalizeCharFilter::default();
        let result = filter.call("Ｐｙｔｈｏｎ").unwrap();
        assert_eq!(result, "Python");
    }
}
```

### Phase 4: Advanced Features and Optimization

#### 4.1 Performance Optimizations
```rust
// Consider using Cow<str> for zero-copy when no changes are needed
impl CharFilter for UnicodeNormalizeCharFilter {
    fn apply(&self, text: &str) -> Result<String, RunomeError> {
        // Check if normalization is needed
        let needs_normalization = match self.form {
            NormalizationForm::NFC => !text.is_normalized_nfc(),
            NormalizationForm::NFKC => !text.is_normalized_nfkc(),
            NormalizationForm::NFD => !text.is_normalized_nfd(),
            NormalizationForm::NFKD => !text.is_normalized_nfkd(),
        };
        
        if !needs_normalization {
            return Ok(text.to_string());
        }
        
        // Apply normalization only if needed
        let normalized = match self.form {
            NormalizationForm::NFC => text.nfc().collect::<String>(),
            NormalizationForm::NFKC => text.nfkc().collect::<String>(),
            NormalizationForm::NFD => text.nfd().collect::<String>(),
            NormalizationForm::NFKD => text.nfkd().collect::<String>(),
        };
        Ok(normalized)
    }
}
```

#### 4.2 Builder Pattern for Complex Filters
```rust
/// Builder for creating complex CharFilter chains
pub struct CharFilterBuilder {
    filters: Vec<Box<dyn CharFilter>>,
}

impl CharFilterBuilder {
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }
    
    pub fn add_unicode_normalize(mut self, form: &str) -> Result<Self, RunomeError> {
        let filter = UnicodeNormalizeCharFilter::new(form)?;
        self.filters.push(Box::new(filter));
        Ok(self)
    }
    
    pub fn add_regex_replace(mut self, pattern: &str, replacement: &str) -> Result<Self, RunomeError> {
        let filter = RegexReplaceCharFilter::new(pattern, replacement)?;
        self.filters.push(Box::new(filter));
        Ok(self)
    }
    
    pub fn build(self) -> CharFilterChain {
        CharFilterChain { filters: self.filters }
    }
}

/// A chain of CharFilters that can be applied sequentially
pub struct CharFilterChain {
    filters: Vec<Box<dyn CharFilter>>,
}

impl CharFilter for CharFilterChain {
    fn apply(&self, text: &str) -> Result<String, RunomeError> {
        let mut result = text.to_string();
        for filter in &self.filters {
            result = filter.apply(&result)?;
        }
        Ok(result)
    }
}
```

#### 4.3 Integration Test with Full Pipeline
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::tokenizer::Tokenizer;
    
    #[test]
    fn test_charfilter_integration_with_tokenizer() {
        // Simulate the full analysis pipeline
        let text = "蛇の目はPure Ｐｙｔｈｏｎな形態素解析器です。";
        
        // Apply CharFilters
        let unicode_filter = UnicodeNormalizeCharFilter::default();
        let regex_filter = RegexReplaceCharFilter::new("蛇の目", "janome").unwrap();
        
        let mut processed_text = unicode_filter.apply(text).unwrap();
        processed_text = regex_filter.apply(&processed_text).unwrap();
        
        // Expected: "janomeはPure Pythonな形態素解析器です。"
        assert!(processed_text.contains("janome"));
        assert!(processed_text.contains("Python")); // Normalized from "Ｐｙｔｈｏｎ"
        
        // Would then be passed to tokenizer
        // let tokenizer = Tokenizer::new();
        // let tokens = tokenizer.tokenize(&processed_text);
    }
}
```

## Implementation Strategy

### Step-by-Step Execution
1. **Add dependencies** - Add regex and unicode-normalization crates
2. **Create base trait** - Establish core CharFilter architecture
3. **Implement RegexReplaceCharFilter** - Start with simpler regex-based filter
4. **Implement UnicodeNormalizeCharFilter** - Add Unicode normalization support
5. **Comprehensive testing** - Port all Python test cases and add edge cases
6. **Integration testing** - Test with real Japanese text and tokenizer
7. **Performance optimization** - Add zero-copy optimizations where possible
8. **Documentation** - Complete API documentation and examples

### Key Design Decisions

#### Memory Efficiency
- Use `&str` input to avoid unnecessary allocations
- Return `String` for owned results (consistent with Python behavior)
- Consider `Cow<str>` for optimization when no changes are needed
- Minimize regex compilation overhead by storing compiled patterns

#### Error Handling
- Validate filter parameters at construction time
- Use `Result` types for all fallible operations
- Provide clear error messages matching Python behavior patterns
- Handle regex compilation errors gracefully

#### Python Compatibility
- Match exact text transformation behavior
- Preserve Unicode normalization semantics
- Handle edge cases consistently (empty strings, invalid patterns)
- Maintain same API patterns (construction, apply, call)

#### Type Safety
- Use strong typing for normalization forms
- Compile-time validation where possible
- Runtime validation for dynamic inputs
- Clear separation of concerns between different filter types

### Performance Considerations
- Minimize string allocations in hot paths
- Use efficient regex compilation and caching
- Consider SIMD for Unicode normalization if needed
- Profile against Python implementation for benchmarking

### Testing Strategy
- Port all existing Python test cases
- Add Rust-specific edge case tests
- Integration tests with Japanese text processing
- Performance benchmarks vs Python implementation
- Memory usage validation
- Error handling validation

## Expected Deliverables

1. **Complete CharFilter implementation** (`src/charfilter.rs`)
   - Core trait and both filter implementations
   - Proper error handling and validation
   - Full Python API compatibility
   - Performance optimizations

2. **Comprehensive test suite**
   - >95% code coverage
   - All Python test cases ported
   - Japanese text processing validation
   - Error handling tests
   - Performance benchmarks

3. **Documentation**
   - Rustdoc for all public APIs
   - Usage examples for each filter
   - Integration examples with tokenizer
   - Migration guide from Python

4. **Integration**
   - Module exports in `src/lib.rs`
   - Dependency management in `Cargo.toml`
   - Error type extensions
   - Future analyzer integration preparation

5. **Quality assurance**
   - Passes cargo fmt, clippy, and test
   - Performance benchmarks
   - Memory efficiency validation
   - Unicode handling correctness

## Future Considerations

### Analyzer Integration
This CharFilter implementation will be designed to integrate seamlessly with a future Analyzer implementation:
```rust
// Future Analyzer structure
pub struct Analyzer {
    char_filters: Vec<Box<dyn CharFilter>>,
    tokenizer: Tokenizer,
    token_filters: Vec<Box<dyn TokenFilter>>,
}

impl Analyzer {
    pub fn analyze(&self, text: &str) -> Result<impl Iterator<Item = Token>, RunomeError> {
        // Apply CharFilters sequentially
        let mut processed_text = text.to_string();
        for filter in &self.char_filters {
            processed_text = filter.apply(&processed_text)?;
        }
        
        // Tokenize processed text
        let tokens = self.tokenizer.tokenize(&processed_text)?;
        
        // Apply TokenFilters
        // ... (use existing TokenFilter implementation)
    }
}
```

### Extension Points
- Custom CharFilter implementations
- Filter composition and chaining
- Configuration-based filter construction
- Streaming/incremental processing support

This implementation will provide a robust, efficient, and Python-compatible character filtering system for the Runome library while leveraging Rust's type safety and performance benefits.