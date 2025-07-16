# Python Bindings Implementation Plan for Analyzer Framework

## Overview
Extend the existing Python bindings to provide complete support for CharFilter, TokenFilter, and Analyzer classes, enabling Python users to access the full text analysis pipeline with compatibility to Python Janome's API.

## Background Analysis

### Current State
1. **Existing Python Bindings** (`src/python_bindings.rs`):
   - `PyToken`: Complete Token wrapper with all properties
   - `PyTokenizer`: Full Tokenizer implementation with user dictionary support
   - `PyTokenIterator`: Iterator for tokenization results
   - Error conversion from `RunomeError` to `PyErr`

2. **Rust Implementation Available**:
   - **CharFilter trait**: String-to-string transformations
     - `RegexReplaceCharFilter`: Pattern replacement
     - `UnicodeNormalizeCharFilter`: Unicode normalization (NFC, NFKC, NFD, NFKD)
   - **TokenFilter trait**: Token stream transformations
     - `LowerCaseFilter`, `UpperCaseFilter`: Case conversion
     - `POSStopFilter`, `POSKeepFilter`: POS-based filtering
     - `CompoundNounFilter`: Compound noun formation
     - `ExtractAttributeFilter`: Attribute extraction (terminal)
     - `TokenCountFilter`: Token counting (terminal)
   - **Analyzer**: Three-stage pipeline with builder pattern

3. **Python Janome API** (target compatibility):
   ```python
   # CharFilter usage
   char_filter = UnicodeNormalizeCharFilter()
   result = char_filter("Ｐｙｔｈｏｎ")  # "Python"
   
   # TokenFilter usage
   token_filter = LowerCaseFilter()
   tokens = token_filter(token_iterator)
   
   # Analyzer usage
   analyzer = Analyzer(
       char_filters=[UnicodeNormalizeCharFilter(), RegexReplaceCharFilter('蛇の目', 'janome')],
       tokenizer=Tokenizer(),
       token_filters=[CompoundNounFilter(), LowerCaseFilter()]
   )
   for token in analyzer.analyze(text):
       print(token)
   ```

### Key Challenges

1. **Dynamic Filter Composition**: Python allows arbitrary filter combinations at runtime
2. **Iterator Protocol**: Python expects iterators with `__iter__` and `__next__`
3. **Different Output Types**: TokenFilters can output Token, String, or (String, usize) tuples
4. **Callable Protocol**: Filters support both `apply()` method and `__call__` protocol
5. **Builder Pattern vs Constructor**: Python uses constructor while Rust uses builder

## Implementation Plan

### Phase 1: CharFilter Python Bindings

#### 1.1 Base CharFilter Protocol
```rust
/// Python CharFilter protocol (abstract base)
#[pyclass(name = "CharFilter", subclass)]
pub struct PyCharFilter;

#[pymethods]
impl PyCharFilter {
    fn apply(&self, text: &str) -> PyResult<String> {
        Err(PyException::new_err("CharFilter.apply() must be implemented"))
    }
    
    fn __call__(&self, text: &str) -> PyResult<String> {
        self.apply(text)
    }
}
```

#### 1.2 RegexReplaceCharFilter
```rust
#[pyclass(name = "RegexReplaceCharFilter", extends = PyCharFilter)]
pub struct PyRegexReplaceCharFilter {
    inner: RegexReplaceCharFilter,
}

#[pymethods]
impl PyRegexReplaceCharFilter {
    #[new]
    fn new(pattern: &str, replacement: &str) -> PyResult<(Self, PyCharFilter)> {
        let inner = RegexReplaceCharFilter::new(pattern, replacement)
            .map_err(|e| PyException::new_err(format!("Invalid regex pattern: {:?}", e)))?;
        Ok((PyRegexReplaceCharFilter { inner }, PyCharFilter))
    }
    
    fn apply(&self, text: &str) -> PyResult<String> {
        self.inner.apply(text)
            .map_err(|e| PyException::new_err(format!("CharFilter error: {:?}", e)))
    }
    
    fn __call__(&self, text: &str) -> PyResult<String> {
        self.apply(text)
    }
}
```

