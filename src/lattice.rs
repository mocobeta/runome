use std::sync::Arc;

use crate::dictionary::{DictEntry, Dictionary};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    SysDict,
    UserDict,
    Unknown,
}

/// Trait for all lattice nodes providing common interface for Viterbi algorithm
pub trait LatticeNode: std::fmt::Debug {
    /// Get the surface form of this node
    fn surface(&self) -> &str;

    /// Get the left part-of-speech ID for connection cost calculation
    fn left_id(&self) -> u16;

    /// Get the right part-of-speech ID for connection cost calculation
    fn right_id(&self) -> u16;

    /// Get the word cost of this node
    fn cost(&self) -> i16;

    /// Get the minimum cost from BOS to this node (Viterbi)
    fn min_cost(&self) -> i32;

    /// Set the minimum cost from BOS to this node (Viterbi)
    fn set_min_cost(&mut self, cost: i32);

    /// Get the position of the best predecessor node (Viterbi)
    fn back_pos(&self) -> i32;

    /// Set the position of the best predecessor node (Viterbi)
    fn set_back_pos(&mut self, pos: i32);

    /// Get the index of the best predecessor node (Viterbi)
    fn back_index(&self) -> i32;

    /// Set the index of the best predecessor node (Viterbi)
    fn set_back_index(&mut self, index: i32);

    /// Get the position of this node in the lattice
    fn pos(&self) -> usize;

    /// Set the position of this node in the lattice
    fn set_pos(&mut self, pos: usize);

    /// Get the index of this node within its position
    fn index(&self) -> usize;

    /// Set the index of this node within its position
    fn set_index(&mut self, index: usize);

    /// Get the type of this node (SysDict, UserDict, Unknown)
    fn node_type(&self) -> NodeType;

    /// Get the length of the surface form in characters
    fn surface_len(&self) -> usize;
}

/// Node backed by a dictionary entry reference (zero-copy for dictionary words)
#[derive(Debug)]
pub struct Node<'a> {
    /// Reference to dictionary entry (avoids copying morphological data)
    dict_entry: &'a DictEntry,
    node_type: NodeType,

    /// Viterbi algorithm fields
    min_cost: i32,
    back_pos: i32,
    back_index: i32,
    pos: usize,
    index: usize,
}

impl<'a> Node<'a> {
    /// Create a new Node from a dictionary entry reference
    pub fn new(dict_entry: &'a DictEntry, node_type: NodeType) -> Self {
        Self {
            dict_entry,
            node_type,
            min_cost: i32::MAX,
            back_pos: -1,
            back_index: -1,
            pos: 0,
            index: 0,
        }
    }

    /// Get the complete dictionary entry for this node
    pub fn dict_entry(&self) -> &DictEntry {
        self.dict_entry
    }
}

impl<'a> LatticeNode for Node<'a> {
    fn surface(&self) -> &str {
        &self.dict_entry.surface
    }

    fn left_id(&self) -> u16 {
        self.dict_entry.left_id
    }

    fn right_id(&self) -> u16 {
        self.dict_entry.right_id
    }

    fn cost(&self) -> i16 {
        self.dict_entry.cost
    }

    fn min_cost(&self) -> i32 {
        self.min_cost
    }

    fn set_min_cost(&mut self, cost: i32) {
        self.min_cost = cost;
    }

    fn back_pos(&self) -> i32 {
        self.back_pos
    }

    fn set_back_pos(&mut self, pos: i32) {
        self.back_pos = pos;
    }

    fn back_index(&self) -> i32 {
        self.back_index
    }

    fn set_back_index(&mut self, index: i32) {
        self.back_index = index;
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    fn node_type(&self) -> NodeType {
        self.node_type.clone()
    }

    fn surface_len(&self) -> usize {
        self.dict_entry.surface.chars().count()
    }
}

/// Node for unknown words that owns its morphological data
#[derive(Debug)]
pub struct UnknownNode {
    /// Morphological data (owned since it's constructed dynamically)
    surface: String,
    left_id: u16,
    right_id: u16,
    cost: i16,
    part_of_speech: String,
    base_form: String,

    /// Viterbi algorithm fields
    min_cost: i32,
    back_pos: i32,
    back_index: i32,
    pos: usize,
    index: usize,
}

impl UnknownNode {
    /// Create a new UnknownNode with owned morphological data
    pub fn new(
        surface: String,
        left_id: u16,
        right_id: u16,
        cost: i16,
        part_of_speech: String,
        base_form: String,
    ) -> Self {
        Self {
            surface,
            left_id,
            right_id,
            cost,
            part_of_speech,
            base_form,
            min_cost: i32::MAX,
            back_pos: -1,
            back_index: -1,
            pos: 0,
            index: 0,
        }
    }

