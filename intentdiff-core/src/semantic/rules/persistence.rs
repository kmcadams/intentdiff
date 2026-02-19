use crate::{RuleId, SignalCategory, SignalStrength, Snapshot};

use crate::semantic::rule::{Rule, RuleMeta};

pub struct EmptyDirRule;

impl Rule for EmptyDirRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::PERSISTENCE_EMPTYDIR,
            category: SignalCategory::Persistence,
            default_severity: SignalStrength::Warning,
        }
    }
    fn evaluate(&self, snapshot: &Snapshot) -> bool {
        snapshot.raw_content.contains("emptyDir")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Snapshot;

    fn snapshot_with(content: &str) -> Snapshot {
        Snapshot::new("test.yaml".into(), content.into())
    }

    #[test]
    fn detects_emptydir_when_present() {
        let rule = EmptyDirRule;
        let snapshot = snapshot_with("volumes:\n  - emptyDir: {}");

        assert!(rule.evaluate(&snapshot));
    }

    #[test]
    fn does_not_detect_emptydir_when_absent() {
        let rule = EmptyDirRule;
        let snapshot = snapshot_with("volumes:\n  - name: data");

        assert!(!rule.evaluate(&snapshot));
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
