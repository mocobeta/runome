use runome::DictionaryBuilder;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Create dictionary builder
    let mecab_dir = Path::new("mecab-ipadic-2.7.0-20070801");
    let encoding = "euc-jp";

    let builder = DictionaryBuilder::new(mecab_dir, encoding);

    // Build dictionary
    println!("Building dictionary from: {:?}", mecab_dir);
    builder.build()?;

    println!("Dictionary built successfully in 'sysdic' directory");
    Ok(())
}
