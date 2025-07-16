#!/usr/bin/env python3
"""
Example demonstrating Janome-compatible usage patterns with Runome.

This example shows how to use Runome with the same import patterns
and API as the original Janome library.
"""

def basic_tokenization():
    """Basic tokenization example matching Janome's API."""
    print("=== Basic Tokenization ===")
    
    from runome.tokenizer import Tokenizer
    
    t = Tokenizer()
    text = "すもももももももものうち"
    
    for token in t.tokenize(text):
        print(f"{token.surface}\t{token.part_of_speech}")
    print()


def analyzer_with_char_filters():
    """Analyzer with character filters example."""
    print("=== Analyzer with Character Filters ===")
    
    from runome.analyzer import Analyzer
    from runome.charfilter import UnicodeNormalizeCharFilter, RegexReplaceCharFilter
    
    analyzer = Analyzer(
        char_filters=[
            UnicodeNormalizeCharFilter(),
            RegexReplaceCharFilter("蛇の目", "janome")
        ]
    )
    
    text = "蛇の目はＰｙｔｈｏｎな形態素解析器です。"
    
    for token in analyzer.analyze(text):
        print(f"{token.surface}\t{token.part_of_speech}")
    print()


def analyzer_with_token_filters():
    """Analyzer with token filters example."""
    print("=== Analyzer with Token Filters ===")
    
    from runome.analyzer import Analyzer
    from runome.tokenfilter import POSKeepFilter, CompoundNounFilter, LowerCaseFilter
    
    analyzer = Analyzer(
        token_filters=[
            POSKeepFilter(["名詞", "動詞"]),
            CompoundNounFilter(),
            LowerCaseFilter()
        ]
    )
    
    text = "東京駅で新幹線に乗って大阪に行きます。"
    
    for token in analyzer.analyze(text):
        print(f"{token.surface}\t{token.part_of_speech}")
    print()


def terminal_filters_example():
    """Terminal filters that change output type."""
    print("=== Terminal Filters Example ===")
    
    from runome.analyzer import Analyzer
    from runome.tokenfilter import POSKeepFilter, ExtractAttributeFilter, TokenCountFilter
    
    # Extract surface forms only
    analyzer = Analyzer(
        token_filters=[
            POSKeepFilter(["名詞"]),
            ExtractAttributeFilter("surface")
        ]
    )
    
    text = "すもももももももものうち"
    results = list(analyzer.analyze(text))
    print("Surface forms:", results)
    
    # Count tokens
    analyzer = Analyzer(
        token_filters=[
            POSKeepFilter(["名詞"]),
            TokenCountFilter("surface", sorted=True)
        ]
    )
    
    counts = list(analyzer.analyze(text))
    print("Token counts:", counts)
    print()


def wildcard_imports_example():
    """Example using explicit imports like Janome documentation."""
    print("=== Complex Pipeline Example ===")
    
    from runome.tokenizer import Tokenizer
    from runome.analyzer import Analyzer
    from runome.charfilter import UnicodeNormalizeCharFilter, RegexReplaceCharFilter
    from runome.tokenfilter import CompoundNounFilter, POSStopFilter, LowerCaseFilter
    
    # Complex pipeline using multiple filters
    analyzer = Analyzer(
        char_filters=[
            UnicodeNormalizeCharFilter(),
            RegexReplaceCharFilter(r"\s+", " ")
        ],
        tokenizer=Tokenizer(),
        token_filters=[
            CompoundNounFilter(),
            POSStopFilter(["助詞", "記号"]),
            LowerCaseFilter()
        ]
    )
    
    text = "東京　　駅で　　新幹線に　　乗る！"
    
    for token in analyzer.analyze(text):
        print(f"{token.surface}\t{token.part_of_speech}")
    print()


if __name__ == "__main__":
    print("Runome - Janome-compatible usage examples")
    print("=" * 40)
    
    basic_tokenization()
    analyzer_with_char_filters()
    analyzer_with_token_filters()
    terminal_filters_example()
    wildcard_imports_example()
    
    print("✓ All examples completed successfully!")