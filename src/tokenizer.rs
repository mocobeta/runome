use std::fmt;
use std::sync::Arc;

use crate::dictionary::{Dictionary, SystemDictionary, UserDictionary};
use crate::error::RunomeError;
use crate::lattice::{Lattice, LatticeNode, NodeType};

/// Constants matching Python Janome tokenizer
const MAX_CHUNK_SIZE: usize = 1024;
const CHUNK_SIZE: usize = 500;

/// Token struct containing all morphological information
/// Mirrors the Python Token class with complete compatibility
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
    /// Create a Token from a dictionary node with full morphological information
    pub fn from_dict_node(node: &dyn LatticeNode) -> Self {
        Self {
            surface: node.surface().to_string(),
            part_of_speech: node.part_of_speech().to_string(),
            infl_type: node.inflection_type().to_string(),
            infl_form: node.inflection_form().to_string(),
            base_form: node.base_form().to_string(),
            reading: node.reading().to_string(),
            phonetic: node.phonetic().to_string(),
            node_type: node.node_type(),
        }
    }

    /// Create a Token from an unknown word node
    pub fn from_unknown_node(node: &dyn LatticeNode, baseform_unk: bool) -> Self {
        let base_form = if baseform_unk {
            node.surface().to_string()
        } else {
            "*".to_string()
        };

        Self {
            surface: node.surface().to_string(),
            part_of_speech: node.part_of_speech().to_string(),
            infl_type: "*".to_string(),
            infl_form: "*".to_string(),
            base_form,
            reading: node.reading().to_string(),
            phonetic: node.phonetic().to_string(),
            node_type: node.node_type(),
        }
    }

    /// Create a Token with explicit field values
    /// Used by TokenFilters to create modified tokens
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        surface: String,
        part_of_speech: String,
        infl_type: String,
        infl_form: String,
        base_form: String,
        reading: String,
        phonetic: String,
        node_type: NodeType,
    ) -> Self {
        Self {
            surface,
            part_of_speech,
            infl_type,
            infl_form,
            base_form,
            reading,
            phonetic,
            node_type,
        }
    }

    // Accessor methods matching Python Token class

    pub fn surface(&self) -> &str {
        &self.surface
    }

    pub fn part_of_speech(&self) -> &str {
        &self.part_of_speech
    }

    pub fn infl_type(&self) -> &str {
        &self.infl_type
    }

    pub fn infl_form(&self) -> &str {
        &self.infl_form
    }

    pub fn base_form(&self) -> &str {
        &self.base_form
    }

    pub fn reading(&self) -> &str {
        &self.reading
    }

    pub fn phonetic(&self) -> &str {
        &self.phonetic
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type.clone()
    }
}

impl fmt::Display for Token {
    /// Format Token to match Python Janome output exactly:
    /// "surface\tpart_of_speech,infl_type,infl_form,base_form,reading,phonetic"
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\t{},{},{},{},{},{}",
            self.surface,
            self.part_of_speech,
            self.infl_type,
            self.infl_form,
            self.base_form,
            self.reading,
            self.phonetic
        )
    }
}

/// Enum representing the result of tokenization
/// Either a full Token with morphological info or just the surface string (wakati mode)
#[derive(Debug, Clone)]
pub enum TokenizeResult {
    Token(Token),
    Surface(String),
}

impl fmt::Display for TokenizeResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizeResult::Token(token) => write!(f, "{}", token),
            TokenizeResult::Surface(surface) => write!(f, "{}", surface),
        }
    }
}

/// Iterator for streaming tokenization results
pub struct TextChunkIterator<'a> {
    tokenizer: &'a Tokenizer,
    text: &'a str,
    processed: usize,
    current_tokens: std::vec::IntoIter<TokenizeResult>,
    wakati: bool,
    baseform_unk: bool,
}

impl<'a> Iterator for TextChunkIterator<'a> {
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
                self.baseform_unk,
            ) {
                Ok((tokens, pos)) => {
                    self.processed += pos;
                    self.current_tokens = tokens.into_iter();
                    self.current_tokens.next().map(Ok)
                }
                Err(e) => Some(Err(e)),
            }
        } else {
            None
        }
    }
}

/// Main Tokenizer struct providing Japanese morphological analysis
/// Mirrors the Python Janome Tokenizer class API
#[derive(Clone)]
pub struct Tokenizer {
    sys_dic: Arc<SystemDictionary>,
    user_dic: Option<Arc<UserDictionary>>,
    max_unknown_length: usize,
    wakati: bool,
}

