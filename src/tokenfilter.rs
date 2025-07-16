use crate::{RunomeError, Token};
use std::collections::HashMap;

/// Core trait for token filtering operations
/// Mirrors Python Janome's TokenFilter abstract base class
///
/// TokenFilters can transform tokens in various ways:
/// - Modify existing token fields (case conversion, field updates)
/// - Filter tokens based on criteria (POS filtering)
/// - Combine tokens (compound noun formation)
/// - Extract specific attributes (terminal filters)
/// - Count token frequencies (terminal filters)
pub trait TokenFilter {
    type Output;

    /// Apply the filter to a stream of tokens
    /// Returns an iterator over the filtered/transformed output
    fn apply<I>(&self, tokens: I) -> Box<dyn Iterator<Item = Self::Output>>
    where
        I: Iterator<Item = Token> + 'static;
}

/// Converts surface and base_form fields to lowercase
///
/// This filter modifies tokens in place, converting the `surface` and `base_form`
/// fields to lowercase using Unicode-aware conversion. Useful for case-insensitive
/// text processing and normalization.
///
/// # Example
/// ```rust
/// use runome::{LowerCaseFilter, TokenFilter};
/// let filter = LowerCaseFilter;
/// // Apply to token stream: "Python" -> "python"
/// ```
#[derive(Debug, Clone)]
pub struct LowerCaseFilter;

impl TokenFilter for LowerCaseFilter {
    type Output = Token;

    fn apply<I>(&self, tokens: I) -> Box<dyn Iterator<Item = Token>>
    where
        I: Iterator<Item = Token> + 'static,
    {
        let iter = tokens.map(|token| {
            // Create new token with lowercase surface and base_form
            let surface = token.surface().to_lowercase();
            let base_form = token.base_form().to_lowercase();

            Token::new(
                surface,
                token.part_of_speech().to_string(),
                token.infl_type().to_string(),
                token.infl_form().to_string(),
                base_form,
                token.reading().to_string(),
                token.phonetic().to_string(),
                token.node_type(),
            )
        });
        Box::new(iter)
    }
}

/// Converts surface and base_form fields to uppercase
///
/// This filter modifies tokens in place, converting the `surface` and `base_form`
/// fields to uppercase using Unicode-aware conversion. Mirror of LowerCaseFilter.
///
/// # Example
/// ```rust
/// use runome::{UpperCaseFilter, TokenFilter};
/// let filter = UpperCaseFilter;
/// // Apply to token stream: "python" -> "PYTHON"
/// ```
#[derive(Debug, Clone)]
pub struct UpperCaseFilter;

impl TokenFilter for UpperCaseFilter {
    type Output = Token;

    fn apply<I>(&self, tokens: I) -> Box<dyn Iterator<Item = Token>>
    where
        I: Iterator<Item = Token> + 'static,
    {
        let iter = tokens.map(|token| {
            // Create new token with uppercase surface and base_form
            let surface = token.surface().to_uppercase();
            let base_form = token.base_form().to_uppercase();

            Token::new(
                surface,
                token.part_of_speech().to_string(),
                token.infl_type().to_string(),
                token.infl_form().to_string(),
                base_form,
                token.reading().to_string(),
                token.phonetic().to_string(),
                token.node_type(),
            )
        });
        Box::new(iter)
    }
}

/// Removes tokens with specified part-of-speech prefixes
///
/// This filter removes tokens whose part-of-speech tags start with any of the
/// specified prefixes. Uses prefix matching to handle hierarchical POS tags.
///
/// # Example
/// ```rust
/// use runome::POSStopFilter;
/// let filter = POSStopFilter::new(vec!["助詞".to_string(), "記号".to_string()]);
/// // Removes particles and symbols
/// ```
#[derive(Debug, Clone)]
pub struct POSStopFilter {
    pos_list: Vec<String>,
}

impl POSStopFilter {
    /// Create a new POSStopFilter with the specified POS prefixes to remove
    pub fn new(pos_list: Vec<String>) -> Self {
        Self { pos_list }
    }
}

