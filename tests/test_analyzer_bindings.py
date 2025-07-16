#!/usr/bin/env python3
"""Test suite for Runome Analyzer Python bindings"""

import pytest


def test_charfilters():
    """Test CharFilter bindings"""
    import runome
    
    # Test UnicodeNormalizeCharFilter
    filter = runome.UnicodeNormalizeCharFilter()
    assert filter("Ｐｙｔｈｏｎ") == "Python"
    assert filter("㍻") == "平成"  # NFKC normalization
    
    # Test with different forms
    filter_nfc = runome.UnicodeNormalizeCharFilter("NFC")
    filter_nfd = runome.UnicodeNormalizeCharFilter("NFD")
    # NFC and NFD should handle combining characters differently
    text = "が"  # Hiragana GA (can be decomposed)
    assert len(filter_nfc(text)) <= len(filter_nfd(text))
    
    # Test RegexReplaceCharFilter
    filter = runome.RegexReplaceCharFilter("蛇の目", "janome")
    assert filter("蛇の目は形態素解析器です。") == "janomeは形態素解析器です。"
    
    # Test regex with groups
    filter = runome.RegexReplaceCharFilter(r"(\d+)年", r"\1 year")
    assert filter("2024年です") == "2024 yearです"
    
    # Test callable interface
    assert filter.__call__("2024年です") == "2024 yearです"


def test_tokenfilters_basic():
    """Test basic TokenFilter bindings"""
    import runome
    
    tokenizer = runome.Tokenizer()
    tokens = list(tokenizer.tokenize("テストTEST"))
    
    # Test LowerCaseFilter
    filter = runome.LowerCaseFilter()
    filtered = list(filter(tokens))
    assert all(hasattr(t, 'surface') for t in filtered)
    assert any(t.surface == "test" for t in filtered)
    
    # Test UpperCaseFilter
    filter = runome.UpperCaseFilter()
    filtered = list(filter(tokens))
    assert any(t.surface == "TEST" for t in filtered)
    
    # Test CompoundNounFilter
    tokens = list(tokenizer.tokenize("形態素解析器"))
    filter = runome.CompoundNounFilter()
    filtered = list(filter(tokens))
    # Should combine consecutive nouns
    assert len(filtered) <= len(tokens)


def test_pos_filters():
    """Test POS-based TokenFilters"""
    import runome
    
    tokenizer = runome.Tokenizer()
    tokens = list(tokenizer.tokenize("東京駅で降りる"))
    
    # Test POSKeepFilter
    filter = runome.POSKeepFilter(["名詞"])
    filtered = list(filter(tokens))
    # Should only keep nouns
    assert all("名詞" in t.part_of_speech for t in filtered)
    assert any(t.surface == "東京" for t in filtered)
    assert any(t.surface == "駅" for t in filtered)
    
    # Test POSStopFilter
    filter = runome.POSStopFilter(["助詞", "動詞"])
    filtered = list(filter(tokens))
    # Should remove particles and verbs
    assert all("助詞" not in t.part_of_speech for t in filtered)
    assert all("動詞" not in t.part_of_speech for t in filtered)


def test_terminal_filters():
    """Test terminal TokenFilters that change output type"""
    import runome
    
    tokenizer = runome.Tokenizer()
    
    # Test ExtractAttributeFilter
    tokens = list(tokenizer.tokenize("東京駅"))
    filter = runome.ExtractAttributeFilter("surface")
    results = list(filter(tokens))
    assert all(isinstance(r, str) for r in results)
    assert "東京" in results
    assert "駅" in results
    
    # Test with base_form
    filter = runome.ExtractAttributeFilter("base_form")
    results = list(filter(tokens))
    assert all(isinstance(r, str) for r in results)
    
    # Test TokenCountFilter
    tokens = list(tokenizer.tokenize("すもももももももものうち"))
    pos_filter = runome.POSKeepFilter(["名詞"])
    count_filter = runome.TokenCountFilter("surface", sorted=True)
    
    filtered_tokens = list(pos_filter(tokens))
    results = list(count_filter(filtered_tokens))
    
    assert all(isinstance(r, tuple) and len(r) == 2 for r in results)
    assert all(isinstance(r[0], str) and isinstance(r[1], int) for r in results)
    
    # Check that results are sorted by count (descending)
    counts = [r[1] for r in results]
    assert counts == sorted(counts, reverse=True)
    
    # Check specific counts
    count_dict = dict(results)
    assert count_dict.get("もも", 0) == 2
    assert count_dict.get("すもも", 0) == 1
    assert count_dict.get("うち", 0) == 1


def test_analyzer_basic():
    """Test basic Analyzer functionality"""
    import runome
    
    # Test default analyzer
    analyzer = runome.Analyzer()
    results = list(analyzer.analyze("テスト"))
    assert len(results) > 0
    assert all(hasattr(t, 'surface') for t in results)
    
    # Test with CharFilters
    analyzer = runome.Analyzer(
        char_filters=[
            runome.UnicodeNormalizeCharFilter(),
            runome.RegexReplaceCharFilter("蛇の目", "janome")
        ]
    )
    results = list(analyzer.analyze("蛇の目はＰｙｔｈｏｎです"))
    surfaces = [t.surface for t in results]
    assert "janome" in surfaces
    assert "Python" in surfaces  # Should be normalized


