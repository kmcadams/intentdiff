use crate::{IntentSignal, RuleId, SignalCategory, SignalStrength, Snapshot};

use crate::semantic::rule::Rule;

pub struct TlsEnabledRule;

impl Rule for TlsEnabledRule {
    fn id(&self) -> RuleId {
        RuleId::TRANSPORT_TLS_ENABLED
    }

    fn evaluate(&self, snapshot: &Snapshot) -> Vec<IntentSignal> {
        if snapshot.raw_content.contains("tls: true") {
            return vec![IntentSignal {
                rule_id: self.id(),
                category: SignalCategory::Transport,
                strength: SignalStrength::Critical,
                description: "TLS explicitly enabled".into(),
                source_path: snapshot.source.display().to_string(),
            }];
        }

        vec![]
    }
}