impl TokenFilter for POSStopFilter {
    type Output = Token;

    fn apply<I>(&self, tokens: I) -> Box<dyn Iterator<Item = Token>>
    where
        I: Iterator<Item = Token> + 'static,
    {
        let pos_list = self.pos_list.clone();
        let iter = tokens.filter(move |token| {
            // Keep tokens that do NOT match any POS prefix
            !pos_list
                .iter()
                .any(|pos| token.part_of_speech().starts_with(pos))
        });
        Box::new(iter)
    }
}

/// Keeps only tokens with specified part-of-speech prefixes
///
/// This filter keeps only tokens whose part-of-speech tags start with any of the
/// specified prefixes. Inverse of POSStopFilter.
///
/// # Example
/// ```rust
/// use runome::POSKeepFilter;
/// let filter = POSKeepFilter::new(vec!["名詞".to_string(), "動詞".to_string()]);
/// // Keeps only nouns and verbs
/// ```
#[derive(Debug, Clone)]
pub struct POSKeepFilter {
    pos_list: Vec<String>,
}

impl POSKeepFilter {
    /// Create a new POSKeepFilter with the specified POS prefixes to keep
    pub fn new(pos_list: Vec<String>) -> Self {
        Self { pos_list }
    }
}

impl TokenFilter for POSKeepFilter {
    type Output = Token;

    fn apply<I>(&self, tokens: I) -> Box<dyn Iterator<Item = Token>>
    where
        I: Iterator<Item = Token> + 'static,
    {
        let pos_list = self.pos_list.clone();
        let iter = tokens.filter(move |token| {
            // Keep tokens that DO match any POS prefix
            pos_list
                .iter()
                .any(|pos| token.part_of_speech().starts_with(pos))
        });
        Box::new(iter)
    }
}

/// Combines contiguous noun tokens into compound nouns
///
/// This filter detects sequences of adjacent tokens with part-of-speech tags
/// starting with "名詞" (noun) and combines them into compound noun tokens.
/// The resulting token has:
/// - Combined surface forms
/// - Combined base forms, reading, and phonetic fields  
/// - Part-of-speech set to "名詞,複合,*,*"
///
/// # Example
/// ```rust
/// use runome::CompoundNounFilter;
/// let filter = CompoundNounFilter;
/// // "東京" + "駅" -> "東京駅" with POS "名詞,複合,*,*"
/// ```
#[derive(Debug, Clone)]
pub struct CompoundNounFilter;

impl TokenFilter for CompoundNounFilter {
    type Output = Token;

    fn apply<I>(&self, tokens: I) -> Box<dyn Iterator<Item = Token>>
    where
        I: Iterator<Item = Token> + 'static,
    {
        Box::new(CompoundNounIterator::new(tokens))
    }
}

/// Iterator that implements the stateful compound noun logic
struct CompoundNounIterator<I: Iterator<Item = Token>> {
    tokens: std::iter::Peekable<I>,
    pending: Option<Token>,
}

impl<I> CompoundNounIterator<I>
where
    I: Iterator<Item = Token>,
{
    fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            pending: None,
        }
    }

    fn is_noun(token: &Token) -> bool {
        token.part_of_speech().starts_with("名詞")
    }

    fn combine_tokens(first: Token, second: Token) -> Token {
        // Combine all relevant fields
        let surface = format!("{}{}", first.surface(), second.surface());
        let base_form = format!("{}{}", first.base_form(), second.base_form());
        let reading = format!("{}{}", first.reading(), second.reading());
        let phonetic = format!("{}{}", first.phonetic(), second.phonetic());

        Token::new(
            surface,
            "名詞,複合,*,*".to_string(), // Compound noun POS
            "*".to_string(),
            "*".to_string(),
            base_form,
            reading,
            phonetic,
            first.node_type(),
        )
    }
}