#### 1.3 UnicodeNormalizeCharFilter
```rust
#[pyclass(name = "UnicodeNormalizeCharFilter", extends = PyCharFilter)]
pub struct PyUnicodeNormalizeCharFilter {
    inner: UnicodeNormalizeCharFilter,
}

#[pymethods]
impl PyUnicodeNormalizeCharFilter {
    #[new]
    #[pyo3(signature = (form = "NFKC"))]
    fn new(form: &str) -> PyResult<(Self, PyCharFilter)> {
        let inner = UnicodeNormalizeCharFilter::new(form)
            .map_err(|e| PyException::new_err(format!("Invalid normalization form: {:?}", e)))?;
        Ok((PyUnicodeNormalizeCharFilter { inner }, PyCharFilter))
    }
    
    fn apply(&self, text: &str) -> PyResult<String> {
        self.inner.apply(text)
            .map_err(|e| PyException::new_err(format!("CharFilter error: {:?}", e)))
    }
    
    fn __call__(&self, text: &str) -> PyResult<String> {
        self.apply(text)
    }
}
```

### Phase 2: TokenFilter Python Bindings

#### 2.1 Base TokenFilter Protocol with Dynamic Output
```rust
/// Enum to handle different TokenFilter output types
#[derive(Clone)]
pub enum PyTokenFilterOutput {
    Token(PyToken),
    String(String),
    Tuple(String, usize),
}

impl IntoPy<PyObject> for PyTokenFilterOutput {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            PyTokenFilterOutput::Token(token) => token.into_py(py),
            PyTokenFilterOutput::String(s) => s.into_py(py),
            PyTokenFilterOutput::Tuple(s, count) => (s, count).into_py(py),
        }
    }
}

/// Python TokenFilter protocol
#[pyclass(name = "TokenFilter", subclass)]
pub struct PyTokenFilter;

#[pymethods]
impl PyTokenFilter {
    fn apply(&self, tokens: &Bound<'_, PyIterator>) -> PyResult<PyObject> {
        Err(PyException::new_err("TokenFilter.apply() must be implemented"))
    }
    
    fn __call__(&self, tokens: &Bound<'_, PyIterator>) -> PyResult<PyObject> {
        self.apply(tokens)
    }
}
```

#### 2.2 Token-to-Token Filters
```rust
/// Generic iterator for token filters
#[pyclass(name = "TokenFilterIterator")]
pub struct PyTokenFilterIterator {
    results: Vec<PyTokenFilterOutput>,
    index: usize,
}

#[pymethods]
impl PyTokenFilterIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }
    
    fn __next__(&mut self) -> PyResult<Option<PyObject>> {
        if self.index >= self.results.len() {
            return Ok(None);
        }
        
        let result = self.results[self.index].clone();
        self.index += 1;
        
        Python::with_gil(|py| Ok(Some(result.into_py(py))))
    }
}

/// Example: LowerCaseFilter
#[pyclass(name = "LowerCaseFilter", extends = PyTokenFilter)]
pub struct PyLowerCaseFilter {
    inner: LowerCaseFilter,
}

#[pymethods]
impl PyLowerCaseFilter {
    #[new]
    fn new() -> (Self, PyTokenFilter) {
        (PyLowerCaseFilter { inner: LowerCaseFilter }, PyTokenFilter)
    }
    
    fn apply(&self, tokens: &Bound<'_, PyIterator>) -> PyResult<PyTokenFilterIterator> {
        // Convert Python iterator to Rust tokens
        let rust_tokens = extract_tokens_from_iterator(tokens)?;
        
        // Apply filter
        let filtered: Vec<Token> = self.inner.apply(rust_tokens.into_iter()).collect();
        
        // Convert back to Python
        let results: Vec<PyTokenFilterOutput> = filtered
            .into_iter()
            .map(|t| PyTokenFilterOutput::Token(PyToken::from_rust_token(t)))
            .collect();
            
        Ok(PyTokenFilterIterator { results, index: 0 })
    }
}
```

#### 2.3 POS-based Filters
```rust
#[pyclass(name = "POSStopFilter", extends = PyTokenFilter)]
pub struct PyPOSStopFilter {
    inner: POSStopFilter,
}

#[pymethods]
impl PyPOSStopFilter {
    #[new]
    fn new(pos_list: Vec<String>) -> (Self, PyTokenFilter) {
        (PyPOSStopFilter { inner: POSStopFilter::new(pos_list) }, PyTokenFilter)
    }
    
    // Similar apply implementation...
}

#[pyclass(name = "POSKeepFilter", extends = PyTokenFilter)]
pub struct PyPOSKeepFilter {
    inner: POSKeepFilter,
}

// Similar implementation...
```

