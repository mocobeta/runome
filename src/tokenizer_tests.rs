use crate::lattice::NodeType;
use crate::tokenizer::{TokenizeResult, Tokenizer};

/// Segmentation tests module - tests for basic tokenization functionality
pub mod segmentation_tests {
    use super::*;

    /// Helper function to check token properties (equivalent to Python's _check_token)
    fn check_token(
        token: &TokenizeResult,
        expected_surface: &str,
        expected_detail: &str,
        expected_node_type: NodeType,
    ) {
        match token {
            TokenizeResult::Token(token) => {
                // Check surface form
                assert_eq!(token.surface(), expected_surface, "Surface mismatch");

                // Check combined detail string (part_of_speech,infl_type,infl_form,base_form,reading,phonetic)
                let actual_detail = format!(
                    "{},{},{},{},{},{}",
                    token.part_of_speech(),
                    token.infl_type(),
                    token.infl_form(),
                    token.base_form(),
                    token.reading(),
                    token.phonetic()
                );
                assert_eq!(
                    actual_detail, expected_detail,
                    "Detail mismatch for '{}'",
                    expected_surface
                );

                // Check string representation (surface + tab + detail)
                let expected_str = format!("{}\t{}", expected_surface, expected_detail);
                assert_eq!(
                    format!("{}", token),
                    expected_str,
                    "String representation mismatch"
                );

                // Check node type
                assert_eq!(
                    token.node_type(),
                    expected_node_type,
                    "Node type mismatch for '{}'",
                    expected_surface
                );
            }
            TokenizeResult::Surface(_) => {
                panic!("Expected Token but got Surface for '{}'", expected_surface);
            }
        }
    }

    #[test]
    fn test_tokenize_basic() {
        // Equivalent to Python's TestTokenizer.test_tokenize_nommap()
        // Tests basic tokenization with the classic "すもももももももものうち" example
        let tokenizer = Tokenizer::new(None, None);
        if tokenizer.is_err() {
            eprintln!("Skipping test: SystemDictionary not available");
            return;
        }
        let tokenizer = tokenizer.unwrap();

        let text = "すもももももももものうち";
        let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();

        assert!(results.is_ok(), "Tokenization should succeed");
        let tokens = results.unwrap();

        // Should produce exactly 7 tokens
        assert_eq!(tokens.len(), 7, "Expected 7 tokens for '{}'", text);

        // Validate each token matches expected structure
        // ✅ FIXED: Now correctly uses SysDict node type for dictionary entries
        // ✅ FIXED: Now correctly extracts reading and phonetic fields from dictionary
        check_token(
            &tokens[0],
            "すもも",
            "名詞,一般,*,*,*,*,すもも,スモモ,スモモ",
            NodeType::SysDict,
        );
        check_token(
            &tokens[1],
            "も",
            "助詞,係助詞,*,*,*,*,も,モ,モ",
            NodeType::SysDict,
        );
        check_token(
            &tokens[2],
            "もも",
            "名詞,一般,*,*,*,*,もも,モモ,モモ",
            NodeType::SysDict,
        );
        check_token(
            &tokens[3],
            "も",
            "助詞,係助詞,*,*,*,*,も,モ,モ",
            NodeType::SysDict,
        );
        check_token(
            &tokens[4],
            "もも",
            "名詞,一般,*,*,*,*,もも,モモ,モモ",
            NodeType::SysDict,
        );
        check_token(
            &tokens[5],
            "の",
            "助詞,連体化,*,*,*,*,の,ノ,ノ",
            NodeType::SysDict,
        );
        check_token(
            &tokens[6],
            "うち",
            "名詞,非自立,副詞可能,*,*,*,うち,ウチ,ウチ",
            NodeType::SysDict,
        );
    }

    #[test]
    fn test_tokenize_mixed_known_unknown() {
        // Equivalent to Python's TestTokenizer.test_tokenize2()
        // Tests tokenization of text with both known and unknown characters
        let tokenizer = Tokenizer::new(None, None);
        if tokenizer.is_err() {
            eprintln!("Skipping test: SystemDictionary not available");
            return;
        }
        let tokenizer = tokenizer.unwrap();

        // Test case 1: Mixed known/unknown characters - '𠮷野屋'
        // 𠮷 is a rare kanji variant (U+20BB7) that should be unknown
        // 野 and 屋 should be found in the dictionary
        let text = "𠮷野屋";
        let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();

        assert!(results.is_ok(), "Tokenization should succeed");
        let tokens = results.unwrap();

        assert_eq!(tokens.len(), 3, "Expected 3 tokens for '{}'", text);

        // 𠮷 should be unknown (rare kanji variant)
        check_token(
            &tokens[0],
            "𠮷",
            "記号,一般,*,*,*,*,𠮷,*,*",
            NodeType::Unknown,
        );

        // 野 should be in dictionary
        check_token(
            &tokens[1],
            "野",
            "名詞,一般,*,*,*,*,野,ノ,ノ",
            NodeType::SysDict,
        );

        // 屋 should be in dictionary
        check_token(
            &tokens[2],
            "屋",
            "名詞,接尾,一般,*,*,*,屋,ヤ,ヤ",
            NodeType::SysDict,
        );

        // Test case 2: Foreign text - Korean '한국어'
        // Should be treated as a single unknown token
        let text = "한국어";
        let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();

        assert!(results.is_ok(), "Tokenization should succeed");
        let tokens = results.unwrap();

        assert_eq!(tokens.len(), 1, "Expected 1 token for '{}'", text);

        // Korean text should be unknown
        check_token(
            &tokens[0],
            "한국어",
            "記号,一般,*,*,*,*,한국어,*,*",
            NodeType::Unknown,
        );
    }

