"""
Japanese morphological analyzer tokenizer.

This module provides the core tokenization functionality with the same API as
the Janome library but with improved performance through Rust implementation.
"""

from .runome import Token, Tokenizer

__all__ = ["Token", "Tokenizer"]
