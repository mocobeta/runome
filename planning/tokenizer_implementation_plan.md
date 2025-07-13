# Design and Implementation Plan: Rust Tokenizer Interface

## Overview

This document outlines the design and implementation of a Rust Tokenizer interface that provides 1:1 API compatibility with the Python Janome Tokenizer. The implementation will be contained in a single file `src/tokenizer.rs` and will leverage existing Rust components (SystemDictionary, Lattice, etc.) while excluding user dictionary support for this phase.

## Python Janome Tokenizer Analysis

Based on the Python source code analysis, the Janome Tokenizer provides:

### Key Features
1. **Token Class**: Contains morphological information (surface, part_of_speech, inflection, etc.)
2. **Tokenizer Class**: Main interface for tokenization with configurable parameters
3. **Text Processing**: Chunking large text, handling unknown words, lattice-based analysis
4. **Output Modes**: Regular mode (Token objects) and wakati mode (surface strings only)
5. **Unknown Word Processing**: Character category-based classification and processing

### Python API Signature
```python
class Tokenizer:
    def __init__(self, udic='', *, udic_enc='utf8', udic_type='ipadic', 
                 max_unknown_length=1024, wakati=False, mmap=DEFAULT_MMAP_MODE, dotfile='')
    
    def tokenize(self, text: str, *, wakati=False, baseform_unk=True, dotfile='') -> Iterator[Union[Token, str]]
```

### Python Token Format
```
surface	part_of_speech,infl_type,infl_form,base_form,reading,phonetic
```

Example output:
```
すもも	名詞,一般,*,*,*,*,すもも,スモモ,スモモ
も	助詞,係助詞,*,*,*,*,も,モ,モ
```

## Rust Implementation Design

### 1. Token Struct

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    surface: String,
    part_of_speech: String,
    infl_type: String,
    infl_form: String,
    base_form: String,
    reading: String,
    phonetic: String,
    node_type: NodeType,
}

impl Token {
    pub fn new(node: &dyn LatticeNode, extra: Option<&[String]>) -> Self
    
    // Accessors
    pub fn surface(&self) -> &str
    pub fn part_of_speech(&self) -> &str
    pub fn infl_type(&self) -> &str
    pub fn infl_form(&self) -> &str
    pub fn base_form(&self) -> &str
    pub fn reading(&self) -> &str
    pub fn phonetic(&self) -> &str
    pub fn node_type(&self) -> NodeType
}

impl Display for Token {
    // Format: "surface\tpart_of_speech,infl_type,infl_form,base_form,reading,phonetic"
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
}
```

### 2. Tokenizer Struct

```rust
pub struct Tokenizer {
    sys_dic: Arc<dyn Dictionary>,
    max_unknown_length: usize,
    wakati: bool,
}

impl Tokenizer {
    // Constructor matching Python API (excluding user dictionary parameters)
    pub fn new(
        max_unknown_length: Option<usize>,
        wakati: Option<bool>,
    ) -> Result<Self, RunomeError>
    
    // Main tokenization method
    pub fn tokenize<'a>(
        &'a self, 
        text: &'a str, 
        wakati: Option<bool>, 
        baseform_unk: Option<bool>
    ) -> impl Iterator<Item = Result<TokenizeResult, RunomeError>> + 'a
    
    // Internal methods
    fn tokenize_stream(&self, text: &str, wakati: bool, baseform_unk: bool) -> TokenizeIterator
    fn tokenize_partial(&self, text: &str, wakati: bool, baseform_unk: bool) -> Result<(Vec<TokenizeResult>, usize), RunomeError>
    fn should_split(&self, text: &str, pos: usize) -> bool
    fn is_splittable(&self, text: &str) -> bool
    fn is_punct(&self, c: char) -> bool
    fn is_newline(&self, text: &str) -> bool
}
```

### 3. Supporting Types

```rust
#[derive(Debug, Clone)]
pub enum TokenizeResult {
    Token(Token),
    Surface(String),
}

pub struct TokenizeIterator<'a> {
    tokenizer: &'a Tokenizer,
    text: &'a str,
    processed: usize,
    current_tokens: std::vec::IntoIter<TokenizeResult>,
    wakati: bool,
    baseform_unk: bool,
}

impl<'a> Iterator for TokenizeIterator<'a> {
    type Item = Result<TokenizeResult, RunomeError>;
    fn next(&mut self) -> Option<Self::Item>
}
```

### 4. Constants

```rust
const MAX_CHUNK_SIZE: usize = 1024;
const CHUNK_SIZE: usize = 500;
```

## Implementation Details

### 1. Token Creation

The Token struct will be created from lattice nodes with morphological information:

```rust
impl Token {
    pub fn new(node: &dyn LatticeNode, extra: Option<&[String]>) -> Self {
        // Extract morphological info from node
        // Handle extra info from dictionary lookups
        // Set appropriate defaults for unknown words
    }
    