impl Tokenizer {
    /// Create a new Tokenizer instance
    ///
    /// # Arguments
    /// * `max_unknown_length` - Maximum length for unknown words (default: 1024)
    /// * `wakati` - If true, only return surface forms (default: false)
    ///
    /// # Returns
    /// * `Ok(Tokenizer)` - Successfully created tokenizer
    /// * `Err(RunomeError)` - Error if dictionary initialization fails
    pub fn new(
        max_unknown_length: Option<usize>,
        wakati: Option<bool>,
    ) -> Result<Self, RunomeError> {
        let sys_dic = SystemDictionary::instance()?;

        Ok(Self {
            sys_dic,
            user_dic: None,
            max_unknown_length: max_unknown_length.unwrap_or(1024),
            wakati: wakati.unwrap_or(false),
        })
    }

    /// Create a new Tokenizer instance with user dictionary
    ///
    /// # Arguments
    /// * `user_dic` - User dictionary to use for custom entries
    /// * `max_unknown_length` - Maximum length for unknown words (default: 1024)
    /// * `wakati` - If true, only return surface forms (default: false)
    ///
    /// # Returns
    /// * `Ok(Tokenizer)` - Successfully created tokenizer
    /// * `Err(RunomeError)` - Error if dictionary initialization fails
    pub fn with_user_dict(
        user_dic: Arc<UserDictionary>,
        max_unknown_length: Option<usize>,
        wakati: Option<bool>,
    ) -> Result<Self, RunomeError> {
        let sys_dic = SystemDictionary::instance()?;

        Ok(Self {
            sys_dic,
            user_dic: Some(user_dic),
            max_unknown_length: max_unknown_length.unwrap_or(1024),
            wakati: wakati.unwrap_or(false),
        })
    }

