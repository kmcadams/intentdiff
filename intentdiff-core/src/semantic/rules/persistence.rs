use crate::{RuleId, SignalCategory, SignalStrength};

use crate::semantic::rule::{Rule, RuleMatch, RuleMeta};
use crate::snapshot::SnapshotDocument;

pub struct EmptyDirRule;

impl Rule for EmptyDirRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::PERSISTENCE_EMPTYDIR,
            category: SignalCategory::Persistence,
            default_severity: SignalStrength::Warning,
        }
    }
    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleMatch> {
        if document.contains_key("emptyDir") {
            Some(RuleMatch {
                strength: self.meta().default_severity,
                description: format!("{} uses emptyDir storage", document.display_name()),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let matched = rule.evaluate(&document).expect("rule should match");

        assert_eq!(matched.strength, SignalStrength::Warning);
        assert!(matched.description.contains("emptyDir"));
    }

    #[test]
    fn does_not_detect_emptydir_when_absent() {
        let rule = EmptyDirRule;
        let document = first_document("volumes:\n  - name: data");

        assert!(rule.evaluate(&document).is_none());
    }

    #[test]
    fn meta_is_correct() {
        let rule = EmptyDirRule;
        let meta = rule.meta();

        assert_eq!(meta.id, RuleId::PERSISTENCE_EMPTYDIR);
        assert_eq!(meta.category, SignalCategory::Persistence);
        assert_eq!(meta.default_severity, SignalStrength::Warning);
    }
}
