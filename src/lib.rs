pub mod dict_builder;
pub mod dictionary;
pub mod error;

pub use dict_builder::DictionaryBuilder;
pub use dictionary::{Dictionary, DictionaryResource, Matcher, RAMDictionary};
pub use error::{Result, RunomeError};