    /// Get the part of speech for this unknown word
    pub fn part_of_speech(&self) -> &str {
        &self.part_of_speech
    }

    /// Get the base form for this unknown word
    pub fn base_form(&self) -> &str {
        &self.base_form
    }
}

impl LatticeNode for UnknownNode {
    fn surface(&self) -> &str {
        &self.surface
    }

    fn left_id(&self) -> u16 {
        self.left_id
    }

    fn right_id(&self) -> u16 {
        self.right_id
    }

    fn cost(&self) -> i16 {
        self.cost
    }

    fn min_cost(&self) -> i32 {
        self.min_cost
    }

    fn set_min_cost(&mut self, cost: i32) {
        self.min_cost = cost;
    }

    fn back_pos(&self) -> i32 {
        self.back_pos
    }

    fn set_back_pos(&mut self, pos: i32) {
        self.back_pos = pos;
    }

    fn back_index(&self) -> i32 {
        self.back_index
    }

    fn set_back_index(&mut self, index: i32) {
        self.back_index = index;
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    fn node_type(&self) -> NodeType {
        NodeType::Unknown
    }

    fn surface_len(&self) -> usize {
        self.surface.chars().count()
    }
}

/// Beginning-of-sentence node
#[derive(Debug)]
pub struct BOS {
    /// Viterbi algorithm fields
    min_cost: i32,
    back_pos: i32,
    back_index: i32,
    pos: usize,
    index: usize,
}

impl BOS {
    /// Create a new BOS node
    pub fn new() -> Self {
        Self {
            min_cost: 0, // BOS starts with cost 0
            back_pos: -1,
            back_index: -1,
            pos: 0,
            index: 0,
        }
    }
}

impl Default for BOS {
    fn default() -> Self {
        Self::new()
    }
}

impl LatticeNode for BOS {
    fn surface(&self) -> &str {
        "__BOS__"
    }

    fn left_id(&self) -> u16 {
        0 // BOS has no left context
    }

    fn right_id(&self) -> u16 {
        0 // BOS connects to any following node
    }

    fn cost(&self) -> i16 {
        0 // BOS has no inherent cost
    }

    fn min_cost(&self) -> i32 {
        self.min_cost
    }

    fn set_min_cost(&mut self, cost: i32) {
        self.min_cost = cost;
    }

    fn back_pos(&self) -> i32 {
        self.back_pos
    }

    fn set_back_pos(&mut self, pos: i32) {
        self.back_pos = pos;
    }

    fn back_index(&self) -> i32 {
        self.back_index
    }

    fn set_back_index(&mut self, index: i32) {
        self.back_index = index;
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    fn node_type(&self) -> NodeType {
        NodeType::SysDict // BOS is treated as system dictionary node
    }

    fn surface_len(&self) -> usize {
        0 // BOS has no surface representation
    }
}

/// End-of-sentence node
#[derive(Debug)]
pub struct EOS {
    /// Viterbi algorithm fields
    min_cost: i32,
    back_pos: i32,
    back_index: i32,
    pos: usize,
    index: usize,
}

impl EOS {
    /// Create a new EOS node at the specified position
    pub fn new(end_pos: usize) -> Self {
        Self {
            min_cost: i32::MAX,
            back_pos: -1,
            back_index: -1,
            pos: end_pos,
            index: 0,
        }
    }
}

impl LatticeNode for EOS {
    fn surface(&self) -> &str {
        "__EOS__"
    }

    fn left_id(&self) -> u16 {
        0 // EOS accepts connections from any preceding node
    }

    fn right_id(&self) -> u16 {
        0 // EOS has no right context
    }

    fn cost(&self) -> i16 {
        0 // EOS has no inherent cost
    }

    fn min_cost(&self) -> i32 {
        self.min_cost
    }

    fn set_min_cost(&mut self, cost: i32) {
        self.min_cost = cost;
    }

    fn back_pos(&self) -> i32 {
        self.back_pos
    }

    fn set_back_pos(&mut self, pos: i32) {
        self.back_pos = pos;
    }

    fn back_index(&self) -> i32 {
        self.back_index
    }