impl<I> Iterator for CompoundNounIterator<I>
where
    I: Iterator<Item = Token>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // If we have a pending token, process it
        if let Some(current) = self.pending.take() {
            if Self::is_noun(&current) {
                // Look ahead to see if next token is also a noun
                if let Some(next_token) = self.tokens.peek() {
                    if Self::is_noun(next_token) {
                        // Combine current with next token
                        let next = self.tokens.next().unwrap();
                        let combined = Self::combine_tokens(current, next);
                        self.pending = Some(combined);
                        return self.next(); // Recursively process the combined token
                    }
                }
            }
            return Some(current);
        }

        // Get next token from the iterator
        if let Some(token) = self.tokens.next() {
            self.pending = Some(token);
            self.next()
        } else {
            None
        }
    }
}

/// Extracts specific token attributes as strings (terminal filter)
///
/// This is a terminal filter that extracts a specific attribute from each token
/// and returns it as a string. The output type changes from Token to String,
/// so this filter cannot be followed by other token filters.
///
/// Valid attributes: surface, part_of_speech, infl_type, infl_form, base_form, reading, phonetic
///
/// # Example
/// ```rust
/// use runome::ExtractAttributeFilter;
/// let filter = ExtractAttributeFilter::new("surface".to_string()).unwrap();
/// // Returns iterator over surface forms as strings
/// ```
#[derive(Debug, Clone)]
pub struct ExtractAttributeFilter {
    attribute: String,
}

impl ExtractAttributeFilter {
    /// Create a new ExtractAttributeFilter for the specified attribute
    ///
    /// # Arguments
    /// * `attribute` - The token attribute to extract
    ///
    /// # Returns
    /// * `Ok(ExtractAttributeFilter)` if the attribute is valid
    /// * `Err(RunomeError)` if the attribute is invalid
    pub fn new(attribute: String) -> Result<Self, RunomeError> {
        // Validate attribute name
        match attribute.as_str() {
            "surface" | "part_of_speech" | "infl_type" | "infl_form" | "base_form" | "reading"
            | "phonetic" => Ok(Self { attribute }),
            _ => Err(RunomeError::DictValidationError {
                reason: format!(
                    "Invalid attribute '{}'. Valid attributes are: surface, part_of_speech, infl_type, infl_form, base_form, reading, phonetic",
                    attribute
                ),
            }),
        }
    }
}

impl TokenFilter for ExtractAttributeFilter {
    type Output = String;

    fn apply<I>(&self, tokens: I) -> Box<dyn Iterator<Item = String>>
    where
        I: Iterator<Item = Token> + 'static,
    {
        let attr = self.attribute.clone();
        let iter = tokens.map(move |token| {
            match attr.as_str() {
                "surface" => token.surface().to_string(),
                "part_of_speech" => token.part_of_speech().to_string(),
                "infl_type" => token.infl_type().to_string(),
                "infl_form" => token.infl_form().to_string(),
                "base_form" => token.base_form().to_string(),
                "reading" => token.reading().to_string(),
                "phonetic" => token.phonetic().to_string(),
                _ => String::new(), // Should not happen due to validation
            }
        });
        Box::new(iter)
    }
}

/// Counts token frequencies (terminal filter)
///
/// This is a terminal filter that counts the frequency of the specified token
/// attribute and returns an iterator over (attribute_value, count) pairs.
/// The output type changes from Token to (String, usize).
///
/// # Example
/// ```rust
/// use runome::TokenCountFilter;
/// let filter = TokenCountFilter::new("surface".to_string(), false).unwrap();
/// // Returns iterator over (surface, count) pairs
/// ```
#[derive(Debug, Clone)]
pub struct TokenCountFilter {
    attribute: String,
    sorted: bool,
}

