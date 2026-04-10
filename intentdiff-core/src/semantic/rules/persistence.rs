use crate::semantic::observation::ObservationValue;
use crate::{RuleId, SignalCategory};

use crate::semantic::rule::{Rule, RuleMeta, RuleObservation};
use crate::snapshot::SnapshotDocument;

pub struct EmptyDirRule;

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
}
