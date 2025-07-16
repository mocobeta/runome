# Analyzer Implementation Plan for Rust

## Overview
Create a complete Rust implementation of Janome's Analyzer class that provides a unified text analysis pipeline. The implementation will combine CharFilter preprocessing, Tokenizer morphological analysis, and TokenFilter post-processing with full Python compatibility and type safety.

## Background Analysis

### Existing Rust Infrastructure
- **CharFilter system** (`src/charfilter.rs`): Complete text preprocessing with RegexReplaceCharFilter and UnicodeNormalizeCharFilter
- **Tokenizer system** (`src/tokenizer.rs`): Full morphological analysis with Token generation
- **TokenFilter system** (`src/tokenfilter.rs`): Complete post-processing with 7 filter types
- **Error handling** (`src/error.rs`): RunomeError type for consistent error propagation
- **Type system**: Strong typing with generics for filter output types

### Python Janome Analyzer System Analysis
Based on examination of `janome/janome/analyzer.py` and related test files:

#### Core Analyzer Architecture
```python
class Analyzer(object):
    def __init__(self, *,
                 char_filters: List[CharFilter] = [],
                 tokenizer: Optional[Tokenizer] = None,
                 token_filters: List[TokenFilter] = []):
        # Validation and initialization
        if not tokenizer:
            self.tokenizer = Tokenizer()
        elif tokenizer.wakati:
            raise Exception('Invalid argument: A Tokenizer with wakati=True option is not accepted.')
        else:
            self.tokenizer = tokenizer
        self.char_filters = char_filters
        self.token_filters = token_filters

    def analyze(self, text: str) -> Iterator[Any]:
        # Three-stage processing pipeline
        for cfilter in self.char_filters:
            text = cfilter(text)
        tokens = self.tokenizer.tokenize(text, wakati=False)
        for tfilter in self.token_filters:
            tokens = tfilter(tokens)
        return tokens
```

**Key characteristics:**
- **Three-stage pipeline**: CharFilter → Tokenizer → TokenFilter
- **Flexible configuration**: Optional components with sensible defaults
- **Type transformation**: Output type depends on final TokenFilter
- **Validation**: Rejects wakati mode tokenizers
- **Iterator-based**: Memory-efficient streaming processing

#### Integration Points
From `janome/janome/analyzer.py` and usage examples:

1. **CharFilter Integration**
   - Sequential application in list order
   - Text preprocessing before tokenization
   - String-to-string transformations

2. **Tokenizer Integration**
   - Uses tokenizer.tokenize(text, wakati=False)
   - Rejects wakati=True tokenizers
   - Generates full Token objects

3. **TokenFilter Integration**
   - Sequential application in list order
   - Type transformations (Token → String, Token → (String, usize), etc.)
   - Iterator-based processing

#### Usage Patterns from Examples
```python
# Basic usage with all three components
text = '蛇の目はPure Ｐｙｔｈｏｎな形態素解析器です。'
char_filters = [UnicodeNormalizeCharFilter(), RegexReplaceCharFilter('蛇の目', 'janome')]
tokenizer = Tokenizer()
token_filters = [CompoundNounFilter(), POSStopFilter(['記号','助詞']), LowerCaseFilter()]
a = Analyzer(char_filters=char_filters, tokenizer=tokenizer, token_filters=token_filters)
for token in a.analyze(text):
    print(token)

# Terminal filter usage (TokenCountFilter)
text = 'すもももももももものうち'
token_filters = [POSKeepFilter(['名詞']), TokenCountFilter()]
a = Analyzer(token_filters=token_filters)
for k, v in a.analyze(text):
    print('%s: %d' % (k, v))
```

## Implementation Plan

### Phase 1: Core Analyzer Structure