    #[test]
    fn test_tokenize_unknown() {
        // Equivalent to Python's TestTokenizer.test_tokenize_unknown()
        // Tests tokenization of text with various unknown word types (numbers, English, etc.)
        let tokenizer = Tokenizer::new(None, None);
        if tokenizer.is_err() {
            eprintln!("Skipping test: SystemDictionary not available");
            return;
        }
        let tokenizer = tokenizer.unwrap();

        // Test case 1: Date text with numbers - '2009年10月16日'
        // Numbers should be unknown, date markers should be in dictionary
        let text = "2009年10月16日";
        let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();

        assert!(results.is_ok(), "Tokenization should succeed");
        let tokens = results.unwrap();

        assert_eq!(tokens.len(), 6, "Expected 6 tokens for '{}'", text);

        // Numbers should be unknown
        check_token(
            &tokens[0],
            "2009",
            "名詞,数,*,*,*,*,2009,*,*",
            NodeType::Unknown,
        );

        // Date markers should be in dictionary
        check_token(
            &tokens[1],
            "年",
            "名詞,接尾,助数詞,*,*,*,年,ネン,ネン",
            NodeType::SysDict,
        );

        check_token(
            &tokens[2],
            "10",
            "名詞,数,*,*,*,*,10,*,*",
            NodeType::Unknown,
        );

        check_token(
            &tokens[3],
            "月",
            "名詞,一般,*,*,*,*,月,ツキ,ツキ",
            NodeType::SysDict,
        );

        check_token(
            &tokens[4],
            "16",
            "名詞,数,*,*,*,*,16,*,*",
            NodeType::Unknown,
        );

        check_token(
            &tokens[5],
            "日",
            "名詞,接尾,助数詞,*,*,*,日,ニチ,ニチ",
            NodeType::SysDict,
        );

        // Test case 2: Mixed Japanese/English text - 'マルチメディア放送（VHF-HIGH帯）「モバキャス」'
        // Tests various punctuation, English words, and compound words
        let text = "マルチメディア放送（VHF-HIGH帯）「モバキャス」";
        let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, None).collect();

        assert!(results.is_ok(), "Tokenization should succeed");
        let tokens = results.unwrap();

        assert_eq!(tokens.len(), 11, "Expected 11 tokens for '{}'", text);

        // Japanese compound word in dictionary
        check_token(
            &tokens[0],
            "マルチメディア",
            "名詞,一般,*,*,*,*,マルチメディア,マルチメディア,マルチメディア",
            NodeType::SysDict,
        );

        // Japanese word in dictionary
        check_token(
            &tokens[1],
            "放送",
            "名詞,サ変接続,*,*,*,*,放送,ホウソウ,ホーソー",
            NodeType::SysDict,
        );

        // Punctuation in dictionary
        check_token(
            &tokens[2],
            "（",
            "記号,括弧開,*,*,*,*,（,（,（",
            NodeType::SysDict,
        );

        // English abbreviation - unknown
        check_token(
            &tokens[3],
            "VHF",
            "名詞,固有名詞,組織,*,*,*,VHF,*,*",
            NodeType::Unknown,
        );

        // Hyphen - unknown
        check_token(
            &tokens[4],
            "-",
            "名詞,サ変接続,*,*,*,*,-,*,*",
            NodeType::Unknown,
        );

        // English word - unknown
        check_token(
            &tokens[5],
            "HIGH",
            "名詞,一般,*,*,*,*,HIGH,*,*",
            NodeType::Unknown,
        );

        // Japanese suffix in dictionary
        check_token(
            &tokens[6],
            "帯",
            "名詞,接尾,一般,*,*,*,帯,タイ,タイ",
            NodeType::SysDict,
        );

        // Closing punctuation in dictionary
        check_token(
            &tokens[7],
            "）",
            "記号,括弧閉,*,*,*,*,）,）,）",
            NodeType::SysDict,
        );

        // Opening quote in dictionary
        check_token(
            &tokens[8],
            "「",
            "記号,括弧開,*,*,*,*,「,「,「",
            NodeType::SysDict,
        );

