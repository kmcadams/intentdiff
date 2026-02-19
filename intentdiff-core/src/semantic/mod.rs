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

        for rule in &self.rules {
            if rule.evaluate(snapshot) {
                let meta = rule.meta();

                signals.push(IntentSignal {
                    rule_id: meta.id,
                    category: meta.category,
                    strength: meta.default_severity,
                    description: format!("Rule triggered: {}", meta.id.0),
                    source_path: snapshot.source.display().to_string(),
                });
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
        );

        let signals = analyzer.analyze(&snapshot);

        assert_eq!(signals.len(), 2);

        assert!(
            signals
                .iter()
                .any(|s| s.rule_id == RuleId::PERSISTENCE_EMPTYDIR
                    && s.category == SignalCategory::Persistence
                    && s.strength == SignalStrength::Warning)
        );

        assert!(
            signals
                .iter()
                .any(|s| s.rule_id == RuleId::TRANSPORT_TLS_ENABLED
                    && s.category == SignalCategory::Transport
                    && s.strength == SignalStrength::Critical)
        );
    }

    #[test]
    fn analyzer_emits_only_relevant_signals() {
        let analyzer = BasicAnalyzer::new(vec![Box::new(EmptyDirRule), Box::new(TlsEnabledRule)]);

        let snapshot = Snapshot::new("test.yaml".into(), "tls: true".into());

        let signals = analyzer.analyze(&snapshot);

        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].rule_id, RuleId::TRANSPORT_TLS_ENABLED);
    }
}
