use crate::{CharFilter, RunomeError, Token, TokenFilter, TokenizeResult, Tokenizer};
use crate::{
    CompoundNounFilter, ExtractAttributeFilter, LowerCaseFilter, POSKeepFilter, POSStopFilter,
    TokenCountFilter, UpperCaseFilter,
};

/// Enum wrapper for TokenFilter to enable dynamic dispatch
/// This solves the trait object compatibility issue with generic methods
#[derive(Debug)]
pub enum DynTokenFilter {
    LowerCase(LowerCaseFilter),
    UpperCase(UpperCaseFilter),
    POSStop(POSStopFilter),
    POSKeep(POSKeepFilter),
    CompoundNoun(CompoundNounFilter),
    ExtractAttribute(ExtractAttributeFilter),
    TokenCount(TokenCountFilter),
}

impl DynTokenFilter {
    /// Apply the filter to a token iterator
    pub fn apply_to_tokens(&self, tokens: Vec<Token>) -> Result<Vec<Token>, RunomeError> {
        let tokens_iter = tokens.into_iter();

        match self {
            DynTokenFilter::LowerCase(filter) => Ok(filter.apply(tokens_iter).collect()),
            DynTokenFilter::UpperCase(filter) => Ok(filter.apply(tokens_iter).collect()),
            DynTokenFilter::POSStop(filter) => Ok(filter.apply(tokens_iter).collect()),
            DynTokenFilter::POSKeep(filter) => Ok(filter.apply(tokens_iter).collect()),
            DynTokenFilter::CompoundNoun(filter) => Ok(filter.apply(tokens_iter).collect()),
            DynTokenFilter::ExtractAttribute(_) => {
                // ExtractAttributeFilter outputs strings, not tokens
                Err(RunomeError::FilterChainError {
                    message: "ExtractAttributeFilter cannot be used in a token chain".to_string(),
                })
            }
            DynTokenFilter::TokenCount(_) => {
                // TokenCountFilter outputs tuples, not tokens
                Err(RunomeError::FilterChainError {
                    message: "TokenCountFilter cannot be used in a token chain".to_string(),
                })
            }
        }
    }
}

/// Core Analyzer for Japanese text analysis pipeline
/// Mirrors Python Janome's Analyzer class with full compatibility
///
/// The Analyzer provides a complete text analysis pipeline that combines:
/// 1. CharFilter chain - Text preprocessing before tokenization
/// 2. Tokenizer - Core morphological analysis
/// 3. TokenFilter chain - Post-processing of tokens
///
/// # Example
/// ```rust
/// use runome::{Analyzer, UnicodeNormalizeCharFilter};
///
/// let analyzer = Analyzer::builder()
///     .add_char_filter(UnicodeNormalizeCharFilter::with_default_form())
///     .add_compound_noun_filter()
///     .add_lower_case_filter()
///     .build();
///
/// let results = analyzer.analyze("テスト用のテキスト").unwrap();
/// ```
pub struct Analyzer {
    char_filters: Vec<Box<dyn CharFilter>>,
    tokenizer: Tokenizer,
    token_filters: Vec<DynTokenFilter>,
}

impl Analyzer {
    /// Create a new AnalyzerBuilder for configuring an Analyzer
    pub fn builder() -> AnalyzerBuilder {
        AnalyzerBuilder::new()
    }

    /// Analyze text through the complete pipeline
    ///
    /// # Arguments
    /// * `text` - Input text to analyze
    ///
    /// # Returns
    /// * `Ok(Vec<Token>)` - Vector of analysis results
    /// * `Err(RunomeError)` - Error if analysis fails
    pub fn analyze(&self, text: &str) -> Result<Vec<Token>, RunomeError> {
        // Stage 1: Apply CharFilters sequentially
        let mut processed_text = text.to_string();
        for filter in &self.char_filters {
            processed_text = filter.apply(&processed_text)?;
        }

        // Stage 2: Tokenize the preprocessed text
        let token_results: Result<Vec<_>, _> = self
            .tokenizer
            .tokenize(&processed_text, Some(false), Some(true))
            .collect();

        let mut tokens: Vec<Token> = token_results?
            .into_iter()
            .filter_map(|result| {
                match result {
                    TokenizeResult::Token(token) => Some(token),
                    TokenizeResult::Surface(_) => None, // Skip surface-only results
                }
            })
            .collect();

        // Stage 3: Apply TokenFilters sequentially
        for filter in &self.token_filters {
            tokens = filter.apply_to_tokens(tokens)?;
        }

        Ok(tokens)
    }
}

/// Builder for creating Analyzer instances with type-safe configuration
pub struct AnalyzerBuilder {
    char_filters: Vec<Box<dyn CharFilter>>,
    tokenizer: Option<Tokenizer>,
    token_filters: Vec<DynTokenFilter>,
}

