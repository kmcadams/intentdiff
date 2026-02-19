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
    pub fn contains_token(&self, token: &str) -> bool {
        self.raw_content.contains(token)
    }

    pub fn key_equals(&self, key: &str, value: &str) -> bool {
        self.raw_content.contains(&format!("{key}: {value}"))
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.raw_content.contains(&format!("{key}:"))
    }
}
