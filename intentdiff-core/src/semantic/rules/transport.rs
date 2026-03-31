use crate::{RuleId, SignalCategory, SignalStrength};

use crate::semantic::rule::{Rule, RuleMatch, RuleMeta};
use crate::snapshot::SnapshotDocument;

pub struct TlsEnabledRule;

impl Rule for TlsEnabledRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::TRANSPORT_TLS_ENABLED,
            category: SignalCategory::Transport,
            default_severity: SignalStrength::Critical,
        }
    }

    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleMatch> {
        match (
            document.key_has_bool_value("tls", true),
            document.key_has_bool_value("tls", false),
        ) {
            (true, _) => Some(RuleMatch {
                strength: SignalStrength::Informational,
                description: format!("{} enables TLS", document.display_name()),
            }),
            (_, true) => Some(RuleMatch {
                strength: SignalStrength::Critical,
                description: format!("{} disables TLS", document.display_name()),
            }),
            _ => None,
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
    fn detects_tls_enabled_when_present() {
        let rule = TlsEnabledRule;
        let document = first_document("tls: true");

        let matched = rule.evaluate(&document).expect("rule should match");

        assert_eq!(matched.strength, SignalStrength::Informational);
        assert!(matched.description.contains("enables TLS"));
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
        assert_eq!(meta.default_severity, SignalStrength::Critical);
    }
}
