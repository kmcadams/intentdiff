use crate::{IntentSignal, RuleId, SignalCategory, SignalStrength, Snapshot};

use crate::semantic::rule::Rule;

pub struct EmptyDirRule;

impl Rule for EmptyDirRule {
    fn id(&self) -> RuleId {
        RuleId::PERSISTENCE_EMPTYDIR
    }
    fn evaluate(&self, snapshot: &Snapshot) -> Vec<IntentSignal> {
        let mut signals = Vec::new();

        if snapshot.raw_content.contains("emptyDir") {
            signals.push(IntentSignal {
                rule_id: self.id(),
                category: SignalCategory::Persistence,
                strength: SignalStrength::Warning,
                description: "Uses emptyDir volume".into(),
                source_path: snapshot.source.display().to_string(),
            });
        }

        signals
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
    fn emits_signal_when_emptydir_present() {
        let rule = EmptyDirRule;
        let snapshot = snapshot_with("volumes:\n  - emptyDir: {}");

        let signals = rule.evaluate(&snapshot);

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].category, SignalCategory::Persistence);
    }

    #[test]
    fn emits_no_signal_when_emptydir_absent() {
        let rule = EmptyDirRule;
        let snapshot = snapshot_with("volumes:\n  - name: data");

        let signals = rule.evaluate(&snapshot);

        assert!(signals.is_empty());
    }
}
