pub mod signal;

use crate::{IntentSignal, SignalCategory, SignalStrength, snapshot::Snapshot};

pub trait SemanticAnalyzer {
    fn analyze(&self, snapshot: &Snapshot) -> Vec<IntentSignal>;
}

pub struct BasicAnalyzer;

impl SemanticAnalyzer for BasicAnalyzer {
    fn analyze(&self, snapshot: &Snapshot) -> Vec<IntentSignal> {
        let mut signals = Vec::new();

        let content = &snapshot.raw_content;

        if content.contains("emptyDir") {
            signals.push(IntentSignal {
                category: SignalCategory::Persistence,
                strength: SignalStrength::Warning,
                description: "Uses emptyDir volume".into(),
                source_path: snapshot.source.display().to_string(),
            });
        }

        if content.contains("tls: true") {
            signals.push(IntentSignal {
                category: SignalCategory::Transport,
                strength: SignalStrength::Critical,
                description: "TLS explicitly enabled".into(),
                source_path: snapshot.source.display().to_string(),
            });
        }

        signals
    }
}