    fn set_back_index(&mut self, index: i32) {
        self.back_index = index;
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    fn node_type(&self) -> NodeType {
        NodeType::SysDict // EOS is treated as system dictionary node
    }

    fn surface_len(&self) -> usize {
        0 // EOS has no surface representation
    }
}

/// Lattice structure for Viterbi algorithm-based morphological analysis
pub struct Lattice<'a> {
    /// Start nodes at each position - snodes[pos][index]
    snodes: Vec<Vec<Box<dyn LatticeNode + 'a>>>,
    /// End nodes at each position - enodes[pos][index]  
    enodes: Vec<Vec<Box<dyn LatticeNode + 'a>>>,
    /// Current position pointer
    p: usize,
    /// Dictionary reference for connection cost lookups
    dic: Arc<dyn Dictionary>,
}

impl<'a> Lattice<'a> {
    /// Create a new lattice with the specified size and dictionary
    ///
    /// Initializes the lattice with BOS node at position 0 and pre-allocates
    /// vectors for the specified size.
    ///
    /// # Arguments
    /// * `size` - Maximum number of positions in the lattice
    /// * `dic` - Dictionary reference for connection cost calculations
    ///
    /// # Returns
    /// * New Lattice instance with BOS node initialized
    pub fn new(size: usize, dic: Arc<dyn Dictionary>) -> Self {
        // Initialize snodes and enodes vectors
        // We need positions 0 through size+1 (size+2 total positions)
        let mut snodes = Vec::with_capacity(size + 2);
        let mut enodes = Vec::with_capacity(size + 2);

        // Initialize all positions as empty first
        for _ in 0..=(size + 1) {
            snodes.push(Vec::new());
            enodes.push(Vec::new());
        }

        // Position 0: BOS node in snodes
        let mut bos = Box::new(BOS::new()) as Box<dyn LatticeNode + 'a>;
        bos.set_pos(0);
        bos.set_index(0);
        snodes[0].push(bos);

        // Position 1: BOS node also appears in enodes[1] for connections
        let mut bos_end = Box::new(BOS::new()) as Box<dyn LatticeNode + 'a>;
        bos_end.set_pos(0);
        bos_end.set_index(0);
        enodes[1].push(bos_end);

        Self {
            snodes,
            enodes,
            p: 1, // Start at position 1 (after BOS)
            dic,
        }
    }

    /// Get the current position in the lattice
    pub fn position(&self) -> usize {
        self.p
    }

    /// Get the total number of positions in the lattice
    pub fn size(&self) -> usize {
        self.snodes.len().saturating_sub(1)
    }

    /// Get reference to start nodes at the specified position
    pub fn start_nodes(&self, pos: usize) -> Option<&Vec<Box<dyn LatticeNode + 'a>>> {
        self.snodes.get(pos)
    }

    /// Get reference to end nodes at the specified position
    pub fn end_nodes(&self, pos: usize) -> Option<&Vec<Box<dyn LatticeNode + 'a>>> {
        self.enodes.get(pos)
    }

    /// Check if the lattice is properly initialized
    pub fn is_valid(&self) -> bool {
        // Must have at least BOS position
        if self.snodes.is_empty() || self.enodes.is_empty() {
            return false;
        }

        // Position 0 must contain exactly one BOS node
        if let Some(start_nodes) = self.snodes.first() {
            if start_nodes.len() != 1 {
                return false;
            }
            // Check if it's actually a BOS node
            if start_nodes[0].surface() != "__BOS__" {
                return false;
            }
        } else {
            return false;
        }

        // Position 1 in enodes must contain the BOS node for connections
        if let Some(end_nodes) = self.enodes.get(1) {
            if end_nodes.is_empty() {
                return false;
            }
        } else {
            return false;
        }

        true
    }