#### 1.1 Create Analyzer Module (`src/analyzer.rs`)
```rust
use crate::{CharFilter, TokenFilter, Tokenizer, Token, RunomeError};
use std::marker::PhantomData;

/// Core Analyzer for Japanese text analysis pipeline
/// Mirrors Python Janome's Analyzer class with full compatibility
pub struct Analyzer<T> {
    char_filters: Vec<Box<dyn CharFilter>>,
    tokenizer: Tokenizer,
    token_filters: Vec<Box<dyn TokenFilter<Output = T>>>,
    _phantom: PhantomData<T>,
}

impl<T> Analyzer<T> {
    /// Create a new AnalyzerBuilder
    pub fn builder() -> AnalyzerBuilder<Token> {
        AnalyzerBuilder::new()
    }
    
    /// Analyze text through the complete pipeline
    pub fn analyze(&self, text: &str) -> Result<Box<dyn Iterator<Item = T>>, RunomeError> {
        // Stage 1: Apply CharFilters sequentially
        let mut processed_text = text.to_string();
        for filter in &self.char_filters {
            processed_text = filter.apply(&processed_text)?;
        }
        
        // Stage 2: Tokenize the preprocessed text
        let tokens = self.tokenizer.tokenize(&processed_text)?;
        
        // Stage 3: Apply TokenFilters sequentially
        let mut current_iter: Box<dyn Iterator<Item = T>> = Box::new(tokens.into_iter());
        for filter in &self.token_filters {
            current_iter = filter.apply(current_iter)?;
        }
        
        Ok(current_iter)
    }
}
```

#### 1.2 Builder Pattern Implementation
```rust
/// Builder for creating Analyzer instances with type safety
pub struct AnalyzerBuilder<T> {
    char_filters: Vec<Box<dyn CharFilter>>,
    tokenizer: Option<Tokenizer>,
    token_filters: Vec<Box<dyn TokenFilter<Output = T>>>,
    _phantom: PhantomData<T>,
}

impl<T> AnalyzerBuilder<T> {
    fn new() -> AnalyzerBuilder<Token> {
        AnalyzerBuilder {
            char_filters: Vec::new(),
            tokenizer: None,
            token_filters: Vec::new(),
            _phantom: PhantomData,
        }
    }
    
    /// Add a CharFilter to the preprocessing chain
    pub fn add_char_filter<F>(mut self, filter: F) -> Self
    where
        F: CharFilter + 'static,
    {
        self.char_filters.push(Box::new(filter));
        self
    }
    
    /// Set the tokenizer (default if not specified)
    pub fn tokenizer(mut self, tokenizer: Tokenizer) -> Result<Self, RunomeError> {
        // Validate tokenizer configuration
        if tokenizer.is_wakati_mode() {
            return Err(RunomeError::AnalyzerError {
                message: "Invalid argument: A Tokenizer with wakati=True option is not accepted.".to_string(),
            });
        }
        self.tokenizer = Some(tokenizer);
        Ok(self)
    }
    
    /// Add a TokenFilter to the post-processing chain
    pub fn add_token_filter<F>(mut self, filter: F) -> AnalyzerBuilder<F::Output>
    where
        F: TokenFilter + 'static,
    {
        // Type transformation: change the output type
        AnalyzerBuilder {
            char_filters: self.char_filters,
            tokenizer: self.tokenizer,
            token_filters: {
                let mut filters = Vec::new();
                // Convert existing filters to new output type
                for old_filter in self.token_filters {
                    // This requires careful type handling
                }
                filters.push(Box::new(filter));
                filters
            },
            _phantom: PhantomData,
        }
    }
    
    /// Build the final Analyzer
    pub fn build(self) -> Analyzer<T> {
        Analyzer {
            char_filters: self.char_filters,
            tokenizer: self.tokenizer.unwrap_or_else(|| Tokenizer::new()),
            token_filters: self.token_filters,
            _phantom: PhantomData,
        }
    }
}
```

### Phase 2: Type System Design

#### 2.1 Handling Type Transformations
The main challenge is handling type transformations as TokenFilters can change output types:

```rust
/// Alternative approach: Use enum for different output types
pub enum AnalyzerOutput {
    Token(Token),
    String(String),
    Tuple(String, usize),
    // Add more as needed
}

/// Simpler Analyzer without complex generics
pub struct Analyzer {
    char_filters: Vec<Box<dyn CharFilter>>,
    tokenizer: Tokenizer,
    token_filters: Vec<Box<dyn TokenFilter<Output = AnalyzerOutput>>>,
}

impl Analyzer {
    pub fn analyze(&self, text: &str) -> Result<Box<dyn Iterator<Item = AnalyzerOutput>>, RunomeError> {
        // Implementation with type erasure
    }
}
```

#### 2.2 Type-Safe Filter Composition
```rust
/// Trait for composable filters
pub trait FilterChain<Input, Output> {
    fn apply(&self, input: Box<dyn Iterator<Item = Input>>) -> Result<Box<dyn Iterator<Item = Output>>, RunomeError>;
}

/// Implement for different filter combinations
impl<F1, F2, T1, T2, T3> FilterChain<T1, T3> for (F1, F2)
where
    F1: TokenFilter<Output = T2>,
    F2: TokenFilter<Output = T3>,
{
    fn apply(&self, input: Box<dyn Iterator<Item = T1>>) -> Result<Box<dyn Iterator<Item = T3>>, RunomeError> {
        let intermediate = self.0.apply(input)?;
        self.1.apply(intermediate)
    }
}
```