#### 2.4 Terminal Filters
```rust
#[pyclass(name = "ExtractAttributeFilter", extends = PyTokenFilter)]
pub struct PyExtractAttributeFilter {
    inner: ExtractAttributeFilter,
}

#[pymethods]
impl PyExtractAttributeFilter {
    #[new]
    #[pyo3(signature = (attr = "surface"))]
    fn new(attr: &str) -> PyResult<(Self, PyTokenFilter)> {
        let inner = ExtractAttributeFilter::new(attr.to_string())
            .map_err(|e| PyException::new_err(format!("Invalid attribute: {:?}", e)))?;
        Ok((PyExtractAttributeFilter { inner }, PyTokenFilter))
    }
    
    fn apply(&self, tokens: &Bound<'_, PyIterator>) -> PyResult<PyTokenFilterIterator> {
        let rust_tokens = extract_tokens_from_iterator(tokens)?;
        let strings: Vec<String> = self.inner.apply(rust_tokens.into_iter()).collect();
        
        let results: Vec<PyTokenFilterOutput> = strings
            .into_iter()
            .map(PyTokenFilterOutput::String)
            .collect();
            
        Ok(PyTokenFilterIterator { results, index: 0 })
    }
}

#[pyclass(name = "TokenCountFilter", extends = PyTokenFilter)]
pub struct PyTokenCountFilter {
    inner: TokenCountFilter,
}

#[pymethods]
impl PyTokenCountFilter {
    #[new]
    #[pyo3(signature = (attr = "surface", sorted = false))]
    fn new(attr: &str, sorted: bool) -> PyResult<(Self, PyTokenFilter)> {
        let inner = TokenCountFilter::new(attr.to_string(), sorted)
            .map_err(|e| PyException::new_err(format!("Invalid attribute: {:?}", e)))?;
        Ok((PyTokenCountFilter { inner }, PyTokenFilter))
    }
    
    fn apply(&self, tokens: &Bound<'_, PyIterator>) -> PyResult<PyTokenFilterIterator> {
        let rust_tokens = extract_tokens_from_iterator(tokens)?;
        let counts: Vec<(String, usize)> = self.inner.apply(rust_tokens.into_iter()).collect();
        
        let results: Vec<PyTokenFilterOutput> = counts
            .into_iter()
            .map(|(s, c)| PyTokenFilterOutput::Tuple(s, c))
            .collect();
            
        Ok(PyTokenFilterIterator { results, index: 0 })
    }
}
```

### Phase 3: Analyzer Python Bindings

#### 3.1 Dynamic Filter Handling
```rust
/// Wrapper to hold Python filter objects
pub enum PyFilterWrapper {
    CharFilter(Py<PyAny>),
    TokenFilter(Py<PyAny>),
}

impl PyFilterWrapper {
    fn apply_char_filter(&self, py: Python, text: &str) -> PyResult<String> {
        match self {
            PyFilterWrapper::CharFilter(filter) => {
                let result = filter.call_method1(py, "__call__", (text,))?;
                result.extract::<String>(py)
            }
            _ => Err(PyException::new_err("Not a CharFilter"))
        }
    }
    
    fn apply_token_filter(&self, py: Python, tokens: PyObject) -> PyResult<PyObject> {
        match self {
            PyFilterWrapper::TokenFilter(filter) => {
                filter.call_method1(py, "__call__", (tokens,))
            }
            _ => Err(PyException::new_err("Not a TokenFilter"))
        }
    }
}
```

