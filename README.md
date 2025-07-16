# Runome

A high-performance Japanese morphological analyzer compatible with [Janome](https://github.com/mocobeta/janome), implemented in Rust with Python bindings.

## Overview

Runome is a Rust-based Japanese text processing library that provides the same API as the popular Janome library but with improved performance. It offers tokenization, morphological analysis, and text processing capabilities for Japanese text.

## Features

- **Janome-compatible API**: Drop-in replacement for Janome with the same interface
- **High Performance**: Implemented in Rust for speed and memory efficiency
- **Comprehensive Analysis**: Supports tokenization, part-of-speech tagging, and morphological analysis
- **Flexible Filtering**: Character filters and token filters for text preprocessing and postprocessing
- **User Dictionary Support**: Custom dictionaries in both IPADIC and simplified formats
- **Cross-platform**: Works on Linux, macOS, and Windows

## Installation

Install from PyPI:

```bash
pip install runome
```

## Quick Start

### Basic Tokenization

```python
from runome.tokenizer import Tokenizer

tokenizer = Tokenizer()
for token in tokenizer.tokenize('すもももももももものうち'):
    print(token)
```

### Analyzer

Three-stage analysis pipeline:

```python
from runome.analyzer import Analyzer
from runome.charfilter import RegexReplaceCharFilter
from runome.tokenfilter import POSKeepFilter

analyzer = Analyzer(
    char_filters=[RegexReplaceCharFilter(r'\d+', '0')],
    token_filters=[POSKeepFilter(['
^'])]
)
```

### Character Filters

- `UnicodeNormalizeCharFilter`: Unicode normalization
- `RegexReplaceCharFilter`: Regular expression replacement

### Token Filters

- `LowerCaseFilter`: Convert to lowercase
- `UpperCaseFilter`: Convert to uppercase
- `POSKeepFilter`: Keep only specified parts of speech
- `POSStopFilter`: Remove specified parts of speech
- `CompoundNounFilter`: Combine consecutive nouns
- `TokenCountFilter`: Count token frequencies
- `ExtractAttributeFilter`: Extract specific token attributes

## References

Runome's API is compatible with Janome. For more details on Janome's API, refer to the Janome documentation

English: https://janome.mocobeta.dev/en/
Japanese: https://janome.mocobeta.dev/ja/

## Examples

See the `examples/` directory for comprehensive usage examples:

- `usage.py`: Basic tokenization examples
- `usage_analyzer.py`: Advanced analysis with filters
- User dictionary examples with sample CSV files

## Development

### Requirements

- Rust 1.70+
- Python 3.8+
- Maturin for building Python wheels

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-repo/runome.git
cd runome

# Install development dependencies
uv sync

# Build the wheel
uv run maturin build --release --features python

# Install the wheel
pip install target/wheels/runome-*.whl
```

### Running Tests

```bash
# Rust tests
cargo test

# Python tests (after building with python feature)
uv run python -m pytest tests/
```

## Copyright notice

The entire codebase is (almost) written by Claude Code, prompted by [@mocobeta](https://github.com/mocobeta), the original author of Janome.