        // Katakana compound (brand name) - unknown
        check_token(
            &tokens[9],
            "モバキャス",
            "名詞,固有名詞,一般,*,*,*,モバキャス,*,*",
            NodeType::Unknown,
        );

        // Closing quote in dictionary
        check_token(
            &tokens[10],
            "」",
            "記号,括弧閉,*,*,*,*,」,」,」",
            NodeType::SysDict,
        );
    }

    #[test]
    fn test_tokenize_unknown_no_baseform() {
        // Equivalent to Python's TestTokenizer.test_tokenize_unknown_no_baseform()
        // Tests tokenization with baseform_unk=False - unknown words should have "*" as base_form
        let tokenizer = Tokenizer::new(None, None);
        if tokenizer.is_err() {
            eprintln!("Skipping test: SystemDictionary not available");
            return;
        }
        let tokenizer = tokenizer.unwrap();

        // Test case 1: Date text with numbers - '2009年10月16日' with baseform_unk=False
        // Numbers should be unknown with "*" as base_form, date markers should be in dictionary
        let text = "2009年10月16日";
        let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, Some(false)).collect();

        assert!(results.is_ok(), "Tokenization should succeed");
        let tokens = results.unwrap();

        assert_eq!(tokens.len(), 6, "Expected 6 tokens for '{}'", text);

        // Numbers should be unknown with "*" as base_form (baseform_unk=False)
        check_token(
            &tokens[0],
            "2009",
            "名詞,数,*,*,*,*,*,*,*",
            NodeType::Unknown,
        );

        // Date markers should be in dictionary (unchanged)
        check_token(
            &tokens[1],
            "年",
            "名詞,接尾,助数詞,*,*,*,年,ネン,ネン",
            NodeType::SysDict,
        );

        check_token(&tokens[2], "10", "名詞,数,*,*,*,*,*,*,*", NodeType::Unknown);

        check_token(
            &tokens[3],
            "月",
            "名詞,一般,*,*,*,*,月,ツキ,ツキ",
            NodeType::SysDict,
        );

        check_token(&tokens[4], "16", "名詞,数,*,*,*,*,*,*,*", NodeType::Unknown);

        check_token(
            &tokens[5],
            "日",
            "名詞,接尾,助数詞,*,*,*,日,ニチ,ニチ",
            NodeType::SysDict,
        );

        // Test case 2: Mixed Japanese/English text with baseform_unk=False
        // 'マルチメディア放送（VHF-HIGH帯）「モバキャス」'
        let text = "マルチメディア放送（VHF-HIGH帯）「モバキャス」";
        let results: Result<Vec<_>, _> = tokenizer.tokenize(text, None, Some(false)).collect();

        assert!(results.is_ok(), "Tokenization should succeed");
        let tokens = results.unwrap();

        assert_eq!(tokens.len(), 11, "Expected 11 tokens for '{}'", text);

        // Dictionary entries should be unchanged
        check_token(
            &tokens[0],
            "マルチメディア",
            "名詞,一般,*,*,*,*,マルチメディア,マルチメディア,マルチメディア",
            NodeType::SysDict,
        );

        check_token(
            &tokens[1],
            "放送",
            "名詞,サ変接続,*,*,*,*,放送,ホウソウ,ホーソー",
            NodeType::SysDict,
        );

        check_token(
            &tokens[2],
            "（",
            "記号,括弧開,*,*,*,*,（,（,（",
            NodeType::SysDict,
        );

        // Unknown words should have "*" as base_form (baseform_unk=False)
        check_token(
            &tokens[3],
            "VHF",
            "名詞,固有名詞,組織,*,*,*,*,*,*",
            NodeType::Unknown,
        );

        check_token(
            &tokens[4],
            "-",
            "名詞,サ変接続,*,*,*,*,*,*,*",
            NodeType::Unknown,
        );

        check_token(
            &tokens[5],
            "HIGH",
            "名詞,一般,*,*,*,*,*,*,*",
            NodeType::Unknown,
        );

        // Dictionary entries continue unchanged
        check_token(
            &tokens[6],
            "帯",
            "名詞,接尾,一般,*,*,*,帯,タイ,タイ",
            NodeType::SysDict,
        );

        check_token(
            &tokens[7],
            "）",
            "記号,括弧閉,*,*,*,*,）,）,）",
            NodeType::SysDict,
        );

        check_token(
            &tokens[8],
            "「",
            "記号,括弧開,*,*,*,*,「,「,「",
            NodeType::SysDict,
        );

        // Unknown compound word with "*" as base_form
        check_token(
            &tokens[9],
            "モバキャス",
            "名詞,固有名詞,一般,*,*,*,*,*,*",
            NodeType::Unknown,
        );

        check_token(
            &tokens[10],
            "」",
            "記号,括弧閉,*,*,*,*,」,」,」",
            NodeType::SysDict,
        );
    }
}