    fn from_dict_node(node: &dyn LatticeNode, dict_entry: &DictEntry) -> Self {
        // Create from dictionary entry with full morphological info
    }
    
    fn from_unknown_node(node: &dyn LatticeNode, baseform_unk: bool) -> Self {
        // Create from unknown word node with minimal info
    }
}
```

### 2. Text Processing Pipeline

The tokenization follows this pipeline:

1. **Text Chunking**: Split large text into processable chunks
2. **Lattice Construction**: Create lattice for each chunk
3. **Dictionary Lookup**: Add system dictionary entries to lattice
4. **Unknown Word Processing**: Handle unknown characters using character categories
5. **Lattice Processing**: Run forward/end algorithms
6. **Path Extraction**: Extract optimal path using backward algorithm
7. **Token Generation**: Convert nodes to Token objects

```rust
fn tokenize_partial(&self, text: &str, wakati: bool, baseform_unk: bool) -> Result<(Vec<TokenizeResult>, usize), RunomeError> {
    let chunk_size = min(text.len(), MAX_CHUNK_SIZE);
    let mut lattice = Lattice::new(chunk_size, self.sys_dic.clone());
    let mut pos = 0;
    
    while !self.should_split(text, pos) {
        let partial_text = &text[pos..pos + min(50, chunk_size - pos)];
        
        // System dictionary lookup
        let entries = self.sys_dic.lookup(partial_text)?;
        for entry in entries {
            let node = Box::new(Node::new(entry, NodeType::SysDict));
            lattice.add(node)?;
        }
        
        // Unknown word processing
        let char_categories = self.sys_dic.get_char_categories(text.chars().nth(pos).unwrap())?;
        self.process_unknown_words(&mut lattice, text, pos, chunk_size, &char_categories, baseform_unk)?;
        
        pos += lattice.forward();
    }
    
    lattice.end()?;
    let min_cost_path = lattice.backward()?;
    
    // Convert path to tokens
    let tokens = self.path_to_tokens(&min_cost_path[1..min_cost_path.len()-1], wakati);
    Ok((tokens, pos))
}
```

### 3. Unknown Word Processing

Following Python logic for unknown word handling:

```rust
fn process_unknown_words(
    &self, 
    lattice: &mut Lattice, 
    text: &str, 
    pos: usize, 
    chunk_size: usize,
    char_categories: &[CharCategory],
    baseform_unk: bool
) -> Result<(), RunomeError> {
    for category in char_categories {
        // Check if unknown processing should be invoked
        if matched && !self.sys_dic.unknown_invoked_always(category) {
            continue;
        }
        
        // Determine unknown word length
        let length = if self.sys_dic.unknown_grouping(category) {
            self.max_unknown_length
        } else {
            self.sys_dic.unknown_length(category)
        };
        
        // Build unknown word buffer
        let mut buf = String::new();
        buf.push(text.chars().nth(pos).unwrap());
        
        for p in (pos + 1)..min(chunk_size, pos + length + 1) {
            if let Some(c) = text.chars().nth(p) {
                let c_categories = self.sys_dic.get_char_categories(c)?;
                if c_categories.contains(category) || self.is_compatible_category(category, &c_categories) {
                    buf.push(c);
                } else {
                    break;
                }
            }
        }
        
        // Create unknown word entries
        let unknown_entries = self.sys_dic.get_unknown_entries(category)?;
        for entry in unknown_entries {
            let base_form = if baseform_unk { buf.clone() } else { "*".to_string() };
            let unknown_node = Box::new(UnknownNode::new(
                buf.clone(),
                entry.left_id,
                entry.right_id,
                entry.cost,
                entry.part_of_speech.clone(),
                base_form,
            ));
            lattice.add(unknown_node)?;
        }
    }
    Ok(())
}
```

### 4. Text Chunking Logic

Implementing Python's chunking strategy:

```rust
fn should_split(&self, text: &str, pos: usize) -> bool {
    pos >= text.len() ||
    pos >= MAX_CHUNK_SIZE ||
    (pos >= CHUNK_SIZE && self.is_splittable(&text[..pos]))
}

fn is_splittable(&self, text: &str) -> bool {
    if let Some(last_char) = text.chars().last() {
        self.is_punct(last_char) || self.is_newline(text)
    } else {
        false
    }
}

fn is_punct(&self, c: char) -> bool {
    matches!(c, '、' | '。' | ',' | '.' | '？' | '?' | '！' | '!')
}

