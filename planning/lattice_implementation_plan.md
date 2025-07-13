# Lattice with Viterbi Algorithm - Design and Implementation Plan

## Overview

This document outlines the design and implementation plan for creating a Rust version of Janome's lattice-based morphological analysis with Viterbi algorithm. The lattice structure enables finding the optimal (minimum cost) path through candidate morphemes using dynamic programming.

## Analysis of Janome's Python Implementation

### Core Components in lattice.py

1. **Node Types**:
   - `NodeType`: Enum with SYS_DICT, USER_DICT, UNKNOWN
   - `Node`: References DictEntry to avoid copying morphological data
   - `UnknownNode`: Owns data for dynamically constructed unknown words
   - `BOS`/`EOS`: Beginning/End of sentence markers

2. **Lattice Structure**:
   - `snodes`: 2D array of start nodes at each position
   - `enodes`: 2D array of end nodes at each position
   - `p`: Current position pointer
   - Dictionary reference for connection costs

3. **Viterbi Algorithm**:
   - Forward pass: Build lattice with cost calculations
   - Backward pass: Trace minimum cost path
   - Connection costs retrieved via `dic.get_trans_cost()`

## Design Decisions

### 1. Node Representation

**Challenge**: Rust's type system requires handling different node types safely.

**Solution**: Use trait objects with a common `LatticeNode` trait:

```rust
trait LatticeNode {
    fn surface(&self) -> &str;
    fn left_id(&self) -> u16;
    fn right_id(&self) -> u16;
    fn cost(&self) -> i16;
    fn min_cost(&self) -> i32;
    fn set_min_cost(&mut self, cost: i32);
    fn back_pos(&self) -> i32;
    fn set_back_pos(&mut self, pos: i32);
    fn back_index(&self) -> i32;
    fn set_back_index(&mut self, index: i32);
    fn pos(&self) -> usize;
    fn set_pos(&mut self, pos: usize);
    fn index(&self) -> usize;
    fn set_index(&mut self, index: usize);
    fn node_type(&self) -> NodeType;
    fn surface_len(&self) -> usize;
}
```

### 2. Memory Management

**Challenge**: Nodes need to be stored in vectors and referenced across lattice positions, while avoiding unnecessary copying of dictionary data.

**Solution**: Use lifetime parameters and references to DictEntry for dictionary nodes:

- `Node<'a>` holds `&'a DictEntry` to avoid copying morphological data
- `UnknownNode` owns its data since it's constructed dynamically  
- `Lattice<'a>` uses `Box<dyn LatticeNode + 'a>` for trait object storage

```rust
pub struct Lattice<'a> {
    snodes: Vec<Vec<Box<dyn LatticeNode + 'a>>>,
    enodes: Vec<Vec<Box<dyn LatticeNode + 'a>>>,
    p: usize,
    dic: Arc<dyn Dictionary>,
}
```

### 3. Viterbi Cost Calculation

**Implementation**: The `add()` method implements the core Viterbi logic:

1. For each existing end node at current position
2. Calculate total cost: `enode.min_cost + connection_cost + node.cost`
3. Keep track of minimum cost and best predecessor
4. Update node's Viterbi fields with optimal path information

### 4. Integration with Existing Code

**Dictionary Interface**: Leverage existing `Dictionary` trait's `get_trans_cost()` method.

**Types**: Use existing `DictEntry` and error types from the codebase.

## Implementation Plan

### File Structure
```
src/lattice.rs  # Single file implementation
```

### Core Components

#### 1. Node Type Definitions

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    SysDict,
    UserDict,
    Unknown,
}

#[derive(Debug)]
pub struct Node<'a> {
    // Reference to dictionary entry (avoids copying morphological data)
    dict_entry: &'a DictEntry,
    node_type: NodeType,
    
    // Viterbi algorithm fields
    min_cost: i32,
    back_pos: i32,
    back_index: i32,
    pos: usize,
    index: usize,
}

#[derive(Debug)]
pub struct UnknownNode {
    // For unknown words, we own the data since it's constructed dynamically
    surface: String,
    left_id: u16,
    right_id: u16,
    cost: i16,
    part_of_speech: String,
    base_form: String,
    