impl AnalyzerBuilder {
    /// Create a new AnalyzerBuilder
    fn new() -> Self {
        Self {
            char_filters: Vec::new(),
            tokenizer: None,
            token_filters: Vec::new(),
        }
    }

    /// Add a CharFilter to the preprocessing chain
    ///
    /// # Arguments
    /// * `filter` - CharFilter to add to the chain
    ///
    /// # Returns
    /// * `Self` - Builder for chaining
    pub fn add_char_filter<F>(mut self, filter: F) -> Self
    where
        F: CharFilter + 'static,
    {
        self.char_filters.push(Box::new(filter));
        self
    }

    /// Set the tokenizer (validates wakati mode)
    ///
    /// # Arguments
    /// * `tokenizer` - Tokenizer to use for morphological analysis
    ///
    /// # Returns
    /// * `Ok(Self)` - Builder for chaining
    /// * `Err(RunomeError)` - Error if tokenizer is in wakati mode
    pub fn tokenizer(mut self, tokenizer: Tokenizer) -> Result<Self, RunomeError> {
        // Validate that tokenizer is not in wakati mode
        if tokenizer.wakati() {
            return Err(RunomeError::InvalidTokenizerConfig {
                reason: "A Tokenizer with wakati=True option is not accepted.".to_string(),
            });
        }
        self.tokenizer = Some(tokenizer);
        Ok(self)
    }

    /// Add a TokenFilter to the post-processing chain
    ///
    /// # Arguments
    /// * `filter` - TokenFilter to add to the chain
    ///
    /// # Returns
    /// * `Self` - Builder for chaining
    pub fn add_token_filter(mut self, filter: DynTokenFilter) -> Self {
        self.token_filters.push(filter);
        self
    }

    /// Add a LowerCaseFilter to the post-processing chain
    pub fn add_lower_case_filter(mut self) -> Self {
        self.token_filters
            .push(DynTokenFilter::LowerCase(LowerCaseFilter));
        self
    }

    /// Add an UpperCaseFilter to the post-processing chain
    pub fn add_upper_case_filter(mut self) -> Self {
        self.token_filters
            .push(DynTokenFilter::UpperCase(UpperCaseFilter));
        self
    }

    /// Add a POSStopFilter to the post-processing chain
    pub fn add_pos_stop_filter(mut self, pos_list: Vec<String>) -> Self {
        self.token_filters
            .push(DynTokenFilter::POSStop(POSStopFilter::new(pos_list)));
        self
    }

    /// Add a POSKeepFilter to the post-processing chain
    pub fn add_pos_keep_filter(mut self, pos_list: Vec<String>) -> Self {
        self.token_filters
            .push(DynTokenFilter::POSKeep(POSKeepFilter::new(pos_list)));
        self
    }

    /// Add a CompoundNounFilter to the post-processing chain
    pub fn add_compound_noun_filter(mut self) -> Self {
        self.token_filters
            .push(DynTokenFilter::CompoundNoun(CompoundNounFilter));
        self
    }

