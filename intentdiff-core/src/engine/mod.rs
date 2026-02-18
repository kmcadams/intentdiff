use crate::{
    Snapshot,
    diff::{diff_signals, DiffResult},
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
