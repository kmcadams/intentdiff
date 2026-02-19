use crate::{RuleId, SignalCategory, SignalStrength, Snapshot};

use crate::semantic::rule::{Rule, RuleMeta};

pub struct TlsEnabledRule;

impl Rule for TlsEnabledRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::TRANSPORT_TLS_ENABLED,
            category: SignalCategory::Transport,
            default_severity: SignalStrength::Critical,
        }
    }

    fn evaluate(&self, snapshot: &Snapshot) -> Option<SignalStrength> {
        let content = &snapshot.raw_content;

        match (
            content.contains("tls: true"),
            content.contains("tls: false"),
        ) {
            (true, _) => Some(SignalStrength::Informational),
            (_, true) => Some(SignalStrength::Critical),
            _ => None,
        }
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
    fn detects_tls_enabled_when_present() {
        let rule = TlsEnabledRule;
        let snapshot = snapshot_with("tls: true");

        assert_eq!(
            rule.evaluate(&snapshot),
            Some(SignalStrength::Informational)
        );
    }

    #[test]
    fn does_not_detect_tls_when_absent() {
        let rule = TlsEnabledRule;
        let snapshot = snapshot_with("no tls here");

        assert_eq!(rule.evaluate(&snapshot), None);
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
