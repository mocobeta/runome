"""
Runome - Japanese morphological analyzer compatible with Janome.

This module provides a Python interface to the Rust-based Runome tokenizer,
offering the same API as the Janome library but with improved performance.

Basic usage:
    >>> from runome.tokenizer import Tokenizer
    >>> t = Tokenizer()
    >>> for token in t.tokenize('形態素解析できるかな'):
    ...     print(token)

Advanced usage with filters:
    >>> from runome.analyzer import Analyzer
    >>> from runome.charfilter import UnicodeNormalizeCharFilter
    >>> from runome.tokenfilter import LowerCaseFilter
    >>> analyzer = Analyzer(
    ...     char_filters=[UnicodeNormalizeCharFilter()],
    ...     token_filters=[LowerCaseFilter()]
    ... )
    >>> for token in analyzer.analyze('テストTEST'):
    ...     print(token)
"""

__version__ = "0.1.0"
