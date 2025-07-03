# Implementation Plan for Rust IPADIC Dictionary Builder

## Overview
Creating a simplified Rust port of the Python `build.py` script that processes small dictionary datasets directly without the collection phase, using the `fst` crate for finite state transducers.

## Architecture

### Module Structure
```
src/
├── lib.rs              # Public API exports
├── dict_builder/
│   ├── mod.rs          # Module exports and main DictionaryBuilder struct
│   ├── types.rs        # Data structure definitions
│   └── build.rs        # Core building logic
```

### Data Flow
```
MeCab CSV files → Parse entries → Build FST → Process auxiliary files → Serialize to sysdic/
```

## Core Components

### 1. Data Structures (`types.rs`)

**DictEntry**: Represents a single dictionary entry from CSV
- Fields: surface, left_id, right_id, cost, part_of_speech, inflection_type, inflection_form, base_form, reading, phonetic
- Serializable with serde

**CharCategory**: Character category definitions from char.def
- Fields: invoke, group, length

**CodePointRange**: Unicode code point ranges for character categories
- Fields: from, to, category, compat_categories

**UnknownEntry**: Unknown word templates from unk.def
- Fields: left_id, right_id, cost, part_of_speech

### 2. Main Builder (`mod.rs`)

**DictionaryBuilder**: Main interface struct
- Fields: mecab_dir, encoding, output_dir (defaults to "sysdic")
- Methods: `new()`, `build()`

### 3. Core Logic (`build.rs`)

**CSV Processing**:
- `parse_csv_files()`: Read all *.csv files, parse into DictEntry structs
- Handle character encoding with encoding_rs

**FST Building**:
- `build_fst()`: Create surface form → morpheme_id mapping using fst::MapBuilder
- Sort entries by surface form (required for FST)
- Generate sequential morpheme IDs

**Auxiliary File Processing**:
- `parse_matrix_def()`: Parse connection cost matrix from matrix.def
- `parse_char_def()`: Parse character categories and code point ranges
- `parse_unk_def()`: Parse unknown word templates

**Serialization**:
- `save_dictionary()`: Write all components to sysdic/ directory
- Use bincode for efficient binary serialization

## Output Format

### sysdic/ Directory Structure
```
sysdic/
├── dic.fst          # FST mapping surface → morpheme_id (fst crate format)
├── entries.bin      # All dictionary entries (bincode serialized)
├── connections.bin  # Connection cost matrix (bincode serialized)
├── char_defs.bin   # Character definitions (bincode serialized)
└── unknowns.bin    # Unknown word templates (bincode serialized)
```

## Implementation Steps

1. **Dependencies**: Add serde, bincode, csv, anyhow, encoding_rs to Cargo.toml ✅
2. **Module Structure**: Create dict_builder/ directory and module files
3. **Data Structures**: Define all structs in types.rs with serde derives
4. **CSV Parsing**: Implement dictionary entry parsing with proper encoding
5. **FST Building**: Use fst::MapBuilder to create surface → ID mapping
6. **Auxiliary Parsing**: Implement matrix.def, char.def, unk.def parsers
7. **Serialization**: Implement binary serialization to sysdic/ directory
8. **Public API**: Export DictionaryBuilder in lib.rs

## Key Design Decisions

**Simplifications for Small Datasets**:
- No collection phase (direct CSV → FST)
- No entry bucketing (single entries.bin file)
- No parallel processing (sequential for simplicity)
- No memory mapping (regular file I/O with bincode)

**Compatibility**:
- Maintains Janome dictionary format expectations
- Uses "sysdic" as output directory name
- Uses "dic.fst" as FST filename
- Preserves all morphological analysis data

**Error Handling**:
- Use anyhow for error propagation with context
- Handle encoding issues gracefully
- Validate file formats and required fields

This plan creates a focused, maintainable Rust implementation that handles the core dictionary building functionality while being optimized for smaller dataset scenarios.