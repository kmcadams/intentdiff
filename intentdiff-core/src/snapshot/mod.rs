use serde::Deserialize;
use serde_yaml::Value;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub source: PathBuf,
    pub raw_content: String,
    documents: Vec<SnapshotDocument>,
}

#[derive(Debug, Clone)]
pub struct SnapshotDocument {
    value: Value,
}

#[derive(Debug, Error)]
pub enum SnapshotParseError {
    #[error("failed to parse YAML document {document_index} in {path}: {message}")]
    InvalidYaml {
        path: PathBuf,
        document_index: usize,
        message: String,
    },
}

impl Snapshot {
    pub fn new(source: PathBuf, raw_content: String) -> Result<Self, SnapshotParseError> {
        let documents = parse_documents(&source, &raw_content)?;

        Ok(Self {
            source,
            raw_content,
            documents,
        })
    }

    pub fn documents(&self) -> &[SnapshotDocument] {
        &self.documents
    }

    pub fn any_key_named(&self, key: &str) -> bool {
        self.documents.iter().any(|document| document.contains_key(key))
    }

    pub fn any_bool_value_for_key(&self, key: &str, expected: bool) -> bool {
        self.documents
            .iter()
            .any(|document| document.key_has_bool_value(key, expected))
    }
}

impl SnapshotDocument {
    pub fn root(&self) -> &Value {
        &self.value
    }

    pub fn contains_key(&self, key: &str) -> bool {
        contains_key(&self.value, key)
    }

    pub fn key_has_bool_value(&self, key: &str, expected: bool) -> bool {
        key_has_bool_value(&self.value, key, expected)
    }
}

fn parse_documents(
    source: &PathBuf,
    raw_content: &str,
) -> Result<Vec<SnapshotDocument>, SnapshotParseError> {
    serde_yaml::Deserializer::from_str(raw_content)
        .enumerate()
        .map(|(document_index, document)| {
            Value::deserialize(document)
                .map(|value| SnapshotDocument { value })
                .map_err(|error| SnapshotParseError::InvalidYaml {
                    path: source.clone(),
                    document_index,
                    message: error.to_string(),
                })
        })
        .collect()
}

fn contains_key(value: &Value, key: &str) -> bool {
    match value {
        Value::Mapping(mapping) => mapping.iter().any(|(candidate, nested)| {
            matches!(candidate, Value::String(candidate_key) if candidate_key == key)
                || contains_key(nested, key)
        }),
        Value::Sequence(sequence) => sequence.iter().any(|item| contains_key(item, key)),
        _ => false,
    }
}

fn key_has_bool_value(value: &Value, key: &str, expected: bool) -> bool {
    match value {
        Value::Mapping(mapping) => mapping.iter().any(|(candidate, nested)| {
            matches!(candidate, Value::String(candidate_key) if candidate_key == key)
                && matches!(nested, Value::Bool(flag) if *flag == expected)
                || key_has_bool_value(nested, key, expected)
        }),
        Value::Sequence(sequence) => sequence
            .iter()
            .any(|item| key_has_bool_value(item, key, expected)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_yaml_documents() {
        let snapshot = Snapshot::new(
            "test.yaml".into(),
            "---\nkind: Service\n---\nkind: Deployment\n".into(),
        )
        .expect("snapshot should parse");

        assert_eq!(snapshot.documents().len(), 2);
    }

    #[test]
    fn recursively_finds_keys() {
        let snapshot = Snapshot::new(
            "test.yaml".into(),
            "spec:\n  template:\n    spec:\n      volumes:\n        - emptyDir: {}\n".into(),
        )
        .expect("snapshot should parse");

        assert!(snapshot.any_key_named("emptyDir"));
    }

    #[test]
    fn recursively_matches_bool_values() {
        let snapshot = Snapshot::new(
            "test.yaml".into(),
            "ingress:\n  tls: true\n".into(),
        )
        .expect("snapshot should parse");

        assert!(snapshot.any_bool_value_for_key("tls", true));
        assert!(!snapshot.any_bool_value_for_key("tls", false));
    }

    #[test]
    fn invalid_yaml_returns_contextual_error() {
        let error = Snapshot::new("broken.yaml".into(), "tls: [".into())
            .expect_err("snapshot parsing should fail");

        assert!(error.to_string().contains("broken.yaml"));
        assert!(error.to_string().contains("document 0"));
    }
}
