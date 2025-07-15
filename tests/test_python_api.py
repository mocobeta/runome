"""
Test cases for Python API compatibility with Janome.

This test suite includes:
1. Basic Python binding functionality tests
2. Equivalent tests from Janome's test_tokenizer.py
3. API compatibility verification
"""

import pytest
import os
import sys
from runome import Tokenizer, Token


class TestBasicPythonBinding:
    """Test basic Python binding functionality."""

    def test_tokenizer_creation(self):
        """Test basic tokenizer creation."""
        tokenizer = Tokenizer()
        assert tokenizer is not None

    def test_tokenizer_with_params(self):
        """Test tokenizer creation with parameters."""
        tokenizer = Tokenizer(max_unknown_length=2048, wakati=True)
        assert tokenizer is not None

    def test_tokenizer_user_dict_not_implemented(self):
        """Test that user dictionary raises appropriate error."""
        with pytest.raises(Exception, match="User dictionary not yet implemented"):
            Tokenizer(udic="test.csv")

    def test_basic_tokenization(self):
        """Test basic tokenization returns tokens."""
        tokenizer = Tokenizer()
        tokens = list(tokenizer.tokenize("テスト"))
        assert len(tokens) > 0
        assert all(isinstance(token, Token) for token in tokens)

    def test_wakati_mode(self):
        """Test wakati mode returns strings."""
        tokenizer = Tokenizer()
        tokens = list(tokenizer.tokenize("テスト", wakati=True))
        assert len(tokens) > 0
        assert all(isinstance(token, str) for token in tokens)

    def test_empty_text(self):
        """Test tokenization of empty string."""
        tokenizer = Tokenizer()
        tokens = list(tokenizer.tokenize(""))
        assert len(tokens) == 0

    def test_token_properties(self):
        """Test all token properties are accessible."""
        tokenizer = Tokenizer()
        tokens = list(tokenizer.tokenize("テスト"))
        assert len(tokens) > 0

        token = tokens[0]

        # Test all properties exist and return strings
        assert isinstance(token.surface, str)
        assert isinstance(token.part_of_speech, str)
        assert isinstance(token.infl_type, str)
        assert isinstance(token.infl_form, str)
        assert isinstance(token.base_form, str)
        assert isinstance(token.reading, str)
        assert isinstance(token.phonetic, str)
        assert isinstance(token.node_type, str)

        # Test surface is not empty
        assert len(token.surface) > 0

        # Test part_of_speech is not empty
        assert len(token.part_of_speech) > 0

    def test_token_string_representation(self):
        """Test token string representation matches Janome format."""
        tokenizer = Tokenizer()
        tokens = list(tokenizer.tokenize("テスト"))
        assert len(tokens) > 0

        token = tokens[0]
        str_repr = str(token)

        # Should contain tab separator
        assert "\t" in str_repr

        # Should start with surface
        assert str_repr.startswith(token.surface)

        # Should contain comma-separated morphological info
        parts = str_repr.split("\t")
        assert len(parts) == 2
        assert "," in parts[1]

    def test_token_repr(self):
        """Test token repr representation."""
        tokenizer = Tokenizer()
        tokens = list(tokenizer.tokenize("テスト"))
        assert len(tokens) > 0

        token = tokens[0]
        repr_str = repr(token)

        # Should contain Token and surface
        assert "Token" in repr_str
        assert token.surface in repr_str

    def test_iterator_protocol(self):
        """Test that tokenize returns proper iterator."""
        tokenizer = Tokenizer()
        result = tokenizer.tokenize("テスト")

        # Should be iterable
        iterator = iter(result)

        # Should support next()
        first_token = next(iterator)
        assert isinstance(first_token, Token)

        # Should raise StopIteration when exhausted
        tokens = list(tokenizer.tokenize(""))
        assert len(tokens) == 0