    /// Get a reference to the dictionary
    pub fn dictionary(&self) -> &Arc<dyn Dictionary> {
        &self.dic
    }
}

impl<'a> std::fmt::Debug for Lattice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lattice")
            .field("p", &self.p)
            .field("size", &self.size())
            .field("snodes_len", &self.snodes.len())
            .field("enodes_len", &self.enodes.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::DictEntry;
    use std::sync::Arc;

    fn create_test_dict_entry() -> DictEntry {
        DictEntry {
            surface: "テスト".to_string(),
            left_id: 100,
            right_id: 200,
            cost: 150,
            part_of_speech: "名詞,一般,*,*,*,*".to_string(),
            inflection_type: "*".to_string(),
            inflection_form: "*".to_string(),
            base_form: "テスト".to_string(),
            reading: "テスト".to_string(),
            phonetic: "テスト".to_string(),
        }
    }

    #[test]
    fn test_node_creation() {
        let dict_entry = create_test_dict_entry();
        let node = Node::new(&dict_entry, NodeType::SysDict);

        assert_eq!(node.surface(), "テスト");
        assert_eq!(node.left_id(), 100);
        assert_eq!(node.right_id(), 200);
        assert_eq!(node.cost(), 150);
        assert_eq!(node.node_type(), NodeType::SysDict);
        assert_eq!(node.surface_len(), 3); // 3 characters

        // Check initial Viterbi values
        assert_eq!(node.min_cost(), i32::MAX);
        assert_eq!(node.back_pos(), -1);
        assert_eq!(node.back_index(), -1);
        assert_eq!(node.pos(), 0);
        assert_eq!(node.index(), 0);
    }

    #[test]
    fn test_unknown_node_creation() {
        let unknown = UnknownNode::new(
            "未知語".to_string(),
            300,
            400,
            500,
            "名詞,一般,*,*,*,*".to_string(),
            "未知語".to_string(),
        );

        assert_eq!(unknown.surface(), "未知語");
        assert_eq!(unknown.left_id(), 300);
        assert_eq!(unknown.right_id(), 400);
        assert_eq!(unknown.cost(), 500);
        assert_eq!(unknown.node_type(), NodeType::Unknown);
        assert_eq!(unknown.surface_len(), 3);
        assert_eq!(unknown.part_of_speech(), "名詞,一般,*,*,*,*");
        assert_eq!(unknown.base_form(), "未知語");
    }

    #[test]
    fn test_bos_node() {
        let bos = BOS::new();

        assert_eq!(bos.surface(), "__BOS__");
        assert_eq!(bos.left_id(), 0);
        assert_eq!(bos.right_id(), 0);
        assert_eq!(bos.cost(), 0);
        assert_eq!(bos.min_cost(), 0); // BOS starts with cost 0
        assert_eq!(bos.node_type(), NodeType::SysDict);
        assert_eq!(bos.surface_len(), 0);
    }

    #[test]
    fn test_eos_node() {
        let eos = EOS::new(10);

        assert_eq!(eos.surface(), "__EOS__");
        assert_eq!(eos.left_id(), 0);
        assert_eq!(eos.right_id(), 0);
        assert_eq!(eos.cost(), 0);
        assert_eq!(eos.min_cost(), i32::MAX); // EOS starts with max cost
        assert_eq!(eos.pos(), 10);
        assert_eq!(eos.node_type(), NodeType::SysDict);
        assert_eq!(eos.surface_len(), 0);
    }

    #[test]
    fn test_viterbi_field_updates() {
        let dict_entry = create_test_dict_entry();
        let mut node = Node::new(&dict_entry, NodeType::SysDict);

        // Test updating Viterbi fields
        node.set_min_cost(1000);
        node.set_back_pos(5);
        node.set_back_index(2);
        node.set_pos(8);
        node.set_index(3);

        assert_eq!(node.min_cost(), 1000);
        assert_eq!(node.back_pos(), 5);
        assert_eq!(node.back_index(), 2);
        assert_eq!(node.pos(), 8);
        assert_eq!(node.index(), 3);
    }

    #[test]
    fn test_node_types() {
        let dict_entry = create_test_dict_entry();

        let sys_node = Node::new(&dict_entry, NodeType::SysDict);
        let user_node = Node::new(&dict_entry, NodeType::UserDict);

        assert_eq!(sys_node.node_type(), NodeType::SysDict);
        assert_eq!(user_node.node_type(), NodeType::UserDict);
    }

    #[test]
    fn test_surface_length_calculation() {
        // Test ASCII
        let dict_entry_ascii = DictEntry {
            surface: "test".to_string(),
            left_id: 1,
            right_id: 1,
            cost: 1,
            part_of_speech: "".to_string(),
            inflection_type: "".to_string(),
            inflection_form: "".to_string(),
            base_form: "".to_string(),
            reading: "".to_string(),
            phonetic: "".to_string(),
        };
        let node_ascii = Node::new(&dict_entry_ascii, NodeType::SysDict);
        assert_eq!(node_ascii.surface_len(), 4);

        // Test Japanese (multi-byte UTF-8)
        let dict_entry_jp = DictEntry {
            surface: "こんにちは".to_string(),
            left_id: 1,
            right_id: 1,
            cost: 1,
            part_of_speech: "".to_string(),
            inflection_type: "".to_string(),
            inflection_form: "".to_string(),
            base_form: "".to_string(),
            reading: "".to_string(),
            phonetic: "".to_string(),
        };
        let node_jp = Node::new(&dict_entry_jp, NodeType::SysDict);
        assert_eq!(node_jp.surface_len(), 5); // 5 characters, not bytes
    }

    // Mock dictionary for testing
    struct MockDictionary;

    impl crate::dictionary::Dictionary for MockDictionary {
        fn lookup(&self, _surface: &str) -> Result<Vec<&DictEntry>, crate::error::RunomeError> {
            Ok(Vec::new()) // Return empty for testing
        }

        fn get_trans_cost(
            &self,
            _left_id: u16,
            _right_id: u16,
        ) -> Result<i16, crate::error::RunomeError> {
            Ok(100) // Return fixed cost for testing
        }
    }

    fn create_mock_dictionary() -> Arc<dyn crate::dictionary::Dictionary> {
        Arc::new(MockDictionary)
    }

    #[test]
    fn test_lattice_creation() {
        let dic = create_mock_dictionary();
        let lattice = Lattice::new(10, dic);

        // Check basic properties
        assert_eq!(lattice.position(), 1); // Starts at position 1
        assert_eq!(lattice.size(), 11); // Should be size + 1
        assert!(lattice.is_valid());

        // Check BOS node at position 0
        let start_nodes = lattice.start_nodes(0).unwrap();
        assert_eq!(start_nodes.len(), 1);
        assert_eq!(start_nodes[0].surface(), "__BOS__");
        assert_eq!(start_nodes[0].pos(), 0);
        assert_eq!(start_nodes[0].index(), 0);

        // Check BOS also appears in enodes[1] for connections
        let end_nodes = lattice.end_nodes(1).unwrap();
        assert_eq!(end_nodes.len(), 1);
        assert_eq!(end_nodes[0].surface(), "__BOS__");
    }

    #[test]
    fn test_lattice_validation() {
        let dic = create_mock_dictionary();
        let lattice = Lattice::new(5, dic);

        // Should be valid after creation
        assert!(lattice.is_valid());

        // Test validation logic
        assert!(lattice.start_nodes(0).is_some());
        assert!(lattice.end_nodes(1).is_some());
    }

    #[test]
    fn test_lattice_empty_positions() {
        let dic = create_mock_dictionary();
        let lattice = Lattice::new(3, dic);

        // Positions 1, 2, 3 should be empty initially (except enodes[1] has BOS)
        assert_eq!(lattice.start_nodes(1).unwrap().len(), 0);
        assert_eq!(lattice.start_nodes(2).unwrap().len(), 0);
        assert_eq!(lattice.start_nodes(3).unwrap().len(), 0);

        // Only enodes[1] should have the BOS node, others empty
        assert_eq!(lattice.end_nodes(0).unwrap().len(), 0);
        assert_eq!(lattice.end_nodes(2).unwrap().len(), 0);
        assert_eq!(lattice.end_nodes(3).unwrap().len(), 0);
    }

    #[test]
    fn test_lattice_bounds_checking() {
        let dic = create_mock_dictionary();
        let lattice = Lattice::new(2, dic);

        // Valid positions
        assert!(lattice.start_nodes(0).is_some());
        assert!(lattice.start_nodes(1).is_some());
        assert!(lattice.start_nodes(2).is_some());

        // Out of bounds positions should return None
        assert!(lattice.start_nodes(10).is_none());
        assert!(lattice.end_nodes(10).is_none());
    }

    #[test]
    fn test_lattice_dictionary_access() {
        let dic = create_mock_dictionary();
        let lattice = Lattice::new(5, dic.clone());

        // Should be able to access the dictionary
        let lattice_dic = lattice.dictionary();

        // Test that it's the same dictionary (using Arc)
        assert!(Arc::ptr_eq(lattice_dic, &dic));
    }

    #[test]
    fn test_lattice_zero_size() {
        let dic = create_mock_dictionary();
        let lattice = Lattice::new(0, dic);

        // Should still be valid with just BOS
        assert!(lattice.is_valid());
        assert_eq!(lattice.position(), 1);
        assert_eq!(lattice.size(), 1); // Just position 0

        // Should have BOS at position 0
        let start_nodes = lattice.start_nodes(0).unwrap();
        assert_eq!(start_nodes.len(), 1);
        assert_eq!(start_nodes[0].surface(), "__BOS__");
    }
}