### Phase 3: Implementation Strategy

#### 3.1 Simplified Type-Erased Approach
```rust
use crate::{CharFilter, TokenFilter, Tokenizer, Token, RunomeError};

/// Analyzer with type erasure for simplicity
pub struct Analyzer {
    char_filters: Vec<Box<dyn CharFilter>>,
    tokenizer: Tokenizer,
    token_filters: Vec<Box<dyn TokenFilter<Output = Box<dyn std::any::Any>>>>,
}

impl Analyzer {
    /// Create a new Analyzer with builder pattern
    pub fn builder() -> AnalyzerBuilder {
        AnalyzerBuilder {
            char_filters: Vec::new(),
            tokenizer: None,
            token_filters: Vec::new(),
        }
    }
    
    /// Analyze text and return iterator over final results
    pub fn analyze<T: 'static>(&self, text: &str) -> Result<Box<dyn Iterator<Item = T>>, RunomeError> {
        // Stage 1: Apply CharFilters
        let mut processed_text = text.to_string();
        for filter in &self.char_filters {
            processed_text = filter.apply(&processed_text)?;
        }
        
        // Stage 2: Tokenize
        let tokens = self.tokenizer.tokenize(&processed_text)?;
        
        // Stage 3: Apply TokenFilters
        let mut current_iter: Box<dyn Iterator<Item = Box<dyn std::any::Any>>> = 
            Box::new(tokens.into_iter().map(|t| Box::new(t) as Box<dyn std::any::Any>));
        
        for filter in &self.token_filters {
            current_iter = filter.apply(current_iter)?;
        }
        
        // Type cast back to T
        let result = current_iter.map(|item| {
            *item.downcast::<T>().map_err(|_| RunomeError::AnalyzerError {
                message: "Type mismatch in analyzer output".to_string(),
            }).unwrap()
        });
        
        Ok(Box::new(result))
    }
}
```

#### 3.2 Builder Pattern Implementation
```rust
pub struct AnalyzerBuilder {
    char_filters: Vec<Box<dyn CharFilter>>,
    tokenizer: Option<Tokenizer>,
    token_filters: Vec<Box<dyn TokenFilter<Output = Box<dyn std::any::Any>>>>,
}

impl AnalyzerBuilder {
    fn new() -> Self {
        Self {
            char_filters: Vec::new(),
            tokenizer: None,
            token_filters: Vec::new(),
        }
    }
    
    /// Add a CharFilter to the preprocessing chain
    pub fn add_char_filter<F>(mut self, filter: F) -> Self
    where
        F: CharFilter + 'static,
    {
        self.char_filters.push(Box::new(filter));
        self
    }
    
    /// Set the tokenizer (validates wakati mode)
    pub fn tokenizer(mut self, tokenizer: Tokenizer) -> Result<Self, RunomeError> {
        if tokenizer.is_wakati_mode() {
            return Err(RunomeError::AnalyzerError {
                message: "Invalid argument: A Tokenizer with wakati=True option is not accepted.".to_string(),
            });
        }
        self.tokenizer = Some(tokenizer);
        Ok(self)
    }
    
    /// Add a TokenFilter to the post-processing chain
    pub fn add_token_filter<F>(mut self, filter: F) -> Self
    where
        F: TokenFilter + 'static,
        F::Output: 'static,
    {
        // Wrap the filter to handle type erasure
        let wrapped_filter = TypeErasedFilter::new(filter);
        self.token_filters.push(Box::new(wrapped_filter));
        self
    }
    
    /// Build the final Analyzer
    pub fn build(self) -> Analyzer {
        Analyzer {
            char_filters: self.char_filters,
            tokenizer: self.tokenizer.unwrap_or_else(|| Tokenizer::new()),
            token_filters: self.token_filters,
        }
    }
}
```

### Phase 4: Error Handling and Validation

#### 4.1 Error Types Extension
```rust
// Add to src/error.rs
pub enum RunomeError {
    // ... existing variants
    
    /// Analyzer-specific errors
    #[error("Analyzer error: {message}")]
    AnalyzerError { message: String },
    
    #[error("Invalid tokenizer configuration: {reason}")]
    InvalidTokenizerConfig { reason: String },
    
    #[error("Filter chain error: {message}")]
    FilterChainError { message: String },
}
```

