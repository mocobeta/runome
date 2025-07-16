"""
Type stubs for runome Rust module.
"""

from typing import Optional, Union

class Token:
    """Token with morphological information."""

    @property
    def surface(self) -> str:
        """Surface form of the token."""
        ...

    @property
    def part_of_speech(self) -> str:
        """Part of speech information."""
        ...

    @property
    def infl_type(self) -> str:
        """Inflection type."""
        ...

    @property
    def infl_form(self) -> str:
        """Inflection form."""
        ...

    @property
    def base_form(self) -> str:
        """Base form of the token."""
        ...

    @property
    def reading(self) -> str:
        """Reading of the token."""
        ...

    @property
    def phonetic(self) -> str:
        """Phonetic transcription."""
        ...

    @property
    def node_type(self) -> str:
        """Type of the node (SysDict, UserDict, Unknown)."""
        ...

    def __str__(self) -> str:
        """String representation in Janome format."""
        ...

    def __repr__(self) -> str:
        """Debug representation."""
        ...

class TokenIterator:
    """Iterator for tokenization results."""

    def __iter__(self) -> "TokenIterator":
        """Return self as iterator."""
        ...

    def __next__(self) -> Union[Token, str]:
        """Return next token or surface string."""
        ...

class Tokenizer:
    """Japanese morphological analyzer."""

    def __init__(
        self,
        udic: str = "",
        *,
        udic_enc: str = "utf8",
        udic_type: str = "ipadic",
        max_unknown_length: int = 1024,
        wakati: bool = False,
    ) -> None:
        """Initialize tokenizer.

        Args:
            udic: User dictionary file path (CSV format) or directory path to compiled dictionary data (default: '')
            udic_enc: Character encoding for user dictionary - 'utf8', 'euc-jp', or 'shift_jis' (default: 'utf8')
            udic_type: User dictionary type - 'ipadic' or 'simpledic' (default: 'ipadic')
            max_unknown_length: Maximum unknown word length (default: 1024)
            wakati: Wakati mode flag (default: False)
        """
        ...

    def tokenize(
        self, text: str, wakati: Optional[bool] = None, baseform_unk: bool = True
    ) -> TokenIterator:
        """Tokenize text.

        Args:
            text: Input text to tokenize
            wakati: Override wakati mode (default: None)
            baseform_unk: Set base form for unknown words (default: True)

        Returns:
            Iterator yielding Token objects (wakati=False) or strings (wakati=True)
        """
        ...
