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
            title: "TLS behavior",
            rationale: "TLS drift changes whether edge traffic is encrypted in transit.",
        }
    }

    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation> {
        match explicit_tls_state(document).or_else(|| inferred_tls_state(document)) {
            Some(true) => Some(RuleObservation {
                value: ObservationValue::Bool(true),
                description: format!("{} enables TLS", document.display_name()),
            }),
            Some(false) => Some(RuleObservation {
                value: ObservationValue::Bool(false),
                description: format!("{} disables TLS", document.display_name()),
            }),
            None => None,
        }
    }
}

fn explicit_tls_state(document: &SnapshotDocument) -> Option<bool> {
    if let Some(value) = document.bool_at_path(&["tls"]) {
        return Some(value);
    }

    if let Some(value) = document.bool_at_path(&["spec", "tls"]) {
        return Some(value);
    }

    match (
        document.key_has_bool_value("tls", true),
        document.key_has_bool_value("tls", false),
    ) {
        (true, _) => Some(true),
        (_, true) => Some(false),
        _ => None,
    }
}

fn inferred_tls_state(document: &SnapshotDocument) -> Option<bool> {
    if document.is_kind("Ingress") {
        return Some(
            document
                .sequence_at_path(&["spec", "tls"])
                .is_some_and(|entries| !entries.is_empty()),
        );
    }

    None
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
    fn detects_ingress_tls_from_structured_spec() {
        let rule = TlsEnabledRule;
        let document = first_document(
            "kind: Ingress\nmetadata:\n  name: web\nspec:\n  tls:\n    - secretName: edge-cert\n",
        );

        let observation = rule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Bool(true));
    }

    #[test]
    fn detects_ingress_without_tls_as_disabled() {
        let rule = TlsEnabledRule;
        let document = first_document(
            "kind: Ingress\nmetadata:\n  name: web\nspec:\n  rules:\n    - host: dev.example.local\n",
        );

        let observation = rule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Bool(false));
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
        assert!(meta.rationale.contains("encrypted"));
    }
}
