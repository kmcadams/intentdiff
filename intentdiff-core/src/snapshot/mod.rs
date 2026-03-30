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
    document_index: usize,
    value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRef {
    pub document_index: usize,
    pub kind: Option<String>,
    pub name: Option<String>,
    pub namespace: Option<String>,
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

    pub fn resources(&self) -> Vec<ResourceRef> {
        self.documents
            .iter()
            .map(SnapshotDocument::resource_ref)
            .collect()
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
    pub fn document_index(&self) -> usize {
        self.document_index
    }

    pub fn root(&self) -> &Value {
        &self.value
    }

    pub fn kind(&self) -> Option<&str> {
        top_level_string_field(&self.value, "kind")
    }

    pub fn name(&self) -> Option<&str> {
        nested_string_field(&self.value, &["metadata", "name"])
    }

    pub fn namespace(&self) -> Option<&str> {
        nested_string_field(&self.value, &["metadata", "namespace"])
    }

    pub fn resource_ref(&self) -> ResourceRef {
        ResourceRef {
            document_index: self.document_index,
            kind: self.kind().map(str::to_owned),
            name: self.name().map(str::to_owned),
            namespace: self.namespace().map(str::to_owned),
        }
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
                .map(|value| SnapshotDocument {
                    document_index,
                    value,
                })
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

fn top_level_string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    match value {
        Value::Mapping(mapping) => mapping
            .get(Value::String(key.to_owned()))
            .and_then(Value::as_str),
        _ => None,
    }
}

fn nested_string_field<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;

    for segment in path {
        current = match current {
            Value::Mapping(mapping) => mapping.get(Value::String((*segment).to_owned()))?,
            _ => return None,
        };
    }

    current.as_str()
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
    fn exposes_resource_identity_for_kubernetes_like_documents() {
        let snapshot = Snapshot::new(
            "test.yaml".into(),
            "---\nkind: Service\nmetadata:\n  name: api\n  namespace: prod\n".into(),
        )
        .expect("snapshot should parse");

        let document = &snapshot.documents()[0];

        assert_eq!(document.document_index(), 0);
        assert_eq!(document.kind(), Some("Service"));
        assert_eq!(document.name(), Some("api"));
        assert_eq!(document.namespace(), Some("prod"));
        assert_eq!(
            document.resource_ref(),
            ResourceRef {
                document_index: 0,
                kind: Some("Service".into()),
                name: Some("api".into()),
                namespace: Some("prod".into()),
            }
        );
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
    fn returns_resource_refs_for_all_documents() {
        let snapshot = Snapshot::new(
            "test.yaml".into(),
            "---\nkind: Service\nmetadata:\n  name: api\n---\nfoo: bar\n".into(),
        )
        .expect("snapshot should parse");

        let resources = snapshot.resources();

        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].kind.as_deref(), Some("Service"));
        assert_eq!(resources[0].name.as_deref(), Some("api"));
        assert_eq!(resources[1].kind, None);
    }

    #[test]
    fn invalid_yaml_returns_contextual_error() {
        let error = Snapshot::new("broken.yaml".into(), "tls: [".into())
            .expect_err("snapshot parsing should fail");

        assert!(error.to_string().contains("broken.yaml"));
        assert!(error.to_string().contains("document 0"));
    }
}
