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

## Planning documentat
When you are asked to create a design or implementation plan doc, save your plan to a Markdown file in /planning folder with appropriate file name.

## Development policy
When you make changes, follow the instructions below.
- Run cargo build --all-targets to check if the entire codebase is successfully compiled.
- Run cargo fmt to make sure all code is formatted.
- Run clippy linter to check if there are no errors and warnings. If there are errors or warnings, try to fix them.
- Make sure "cargo test" passes before making a git commit.
- Follow design docs in 'planning' folder.
- Proceed with the smallest possible steps.
- Never use println!() except for debugging purposes.

## Development steps
1. Review design and implementation plan docs in /planning folder. If there is no planning doc for the changes you are going to make, make a proposal to create it.
2. Your code should be consistent with Janome Python code. Examine the corresponding Janome Python source and test code when you add new modules, structs, methods, test cases, and so on.
3. When you add new methods,
   1. Create empty methods. An empty method should have the correct method signatures but no actual code.
   2. Prepare test cases to test the methods you are going to make. Ask the user to review the test cases.
   3. Implement the methods so that all test cases pass.
4. When you make changes to existing methods,
   1. Prepare test cases to test the changes you are going to make. Ask the user to review the test cases.
   2. Implement the changes so that all test cases pass.
5. When you make any changes, always review the existing code and consider utilizing them effectively.