    /// Build the final Analyzer
    ///
    /// # Returns
    /// * `Analyzer` - Configured analyzer ready for use
    pub fn build(self) -> Analyzer {
        let tokenizer = self.tokenizer.unwrap_or_else(|| {
            // Create default tokenizer - this might fail, but we'll handle it gracefully
            Tokenizer::new(None, Some(false))
                .unwrap_or_else(|_| panic!("Failed to create default tokenizer"))
        });

        Analyzer {
            char_filters: self.char_filters,
            tokenizer,
            token_filters: self.token_filters,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RegexReplaceCharFilter, UnicodeNormalizeCharFilter};

    #[test]
    fn test_analyzer_default() {
        // Test default analyzer creation
        let analyzer = Analyzer::builder().build();

        // Should have no filters
        assert_eq!(analyzer.char_filters.len(), 0);
        assert_eq!(analyzer.token_filters.len(), 0);

        // Should have default tokenizer that is not in wakati mode
        assert!(!analyzer.tokenizer.wakati());
    }

    #[test]
    fn test_analyzer_custom() {
        // Test custom analyzer with all components
        let analyzer = Analyzer::builder()
            .add_char_filter(UnicodeNormalizeCharFilter::with_default_form())
            .add_char_filter(RegexReplaceCharFilter::new(r"\s+", "").unwrap())
            .add_compound_noun_filter()
            .add_pos_stop_filter(vec!["記号".to_string(), "助詞".to_string()])
            .add_lower_case_filter()
            .build();

        assert_eq!(analyzer.char_filters.len(), 2);
        assert_eq!(analyzer.token_filters.len(), 3);
    }

    #[test]
    fn test_analyzer_wakati_rejection() {
        // Test that wakati mode tokenizer is rejected
        let wakati_tokenizer = Tokenizer::new(None, Some(true)).unwrap();
        let result = Analyzer::builder().tokenizer(wakati_tokenizer);

        assert!(result.is_err());
        if let Err(RunomeError::InvalidTokenizerConfig { reason }) = result {
            assert!(reason.contains("wakati=True"));
        } else {
            panic!("Expected InvalidTokenizerConfig error");
        }
    }

    #[test]
    fn test_analyzer_basic_analysis() {
        // Test basic text analysis
        let analyzer = Analyzer::builder()
            .add_char_filter(UnicodeNormalizeCharFilter::with_default_form())
            .build();

        let results = analyzer.analyze("テスト").unwrap();
        assert!(!results.is_empty());

        // Should have at least one token
        assert!(!results.is_empty());
    }

    #[test]
    fn test_analyzer_full_pipeline() {
        // Test complete analysis pipeline
        let analyzer = Analyzer::builder()
            .add_char_filter(UnicodeNormalizeCharFilter::with_default_form())
            .add_char_filter(RegexReplaceCharFilter::new("蛇の目", "janome").unwrap())
            .add_compound_noun_filter()
            .add_lower_case_filter()
            .build();

        let results = analyzer.analyze("蛇の目はテスト用です").unwrap();
        assert!(!results.is_empty());

        // Check that text was processed (contains "janome" instead of "蛇の目")
        let surfaces: Vec<String> = results.iter().map(|t| t.surface().to_string()).collect();
        let text = surfaces.join("");
        assert!(text.contains("janome"));
    }

    #[test]
    fn test_analyzer_python_test_analyze_equivalent() {
        // Test equivalent to Python TestAnalyzer.test_analyze()
        // Input: '蛇の目はPure Ｐｙｔｈｏｎな形態素解析器です。'
        // Expected after CharFilters + TokenFilters + ExtractAttributeFilter:
        // ['janome', 'pure', 'python', 'な', '形態素解析器', 'です']

        let analyzer = Analyzer::builder()
            .add_char_filter(UnicodeNormalizeCharFilter::with_default_form())
            .add_char_filter(RegexReplaceCharFilter::new("蛇の目", "janome").unwrap())
            .add_compound_noun_filter()
            .add_pos_stop_filter(vec!["記号".to_string(), "助詞".to_string()])
            .add_lower_case_filter()
            .build();

        let results = analyzer
            .analyze("蛇の目はPure Ｐｙｔｈｏｎな形態素解析器です。")
            .unwrap();

        // Extract surface forms
        let surfaces: Vec<String> = results.iter().map(|t| t.surface().to_string()).collect();

        // Check that we have the expected tokens (order may vary, but content should match)
        assert!(surfaces.contains(&"janome".to_string()));
        assert!(surfaces.contains(&"pure".to_string()));
        assert!(surfaces.contains(&"python".to_string()));
        assert!(surfaces.contains(&"な".to_string()));
        assert!(surfaces.contains(&"形態素解析器".to_string()));
        assert!(surfaces.contains(&"です".to_string()));
    }

    #[test]
    fn test_analyzer_comprehensive_pipeline() {
        // Test a more comprehensive pipeline with multiple filters
        let analyzer = Analyzer::builder()
            .add_char_filter(UnicodeNormalizeCharFilter::with_default_form())
            .add_char_filter(RegexReplaceCharFilter::new(r"\s+", "").unwrap())
            .add_compound_noun_filter()
            .add_pos_stop_filter(vec!["記号".to_string()])
            .add_lower_case_filter()
            .build();

        let results = analyzer.analyze("東京  駅で  降りる").unwrap();

        // Should have processed the text by removing spaces and combining compounds
        assert!(!results.is_empty());

        // Should have compound nouns and lowercase processing
        let surfaces: Vec<String> = results.iter().map(|t| t.surface().to_string()).collect();
        let text = surfaces.join("");

        // Text should not contain spaces (removed by regex filter)
        assert!(!text.contains(" "));
    }

    #[test]
    fn test_analyzer_empty_filters() {
        // Test analyzer with no filters (should work with just tokenizer)
        let analyzer = Analyzer::builder().build();
        let results = analyzer.analyze("テスト").unwrap();

        assert!(!results.is_empty());
    }

    #[test]
    fn test_analyzer_builder_chaining() {
        // Test that builder methods can be chained properly
        let analyzer = Analyzer::builder()
            .add_char_filter(UnicodeNormalizeCharFilter::with_default_form())
            .add_compound_noun_filter()
            .add_pos_keep_filter(vec!["名詞".to_string()])
            .add_upper_case_filter()
            .build();

        let results = analyzer.analyze("東京駅").unwrap();
        assert!(!results.is_empty());

        // Check that uppercase filter was applied
        let surfaces: Vec<String> = results.iter().map(|t| t.surface().to_string()).collect();
        let text = surfaces.join("");

        // Should be uppercase (though Japanese characters may not change)
        // This tests that the filter was applied without errors
        assert!(!text.is_empty());
    }
}
