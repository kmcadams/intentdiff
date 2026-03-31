use crate::{
    Snapshot,
    diff::{DiffResult, diff_observations},
    semantic::SemanticAnalyzer,
};

pub struct Engine {
    analyzer: Box<dyn SemanticAnalyzer>,
}

impl Engine {
    pub fn new(analyzer: Box<dyn SemanticAnalyzer>) -> Self {
        Self { analyzer }
    }

    pub fn run(&self, left: Snapshot, right: Snapshot) -> DiffResult {
        let left_signals = self.analyzer.analyze(&left);
        let right_signals = self.analyzer.analyze(&right);

        diff_observations(&left_signals, &right_signals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{
        BasicAnalyzer,
        rules::{persistence::EmptyDirRule, transport::TlsEnabledRule},
    };

    #[test]
    fn engine_diff_detects_rule_difference() {
        let analyzer = Box::new(BasicAnalyzer::new(vec![
            Box::new(EmptyDirRule),
            Box::new(TlsEnabledRule),
        ]));

        let engine = Engine::new(analyzer);

        let left = Snapshot::new("left.yaml".into(), "volumes:\n  - emptyDir: {}\n".into())
            .expect("left snapshot should parse");
        let right = Snapshot::new("right.yaml".into(), "tls: true".into())
            .expect("right snapshot should parse");

        let result = engine.run(left, right);

        assert_eq!(
            result.removed[0].rule_id,
            crate::RuleId::PERSISTENCE_EMPTYDIR
        );
        assert_eq!(
            result.added[0].rule_id,
            crate::RuleId::TRANSPORT_TLS_ENABLED
        );

        assert!(result.changed.is_empty());
    }
}