def test_analyzer_full_pipeline():
    """Test complete Analyzer pipeline"""
    import runome
    
    analyzer = runome.Analyzer(
        char_filters=[
            runome.UnicodeNormalizeCharFilter(),
            runome.RegexReplaceCharFilter("蛇の目", "janome")
        ],
        token_filters=[
            runome.CompoundNounFilter(),
            runome.POSStopFilter(["記号", "助詞"]),
            runome.LowerCaseFilter()
        ]
    )
    
    text = "蛇の目はPure Ｐｙｔｈｏｎな形態素解析器です。"
    results = list(analyzer.analyze(text))
    
    # Extract surfaces
    surfaces = [t.surface for t in results]
    
    # Check expected tokens
    assert "janome" in surfaces
    assert "pure" in surfaces
    assert "python" in surfaces
    assert "な" in surfaces
    assert "形態素解析器" in surfaces
    assert "です" in surfaces
    
    # Check that particles were filtered out
    assert "は" not in surfaces  # particle
    assert "。" not in surfaces  # symbol


def test_analyzer_with_terminal_filter():
    """Test Analyzer with terminal filters"""
    import runome
    
    # Test with ExtractAttributeFilter
    analyzer = runome.Analyzer(
        token_filters=[
            runome.POSKeepFilter(["名詞"]),
            runome.ExtractAttributeFilter("surface")
        ]
    )
    
    results = list(analyzer.analyze("東京駅で降りる"))
    assert all(isinstance(r, str) for r in results)
    assert "東京" in results
    assert "駅" in results
    assert "降りる" not in results  # verb should be filtered out
    
    # Test with TokenCountFilter
    analyzer = runome.Analyzer(
        token_filters=[
            runome.POSKeepFilter(["名詞"]),
            runome.TokenCountFilter("surface", sorted=True)
        ]
    )
    
    results = list(analyzer.analyze("すもももももももものうち"))
    assert all(isinstance(r, tuple) for r in results)
    
    count_dict = dict(results)
    assert count_dict["もも"] == 2
    assert count_dict["すもも"] == 1
    assert count_dict["うち"] == 1


def test_analyzer_wakati_rejection():
    """Test that Analyzer rejects wakati mode tokenizer"""
    import runome
    
    # Create tokenizer with wakati=True
    wakati_tokenizer = runome.Tokenizer(wakati=True)
    
    # Should raise exception
    with pytest.raises(Exception) as excinfo:
        runome.Analyzer(tokenizer=wakati_tokenizer)
    
    assert "wakati=True" in str(excinfo.value)


def test_custom_tokenizer():
    """Test Analyzer with custom tokenizer settings"""
    import runome
    
    # Create tokenizer with user dictionary
    # Note: This assumes no user dictionary file, but tests the parameter passing
    tokenizer = runome.Tokenizer(max_unknown_length=512)
    
    analyzer = runome.Analyzer(
        tokenizer=tokenizer,
        token_filters=[runome.LowerCaseFilter()]
    )
    
    results = list(analyzer.analyze("TEST"))
    assert any(t.surface == "test" for t in results)


def test_filter_chaining():
    """Test complex filter chaining"""
    import runome
    
    # Create a complex pipeline
    analyzer = runome.Analyzer(
        char_filters=[
            runome.RegexReplaceCharFilter(r"\s+", " "),  # Normalize whitespace
            runome.RegexReplaceCharFilter(r"[！-～]", ""),  # Remove full-width symbols
            runome.UnicodeNormalizeCharFilter()
        ],
        token_filters=[
            runome.CompoundNounFilter(),
            runome.POSKeepFilter(["名詞", "動詞", "形容詞"]),
            runome.LowerCaseFilter()
        ]
    )
    
    text = "東京　　駅で　　降りる！！"
    results = list(analyzer.analyze(text))
    
    surfaces = [t.surface for t in results]
    assert "東京" in surfaces
    assert "駅" in surfaces
    assert "降りる" in surfaces


def test_error_handling():
    """Test error handling in bindings"""
    import runome
    
    # Test invalid regex pattern
    with pytest.raises(Exception):
        runome.RegexReplaceCharFilter("[invalid", "replacement")
    
    # Test invalid normalization form
    with pytest.raises(Exception):
        runome.UnicodeNormalizeCharFilter("INVALID")
    
    # Test invalid attribute name
    with pytest.raises(Exception):
        runome.ExtractAttributeFilter("invalid_attribute")
    
    with pytest.raises(Exception):
        runome.TokenCountFilter("invalid_attribute")


if __name__ == "__main__":
    # Run basic smoke test
    print("Running Analyzer bindings smoke test...")
    
    test_charfilters()
    print("✓ CharFilters working")
    
    test_tokenfilters_basic()
    print("✓ Basic TokenFilters working")
    
    test_pos_filters()
    print("✓ POS filters working")
    
    test_terminal_filters()
    print("✓ Terminal filters working")
    
    test_analyzer_basic()
    print("✓ Basic Analyzer working")
    
    test_analyzer_full_pipeline()
    print("✓ Full pipeline working")
    
    test_analyzer_with_terminal_filter()
    print("✓ Terminal filter integration working")
    
    test_filter_chaining()
    print("✓ Filter chaining working")
    
    print("\nAll tests passed! 🎉")