#### 3.2 Analyzer Implementation
```rust
#[pyclass(name = "Analyzer")]
pub struct PyAnalyzer {
    char_filters: Vec<PyFilterWrapper>,
    tokenizer: PyTokenizer,
    token_filters: Vec<PyFilterWrapper>,
}

#[pymethods]
impl PyAnalyzer {
    #[new]
    #[pyo3(signature = (*, char_filters = vec![], tokenizer = None, token_filters = vec![]))]
    fn new(
        py: Python,
        char_filters: Vec<Bound<'_, PyAny>>,
        tokenizer: Option<PyTokenizer>,
        token_filters: Vec<Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        // Validate tokenizer
        let tokenizer = if let Some(t) = tokenizer {
            if t.inner.wakati() {
                return Err(PyException::new_err(
                    "Invalid argument: A Tokenizer with wakati=True option is not accepted."
                ));
            }
            t
        } else {
            // Create default tokenizer
            PyTokenizer::new("", "utf8", "ipadic", 1024, false)?
        };
        
        // Wrap filters
        let char_filters: Vec<PyFilterWrapper> = char_filters
            .into_iter()
            .map(|f| PyFilterWrapper::CharFilter(f.unbind()))
            .collect();
            
        let token_filters: Vec<PyFilterWrapper> = token_filters
            .into_iter()
            .map(|f| PyFilterWrapper::TokenFilter(f.unbind()))
            .collect();
        
        Ok(PyAnalyzer {
            char_filters,
            tokenizer,
            token_filters,
        })
    }
    
    fn analyze(&self, py: Python, text: &str) -> PyResult<PyObject> {
        // Stage 1: Apply CharFilters
        let mut processed_text = text.to_string();
        for filter in &self.char_filters {
            processed_text = filter.apply_char_filter(py, &processed_text)?;
        }
        
        // Stage 2: Tokenize
        let tokens = self.tokenizer.tokenize(&processed_text, None, true)?;
        let tokens_obj = tokens.into_py(py);
        
        // Stage 3: Apply TokenFilters
        let mut current_iter = tokens_obj;
        for filter in &self.token_filters {
            current_iter = filter.apply_token_filter(py, current_iter)?;
        }
        
        Ok(current_iter)
    }
}
```

### Phase 4: Helper Functions

#### 4.1 Token Extraction from Python Iterator
```rust
fn extract_tokens_from_iterator(tokens: &Bound<'_, PyIterator>) -> PyResult<Vec<Token>> {
    let mut rust_tokens = Vec::new();
    
    for item in tokens.iter()? {
        let item = item?;
        
        // Try to extract PyToken
        if let Ok(py_token) = item.extract::<PyToken>() {
            rust_tokens.push(py_token.inner.clone());
        } else {
            return Err(PyException::new_err(
                "TokenFilter expects Token objects in the input iterator"
            ));
        }
    }
    
    Ok(rust_tokens)
}
```

#### 4.2 Support for Custom Python Filters
```rust
/// Allow Python-defined filters to work with Rust Analyzer
fn is_char_filter(obj: &Bound<'_, PyAny>) -> bool {
    obj.hasattr("apply").unwrap_or(false) && 
    obj.hasattr("__call__").unwrap_or(false)
}

fn is_token_filter(obj: &Bound<'_, PyAny>) -> bool {
    obj.hasattr("apply").unwrap_or(false) && 
    obj.hasattr("__call__").unwrap_or(false)
}
```

### Phase 5: Module Registration

```rust
/// Update Python module definition
#[pymodule]
fn runome(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Existing classes
    m.add_class::<PyToken>()?;
    m.add_class::<PyTokenizer>()?;
    m.add_class::<PyTokenIterator>()?;
    
    // CharFilter classes
    m.add_class::<PyCharFilter>()?;
    m.add_class::<PyRegexReplaceCharFilter>()?;
    m.add_class::<PyUnicodeNormalizeCharFilter>()?;
    
    // TokenFilter classes
    m.add_class::<PyTokenFilter>()?;
    m.add_class::<PyLowerCaseFilter>()?;
    m.add_class::<PyUpperCaseFilter>()?;
    m.add_class::<PyPOSStopFilter>()?;
    m.add_class::<PyPOSKeepFilter>()?;
    m.add_class::<PyCompoundNounFilter>()?;
    m.add_class::<PyExtractAttributeFilter>()?;
    m.add_class::<PyTokenCountFilter>()?;
    m.add_class::<PyTokenFilterIterator>()?;
    
    // Analyzer
    m.add_class::<PyAnalyzer>()?;
    
    Ok(())
}
```

### Phase 6: Testing Strategy

