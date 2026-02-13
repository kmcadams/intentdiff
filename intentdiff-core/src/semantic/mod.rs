pub mod signal;

use crate::snapshot::Snapshot;
use signal::IntentSignal;

pub trait SemanticAnalyzer {
    fn analyze(&self, snapshot: &Snapshot) -> Vec<IntentSignal>;
}
