use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub source: PathBuf,
    pub raw_content: String,
}

impl Snapshot {
    pub fn new(source: PathBuf, raw_content: String) -> Self {
        Self {
            source,
            raw_content,
        }
    }
}
