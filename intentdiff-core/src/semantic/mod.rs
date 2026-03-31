pub mod profile;
pub mod rule;
pub mod rule_id;
pub mod rules;
pub mod signal;

use crate::{IntentSignal, snapshot::Snapshot};
use rule::Rule;
pub use rules::persistence::EmptyDirRule;

pub trait SemanticAnalyzer {
    fn analyze(&self, snapshot: &Snapshot) -> Vec<IntentSignal>;
}

pub struct BasicAnalyzer {
    rules: Vec<Box<dyn Rule>>,
}

impl BasicAnalyzer {
    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
    }
}

impl SemanticAnalyzer for BasicAnalyzer {
    fn analyze(&self, snapshot: &Snapshot) -> Vec<IntentSignal> {
        let mut signals = Vec::new();

        for document in snapshot.documents() {
            for rule in &self.rules {
                if let Some(rule_match) = rule.evaluate(document) {
                    let meta = rule.meta();

                    signals.push(IntentSignal {
                        rule_id: meta.id,
                        resource: document.resource_ref(),
                        category: meta.category,
                        strength: rule_match.strength,
                        description: rule_match.description,
                        source_path: snapshot.source.display().to_string(),
                    });
                }
            }
        }

        signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::rules::{persistence::EmptyDirRule, transport::TlsEnabledRule};
    use crate::{RuleId, SignalCategory, SignalStrength, Snapshot};

    #[test]
    fn analyzer_emits_multiple_signals_when_multiple_rules_trigger() {
        let analyzer = BasicAnalyzer::new(vec![Box::new(EmptyDirRule), Box::new(TlsEnabledRule)]);

        let snapshot = Snapshot::new(
            "test.yaml".into(),
            r#"
            volumes:
              - emptyDir: {}
            tls: true
            "#
            .into(),
        )
        .expect("snapshot should parse");

        let signals = analyzer.analyze(&snapshot);

        assert_eq!(signals.len(), 2);
        let tls_signal = signals
            .iter()
            .find(|s| s.rule_id == RuleId::TRANSPORT_TLS_ENABLED)
            .expect("TLS signal missing");

        assert_eq!(tls_signal.category, SignalCategory::Transport);
        assert_eq!(tls_signal.strength, SignalStrength::Informational);
        assert_eq!(tls_signal.resource.document_index, 0);
    }

    #[test]
    fn analyzer_emits_only_relevant_signals() {
        let analyzer = BasicAnalyzer::new(vec![Box::new(EmptyDirRule), Box::new(TlsEnabledRule)]);

        let snapshot =
            Snapshot::new("test.yaml".into(), "tls: true".into()).expect("snapshot should parse");

        let signals = analyzer.analyze(&snapshot);

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].rule_id, RuleId::TRANSPORT_TLS_ENABLED);
    }

    #[test]
    fn analyzer_emits_one_signal_per_matching_document() {
        let analyzer = BasicAnalyzer::new(vec![Box::new(TlsEnabledRule)]);

        let snapshot = Snapshot::new(
            "test.yaml".into(),
            "---\nkind: Service\nmetadata:\n  name: api\nspec:\n  tls: true\n---\nkind: Service\nmetadata:\n  name: admin\nspec:\n  tls: true\n"
                .into(),
        )
        .expect("snapshot should parse");

        let signals = analyzer.analyze(&snapshot);

        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].resource.name.as_deref(), Some("api"));
        assert_eq!(signals[1].resource.name.as_deref(), Some("admin"));
    }
}