#### 4.2 Validation Logic
```rust
impl Analyzer {
    /// Validate the analyzer configuration
    fn validate(&self) -> Result<(), RunomeError> {
        // Check tokenizer configuration
        if self.tokenizer.is_wakati_mode() {
            return Err(RunomeError::InvalidTokenizerConfig {
                reason: "wakati mode is not supported in Analyzer".to_string(),
            });
        }
        
        // Validate filter chains
        // ... additional validation logic
        
        Ok(())
    }
}
```

### Phase 5: Testing Strategy

#### 5.1 Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UnicodeNormalizeCharFilter, RegexReplaceCharFilter, CompoundNounFilter, POSStopFilter, LowerCaseFilter, ExtractAttributeFilter, TokenCountFilter};
    
    #[test]
    fn test_analyzer_default() {
        // Test default analyzer creation
        let analyzer = Analyzer::builder().build();
        assert!(analyzer.char_filters.is_empty());
        assert!(analyzer.token_filters.is_empty());
        assert!(!analyzer.tokenizer.is_wakati_mode());
    }
    
    #[test]
    fn test_analyzer_custom() {
        // Test custom analyzer with all components
        let analyzer = Analyzer::builder()
            .add_char_filter(UnicodeNormalizeCharFilter::with_default_form())
            .add_char_filter(RegexReplaceCharFilter::new(r"\s+", "").unwrap())
            .add_token_filter(CompoundNounFilter)
            .add_token_filter(POSStopFilter::new(vec!["記号".to_string(), "助詞".to_string()]))
            .add_token_filter(LowerCaseFilter)
            .build();
        
        assert_eq!(analyzer.char_filters.len(), 2);
        assert_eq!(analyzer.token_filters.len(), 3);
    }
    
    #[test]
    fn test_analyzer_full_pipeline() {
        // Test complete analysis pipeline
        let analyzer = Analyzer::builder()
            .add_char_filter(UnicodeNormalizeCharFilter::with_default_form())
            .add_char_filter(RegexReplaceCharFilter::new("蛇の目", "janome").unwrap())
            .add_token_filter(CompoundNounFilter)
            .add_token_filter(POSStopFilter::new(vec!["記号".to_string(), "助詞".to_string()]))
            .add_token_filter(LowerCaseFilter)
            .add_token_filter(ExtractAttributeFilter::new("surface".to_string()).unwrap())
            .build();
        
        let results: Vec<String> = analyzer.analyze("蛇の目はPure Ｐｙｔｈｏｎな形態素解析器です。").unwrap().collect();
        let expected = vec!["janome", "pure", "python", "な", "形態素解析器", "です"];
        assert_eq!(results, expected);
    }
    
    #[test]
    fn test_analyzer_token_count_filter() {
        // Test terminal TokenCountFilter
        let analyzer = Analyzer::builder()
            .add_token_filter(POSKeepFilter::new(vec!["名詞".to_string()]))
            .add_token_filter(TokenCountFilter::new("surface".to_string(), false).unwrap())
            .build();
        
        let results: Vec<(String, usize)> = analyzer.analyze("すもももももももものうち").unwrap().collect();
        
        let counts: std::collections::HashMap<String, usize> = results.into_iter().collect();
        assert_eq!(counts.get("もも"), Some(&2));
        assert_eq!(counts.get("すもも"), Some(&1));
        assert_eq!(counts.get("うち"), Some(&1));
    }
    
    #[test]
    fn test_analyzer_wakati_rejection() {
        // Test wakati mode rejection
        let wakati_tokenizer = Tokenizer::new_with_wakati(true);
        let result = Analyzer::builder()
            .tokenizer(wakati_tokenizer);
        
        assert!(result.is_err());
        if let Err(RunomeError::AnalyzerError { message }) = result {
            assert!(message.contains("wakati=True"));
        }
    }
}
```

### Phase 6: Performance Optimizations

#### 6.1 Iterator Chain Optimization
```rust
impl Analyzer {
    /// Optimized analyze method with minimal allocations
    pub fn analyze_streaming<'a>(&'a self, text: &'a str) -> Result<impl Iterator<Item = AnalyzerOutput> + 'a, RunomeError> {
        // Use iterator adapters for zero-copy processing where possible
        let processed_text = self.apply_char_filters(text)?;
        let tokens = self.tokenizer.tokenize_streaming(&processed_text)?;
        let filtered_tokens = self.apply_token_filters(tokens)?;
        Ok(filtered_tokens)
    }
}
```

#### 6.2 Memory Management
```rust
/// Efficient memory management for large text processing
pub struct StreamingAnalyzer {
    analyzer: Analyzer,
    buffer_size: usize,
}