    // Viterbi algorithm fields
    min_cost: i32,
    back_pos: i32,
    back_index: i32,
    pos: usize,
    index: usize,
}
```

#### 2. Lattice Structure

```rust
pub struct Lattice<'a> {
    snodes: Vec<Vec<Box<dyn LatticeNode + 'a>>>,
    enodes: Vec<Vec<Box<dyn LatticeNode + 'a>>>,
    p: usize,
    dic: Arc<dyn Dictionary>,
}

impl<'a> Lattice<'a> {
    pub fn new(size: usize, dic: Arc<dyn Dictionary>) -> Self;
    pub fn add(&mut self, node: Box<dyn LatticeNode + 'a>) -> Result<(), RunomeError>;
    pub fn forward(&mut self) -> usize;
    pub fn end(&mut self) -> Result<(), RunomeError>;
    pub fn backward(&self) -> Result<Vec<&dyn LatticeNode>, RunomeError>;
}
```

#### 3. Viterbi Algorithm Implementation

**Add Method**: Core Viterbi logic for cost calculation and path tracking.

**Forward Method**: Position advancement with validation.

**Backward Method**: Path reconstruction from EOS to BOS.

### Key Methods Detail

#### `add()` Method Logic:
1. Initialize with maximum cost
2. Iterate through end nodes at current position
3. Calculate connection cost via dictionary
4. Find minimum cost path
5. Update node's Viterbi fields
6. Add to appropriate lattice positions

#### `backward()` Method Logic:
1. Start from EOS node (last position, index 0)
2. Follow back_pos and back_index pointers
3. Build path in reverse order
4. Return reversed path vector

### Error Handling

Use existing `RunomeError` types:
- `DictValidationError` for invalid connection IDs
- `InvalidInput` for malformed lattice states
- Custom error variants as needed

### Testing Strategy

#### Unit Tests:
1. Node creation and trait implementations
2. Lattice initialization and basic operations
3. Viterbi cost calculations with mock data
4. Path reconstruction accuracy

#### Integration Tests:
1. Real dictionary integration
2. Known morpheme sequence verification
3. Performance with various input sizes
4. Edge cases (empty input, single character)

#### Test Data:
- Use existing SystemDictionary test infrastructure
- Create minimal test cases with known costs
- Test with real Japanese text samples

### Performance Considerations

1. **Memory Efficiency**: 
   - Use references to `DictEntry` to avoid copying morphological data
   - `Box<dyn LatticeNode + 'a>` minimizes heap fragmentation
   - Only `UnknownNode` owns its data when dictionary lookup fails
2. **Connection Cost Caching**: Leverage dictionary's efficient cost lookup
3. **Vector Pre-allocation**: Size lattice vectors appropriately
4. **Zero-Copy Dictionary Access**: Lifetime parameters ensure data references remain valid

### Dependencies

- `std::sync::Arc` for shared dictionary reference
- Existing `Dictionary` trait for connection costs
- Existing `DictEntry` and `RunomeError` types
- No external crates beyond current dependencies

## Implementation Steps

### Phase 1: Core Structure (Day 1)
1. Define `LatticeNode` trait and implementations
2. Create `Lattice` struct with basic methods
3. Implement `new()` and basic initialization
4. Write initial unit tests

### Phase 2: Viterbi Algorithm (Day 2)
1. Implement `add()` method with cost calculation
2. Add `forward()` position management
3. Implement `end()` EOS handling
4. Create comprehensive cost calculation tests

### Phase 3: Path Reconstruction (Day 3)
1. Implement `backward()` method
2. Add path validation and error handling
3. Create integration tests with real data
4. Performance testing and optimization

### Phase 4: Integration and Polish (Day 4)
1. Integration with existing codebase
2. Documentation and examples
3. Edge case testing
4. Code review and refinement

## Success Criteria

1. **Functional**: Correctly implements Viterbi algorithm matching Janome's behavior
2. **Performance**: Handles typical Japanese text efficiently (< 10ms for short sentences)
3. **Integration**: Seamlessly works with existing Dictionary implementations
4. **Testing**: Comprehensive test coverage (>90%) with real data validation
5. **Documentation**: Clear API documentation with usage examples

This implementation will provide the foundation for Japanese morphological analysis by finding optimal morpheme segmentations using the Viterbi algorithm on a lattice of candidate morphemes.