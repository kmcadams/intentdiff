use crate::{
    Snapshot,
    diff::{DiffResult, diff_observations},
    policy::{DefaultPolicyEvaluator, PolicyEvaluator, PolicyReport},
    semantic::SemanticAnalyzer,
};

pub struct Engine {
    analyzer: Box<dyn SemanticAnalyzer>,
    policy: Box<dyn PolicyEvaluator>,
}

pub struct AnalysisResult {
    pub diff: DiffResult,
    pub policy: PolicyReport,
}

impl Engine {
    pub fn new(analyzer: Box<dyn SemanticAnalyzer>) -> Self {
        Self::with_policy(analyzer, Box::new(DefaultPolicyEvaluator))
    }

    pub fn with_policy(
        analyzer: Box<dyn SemanticAnalyzer>,
        policy: Box<dyn PolicyEvaluator>,
    ) -> Self {
        Self { analyzer, policy }
    }

    pub fn run(&self, left: Snapshot, right: Snapshot) -> AnalysisResult {
        let left_observations = self.analyzer.analyze(&left);
        let right_observations = self.analyzer.analyze(&right);

        let diff = diff_observations(&left_observations, &right_observations);
        let policy = self.policy.evaluate(&diff);

        AnalysisResult { diff, policy }
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
            result.diff.removed[0].rule_id,
            crate::RuleId::PERSISTENCE_EMPTYDIR
        );
        assert_eq!(
            result.diff.added[0].rule_id,
            crate::RuleId::TRANSPORT_TLS_ENABLED
        );

        assert!(result.diff.changed.is_empty());
        assert_eq!(result.policy.findings.len(), 2);
    }
}
