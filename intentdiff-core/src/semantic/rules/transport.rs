use crate::semantic::observation::ObservationValue;
use crate::{RuleId, SignalCategory};

use crate::semantic::rule::{Rule, RuleMeta, RuleObservation};
use crate::snapshot::SnapshotDocument;

pub struct TlsEnabledRule;

impl Rule for TlsEnabledRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::TRANSPORT_TLS_ENABLED,
            category: SignalCategory::Transport,
        }
    }

    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation> {
        match (
            document.key_has_bool_value("tls", true),
            document.key_has_bool_value("tls", false),
        ) {
            (true, _) => Some(RuleObservation {
                value: ObservationValue::Bool(true),
                description: format!("{} enables TLS", document.display_name()),
            }),
            (_, true) => Some(RuleObservation {
                value: ObservationValue::Bool(false),
                description: format!("{} disables TLS", document.display_name()),
            }),
            _ => None,
        }
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
    fn detects_tls_enabled_when_present() {
        let rule = TlsEnabledRule;
        let document = first_document("tls: true");

        let observation = rule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Bool(true));
        assert!(observation.description.contains("enables TLS"));
    }

    #[test]
    fn does_not_detect_tls_when_absent() {
        let rule = TlsEnabledRule;
        let document = first_document("foo: bar");

        assert!(rule.evaluate(&document).is_none());
    }

    #[test]
    fn meta_is_correct() {
        let rule = TlsEnabledRule;
        let meta = rule.meta();

        assert_eq!(meta.id, RuleId::TRANSPORT_TLS_ENABLED);
        assert_eq!(meta.category, SignalCategory::Transport);
    }
}
