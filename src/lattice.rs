use crate::dictionary::DictEntry;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    SysDict,
    UserDict,
    Unknown,
}

/// Trait for all lattice nodes providing common interface for Viterbi algorithm
pub trait LatticeNode {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::DictEntry;

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
}