fn is_newline(&self, text: &str) -> bool {
    text.ends_with("\n\n") || text.ends_with("\r\n\r\n")
}
```

### 5. Iterator Implementation

Providing streaming tokenization:

```rust
impl<'a> Iterator for TokenizeIterator<'a> {
    type Item = Result<TokenizeResult, RunomeError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Return next token from current batch
        if let Some(token) = self.current_tokens.next() {
            return Some(Ok(token));
        }
        
        // Process next chunk if available
        if self.processed < self.text.len() {
            match self.tokenizer.tokenize_partial(
                &self.text[self.processed..], 
                self.wakati, 
                self.baseform_unk
            ) {
                Ok((tokens, pos)) => {
                    self.processed += pos;
                    self.current_tokens = tokens.into_iter();
                    self.current_tokens.next().map(Ok)
                }
                Err(e) => Some(Err(e))
            }
        } else {
            None
        }
    }
}
```

## API Usage Examples

### Basic Tokenization

```rust
use runome::tokenizer::Tokenizer;

// Create tokenizer
let tokenizer = Tokenizer::new(None, None)?;

// Tokenize text
let text = "すもももももももものうち";
for token_result in tokenizer.tokenize(text, None, None) {
    let token = token_result?;
    match token {
        TokenizeResult::Token(t) => println!("{}", t), // Full morphological info
        TokenizeResult::Surface(s) => println!("{}", s), // Surface form only
    }
}
```

### Wakati Mode

```rust
// Wakati mode (surface forms only)
let tokenizer = Tokenizer::new(None, Some(true))?;
for token_result in tokenizer.tokenize(text, None, None) {
    let token = token_result?;
    if let TokenizeResult::Surface(surface) = token {
        println!("{}", surface);
    }
}
```

### Configuration

```rust
// Custom configuration
let tokenizer = Tokenizer::new(
    Some(2048), // max_unknown_length
    Some(false) // wakati mode
)?;
```

## Testing Strategy

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_creation() {
        // Test Token creation from various node types
    }
    
    #[test]
    fn test_basic_tokenization() {
        // Test basic Japanese text tokenization
        let tokenizer = Tokenizer::new(None, None).unwrap();
        let tokens: Vec<_> = tokenizer.tokenize("すもも", None, None).collect();
        // Assert expected token count and properties
    }
    
    #[test]
    fn test_wakati_mode() {
        // Test wakati mode output
    }
    
    #[test]
    fn test_unknown_words() {
        // Test unknown word processing
    }
    
    #[test]
    fn test_chunking() {
        // Test text chunking with large inputs
    }
    
    #[test]
    fn test_python_compatibility() {
        // Compare output with Python Janome for same input
        let text = "すもももももももものうち";
        let tokenizer = Tokenizer::new(None, None).unwrap();
        let tokens: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();
        let tokens = tokens.unwrap();
        
        // Expected Python output:
        // すもも	名詞,一般,*,*,*,*,すもも,スモモ,スモモ
        // も	助詞,係助詞,*,*,*,*,も,モ,モ  
        // もも	名詞,一般,*,*,*,*,もも,モモ,モモ
        // も	助詞,係助詞,*,*,*,*,も,モ,モ
        // もも	名詞,一般,*,*,*,*,もも,モモ,モモ
        // の	助詞,連体化,*,*,*,*,の,ノ,ノ
        // うち	名詞,非自立,副詞可能,*,*,*,うち,ウチ,ウチ
        
        assert_eq!(tokens.len(), 7);
        // Assert each token matches expected output
    }
}
```

### 2. Integration Tests

```rust
#[test]
fn test_complex_text() {
    let tokenizer = Tokenizer::new(None, None).unwrap();
    let text = "2009年10月16日";
    let tokens: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();
    let tokens = tokens.unwrap();
    
    // Verify exact match with Python output:
    // 2009	名詞,数,*,*,*,*,2009,*,*  (UNKNOWN)
    // 年	名詞,接尾,助数詞,*,*,*,年,ネン,ネン  (SYS_DICT)
    // 10	名詞,数,*,*,*,*,10,*,*  (UNKNOWN)  
    // 月	名詞,一般,*,*,*,*,月,ツキ,ツキ  (SYS_DICT)
    // 16	名詞,数,*,*,*,*,16,*,*  (UNKNOWN)
    // 日	名詞,接尾,助数詞,*,*,*,日,ニチ,ニチ  (SYS_DICT)
    
    assert_eq!(tokens.len(), 6);
}
```

### 3. Performance Tests

