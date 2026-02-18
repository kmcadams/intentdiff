use crate::{
    Snapshot,
    diff::{DiffResult, diff_signals},
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

        diff_signals(&left_signals, &right_signals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Snapshot;
    use crate::semantic::{BasicAnalyzer, rules::persistence::EmptyDirRule};

    #[test]
    fn engine_runs_and_diffs() {
        let analyzer = Box::new(BasicAnalyzer::new(vec![Box::new(EmptyDirRule)]));

        let engine = Engine::new(analyzer);

        let left = Snapshot::new("a.yaml".into(), "emptyDir".into());
        let right = Snapshot::new("b.yaml".into(), "".into());

        let result = engine.run(left, right);

        assert_eq!(result.only_in_left.len(), 1);
    }
}
