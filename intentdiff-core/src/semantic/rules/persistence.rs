use crate::semantic::observation::ObservationValue;
use crate::{RuleId, SignalCategory};

use crate::semantic::rule::{Rule, RuleMeta, RuleObservation};
use crate::snapshot::SnapshotDocument;

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
        }
    }
    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation> {
        if !document.contains_key("volumes") && !document.contains_key("emptyDir") {
            return None;
        }

        let uses_empty_dir = document.contains_key("emptyDir");

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
        }
    }

    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation> {
        if !document.contains_key("volumes") {
            return None;
        }

        let has_empty_dir = document.contains_key("emptyDir");
        let has_pvc = document.contains_key("persistentVolumeClaim");

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
        let document = first_document("volumes:\n  - emptyDir: {}");

        let observation = rule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Bool(true));
        assert!(observation.description.contains("emptyDir"));
    }

    #[test]
    fn does_not_detect_emptydir_when_absent() {
        let rule = EmptyDirRule;
        let document = first_document("volumes:\n  - name: data");

        let observation = rule.evaluate(&document).expect("rule should emit observation");

        assert_eq!(observation.value, ObservationValue::Bool(false));
    }

    #[test]
    fn meta_is_correct() {
        let rule = EmptyDirRule;
        let meta = rule.meta();

        assert_eq!(meta.id, RuleId::PERSISTENCE_EMPTYDIR);
        assert_eq!(meta.category, SignalCategory::Persistence);
    }

    #[test]
    fn detects_storage_mode_as_pvc() {
        let rule = StorageModeRule;
        let document = first_document(
            "volumes:\n  - name: data\n    persistentVolumeClaim:\n      claimName: app-data",
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
            "volumes:\n  - name: cache\n    emptyDir: {}\n  - name: data\n    persistentVolumeClaim:\n      claimName: app-data",
        );

        let observation = rule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("mixed"));
    }
}
