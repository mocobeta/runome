# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Structure

This is a hybrid Rust/Python project that combines:
- **Rust crate**: Main project in `src/lib.rs` with basic starter code
- **Janome submodule**: Japanese morphological analysis library (Pure Python) in `janome/` directory

The repository contains the Janome library as a git submodule, which is a mature Japanese text processing library for tokenization and morphological analysis.

## Development Commands

### Rust Development
- **Build**: `cargo build`
- **Test**: `cargo test`
- **Run**: `cargo run`

## Architecture Overview

### Rust Component
- Basic library crate with an `add` function and test
- Uses Rust 2024 edition
- No external dependencies currently

### Janome Component (Python)
- **Core modules**:
  - `tokenizer.py`: Main tokenization interface with `Tokenizer` class
  - `analyzer.py`: Analysis framework with preprocessing/postprocessing filters
  - `dic.py`: Dictionary management and loading
  - `fst.py`: Finite State Transducer implementation
  - `lattice.py`: Lattice-based parsing for morphological analysis
  
- **Filter system**:
  - Character filters (`charfilter.py`): Text preprocessing
  - Token filters (`tokenfilter.py`): Post-processing of tokens
  
- **Dictionary**: Uses MeCab-IPADIC format, stored in `ipadic/sysdic.zip`

### Integration Pattern
The project aims to port janome library to Rust and provide the very same API to Janeme.


## Key Files for Development
- `Cargo.toml`: Rust project configuration
- `janome/setup.py`: Python package setup and installation
- `janome/janome/tokenizer.py`: Primary API for text tokenization
- `janome/janome/analyzer.py`: Advanced analysis with filters
- `janome/tests/`: Comprehensive test suite

## Testing Strategy
- Rust: Standard `cargo test` with unit tests in `src/lib.rs`

## Development policy
- Always run code formatter before making a commit.
- Always run clippy linter before making a commit.
