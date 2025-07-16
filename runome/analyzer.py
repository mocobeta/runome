"""
Japanese morphological analyzer with filter pipeline.

This module provides the Analyzer class for advanced text analysis
with composable character filters and token filters.
"""

from .runome import Analyzer

__all__ = ["Analyzer"]
