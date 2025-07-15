# Python Bindings Implementation Plan for Runome with PyO3 and Maturin

## Overview
Create Python bindings for the Runome Tokenizer using PyO3 and maturin, with Python code located in the `/runome` directory to match the requested structure.

## Project Structure
Based on maturin best practices, we'll use a mixed Python-Rust project structure:

```
runome/
├── Cargo.toml (updated with PyO3 and cdylib)
├── pyproject.toml (configured for maturin)
├── runome/              # Python package directory
│   ├── __init__.py      # Python module initialization
│   ├── tokenizer.py     # Pure Python wrapper/extensions
│   └── py.typed         # Type information marker
├── runome.pyi           # Type stubs for Rust bindings
├── src/
│   ├── lib.rs           # Updated to expose Python module
│   ├── python_bindings.rs # PyO3 bindings implementation
│   └── (existing Rust modules)
└── tests/
    └── test_python_api.py
```

## Implementation Steps

### 1. Configure Project Structure
- Update `Cargo.toml` to include PyO3 dependencies and configure `crate-type = ["cdylib", "rlib"]`
- Configure `pyproject.toml` to use maturin as build backend
- Create `/runome` directory for Python package

### 2. Implement PyO3 Bindings
- **Token class**: Create `#[pyclass]` wrapper for Rust `Token` struct
  - Properties: `surface`, `part_of_speech`, `infl_type`, `infl_form`, `base_form`, `reading`, `phonetic`, `node_type`
  - `__str__` method matching Janome format: `surface\tpart_of_speech,infl_type,infl_form,base_form,reading,phonetic`
  - `__repr__` method for debugging

- **Tokenizer class**: Create `#[pyclass]` wrapper for Rust `Tokenizer` struct
  - Constructor: `__init__(udic='', max_unknown_length=1024, wakati=False)`
  - Method: `tokenize(text, wakati=False, baseform_unk=True)` returning Python iterator
  - Support for user dictionary loading

- **Iterator Implementation**: Create `#[pyclass]` for `TextChunkIterator`
  - `__iter__` and `__next__` methods
  - Handle `StopIteration` exception properly

### 3. Error Handling
- Implement `From<RunomeError> for PyErr` conversion
- Map specific Rust errors to appropriate Python exceptions
- Ensure proper error propagation from Rust to Python

### 4. Python Module Structure
- Create `runome/__init__.py` to expose the Rust module:
  ```python
  from .runome import Token, Tokenizer
  __all__ = ['Token', 'Tokenizer']
  ```
- Add optional pure Python extensions in `runome/tokenizer.py`

### 5. Type Information
- Create `runome.pyi` stub file with complete type information
- Add `py.typed` marker file to indicate typing support
- Include proper type annotations for all public APIs

### 6. Testing and Validation
- Create comprehensive Python test suite
- Test API compatibility with original Janome
- Validate error handling and edge cases
- Performance comparison tests

## Key Implementation Details

### Cargo.toml Configuration
```toml
[lib]
name = "runome"
crate-type = ["cdylib", "rlib"]

[dependencies]
pyo3 = { version = "0.23", features = ["abi3-py38"] }
```

### Python Module Definition
```rust
use pyo3::prelude::*;

#[pymodule]
fn runome(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyToken>()?;
    m.add_class::<PyTokenizer>()?;
    m.add_class::<PyTextChunkIterator>()?;
    Ok(())
}
```

### API Compatibility
- Maintain 100% API compatibility with Janome's tokenizer
- Support both wakati mode and full morphological analysis
- Handle user dictionaries in the same way as Janome

## Benefits
- Provides Python API identical to Janome for easy migration
- Leverages Rust's performance for tokenization
- Maintains Python ecosystem integration
- Supports proper type checking and IDE integration
- Follows maturin best practices for mixed projects

## Research Summary

### PyO3 and Maturin Survey Results

#### PyO3 Key Features:
- **#[pyclass]** attribute for exposing Rust structs as Python classes
- **#[pymethods]** for defining Python methods on Rust structs
- **PyResult<T>** for error handling (alias for `Result<T, PyErr>`)
- Automatic type conversion between Rust and Python types
- Support for Python iterators with `__iter__` and `__next__`

#### Maturin Project Structure Options:
1. **Standard Mixed Layout**: Python package directory at project root
2. **Configurable Python Source**: Use `[tool.maturin] python-source = "runome"`
3. **Src Layout**: Python in `src/` directory for better organization

#### Error Handling Patterns:
- Implement `From<RunomeError> for PyErr` for automatic conversion
- Use `pyo3::exceptions` module for standard Python exceptions
- Support for custom exception types with `create_exception!` macro

#### Type Hints and Stubs:
- Manual `.pyi` file creation (automatic generation still in development)
- `py.typed` marker file for typing support
- `pyo3-stub-gen` crate for semi-automatic stub generation

#### Best Practices:
- Use `abi3-py38` feature for stable Python ABI
- Configure `crate-type = ["cdylib", "rlib"]` for library distribution
- Follow Python naming conventions in PyO3 bindings
- Maintain API compatibility with existing Python libraries

This plan provides a comprehensive approach to creating Python bindings that will make the Rust Runome tokenizer accessible to Python developers while maintaining performance and compatibility with the existing Janome ecosystem.