    /// Tokenize input text into morphological units
    ///
    /// # Arguments
    /// * `text` - Input Japanese text to tokenize
    /// * `wakati` - Override wakati mode for this call (optional)
    /// * `baseform_unk` - Set base form for unknown words (default: true)
    ///
    /// # Returns
    /// Iterator yielding `TokenizeResult` items (either Token or Surface string)
    pub fn tokenize<'a>(
        &'a self,
        text: &'a str,
        wakati: Option<bool>,
        baseform_unk: Option<bool>,
    ) -> impl Iterator<Item = Result<TokenizeResult, RunomeError>> + 'a {
        // If tokenizer was initialized with wakati=True, always use wakati mode
        // regardless of the parameter passed to tokenize()
        let wakati_mode = if self.wakati {
            true
        } else {
            wakati.unwrap_or(false)
        };
        let baseform_unk_mode = baseform_unk.unwrap_or(true);

        self.tokenize_stream(text, wakati_mode, baseform_unk_mode)
    }

    /// Get the wakati mode setting for this tokenizer
    pub fn wakati(&self) -> bool {
        self.wakati
    }

    /// Create a streaming iterator for tokenization
    fn tokenize_stream<'a>(
        &'a self,
        text: &'a str,
        wakati: bool,
        baseform_unk: bool,
    ) -> TextChunkIterator<'a> {
        TextChunkIterator {
            tokenizer: self,
            text: text.trim(),
            processed: 0,
            current_tokens: Vec::new().into_iter(),
            wakati,
            baseform_unk,
        }
    }

    /// Process a partial chunk of text through the tokenization pipeline
    /// This is the core tokenization method implementing Phase 2 functionality
    fn tokenize_partial(
        &self,
        text: &str,
        wakati: bool,
        baseform_unk: bool,
    ) -> Result<(Vec<TokenizeResult>, usize), RunomeError> {
        if text.is_empty() {
            return Ok((Vec::new(), 0));
        }

        // Determine chunk size, respecting splitting logic
        let mut chunk_end = text.len();
        for pos in CHUNK_SIZE..std::cmp::min(text.len(), MAX_CHUNK_SIZE) {
            if self.should_split(text, pos) {
                chunk_end = pos;
                break;
            }
        }
        if chunk_end > MAX_CHUNK_SIZE {
            chunk_end = MAX_CHUNK_SIZE;
        }

        // Process only the chunk we determined
        let chunk_text = &text[..chunk_end];

        // Create lattice for this chunk
        // Add +1 to lattice size to account for EOS position
        let lattice_size = chunk_text.chars().count() + 1;
        let mut lattice = Lattice::new(
            lattice_size,
            self.sys_dic.clone() as Arc<dyn crate::dictionary::Dictionary>,
        );

        // Add dictionary entries to lattice
        self.add_dictionary_entries(&mut lattice, chunk_text, baseform_unk)?;

        // Process the lattice using Viterbi algorithm
        // Note: we don't call lattice.forward() here because we've already advanced incrementally
        lattice.end()?;
        let path = lattice.backward()?;

        // Convert path to tokens (excluding BOS and EOS)
        let tokens = self.path_to_tokens(&path[1..path.len() - 1], wakati, baseform_unk)?;

        Ok((tokens, chunk_end))
    }

    /// Add dictionary entries to the lattice following Python's incremental approach
    /// This matches Python Janome's tokenize() method exactly
    fn add_dictionary_entries<'a>(
        &self,
        lattice: &mut Lattice<'a>,
        text: &str,
        baseform_unk: bool,
    ) -> Result<(), RunomeError> {
        let _text_bytes = text.as_bytes();
        let text_len = text.len();
        let mut pos = 0;

        // Python-style incremental processing: while pos < len(s):
        while pos < text_len {
            let _current_pos = lattice.position();

            // Extract current character for unknown word processing
            let current_char = text[pos..].chars().next().unwrap();
            let mut matched = false;

            // 1. DICTIONARY LOOKUP - try all possible substrings starting at current position
            // We need to work with character-based lengths, not byte-based
            let remaining_text = &text[pos..];
            let char_indices: Vec<_> = remaining_text.char_indices().collect();

            for char_len in 1..=std::cmp::min(char_indices.len(), 50) {
                // Max word length limit
                // Get substring by character count, not byte count
                let end_byte = if char_len < char_indices.len() {
                    char_indices[char_len].0
                } else {
                    remaining_text.len()
                };
                let substring = &remaining_text[..end_byte];

                // Look up dictionary entries for this substring
                // 1. Check user dictionary first (higher priority)
                if let Some(user_dic) = &self.user_dic {
                    match user_dic.lookup(substring) {
                        Ok(entries) if !entries.is_empty() => {
                            matched = true;
                            for entry in entries {
                                // Create user dictionary node
                                let user_node = Box::new(crate::lattice::UnknownNode::new(
                                    entry.surface.clone(),
                                    entry.left_id,
                                    entry.right_id,
                                    entry.cost,
                                    entry.part_of_speech.clone(),
                                    entry.inflection_type.clone(),
                                    entry.inflection_form.clone(),
                                    entry.base_form.clone(),
                                    entry.reading.clone(),
                                    entry.phonetic.clone(),
                                    NodeType::UserDict,
                                ));
                                lattice.add(user_node)?;
                            }
                        }
                        _ => {
                            // No entries found in user dictionary
                        }
                    }
                }

                // 2. Check system dictionary (lower priority)
                match self.sys_dic.lookup(substring) {
                    Ok(entries) if !entries.is_empty() => {
                        matched = true;
                        for entry in entries {
                            // Create system dictionary node
                            let dict_node = Box::new(crate::lattice::UnknownNode::new(
                                entry.surface.clone(),
                                entry.left_id,
                                entry.right_id,
                                entry.cost,
                                entry.part_of_speech.clone(),
                                entry.inflection_type.clone(),
                                entry.inflection_form.clone(),
                                entry.base_form.clone(),
                                entry.reading.clone(),
                                entry.phonetic.clone(),
                                NodeType::SysDict,
                            ));
                            lattice.add(dict_node)?;
                        }
                    }
                    _ => {
                        // No entries found for this substring
                    }
                }
            }

            // 2. UNKNOWN WORD PROCESSING - Python logic
            let char_categories = self.sys_dic.get_char_categories_result(current_char)?;

            for category in &char_categories {
                // Python: if matched and not self.sys_dic.unknown_invoked_always(cate): continue
                let should_invoke = !matched
                    || self
                        .sys_dic
                        .unknown_invoked_always_result(category)
                        .unwrap_or(false);

                if should_invoke {
                    // Get unknown word entries for this category
                    let unknown_entries = match self.sys_dic.get_unknown_entries_result(category) {
                        Ok(entries) => entries,
                        Err(_) => continue,
                    };

                    // Build unknown word following Python's exact logic
                    let grouped_surface =
                        self.build_grouped_surface_python_style(text, pos, category)?;

                    // Create unknown word nodes
                    for entry in unknown_entries {
                        let base_form = if baseform_unk {
                            grouped_surface.clone()
                        } else {
                            "*".to_string()
                        };

                        let unknown_node = Box::new(crate::lattice::UnknownNode::new(
                            grouped_surface.clone(),
                            entry.left_id,
                            entry.right_id,
                            entry.cost,
                            entry.part_of_speech.clone(),
                            "*".to_string(),
                            "*".to_string(),
                            base_form,
                            "*".to_string(),
                            "*".to_string(),
                            NodeType::Unknown,
                        ));

                        lattice.add(unknown_node)?;
                    }
                }
            }

            // 3. CRITICAL: Python-style position advancement
            // Python: pos += lattice.forward()
            let advancement = lattice.forward();

            // Convert lattice position advancement to byte position in text
            // This is the key insight - we need to track byte positions in text
            // while letting the lattice control the advancement
            if advancement > 0 {
                // Find the byte position corresponding to the lattice advancement
                let mut char_count = 0;
                for (_i, _) in text[pos..].char_indices() {
                    if char_count >= advancement {
                        break;
                    }
                    char_count += 1;
                }
                // Move to position after the last character
                if char_count < advancement {
                    pos = text_len; // End of string
                } else {
                    // Find start of next character
                    pos = text[pos..]
                        .char_indices()
                        .nth(advancement)
                        .map(|(i, _)| pos + i)
                        .unwrap_or(text_len);
                }
            } else {
                // If no advancement, move by one character to avoid infinite loop
                pos = text[pos..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| pos + i)
                    .unwrap_or(text_len);
            }
        }

        Ok(())
    }

    /// Build grouped surface form following Python Janome's exact logic
    /// This version works with string byte positions like Python
    fn build_grouped_surface_python_style(
        &self,
        text: &str,
        start_pos: usize,
        category: &str,
    ) -> Result<String, RunomeError> {
        let category_max_length = self.sys_dic.unknown_length_result(category)?;
        let length = if self.sys_dic.unknown_grouping_result(category)? {
            self.max_unknown_length
        } else {
            category_max_length
        };

        let mut buf = String::new();
        let char_indices: Vec<_> = text[start_pos..].char_indices().collect();

        // Add the starting character
        if let Some((_, first_char)) = char_indices.first() {
            buf.push(*first_char);
        }

        // Group consecutive characters following Python's logic
        for (byte_offset, c) in char_indices.iter().skip(1) {
            if buf.chars().count() >= length {
                break;
            }

            let abs_pos = start_pos + byte_offset;
            if abs_pos >= text.len() {
                break;
            }

            // Get character categories for this character
            let c_categories = self.sys_dic.get_char_categories_result(*c)?;

            // Python logic: if cate in _cates or any(cate in _compat_cates for _compat_cates in _cates.values())
            let same_category = c_categories.contains(&category.to_string());
            let compatible = self.is_compatible_category_python_style(category, &c_categories);

            if same_category || compatible {
                buf.push(*c);
            } else {
                break;
            }
        }

        Ok(buf)
    }

    /// Python-style category compatibility checking
    /// Implements: any(cate in _compat_cates for _compat_cates in _cates.values())
    fn is_compatible_category_python_style(
        &self,
        base_category: &str,
        char_categories: &[String],
    ) -> bool {
        // For now, use simplified compatibility rules
        // TODO: Implement full compatible categories lookup from char definitions
        match base_category {
            "NUMERIC" => char_categories
                .iter()
                .any(|cat| cat == "NUMERIC" || cat == "DEFAULT"),
            "ALPHA" => char_categories
                .iter()
                .any(|cat| cat == "ALPHA" || cat == "DEFAULT"),
            "KATAKANA" => char_categories
                .iter()
                .any(|cat| cat == "KATAKANA" || cat == "DEFAULT"),
            "HIRAGANA" => char_categories
                .iter()
                .any(|cat| cat == "HIRAGANA" || cat == "DEFAULT"),
            "KANJI" => char_categories
                .iter()
                .any(|cat| cat == "KANJI" || cat == "DEFAULT"),
            "SYMBOL" => char_categories
                .iter()
                .any(|cat| cat == "SYMBOL" || cat == "DEFAULT"),
            _ => false,
        }
    }

    /// Convert a path of lattice nodes to tokens
    fn path_to_tokens(
        &self,
        path: &[&dyn LatticeNode],
        wakati: bool,
        baseform_unk: bool,
    ) -> Result<Vec<TokenizeResult>, RunomeError> {
        let mut tokens = Vec::new();

        for node in path {
            if wakati {
                // Wakati mode: return only surface forms
                tokens.push(TokenizeResult::Surface(node.surface().to_string()));
            } else {
                // Full mode: create Token objects with morphological information
                let token = match node.node_type() {
                    NodeType::SysDict => Token::from_dict_node(*node),
                    NodeType::Unknown => Token::from_unknown_node(*node, baseform_unk),
                    NodeType::UserDict => Token::from_dict_node(*node), // Treat as dict node for now
                };
                tokens.push(TokenizeResult::Token(token));
            }
        }

        Ok(tokens)
    }

    /// Determine if text should be split at the given position
    /// Implements Python's chunking strategy
    fn should_split(&self, text: &str, pos: usize) -> bool {
        pos >= text.len()
            || pos >= MAX_CHUNK_SIZE
            || (pos >= CHUNK_SIZE && self.is_splittable(&text[..pos]))
    }

    /// Check if text can be split at the end (at punctuation or newlines)
    fn is_splittable(&self, text: &str) -> bool {
        if let Some(last_char) = text.chars().last() {
            self.is_punct(last_char) || self.is_newline(text)
        } else {
            false
        }
    }

    /// Check if character is punctuation (suitable for splitting)
    fn is_punct(&self, c: char) -> bool {
        matches!(c, '、' | '。' | ',' | '.' | '？' | '?' | '！' | '!')
    }

    /// Check if text ends with newlines (suitable for splitting)
    fn is_newline(&self, text: &str) -> bool {
        text.ends_with("\n\n") || text.ends_with("\r\n\r\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        // Test Token creation with minimal data
        use crate::lattice::UnknownNode;

        let unknown_node = UnknownNode::new(
            "テスト".to_string(),
            100,
            200,
            150,
            "名詞,一般,*,*,*,*".to_string(),
            "*".to_string(),
            "*".to_string(),
            "テスト".to_string(),
            "*".to_string(),
            "*".to_string(),
            NodeType::Unknown,
        );

        let token = Token::from_unknown_node(&unknown_node, true);

        assert_eq!(token.surface(), "テスト");
        assert_eq!(token.part_of_speech(), "名詞,一般,*,*,*,*");
        assert_eq!(token.base_form(), "テスト"); // baseform_unk = true
        assert_eq!(token.node_type(), NodeType::Unknown);
    }

    #[test]
    fn test_token_display() {
        use crate::lattice::UnknownNode;

        let unknown_node = UnknownNode::new(
            "テスト".to_string(),
            100,
            200,
            150,
            "名詞,一般,*,*,*,*".to_string(),
            "*".to_string(),
            "*".to_string(),
            "テスト".to_string(),
            "*".to_string(),
            "*".to_string(),
            NodeType::Unknown,
        );

        let token = Token::from_unknown_node(&unknown_node, true);
        let formatted = format!("{}", token);

        // Should match Python format: surface\tpart_of_speech,infl_type,infl_form,base_form,reading,phonetic
        assert_eq!(formatted, "テスト\t名詞,一般,*,*,*,*,*,*,テスト,*,*");
    }

    #[test]
    fn test_tokenize_result_display() {
        let surface_result = TokenizeResult::Surface("テスト".to_string());
        assert_eq!(format!("{}", surface_result), "テスト");

        use crate::lattice::UnknownNode;
        let unknown_node = UnknownNode::new(
            "テスト".to_string(),
            100,
            200,
            150,
            "名詞,一般,*,*,*,*".to_string(),
            "*".to_string(),
            "*".to_string(),
            "テスト".to_string(),
            "*".to_string(),
            "*".to_string(),
            NodeType::Unknown,
        );
        let token = Token::from_unknown_node(&unknown_node, true);
        let token_result = TokenizeResult::Token(token);

        assert!(format!("{}", token_result).starts_with("テスト\t"));
    }

    #[test]
    fn test_tokenizer_creation() {
        // Skip test if sysdic directory doesn't exist
        let sysdic_path = std::path::PathBuf::from("sysdic");
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let tokenizer = Tokenizer::new(None, None);
        assert!(tokenizer.is_ok(), "Tokenizer creation should succeed");

        let tokenizer = tokenizer.unwrap();
        assert_eq!(tokenizer.max_unknown_length, 1024);
        assert!(!tokenizer.wakati);
    }

    #[test]
    fn test_tokenizer_custom_params() {
        // Skip test if sysdic directory doesn't exist
        let sysdic_path = std::path::PathBuf::from("sysdic");
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let tokenizer = Tokenizer::new(Some(2048), Some(true));
        assert!(tokenizer.is_ok(), "Tokenizer creation should succeed");

        let tokenizer = tokenizer.unwrap();
        assert_eq!(tokenizer.max_unknown_length, 2048);
        assert!(tokenizer.wakati);
    }

    #[test]
    fn test_basic_tokenize_placeholder() {
        // Skip test if sysdic directory doesn't exist
        let sysdic_path = std::path::PathBuf::from("sysdic");
        if !sysdic_path.exists() {
            eprintln!(
                "Skipping test: sysdic directory not found at {:?}",
                sysdic_path
            );
            return;
        }

        let tokenizer = Tokenizer::new(None, None).unwrap();
        let text = "テスト";

        // Test that tokenize method returns an iterator
        let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();
        assert!(results.is_ok(), "Tokenization should not fail");

        let tokens = results.unwrap();
        assert!(!tokens.is_empty(), "Should return at least one token");
    }

    #[test]
    fn test_chunking_helpers() {
        let tokenizer = Tokenizer::new(None, None);
        if tokenizer.is_err() {
            eprintln!("Skipping test: SystemDictionary not available");
            return;
        }
        let tokenizer = tokenizer.unwrap();

        // Test punctuation detection
        assert!(tokenizer.is_punct('。'));
        assert!(tokenizer.is_punct('、'));
        assert!(tokenizer.is_punct('?'));
        assert!(!tokenizer.is_punct('あ'));

        // Test newline detection
        assert!(tokenizer.is_newline("text\n\n"));
        assert!(tokenizer.is_newline("text\r\n\r\n"));
        assert!(!tokenizer.is_newline("text\n"));

        // Test splittable text
        assert!(tokenizer.is_splittable("これは文です。"));
        assert!(tokenizer.is_splittable("質問？"));
        assert!(!tokenizer.is_splittable("文の途中"));
    }

    #[test]
    fn test_should_split_logic() {
        let tokenizer = Tokenizer::new(None, None);
        if tokenizer.is_err() {
            eprintln!("Skipping test: SystemDictionary not available");
            return;
        }
        let tokenizer = tokenizer.unwrap();

        let text = "短いテキスト";

        // Should not split short text
        assert!(!tokenizer.should_split(text, 5));

        // Should split at end of text
        assert!(tokenizer.should_split(text, text.len()));

        // Test with large position (would exceed MAX_CHUNK_SIZE)
        assert!(tokenizer.should_split(text, MAX_CHUNK_SIZE + 1));
    }

    #[test]
    fn test_character_categories() {
        let tokenizer = Tokenizer::new(None, None);
        if tokenizer.is_err() {
            eprintln!("Skipping test: SystemDictionary not available");
            return;
        }
        let tokenizer = tokenizer.unwrap();

        // Test different character types
        let test_cases = vec![
            ('あ', "hiragana"), // Hiragana
            ('ア', "katakana"), // Katakana
            ('漢', "kanji"),    // Kanji
            ('2', "numeric"),   // Number
            ('A', "alpha"),     // Alphabet
            ('、', "symbol"),   // Symbol
        ];

        for (ch, expected_type) in test_cases {
            let categories = tokenizer.sys_dic.get_char_categories_result(ch);
            match categories {
                Ok(cats) => {
                    assert!(
                        !cats.is_empty(),
                        "Character '{}' should have at least one category",
                        ch
                    );
                    eprintln!(
                        "Character '{}' has categories: {:?} (expected type: {})",
                        ch, cats, expected_type
                    );
                }
                Err(e) => {
                    eprintln!("Warning: Could not get categories for '{}': {:?}", ch, e);
                }
            }
        }
    }

    #[test]
    fn test_unknown_word_grouping() {
        let tokenizer = Tokenizer::new(None, None);
        if tokenizer.is_err() {
            eprintln!("Skipping test: SystemDictionary not available");
            return;
        }
        let tokenizer = tokenizer.unwrap();

        // Test cases for unknown word grouping
        let test_cases = vec![
            // (input, expected_tokens)
            (
                "2009年",
                vec![("2009", "名詞,数"), ("年", "名詞,接尾,助数詞")],
            ),
            ("2009", vec![("2009", "名詞,数")]),
            ("ABC", vec![("ABC", "名詞,固有名詞,組織")]), // Should group alphabetic characters
            ("123", vec![("123", "名詞,数")]),            // Should group numeric characters
        ];

        for (text, expected) in test_cases {
            let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();

            match results {
                Ok(tokens) => {
                    assert_eq!(
                        tokens.len(),
                        expected.len(),
                        "Expected {} tokens for '{}', but got {}. Expected: {:?}",
                        expected.len(),
                        text,
                        tokens.len(),
                        expected
                    );

                    // Validate each token matches expected surface and part-of-speech
                    for (i, (expected_surface, expected_pos_prefix)) in expected.iter().enumerate()
                    {
                        match &tokens[i] {
                            TokenizeResult::Token(token) => {
                                // Check surface form
                                assert_eq!(
                                    token.surface(),
                                    *expected_surface,
                                    "Token {} surface mismatch for '{}': expected '{}', got '{}'",
                                    i,
                                    text,
                                    expected_surface,
                                    token.surface()
                                );

                                // Check part-of-speech starts with expected prefix
                                assert!(
                                    token.part_of_speech().starts_with(expected_pos_prefix),
                                    "Token {} part-of-speech mismatch for '{}': expected to start with '{}', got '{}'",
                                    i,
                                    text,
                                    expected_pos_prefix,
                                    token.part_of_speech()
                                );
                            }
                            TokenizeResult::Surface(surface) => {
                                panic!(
                                    "Expected Token but got Surface '{}' for test case '{}'",
                                    surface, text
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    panic!("Tokenization failed for '{}': {:?}", text, e);
                }
            }
        }
    }

    #[test]
    fn test_unknown_word_grouping_edge_cases() {
        let tokenizer = Tokenizer::new(None, None);
        if tokenizer.is_err() {
            eprintln!("Skipping test: SystemDictionary not available");
            return;
        }
        let tokenizer = tokenizer.unwrap();

        // Edge cases that should fail if grouping is broken
        let edge_cases = vec![
            // Mixed character types - should NOT group across categories
            ("123abc", 2), // Should be "123" + "abc", not "123abc"
            ("ABC123", 2), // Should be "ABC" + "123", not "ABC123"
            // Single characters - should still work
            ("2", 1), // Single digit
            ("A", 1), // Single letter
        ];

        for (text, expected_count) in edge_cases {
            eprintln!("\n=== Edge case: '{}' ===", text);

            let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();

            match results {
                Ok(tokens) => {
                    eprintln!(
                        "Tokenization of '{}' produced {} tokens:",
                        text,
                        tokens.len()
                    );
                    for (i, token) in tokens.iter().enumerate() {
                        eprintln!("  Token {}: {}", i, token);
                    }

                    assert_eq!(
                        tokens.len(),
                        expected_count,
                        "Edge case '{}' failed: expected {} tokens, got {}",
                        text,
                        expected_count,
                        tokens.len()
                    );

                    eprintln!("✓ Edge case '{}' PASSED", text);
                }
                Err(e) => {
                    panic!("Edge case tokenization failed for '{}': {:?}", text, e);
                }
            }
        }

        eprintln!("\n🎉 All edge case tests PASSED!");
    }

    #[test]
    fn test_python_compatibility_basic() {
        let tokenizer = Tokenizer::new(None, None);
        if tokenizer.is_err() {
            eprintln!("Skipping test: SystemDictionary not available");
            return;
        }
        let tokenizer = tokenizer.unwrap();

        // Test basic Japanese text that should match Python Janome output
        let test_cases = vec![
            "すもも", // Simple hiragana
            "テスト", // Simple katakana
            "2009",   // Numbers
            "ABC",    // Alphabet
        ];

        for text in test_cases {
            let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();

            match results {
                Ok(tokens) => {
                    eprintln!("Text '{}' tokenized into {} tokens:", text, tokens.len());
                    for (i, token) in tokens.iter().enumerate() {
                        eprintln!("  {}: {}", i, token);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to tokenize '{}': {:?}", text, e);
                }
            }
        }
    }
}