#### 6.1 Python Test Suite
```python
import runome

def test_char_filters():
    # Test UnicodeNormalizeCharFilter
    filter = runome.UnicodeNormalizeCharFilter()
    assert filter("Ｐｙｔｈｏｎ") == "Python"
    
    # Test RegexReplaceCharFilter
    filter = runome.RegexReplaceCharFilter("蛇の目", "janome")
    assert filter("蛇の目は形態素解析器です。") == "janomeは形態素解析器です。"

def test_token_filters():
    tokenizer = runome.Tokenizer()
    tokens = list(tokenizer.tokenize("テストTEST"))
    
    # Test LowerCaseFilter
    filter = runome.LowerCaseFilter()
    filtered = list(filter(iter(tokens)))
    assert all(t.surface == t.surface.lower() for t in filtered)

def test_analyzer():
    # Test complete pipeline
    analyzer = runome.Analyzer(
        char_filters=[
            runome.UnicodeNormalizeCharFilter(),
            runome.RegexReplaceCharFilter("蛇の目", "janome")
        ],
        token_filters=[
            runome.CompoundNounFilter(),
            runome.LowerCaseFilter()
        ]
    )
    
    results = list(analyzer.analyze("蛇の目はPure Ｐｙｔｈｏｎな形態素解析器です。"))
    assert any(t.surface == "janome" for t in results)
    assert any(t.surface == "pure" for t in results)
    assert any(t.surface == "python" for t in results)

def test_terminal_filters():
    # Test ExtractAttributeFilter
    analyzer = runome.Analyzer(
        token_filters=[
            runome.POSKeepFilter(["名詞"]),
            runome.ExtractAttributeFilter("surface")
        ]
    )
    
    results = list(analyzer.analyze("東京駅で降りる"))
    assert all(isinstance(r, str) for r in results)
    
    # Test TokenCountFilter
    analyzer = runome.Analyzer(
        token_filters=[
            runome.POSKeepFilter(["名詞"]),
            runome.TokenCountFilter("surface", sorted=True)
        ]
    )
    
    results = list(analyzer.analyze("すもももももももものうち"))
    assert all(isinstance(r, tuple) and len(r) == 2 for r in results)
```

#### 6.2 Compatibility Tests
```python
# Verify API compatibility with Python Janome
import janome.analyzer
import janome.charfilter
import janome.tokenfilter
import runome

def test_api_compatibility():
    # Both should have same interface
    janome_analyzer = janome.analyzer.Analyzer()
    runome_analyzer = runome.Analyzer()
    
    assert hasattr(runome_analyzer, 'analyze')
    assert callable(runome_analyzer.analyze)
```

### Phase 7: Performance Optimizations

#### 7.1 Minimize Python/Rust Boundary Crossings
- Cache filter results when possible
- Batch token processing
- Use PyO3's buffer protocol for large text

#### 7.2 Memory Management
- Use `Py<T>` for Python object references
- Implement proper `Drop` traits
- Handle GIL correctly in iterators

### Phase 8: Documentation

#### 8.1 API Documentation
```python
"""
runome.Analyzer
==============

A text analysis pipeline that combines CharFilter, Tokenizer, and TokenFilter.

Example:
    >>> from runome import Analyzer, UnicodeNormalizeCharFilter, LowerCaseFilter
    >>> analyzer = Analyzer(
    ...     char_filters=[UnicodeNormalizeCharFilter()],
    ...     token_filters=[LowerCaseFilter()]
    ... )
    >>> for token in analyzer.analyze("テストTEST"):
    ...     print(token.surface)
"""
```

#### 8.2 Migration Guide
- Document differences from Python Janome
- Provide examples for common use cases
- Include performance comparisons

## Implementation Order

1. **Phase 1**: CharFilter bindings (RegexReplaceCharFilter, UnicodeNormalizeCharFilter)
2. **Phase 2**: Basic TokenFilter bindings (LowerCaseFilter, UpperCaseFilter)
3. **Phase 3**: Core Analyzer implementation
4. **Phase 4**: Remaining TokenFilters (POS filters, CompoundNounFilter)
5. **Phase 5**: Terminal filters (ExtractAttributeFilter, TokenCountFilter)
6. **Phase 6**: Testing and debugging
7. **Phase 7**: Performance optimization
8. **Phase 8**: Documentation and examples

## Success Criteria

1. **API Compatibility**: Drop-in replacement for Python Janome's Analyzer
2. **Performance**: Faster than pure Python implementation
3. **Correctness**: Pass all compatibility tests
4. **Usability**: Clear error messages and documentation
5. **Extensibility**: Support for custom Python filters

## Risks and Mitigations

1. **Risk**: Complex iterator conversions between Python and Rust
   - **Mitigation**: Use PyO3's iterator support and proper error handling

2. **Risk**: Different output types from TokenFilters
   - **Mitigation**: Use enum wrapper with proper type conversion

3. **Risk**: GIL contention in filter chains
   - **Mitigation**: Release GIL when safe, batch operations

4. **Risk**: Memory leaks from Python object references
   - **Mitigation**: Use RAII patterns and proper PyO3 reference handling

This implementation will provide Python users with a high-performance, fully-compatible alternative to Janome's Analyzer framework while maintaining the flexibility to use custom Python filters alongside Rust implementations.