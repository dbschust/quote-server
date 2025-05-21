use std::path::Path;
use crate::QuoteError;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct Quote {
    pub words: String,
    pub author: String,
}

pub fn read_quotes<P: AsRef<Path>>(quotes_path: P) -> Result<Vec<Quote>, QuoteError> {
    let f = std::fs::File::open(quotes_path.as_ref())?;
    let quotes = serde_json::from_reader(f)?;
    Ok(quotes)
}