class TestJanomeEquivalent:
    """Test cases equivalent to Janome's test_tokenizer.py."""

    def setup_method(self):
        """Setup tokenizer for each test."""
        self.tokenizer = Tokenizer()

    def _check_token(self, token, surface, detail, node_type_str):
        """Helper method to check token properties (equivalent to Janome's _check_token)."""
        assert token.surface == surface

        # Reconstruct detail string from token properties
        detail_parts = [
            token.part_of_speech,
            token.infl_type,
            token.infl_form,
            token.base_form,
            token.reading,
            token.phonetic,
        ]
        reconstructed_detail = ",".join(detail_parts)
        assert reconstructed_detail == detail

        # Check string representation
        expected_str = f"{surface}\t{detail}"
        assert str(token) == expected_str

        # Check node type (convert to string for comparison)
        assert node_type_str.lower() in token.node_type.lower()

    def test_tokenize_basic(self):
        """Test basic tokenization (equivalent to test_tokenize_nommap)."""
        text = "すもももももももものうち"
        tokens = list(self.tokenizer.tokenize(text))
        assert len(tokens) == 7

        # Check each token (using relaxed assertions for now)
        assert tokens[0].surface == "すもも"
        assert tokens[1].surface == "も"
        assert tokens[2].surface == "もも"
        assert tokens[3].surface == "も"
        assert tokens[4].surface == "もも"
        assert tokens[5].surface == "の"
        assert tokens[6].surface == "うち"

        # Check that all are system dictionary tokens
        for token in tokens:
            assert "sys" in token.node_type.lower() or "dict" in token.node_type.lower()

    def test_tokenize_unicode_unknown(self):
        """Test tokenization with unicode unknown characters (equivalent to test_tokenize2)."""
        text = "𠮷野屋"
        tokens = list(self.tokenizer.tokenize(text))
        assert len(tokens) == 3

        # First token should be unknown
        assert tokens[0].surface == "𠮷"
        assert "unknown" in tokens[0].node_type.lower()

        # Other tokens should be from system dictionary
        assert tokens[1].surface == "野"
        assert tokens[2].surface == "屋"

        # Test Korean text
        text = "한국어"
        tokens = list(self.tokenizer.tokenize(text))
        assert len(tokens) == 1
        assert tokens[0].surface == "한국어"
        assert "unknown" in tokens[0].node_type.lower()

    def test_tokenize_unknown_numbers(self):
        """Test tokenization with unknown numbers (equivalent to test_tokenize_unknown)."""
        text = "2009年10月16日"
        tokens = list(self.tokenizer.tokenize(text))
        assert len(tokens) == 6

        # Check surfaces
        assert tokens[0].surface == "2009"
        assert tokens[1].surface == "年"
        assert tokens[2].surface == "10"
        assert tokens[3].surface == "月"
        assert tokens[4].surface == "16"
        assert tokens[5].surface == "日"

        # Check node types
        assert "unknown" in tokens[0].node_type.lower()  # 2009
        assert "unknown" in tokens[2].node_type.lower()  # 10
        assert "unknown" in tokens[4].node_type.lower()  # 16

    def test_tokenize_complex_unknown(self):
        """Test complex text with unknown words (equivalent to test_tokenize_unknown part 2)."""
        text = "マルチメディア放送（VHF-HIGH帯）「モバキャス」"
        tokens = list(self.tokenizer.tokenize(text))
        assert len(tokens) == 11

        # Check some key tokens
        assert tokens[0].surface == "マルチメディア"
        assert tokens[1].surface == "放送"
        assert tokens[2].surface == "（"
        assert tokens[3].surface == "VHF"
        assert tokens[4].surface == "-"
        assert tokens[5].surface == "HIGH"
        assert tokens[6].surface == "帯"
        assert tokens[7].surface == "）"
        assert tokens[8].surface == "「"
        assert tokens[9].surface == "モバキャス"
        assert tokens[10].surface == "」"

        # Check that VHF, -, HIGH, モバキャス are unknown
        assert "unknown" in tokens[3].node_type.lower()  # VHF
        assert "unknown" in tokens[4].node_type.lower()  # -
        assert "unknown" in tokens[5].node_type.lower()  # HIGH
        assert "unknown" in tokens[9].node_type.lower()  # モバキャス

    def test_tokenize_unknown_no_baseform(self):
        """Test tokenization with baseform_unk=False (equivalent to test_tokenize_unknown_no_baseform)."""
        text = "2009年10月16日"
        tokens = list(self.tokenizer.tokenize(text, baseform_unk=False))
        assert len(tokens) == 6

        # Check that unknown words have "*" as base form
        assert tokens[0].surface == "2009"
        assert tokens[0].base_form == "*"  # baseform_unk=False
        assert tokens[2].surface == "10"
        assert tokens[2].base_form == "*"  # baseform_unk=False
        assert tokens[4].surface == "16"
        assert tokens[4].base_form == "*"  # baseform_unk=False

        # System dictionary tokens should still have proper base forms
        assert tokens[1].surface == "年"
        assert tokens[1].base_form == "年"

    def test_tokenize_wakati_mode(self):
        """Test wakati mode (equivalent to test_tokenize_wakati)."""
        text = "すもももももももものうち"
        tokenizer = Tokenizer(wakati=True)
        tokens = list(tokenizer.tokenize(text, wakati=True))
        assert len(tokens) == 7

        # In wakati mode, all tokens should be strings
        assert all(isinstance(token, str) for token in tokens)

        # Check surfaces
        assert tokens[0] == "すもも"
        assert tokens[1] == "も"
        assert tokens[2] == "もも"
        assert tokens[3] == "も"
        assert tokens[4] == "もも"
        assert tokens[5] == "の"
        assert tokens[6] == "うち"

    def test_tokenize_wakati_mode_only(self):
        """Test wakati mode when initialized with wakati=True (equivalent to test_tokenize_wakati_mode_only)."""
        text = "すもももももももものうち"
        tokenizer = Tokenizer(wakati=True)
        tokens = list(tokenizer.tokenize(text, wakati=False))

        # When tokenizer is initialized with wakati=True, wakati=False parameter should be ignored
        assert len(tokens) == 7
        print(tokens)
        assert all(isinstance(token, str) for token in tokens)

        # Check surfaces
        assert tokens[0] == "すもも"
        assert tokens[1] == "も"
        assert tokens[2] == "もも"
        assert tokens[3] == "も"
        assert tokens[4] == "もも"
        assert tokens[5] == "の"
        assert tokens[6] == "うち"

    def test_baseform_unk_parameter(self):
        """Test baseform_unk parameter works correctly."""
        text = "2009年"

        # Test with baseform_unk=True (default)
        tokens_true = list(self.tokenizer.tokenize(text, baseform_unk=True))

        # Test with baseform_unk=False
        tokens_false = list(self.tokenizer.tokenize(text, baseform_unk=False))

        # Both should return same number of tokens
        assert len(tokens_true) == len(tokens_false)

        # Both should have same surface
        assert tokens_true[0].surface == tokens_false[0].surface == "2009"

        # But different base forms for unknown words
        assert tokens_true[0].base_form == "2009"  # baseform_unk=True
        assert tokens_false[0].base_form == "*"  # baseform_unk=False

    def test_multiple_character_types(self):
        """Test various character types."""
        test_cases = [
            ("2009", "numeric"),
            ("ABC", "alphabetic"),
            ("すもも", "hiragana"),
            ("テスト", "katakana"),
        ]

        for text, char_type in test_cases:
            tokens = list(self.tokenizer.tokenize(text))
            assert len(tokens) >= 1, f"Failed to tokenize {char_type} text: {text}"
            assert tokens[0].surface == text, (
                f"Surface mismatch for {char_type} text: {text}"
            )


class TestCompatibility:
    """Test compatibility with Janome API."""

    def test_parameter_compatibility(self):
        """Test parameter compatibility with Janome."""
        # Test all supported parameters
        tokenizer = Tokenizer(
            udic="",  # Empty string should work
            max_unknown_length=2048,
            wakati=False,
        )

        tokens = list(tokenizer.tokenize("テスト"))
        assert len(tokens) > 0

    def test_tokenize_parameters(self):
        """Test tokenize method parameters."""
        tokenizer = Tokenizer()

        # Test with all parameters
        tokens = list(tokenizer.tokenize("テスト", wakati=False, baseform_unk=True))
        assert len(tokens) > 0
        assert all(isinstance(token, Token) for token in tokens)

        # Test with wakati=True
        tokens = list(tokenizer.tokenize("テスト", wakati=True, baseform_unk=True))
        assert len(tokens) > 0
        assert all(isinstance(token, str) for token in tokens)


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
