use crate::RunomeError;
use regex::Regex;
use unicode_normalization::{UnicodeNormalization, is_nfc, is_nfd, is_nfkc, is_nfkd};

/// Core trait for character filtering operations
/// Mirrors Python Janome's CharFilter abstract base class
///
/// CharFilters transform input text before tokenization occurs.
/// They provide a simple string-to-string transformation interface
/// and can be chained together for complex preprocessing operations.
///
/// # Example
/// ```rust
/// use runome::{CharFilter, UnicodeNormalizeCharFilter};
/// let filter = UnicodeNormalizeCharFilter::with_default_form();
/// let result = filter.apply("Ｐｙｔｈｏｎ").unwrap();
/// assert_eq!(result, "Python");
/// ```
pub trait CharFilter {
    /// Apply the filter to input text
    /// Returns the transformed text
    fn apply(&self, text: &str) -> Result<String, RunomeError>;

    /// Convenience method for direct calling (mimics Python __call__)
    fn call(&self, text: &str) -> Result<String, RunomeError> {
        self.apply(text)
    }
}

/// Replaces text patterns using regular expressions
/// Mirrors Python's RegexReplaceCharFilter
///
/// This filter uses regular expressions to find and replace text patterns.
/// The pattern is compiled once during construction for efficiency.
///
/// # Example
/// ```rust
/// use runome::{CharFilter, RegexReplaceCharFilter};
/// let filter = RegexReplaceCharFilter::new("蛇の目", "janome").unwrap();
/// let result = filter.apply("蛇の目は形態素解析器です。").unwrap();
/// assert_eq!(result, "janomeは形態素解析器です。");
/// ```
#[derive(Debug, Clone)]
pub struct RegexReplaceCharFilter {
    pattern: Regex,
    replacement: String,
}

impl RegexReplaceCharFilter {
    /// Create a new RegexReplaceCharFilter
    ///
    /// # Arguments
    /// * `pattern` - Regular expression pattern string
    /// * `replacement` - Replacement string (supports \1, \2, etc. for backreferences)
    ///
    /// # Returns
    /// * `Ok(RegexReplaceCharFilter)` if pattern is valid
    /// * `Err(RunomeError)` if pattern is invalid
    pub fn new(pattern: &str, replacement: &str) -> Result<Self, RunomeError> {
        let regex = Regex::new(pattern).map_err(|source| RunomeError::InvalidRegexPattern {
            pattern: pattern.to_string(),
            source,
        })?;

        // Convert Python-style backreferences (\1, \2, etc.) to Rust regex format ($1, $2, etc.)
        let rust_replacement = Self::convert_backreferences(replacement);

        Ok(Self {
            pattern: regex,
            replacement: rust_replacement,
        })
    }

    /// Convert Python-style backreferences (\1, \2, etc.) to Rust regex format ($1, $2, etc.)
    fn convert_backreferences(replacement: &str) -> String {
        // Use regex to replace \1, \2, etc. with $1, $2, etc.
        let backref_regex = Regex::new(r"\\(\d+)").unwrap();
        backref_regex.replace_all(replacement, "$$$1").to_string()
    }
}

impl CharFilter for RegexReplaceCharFilter {
    fn apply(&self, text: &str) -> Result<String, RunomeError> {
        Ok(self
            .pattern
            .replace_all(text, &self.replacement)
            .to_string())
    }
}

/// Unicode normalization for text standardization
/// Mirrors Python's UnicodeNormalizeCharFilter
///
/// This filter normalizes Unicode text according to the specified normalization form.
/// It's particularly useful for Japanese text processing to convert between
/// fullwidth/halfwidth characters and normalize composed/decomposed forms.
///
/// # Example
/// ```rust
/// use runome::{CharFilter, UnicodeNormalizeCharFilter};
/// let filter = UnicodeNormalizeCharFilter::with_default_form();
/// let result = filter.apply("Ｐｙｔｈｏｎ").unwrap();
/// assert_eq!(result, "Python");
/// ```
#[derive(Debug, Clone)]
pub struct UnicodeNormalizeCharFilter {
    form: NormalizationForm,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
enum NormalizationForm {
    NFC,
    NFKC,
    NFD,
    NFKD,
}

impl UnicodeNormalizeCharFilter {
    /// Create a new UnicodeNormalizeCharFilter
    ///
    /// # Arguments
    /// * `form` - Normalization form ('NFC', 'NFKC', 'NFD', 'NFKD')
    ///
    /// # Returns
    /// * `Ok(UnicodeNormalizeCharFilter)` if form is valid
    /// * `Err(RunomeError)` if form is invalid
    pub fn new(form: &str) -> Result<Self, RunomeError> {
        let norm_form = match form {
            "NFC" => NormalizationForm::NFC,
            "NFKC" => NormalizationForm::NFKC,
            "NFD" => NormalizationForm::NFD,
            "NFKD" => NormalizationForm::NFKD,
            _ => {
                return Err(RunomeError::InvalidNormalizationForm {
                    form: form.to_string(),
                });
            }
        };

        Ok(Self { form: norm_form })
    }