impl StreamingAnalyzer {
    pub fn new(analyzer: Analyzer, buffer_size: usize) -> Self {
        Self { analyzer, buffer_size }
    }
    
    pub fn analyze_large_text<'a>(&'a self, text: &'a str) -> impl Iterator<Item = AnalyzerOutput> + 'a {
        // Process text in chunks to manage memory usage
        text.chars()
            .collect::<Vec<_>>()
            .chunks(self.buffer_size)
            .flat_map(move |chunk| {
                let chunk_str: String = chunk.iter().collect();
                self.analyzer.analyze(&chunk_str).unwrap()
            })
    }
}
```

### Phase 7: Integration and Documentation

#### 7.1 Module Integration
```rust
// Add to src/lib.rs
pub mod analyzer;
pub use analyzer::{Analyzer, AnalyzerBuilder, AnalyzerOutput};
```

#### 7.2 Documentation Examples
```rust
/// Complete text analysis pipeline
/// 
/// # Example: Basic Usage
/// ```rust
/// use runome::{Analyzer, UnicodeNormalizeCharFilter, RegexReplaceCharFilter, CompoundNounFilter, LowerCaseFilter};
/// 
/// let analyzer = Analyzer::builder()
///     .add_char_filter(UnicodeNormalizeCharFilter::with_default_form())
///     .add_char_filter(RegexReplaceCharFilter::new("蛇の目", "janome").unwrap())
///     .add_token_filter(CompoundNounFilter)
///     .add_token_filter(LowerCaseFilter)
///     .build();
/// 
/// let results: Vec<Token> = analyzer.analyze("蛇の目はPure Ｐｙｔｈｏｎな形態素解析器です。").unwrap().collect();
/// ```
/// 
/// # Example: Token Counting
/// ```rust
/// use runome::{Analyzer, POSKeepFilter, TokenCountFilter};
/// 
/// let analyzer = Analyzer::builder()
///     .add_token_filter(POSKeepFilter::new(vec!["名詞".to_string()]))
///     .add_token_filter(TokenCountFilter::new("surface".to_string(), false).unwrap())
///     .build();
/// 
/// let counts: Vec<(String, usize)> = analyzer.analyze("すもももももももものうち").unwrap().collect();
/// ```
impl Analyzer {
    // Implementation
}
```

## Implementation Strategy

### Step-by-Step Execution
1. **Create analyzer module** - Establish basic structure and types
2. **Implement builder pattern** - Type-safe configuration with validation
3. **Add core analyze method** - Three-stage pipeline with error handling
4. **Implement type handling** - Support for different TokenFilter outputs
5. **Add comprehensive tests** - Port all Python test cases
6. **Performance optimization** - Iterator-based streaming processing
7. **Documentation** - Complete API documentation with examples

### Key Design Decisions

#### Type System Approach
- **Type erasure with Any trait** for simplicity and flexibility
- **Builder pattern** for configuration with compile-time validation
- **Generic output types** to handle different TokenFilter results
- **Error handling** with specific error types for debugging

#### Performance Considerations
- **Iterator-based processing** for memory efficiency
- **Streaming analysis** for large text processing
- **Zero-copy optimizations** where possible
- **Chunked processing** for memory management

#### Python Compatibility
- **Exact API behavior** matching Python Analyzer
- **Same validation rules** (wakati mode rejection)
- **Error message compatibility** for debugging
- **Usage pattern support** for all documented scenarios

## Expected Deliverables

1. **Complete Analyzer implementation** (`src/analyzer.rs`)
   - Core Analyzer struct with builder pattern
   - Type-safe filter composition
   - Full Python API compatibility
   - Comprehensive error handling

2. **Test suite**
   - All Python test cases ported
   - Edge case coverage
   - Performance benchmarks
   - Integration tests

3. **Documentation**
   - Complete API documentation
   - Usage examples
   - Migration guide from Python
   - Performance characteristics

4. **Integration**
   - Module exports in `src/lib.rs`
   - Seamless integration with existing filters
   - Future extension points

This implementation will provide a robust, type-safe, and performant text analysis pipeline that maintains full compatibility with Python Janome while leveraging Rust's advantages in safety and performance.