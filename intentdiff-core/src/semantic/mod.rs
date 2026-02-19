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
    use crate::Snapshot;
    use crate::semantic::rules::persistence::EmptyDirRule;

    #[test]
    fn analyzer_aggregates_rule_results() {
        let analyzer = BasicAnalyzer::new(vec![Box::new(EmptyDirRule)]);

        let snapshot = Snapshot::new("test.yaml".into(), "emptyDir".into());

        let signals = analyzer.analyze(&snapshot);

        assert_eq!(signals.len(), 1);
    }
}