impl TokenCountFilter {
    /// Create a new TokenCountFilter for the specified attribute
    ///
    /// # Arguments
    /// * `attribute` - The token attribute to count
    /// * `sorted` - Whether to sort results by frequency (descending)
    ///
    /// # Returns
    /// * `Ok(TokenCountFilter)` if the attribute is valid
    /// * `Err(RunomeError)` if the attribute is invalid
    pub fn new(attribute: String, sorted: bool) -> Result<Self, RunomeError> {
        // Validate attribute name (same as ExtractAttributeFilter)
        match attribute.as_str() {
            "surface" | "part_of_speech" | "infl_type" | "infl_form" | "base_form" | "reading"
            | "phonetic" => Ok(Self { attribute, sorted }),
            _ => Err(RunomeError::DictValidationError {
                reason: format!(
                    "Invalid attribute '{}'. Valid attributes are: surface, part_of_speech, infl_type, infl_form, base_form, reading, phonetic",
                    attribute
                ),
            }),
        }
    }
}

impl TokenFilter for TokenCountFilter {
    type Output = (String, usize);

    fn apply<I>(&self, tokens: I) -> Box<dyn Iterator<Item = (String, usize)>>
    where
        I: Iterator<Item = Token> + 'static,
    {
        let attr = self.attribute.clone();
        let sorted = self.sorted;

        // Collect all tokens and count frequencies
        let tokens: Vec<Token> = tokens.collect();
        let mut counts: HashMap<String, usize> = HashMap::new();

        for token in tokens {
            let value = match attr.as_str() {
                "surface" => token.surface().to_string(),
                "part_of_speech" => token.part_of_speech().to_string(),
                "infl_type" => token.infl_type().to_string(),
                "infl_form" => token.infl_form().to_string(),
                "base_form" => token.base_form().to_string(),
                "reading" => token.reading().to_string(),
                "phonetic" => token.phonetic().to_string(),
                _ => String::new(), // Should not happen due to validation
            };
            *counts.entry(value).or_insert(0) += 1;
        }

        // Convert to vector of pairs
        let mut result: Vec<(String, usize)> = counts.into_iter().collect();

        if sorted {
            // Sort by frequency descending, then by key ascending for stability
            result.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        }

        Box::new(result.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeType;

    fn create_test_token(surface: &str, pos: &str, base_form: &str) -> Token {
        Token::new(
            surface.to_string(),
            pos.to_string(),
            "*".to_string(),
            "*".to_string(),
            base_form.to_string(),
            "*".to_string(),
            "*".to_string(),
            NodeType::SysDict,
        )
    }

    #[test]
    fn test_lower_case_filter() {
        let filter = LowerCaseFilter;
        let tokens = vec![
            create_test_token("Python", "名詞,固有名詞", "Python"),
            create_test_token("JavaScript", "名詞,固有名詞", "JavaScript"),
        ];

        let results: Vec<Token> = filter.apply(tokens.into_iter()).collect();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].surface(), "python");
        assert_eq!(results[0].base_form(), "python");
        assert_eq!(results[1].surface(), "javascript");
        assert_eq!(results[1].base_form(), "javascript");
    }