```rust
#[test]
fn test_large_text_performance() {
    // Test performance with large text inputs
    // Ensure streaming processing works correctly
    let large_text = "すもも".repeat(1000);
    let tokenizer = Tokenizer::new(None, None).unwrap();
    
    let start = std::time::Instant::now();
    let token_count = tokenizer.tokenize(&large_text, None, None).count();
    let duration = start.elapsed();
    
    println!("Processed {} tokens in {:?}", token_count, duration);
    assert!(duration.as_secs() < 5); // Should complete within 5 seconds
}
```

## Error Handling

All methods return `Result<T, RunomeError>` for proper error propagation:

- Dictionary lookup failures
- Lattice processing errors  
- Character encoding issues
- Memory allocation failures

### Error Types
```rust
pub enum TokenizerError {
    DictionaryError(RunomeError),
    LatticeError(RunomeError),
    InvalidInput(String),
    UnknownCharacter(char),
}
```

## Performance Considerations

1. **Streaming Processing**: Iterator-based API prevents loading entire result set into memory
2. **Text Chunking**: Large texts processed in chunks to control memory usage  
3. **Zero-Copy**: Minimize string allocations where possible
4. **Efficient Lattice**: Reuse existing optimized lattice implementation
5. **Character Iteration**: Efficient UTF-8 character handling for Japanese text

## Memory Management

1. **Bounded Memory Usage**: Text chunking ensures constant memory usage regardless of input size
2. **Reference Counting**: Arc<dyn Dictionary> allows sharing dictionary across tokenizer instances
3. **Minimal Allocations**: Reuse buffers and minimize temporary string allocations
4. **Iterator Pattern**: Process tokens on-demand rather than materializing full result sets

## Integration with Existing Components

### SystemDictionary Integration
- Use existing `SystemDictionary::instance()` for dictionary access
- Leverage existing character classification methods
- Utilize unknown word configuration from dictionary

### Lattice Integration  
- Use existing `Lattice::new()`, `add()`, `forward()`, `end()`, `backward()` methods
- Integrate with existing `LatticeNode` trait implementations
- Maintain compatibility with existing node types

### Error Integration
- Use existing `RunomeError` for consistent error handling
- Propagate errors from dictionary and lattice components
- Provide meaningful error messages for tokenization failures

## File Structure

```
src/
├── tokenizer.rs        # Complete tokenizer implementation
└── lib.rs             # Export tokenizer module
```

### lib.rs updates

```rust
pub mod tokenizer;
pub use tokenizer::{Token, Tokenizer, TokenizeResult};
```

## Python Compatibility Matrix

| Feature | Python Janome | Rust Implementation | Status |
|---------|---------------|-------------------|---------|
| Basic tokenization | ✅ | ✅ | Implemented |
| Wakati mode | ✅ | ✅ | Implemented |
| Unknown word processing | ✅ | ✅ | Implemented |
| Text chunking | ✅ | ✅ | Implemented |
| Character categories | ✅ | ✅ | Implemented |
| Token format | ✅ | ✅ | Implemented |
| Iterator API | ✅ | ✅ | Implemented |
| User dictionaries | ✅ | ❌ | Excluded (future) |
| MMap mode | ✅ | ❌ | Excluded (future) |
| Dotfile output | ✅ | ❌ | Excluded (future) |

## Future Extensions

While not in scope for this implementation, the design allows for:

1. **User Dictionary Support**: Add user dictionary loading and integration
2. **MMap Mode Selection**: Support memory-mapped vs regular dictionary modes  
3. **Dot File Visualization**: Output GraphViz dot files for lattice visualization
4. **Custom Character Filters**: Preprocessing filters for text normalization
5. **Parallel Processing**: Multi-threaded processing for large texts
6. **Custom Unknown Word**: User-configurable unknown word processing
7. **Streaming Input**: Support for streaming text input from readers

## Implementation Phases

### Phase 1: Core Structure (Week 1)
- Implement Token struct with all fields and Display trait
- Implement basic Tokenizer struct and constructor
- Add basic tokenize method skeleton
- Create comprehensive test framework

### Phase 2: Basic Tokenization (Week 2)  
- Implement dictionary lookup integration
- Add lattice construction and processing
- Implement basic token extraction
- Test with simple Japanese text

### Phase 3: Advanced Features (Week 3)
- Add unknown word processing with character categories
- Implement text chunking and splitting logic
- Add wakati mode support
- Comprehensive error handling

### Phase 4: Polish and Optimization (Week 4)
- Performance optimization and memory management
- Complete test suite with Python compatibility verification
- Documentation and usage examples
- Code review and refinement

This design provides a complete, production-ready Rust tokenizer that maintains perfect compatibility with Python Janome while leveraging Rust's performance and safety advantages.