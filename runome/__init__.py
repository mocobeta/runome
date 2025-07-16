"""
Runome - Japanese morphological analyzer compatible with Janome.

This module provides a Python interface to the Rust-based Runome tokenizer,
offering the same API as the Janome library but with improved performance.

Basic usage:
    >>> from runome import Tokenizer
    >>> t = Tokenizer()
    >>> for token in t.tokenize('形態素解析できるかな'):
    ...     print(token)
"""

from .runome import (
    Token, Tokenizer,
    # CharFilter classes
    CharFilter, RegexReplaceCharFilter, UnicodeNormalizeCharFilter,
    # TokenFilter classes
    TokenFilter, LowerCaseFilter, UpperCaseFilter, POSStopFilter, POSKeepFilter,
    CompoundNounFilter, ExtractAttributeFilter, TokenCountFilter, TokenFilterIterator,
    # Analyzer
    Analyzer
)

__all__ = [
    'Token', 'Tokenizer',
    # CharFilter classes
    'CharFilter', 'RegexReplaceCharFilter', 'UnicodeNormalizeCharFilter',
    # TokenFilter classes
    'TokenFilter', 'LowerCaseFilter', 'UpperCaseFilter', 'POSStopFilter', 'POSKeepFilter',
    'CompoundNounFilter', 'ExtractAttributeFilter', 'TokenCountFilter', 'TokenFilterIterator',
    # Analyzer
    'Analyzer'
]
__version__ = "0.1.0"