    #[test]
    fn test_upper_case_filter() {
        let filter = UpperCaseFilter;
        let tokens = vec![create_test_token("python", "名詞,固有名詞", "python")];

        let results: Vec<Token> = filter.apply(tokens.into_iter()).collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].surface(), "PYTHON");
        assert_eq!(results[0].base_form(), "PYTHON");
    }

    #[test]
    fn test_pos_stop_filter() {
        let filter = POSStopFilter::new(vec!["助詞".to_string(), "記号".to_string()]);
        let tokens = vec![
            create_test_token("私", "名詞,代名詞", "私"),
            create_test_token("は", "助詞,係助詞", "は"),
            create_test_token("学生", "名詞,一般", "学生"),
            create_test_token("。", "記号,句点", "。"),
        ];

        let results: Vec<Token> = filter.apply(tokens.into_iter()).collect();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].surface(), "私");
        assert_eq!(results[1].surface(), "学生");
    }

    #[test]
    fn test_pos_keep_filter() {
        let filter = POSKeepFilter::new(vec!["名詞".to_string()]);
        let tokens = vec![
            create_test_token("私", "名詞,代名詞", "私"),
            create_test_token("は", "助詞,係助詞", "は"),
            create_test_token("学生", "名詞,一般", "学生"),
        ];

        let results: Vec<Token> = filter.apply(tokens.into_iter()).collect();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].surface(), "私");
        assert_eq!(results[1].surface(), "学生");
    }

    #[test]
    fn test_compound_noun_filter() {
        let filter = CompoundNounFilter;
        let tokens = vec![
            create_test_token("東京", "名詞,固有名詞", "東京"),
            create_test_token("駅", "名詞,一般", "駅"),
            create_test_token("で", "助詞,格助詞", "で"),
            create_test_token("羽田", "名詞,固有名詞", "羽田"),
            create_test_token("空港", "名詞,一般", "空港"),
        ];

        let results: Vec<Token> = filter.apply(tokens.into_iter()).collect();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].surface(), "東京駅");
        assert_eq!(results[0].part_of_speech(), "名詞,複合,*,*");
        assert_eq!(results[1].surface(), "で");
        assert_eq!(results[2].surface(), "羽田空港");
        assert_eq!(results[2].part_of_speech(), "名詞,複合,*,*");
    }

    #[test]
    fn test_extract_attribute_filter_surface() {
        let filter = ExtractAttributeFilter::new("surface".to_string()).unwrap();
        let tokens = vec![
            create_test_token("私", "名詞,代名詞", "私"),
            create_test_token("は", "助詞,係助詞", "は"),
        ];

        let results: Vec<String> = filter.apply(tokens.into_iter()).collect();

        assert_eq!(results, vec!["私", "は"]);
    }

    #[test]
    fn test_extract_attribute_filter_invalid() {
        let result = ExtractAttributeFilter::new("invalid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_token_count_filter_basic() {
        let filter = TokenCountFilter::new("surface".to_string(), false).unwrap();
        let tokens = vec![
            create_test_token("も", "助詞", "も"),
            create_test_token("も", "助詞", "も"),
            create_test_token("の", "助詞", "の"),
        ];

        let results: Vec<(String, usize)> = filter.apply(tokens.into_iter()).collect();

        assert_eq!(results.len(), 2);
        // Results may be in any order since we didn't sort
        let mut results = results;
        results.sort();
        assert_eq!(results[0], ("の".to_string(), 1));
        assert_eq!(results[1], ("も".to_string(), 2));
    }

    #[test]
    fn test_token_count_filter_basic_sorted() {
        let filter = TokenCountFilter::new("surface".to_string(), true).unwrap();
        let tokens = vec![
            create_test_token("も", "助詞", "も"),
            create_test_token("の", "助詞", "の"),
            create_test_token("も", "助詞", "も"),
        ];

        let results: Vec<(String, usize)> = filter.apply(tokens.into_iter()).collect();

        assert_eq!(results.len(), 2);
        // Results should be sorted by frequency descending
        assert_eq!(results[0], ("も".to_string(), 2));
        assert_eq!(results[1], ("の".to_string(), 1));
    }

    #[test]
    fn test_compound_noun_filter_japanese_text() {
        // Test equivalent to Python TestTokenFilter.test_compound_noun_filter()
        // Input: '浜松町駅から東京モノレールで羽田空港ターミナルへ向かう'
        // Expected: ['浜松町駅', 'から', '東京モノレール', 'で', '羽田空港ターミナル', 'へ', '向かう']

        let filter = CompoundNounFilter;

        // Create tokens that simulate the tokenization of the Japanese text
        // This represents how the text would be tokenized before the compound noun filter
        let tokens = vec![
            // "浜松町駅" - should be compound of 浜松町 + 駅
            create_test_token("浜松町", "名詞,固有名詞,地域,一般", "浜松町"),
            create_test_token("駅", "名詞,一般", "駅"),
            // "から" - particle, should remain separate
            create_test_token("から", "助詞,格助詞,一般", "から"),
            // "東京モノレール" - should be compound of 東京 + モノレール
            create_test_token("東京", "名詞,固有名詞,地域,一般", "東京"),
            create_test_token("モノレール", "名詞,一般", "モノレール"),
            // "で" - particle, should remain separate
            create_test_token("で", "助詞,格助詞,一般", "で"),
            // "羽田空港ターミナル" - should be compound of 羽田 + 空港 + ターミナル
            create_test_token("羽田", "名詞,固有名詞,地域,一般", "羽田"),
            create_test_token("空港", "名詞,一般", "空港"),
            create_test_token("ターミナル", "名詞,一般", "ターミナル"),
            // "へ" - particle, should remain separate
            create_test_token("へ", "助詞,格助詞,一般", "へ"),
            // "向かう" - verb, should remain separate
            create_test_token("向かう", "動詞,自立", "向かう"),
        ];

        let results: Vec<Token> = filter.apply(tokens.into_iter()).collect();

        // Extract surface forms for comparison
        let surfaces: Vec<&str> = results.iter().map(|token| token.surface()).collect();

        // Validate the expected compound noun formation
        let expected = vec![
            "浜松町駅",
            "から",
            "東京モノレール",
            "で",
            "羽田空港ターミナル",
            "へ",
            "向かう",
        ];
        assert_eq!(surfaces, expected);

        // Validate that compound nouns have the correct POS tag
        assert_eq!(results[0].part_of_speech(), "名詞,複合,*,*"); // 浜松町駅
        assert_eq!(results[2].part_of_speech(), "名詞,複合,*,*"); // 東京モノレール
        assert_eq!(results[4].part_of_speech(), "名詞,複合,*,*"); // 羽田空港ターミナル

        // Validate that non-noun tokens retain their original POS
        assert_eq!(results[1].part_of_speech(), "助詞,格助詞,一般"); // から
        assert_eq!(results[3].part_of_speech(), "助詞,格助詞,一般"); // で
        assert_eq!(results[5].part_of_speech(), "助詞,格助詞,一般"); // へ
        assert_eq!(results[6].part_of_speech(), "動詞,自立"); // 向かう
    }

    #[test]
    fn test_count_token_filter() {
        // Test equivalent to Python TestTokenFilter.test_count_token_filter()

        // Test 1: Basic token counting with surface forms (default)
        // Input: 'すもももももももものうち' - Japanese text meaning "plums and peaches are among peaches"
        let filter = TokenCountFilter::new("surface".to_string(), false).unwrap();

        // Create tokens that simulate the tokenization of 'すもももももももものうち'
        let tokens = vec![
            create_test_token("すもも", "名詞,一般", "すもも"), // 1 occurrence
            create_test_token("も", "助詞,係助詞", "も"),       // 1st occurrence
            create_test_token("もも", "名詞,一般", "もも"),     // 1st occurrence
            create_test_token("も", "助詞,係助詞", "も"),       // 2nd occurrence
            create_test_token("もも", "名詞,一般", "もも"),     // 2nd occurrence
            create_test_token("の", "助詞,連体化", "の"),       // 1 occurrence
            create_test_token("うち", "名詞,非自立,副詞可能", "うち"), // 1 occurrence
        ];

        let results: Vec<(String, usize)> = filter.apply(tokens.into_iter()).collect();

        // Convert to HashMap for easier validation (since order is not guaranteed when not sorted)
        let counts: std::collections::HashMap<String, usize> = results.into_iter().collect();

        // Validate expected counts - matching Python test assertions
        assert_eq!(counts.get("すもも"), Some(&1));
        assert_eq!(counts.get("もも"), Some(&2));
        assert_eq!(counts.get("も"), Some(&2));
        assert_eq!(counts.get("の"), Some(&1));
        assert_eq!(counts.get("うち"), Some(&1));
        assert_eq!(counts.len(), 5); // Should have exactly 5 unique tokens
    }

    #[test]
    fn test_count_token_filter_sorted() {
        // Test 2: Sorted mode testing
        let filter = TokenCountFilter::new("surface".to_string(), true).unwrap();

        // Same tokens as above
        let tokens = vec![
            create_test_token("すもも", "名詞,一般", "すもも"),
            create_test_token("も", "助詞,係助詞", "も"),
            create_test_token("もも", "名詞,一般", "もも"),
            create_test_token("も", "助詞,係助詞", "も"),
            create_test_token("もも", "名詞,一般", "もも"),
            create_test_token("の", "助詞,連体化", "の"),
            create_test_token("うち", "名詞,非自立,副詞可能", "うち"),
        ];

        let results: Vec<(String, usize)> = filter.apply(tokens.into_iter()).collect();

        // Extract just the counts to validate sorting (frequencies should be [2, 2, 1, 1, 1])
        let frequencies: Vec<usize> = results.iter().map(|(_, count)| *count).collect();
        assert_eq!(frequencies, vec![2, 2, 1, 1, 1]); // Sorted by frequency descending

        // Validate that items with count 2 come first
        assert_eq!(results[0].1, 2); // First item has count 2
        assert_eq!(results[1].1, 2); // Second item has count 2
        assert_eq!(results[2].1, 1); // Third item has count 1
        assert_eq!(results[3].1, 1); // Fourth item has count 1  
        assert_eq!(results[4].1, 1); // Fifth item has count 1
    }

    #[test]
    fn test_count_token_filter_base_form() {
        // Test 3: Base form attribute testing
        // Input: 'CountFilterで簡単に単語数が数えられます' - Japanese text about counting words
        let filter = TokenCountFilter::new("base_form".to_string(), false).unwrap();

        // Create tokens that simulate the tokenization with base forms
        let tokens = vec![
            create_test_token("CountFilter", "名詞,固有名詞", "CountFilter"),
            create_test_token("で", "助詞,格助詞", "で"),
            create_test_token("簡単", "名詞,形容動詞語幹", "簡単"),
            create_test_token("に", "助詞,格助詞", "に"),
            create_test_token("単語", "名詞,一般", "単語"),
            create_test_token("数", "名詞,一般", "数"),
            create_test_token("が", "助詞,格助詞", "が"),
            create_test_token("数えられ", "動詞,自立", "数える"), // base_form differs from surface
            create_test_token("られる", "動詞,接尾", "られる"),
            create_test_token("ます", "助動詞", "ます"),
        ];

        let results: Vec<(String, usize)> = filter.apply(tokens.into_iter()).collect();
        let counts: std::collections::HashMap<String, usize> = results.into_iter().collect();

        // Validate that each base form appears exactly once
        assert_eq!(counts.get("CountFilter"), Some(&1));
        assert_eq!(counts.get("で"), Some(&1));
        assert_eq!(counts.get("簡単"), Some(&1));
        assert_eq!(counts.get("に"), Some(&1));
        assert_eq!(counts.get("単語"), Some(&1));
        assert_eq!(counts.get("数"), Some(&1));
        assert_eq!(counts.get("が"), Some(&1));
        assert_eq!(counts.get("数える"), Some(&1)); // Note: base form, not surface "数えられ"
        assert_eq!(counts.get("られる"), Some(&1));
        assert_eq!(counts.get("ます"), Some(&1));
        assert_eq!(counts.len(), 10); // Should have exactly 10 unique base forms
    }

    #[test]
    fn test_count_token_filter_invalid_attribute() {
        // Test 4: Invalid attribute error testing
        let result = TokenCountFilter::new("foo".to_string(), false);
        assert!(result.is_err());

        // Also test that the error message is informative
        if let Err(error) = result {
            let error_msg = format!("{:?}", error);
            assert!(error_msg.contains("Invalid attribute"));
            assert!(error_msg.contains("foo"));
        }
    }
}
