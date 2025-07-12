use anyhow::Result;
use std::path::{Path, PathBuf};

pub mod build;

pub struct DictionaryBuilder {
    pub mecab_dir: PathBuf,
    pub encoding: String,
    pub output_dir: PathBuf,
}

impl DictionaryBuilder {
    pub fn new(mecab_dir: &Path, encoding: &str) -> Self {
        Self {
            mecab_dir: mecab_dir.to_path_buf(),
            encoding: encoding.to_string(),
            output_dir: PathBuf::from("sysdic"),
        }
    }

    pub fn with_output_dir(mut self, output_dir: &Path) -> Self {
        self.output_dir = output_dir.to_path_buf();
        self
    }

    pub fn build(&self) -> Result<()> {
        build::build_dictionary(self)
    }
}