    /// Create with default NFKC normalization
    /// NFKC is the most commonly used form for Japanese text processing
    pub fn with_default_form() -> Self {
        Self {
            form: NormalizationForm::NFKC,
        }
    }
}

impl CharFilter for UnicodeNormalizeCharFilter {
    fn apply(&self, text: &str) -> Result<String, RunomeError> {
        // Optimization: Check if normalization is needed to avoid unnecessary work
        let needs_normalization = match self.form {
            NormalizationForm::NFC => !is_nfc(text),
            NormalizationForm::NFKC => !is_nfkc(text),
            NormalizationForm::NFD => !is_nfd(text),
            NormalizationForm::NFKD => !is_nfkd(text),
        };

        if !needs_normalization {
            return Ok(text.to_string());
        }

        // Apply normalization only if needed
        let normalized = match self.form {
            NormalizationForm::NFC => text.nfc().collect::<String>(),
            NormalizationForm::NFKC => text.nfkc().collect::<String>(),
            NormalizationForm::NFD => text.nfd().collect::<String>(),
            NormalizationForm::NFKD => text.nfkd().collect::<String>(),
        };
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_replace_charfilter_japanese() {
        // Test case from Python: RegexReplaceCharFilter('蛇の目', 'janome')
        let filter = RegexReplaceCharFilter::new("蛇の目", "janome").unwrap();
        let result = filter.apply("蛇の目は形態素解析器です。").unwrap();
        assert_eq!(result, "janomeは形態素解析器です。");
    }

    #[test]
    fn test_regex_replace_charfilter_whitespace() {
        // Test case from Python: RegexReplaceCharFilter('\s+', '')
        let filter = RegexReplaceCharFilter::new(r"\s+", "").unwrap();
        let result = filter.apply(" a  b c   d  ").unwrap();
        assert_eq!(result, "abcd");
    }

    #[test]
    fn test_regex_replace_charfilter_empty() {
        // Test edge case with empty replacement
        let filter = RegexReplaceCharFilter::new("", "").unwrap();
        let result = filter.apply("abc あいうえお").unwrap();
        assert_eq!(result, "abc あいうえお");
    }

    #[test]
    fn test_regex_replace_charfilter_no_match() {
        // Test case where pattern doesn't match
        let filter = RegexReplaceCharFilter::new("xyz", "replacement").unwrap();
        let result = filter.apply("abc def").unwrap();
        assert_eq!(result, "abc def");
    }

    #[test]
    fn test_regex_replace_charfilter_multiple_matches() {
        // Test case with multiple matches
        let filter = RegexReplaceCharFilter::new("o", "0").unwrap();
        let result = filter.apply("hello world").unwrap();
        assert_eq!(result, "hell0 w0rld");
    }

    #[test]
    fn test_unicode_normalize_charfilter_default() {
        // Test default NFKC normalization
        let filter = UnicodeNormalizeCharFilter::with_default_form();

        // Fullwidth to halfwidth conversion
        let result = filter.apply("Ｐｙｔｈｏｎ").unwrap();
        assert_eq!(result, "Python");

        // Halfwidth katakana to fullwidth
        let result = filter.apply("ﾒｶﾞﾊﾞｲﾄ").unwrap();
        assert_eq!(result, "メガバイト");
    }

    #[test]
    fn test_unicode_normalize_charfilter_nfkc() {
        // Test NFKC normalization explicitly
        let filter = UnicodeNormalizeCharFilter::new("NFKC").unwrap();

        // Fullwidth to halfwidth conversion
        let result = filter.apply("Ｐｙｔｈｏｎ").unwrap();
        assert_eq!(result, "Python");

        // Halfwidth katakana to fullwidth
        let result = filter.apply("ﾒｶﾞﾊﾞｲﾄ").unwrap();
        assert_eq!(result, "メガバイト");
    }

    #[test]
    fn test_unicode_normalize_charfilter_forms() {
        // Test different normalization forms
        let nfc = UnicodeNormalizeCharFilter::new("NFC").unwrap();
        let nfkc = UnicodeNormalizeCharFilter::new("NFKC").unwrap();
        let nfd = UnicodeNormalizeCharFilter::new("NFD").unwrap();
        let nfkd = UnicodeNormalizeCharFilter::new("NFKD").unwrap();

        // Test with composed/decomposed characters
        let text = "é"; // U+00E9 (composed) 

        // All should handle the text appropriately
        assert!(nfc.apply(text).is_ok());
        assert!(nfkc.apply(text).is_ok());
        assert!(nfd.apply(text).is_ok());
        assert!(nfkd.apply(text).is_ok());
    }

    #[test]
    fn test_unicode_normalize_charfilter_already_normalized() {
        // Test optimization for already normalized text
        let filter = UnicodeNormalizeCharFilter::new("NFC").unwrap();
        let text = "hello world"; // Already NFC normalized
        let result = filter.apply(text).unwrap();
        assert_eq!(result, text);
    }

    #[test]
    fn test_unicode_normalize_charfilter_invalid_form() {
        // Test invalid normalization form
        let result = UnicodeNormalizeCharFilter::new("INVALID");
        assert!(result.is_err());

        if let Err(RunomeError::InvalidNormalizationForm { form }) = result {
            assert_eq!(form, "INVALID");
        } else {
            panic!("Expected InvalidNormalizationForm error");
        }
    }

    #[test]
    fn test_regex_replace_charfilter_invalid_pattern() {
        // Test invalid regex pattern
        let result = RegexReplaceCharFilter::new("[", "replacement");
        assert!(result.is_err());

        if let Err(RunomeError::InvalidRegexPattern { pattern, source: _ }) = result {
            assert_eq!(pattern, "[");
        } else {
            panic!("Expected InvalidRegexPattern error");
        }
    }

    #[test]
    fn test_charfilter_call_method() {
        // Test the convenience call method
        let filter = UnicodeNormalizeCharFilter::with_default_form();
        let result = filter.call("Ｐｙｔｈｏｎ").unwrap();
        assert_eq!(result, "Python");
    }

    #[test]
    fn test_regex_replace_charfilter_call_method() {
        // Test the convenience call method for RegexReplaceCharFilter
        let filter = RegexReplaceCharFilter::new("test", "TEST").unwrap();
        let result = filter.call("this is a test").unwrap();
        assert_eq!(result, "this is a TEST");
    }

    #[test]
    fn test_complex_japanese_text_processing() {
        // Test realistic Japanese text processing scenario
        let unicode_filter = UnicodeNormalizeCharFilter::with_default_form();
        let regex_filter = RegexReplaceCharFilter::new("蛇の目", "janome").unwrap();

        let text = "蛇の目はPure Ｐｙｔｈｏｎな形態素解析器です。";

        // Apply Unicode normalization first
        let normalized = unicode_filter.apply(text).unwrap();
        // Then apply regex replacement
        let replaced = regex_filter.apply(&normalized).unwrap();

        // Should contain both transformations
        assert!(replaced.contains("janome"));
        assert!(replaced.contains("Python")); // Normalized from "Ｐｙｔｈｏｎ"
        assert_eq!(replaced, "janomeはPure Pythonな形態素解析器です。");
    }

    #[test]
    fn test_empty_and_whitespace_handling() {
        // Test edge cases with empty and whitespace strings
        let unicode_filter = UnicodeNormalizeCharFilter::with_default_form();
        let regex_filter = RegexReplaceCharFilter::new(r"\s+", " ").unwrap();

        // Empty string
        assert_eq!(unicode_filter.apply("").unwrap(), "");
        assert_eq!(regex_filter.apply("").unwrap(), "");

        // Whitespace only
        assert_eq!(unicode_filter.apply("   ").unwrap(), "   ");
        assert_eq!(regex_filter.apply("   ").unwrap(), " ");

        // Mixed whitespace
        assert_eq!(regex_filter.apply("a  b\t\nc").unwrap(), "a b c");
    }

    #[test]
    fn test_unicode_variants_handling() {
        // Test various Unicode scenarios
        let filter = UnicodeNormalizeCharFilter::with_default_form();

        // Fullwidth numbers
        let result = filter.apply("１２３４５").unwrap();
        assert_eq!(result, "12345");

        // Fullwidth punctuation
        let result = filter.apply("！？．，").unwrap();
        assert_eq!(result, "!?.,");

        // Mixed fullwidth and halfwidth
        let result = filter.apply("Ｈｅｌｌｏ World").unwrap();
        assert_eq!(result, "Hello World");
    }
}
