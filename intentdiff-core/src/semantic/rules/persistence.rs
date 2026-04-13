use crate::semantic::observation::ObservationValue;
use crate::{RuleId, SignalCategory};

use crate::semantic::rule::{Rule, RuleMeta, RuleObservation};
use crate::snapshot::SnapshotDocument;
use serde_yaml::Value;

pub struct EmptyDirRule;
pub struct StorageModeRule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageMode {
    EmptyDir,
    PersistentVolumeClaim,
    Mixed,
    None,
}

impl From<StorageMode> for ObservationValue {
    fn from(value: StorageMode) -> Self {
        match value {
            StorageMode::EmptyDir => ObservationValue::Keyword("empty_dir"),
            StorageMode::PersistentVolumeClaim => ObservationValue::Keyword("persistent_volume_claim"),
            StorageMode::Mixed => ObservationValue::Keyword("mixed"),
            StorageMode::None => ObservationValue::Keyword("none"),
        }
    }
}

impl Rule for EmptyDirRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::PERSISTENCE_EMPTYDIR,
            category: SignalCategory::Persistence,
            title: "emptyDir usage",
            rationale: "Ephemeral storage changes durability guarantees and restart behavior.",
        }
    }
    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation> {
        let volumes = workload_volumes(document)?;
        let uses_empty_dir = volumes.iter().any(volume_uses_empty_dir);

        Some(RuleObservation {
            value: ObservationValue::Bool(uses_empty_dir),
            description: if uses_empty_dir {
                format!("{} uses emptyDir storage", document.display_name())
            } else {
                format!("{} does not use emptyDir storage", document.display_name())
            },
        })
    }
}

impl Rule for StorageModeRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::PERSISTENCE_STORAGE_MODE,
            category: SignalCategory::Persistence,
            title: "Storage mode",
            rationale: "Storage mode drift changes whether data is ephemeral, durable, or mixed.",
        }
    }

    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation> {
        let volumes = workload_volumes(document)?;

        let has_empty_dir = volumes.iter().any(volume_uses_empty_dir);
        let has_pvc = volumes.iter().any(volume_uses_pvc);

        let mode = match (has_empty_dir, has_pvc) {
            (true, true) => StorageMode::Mixed,
            (true, false) => StorageMode::EmptyDir,
            (false, true) => StorageMode::PersistentVolumeClaim,
            (false, false) => StorageMode::None,
        };

        Some(RuleObservation {
            value: mode.into(),
            description: format!(
                "{} uses {} storage mode",
                document.display_name(),
                match mode {
                    StorageMode::EmptyDir => "empty_dir",
                    StorageMode::PersistentVolumeClaim => "persistent_volume_claim",
                    StorageMode::Mixed => "mixed",
                    StorageMode::None => "none",
                }
            ),
        })
    }
}

fn workload_volumes<'a>(document: &'a SnapshotDocument) -> Option<&'a [Value]> {
    if document.is_kind("Pod") {
        return document.sequence_at_path(&["spec", "volumes"]);
    }

    match document.kind() {
        Some("Deployment" | "StatefulSet" | "DaemonSet" | "ReplicaSet" | "Job") => {
            document.sequence_at_path(&["spec", "template", "spec", "volumes"])
        }
        Some("CronJob") => {
            document.sequence_at_path(&["spec", "jobTemplate", "spec", "template", "spec", "volumes"])
        }
        _ => {
            if document.contains_key("volumes") {
                document.sequence_at_path(&["volumes"])
            } else {
                None
            }
        }
    }
}

fn volume_uses_empty_dir(volume: &Value) -> bool {
    matches!(volume, Value::Mapping(mapping) if mapping.contains_key(Value::String("emptyDir".into())))
}

fn volume_uses_pvc(volume: &Value) -> bool {
    matches!(volume, Value::Mapping(mapping) if mapping.contains_key(Value::String("persistentVolumeClaim".into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::observation::ObservationValue;
    use crate::{Snapshot, SnapshotDocument};

    fn snapshot_with(content: &str) -> Snapshot {
        Snapshot::new("test.yaml".into(), content.into()).expect("snapshot should parse")
    }

    fn first_document(content: &str) -> SnapshotDocument {
        snapshot_with(content).documents()[0].clone()
    }

    #[test]
    fn detects_emptydir_when_present() {
        let rule = EmptyDirRule;
        let document = first_document(
            "kind: Deployment\nspec:\n  template:\n    spec:\n      volumes:\n        - emptyDir: {}\n",
        );

        let observation = rule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Bool(true));
        assert!(observation.description.contains("emptyDir"));
    }

    #[test]
    fn does_not_detect_emptydir_when_absent() {
        let rule = EmptyDirRule;
        let document = first_document(
            "kind: Deployment\nspec:\n  template:\n    spec:\n      volumes:\n        - name: data\n          persistentVolumeClaim:\n            claimName: app-data\n",
        );

        let observation = rule.evaluate(&document).expect("rule should emit observation");

        assert_eq!(observation.value, ObservationValue::Bool(false));
    }

    #[test]
    fn ignores_documents_without_volume_context() {
        let rule = EmptyDirRule;
        let document = first_document("kind: Service\nmetadata:\n  name: web\n");

        assert!(rule.evaluate(&document).is_none());
    }

    #[test]
    fn meta_is_correct() {
        let rule = EmptyDirRule;
        let meta = rule.meta();

        assert_eq!(meta.id, RuleId::PERSISTENCE_EMPTYDIR);
        assert_eq!(meta.category, SignalCategory::Persistence);
        assert!(meta.rationale.contains("durability"));
    }

    #[test]
    fn detects_storage_mode_as_pvc() {
        let rule = StorageModeRule;
        let document = first_document(
            "kind: Deployment\nspec:\n  template:\n    spec:\n      volumes:\n        - name: data\n          persistentVolumeClaim:\n            claimName: app-data\n",
        );

        let observation = rule.evaluate(&document).expect("rule should match");

        assert_eq!(
            observation.value,
            ObservationValue::Keyword("persistent_volume_claim")
        );
    }

    #[test]
    fn detects_storage_mode_as_mixed() {
        let rule = StorageModeRule;
        let document = first_document(
            "kind: Deployment\nspec:\n  template:\n    spec:\n      volumes:\n        - name: cache\n          emptyDir: {}\n        - name: data\n          persistentVolumeClaim:\n            claimName: app-data\n",
        );

        let observation = rule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("mixed"));
    }
}
