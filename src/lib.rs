pub mod analyzer;
pub mod charfilter;
pub mod dict_builder;
pub mod dictionary;
pub mod error;
pub mod lattice;
pub mod tokenfilter;
pub mod tokenizer;

#[cfg(feature = "python")]
pub mod python_bindings;

#[cfg(test)]
pub mod tokenizer_tests;

pub use analyzer::{Analyzer, AnalyzerBuilder};
pub use charfilter::{CharFilter, RegexReplaceCharFilter, UnicodeNormalizeCharFilter};
pub use dict_builder::DictionaryBuilder;
pub use dictionary::{Dictionary, DictionaryResource, Matcher, RAMDictionary};
pub use error::{Result, RunomeError};
pub use lattice::{BOS, EOS, Lattice, LatticeNode, Node, NodeType, UnknownNode};
pub use tokenfilter::{
    CompoundNounFilter, ExtractAttributeFilter, LowerCaseFilter, POSKeepFilter, POSStopFilter,
    TokenCountFilter, TokenFilter, UpperCaseFilter,
};
pub use tokenizer::{Token, TokenizeResult, Tokenizer};

#[cfg(feature = "python")]
pub use python_bindings::*;
