pub mod dict_resource;
pub mod dict;
pub mod loader;
pub mod types;

pub use dict_resource::DictionaryResource;
pub use dict::{Dictionary, Matcher, RAMDictionary};
pub use types::*;
