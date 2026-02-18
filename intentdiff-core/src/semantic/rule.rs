use crate::{IntentSignal, Snapshot};

pub trait Rule {
    fn evaluate(&self, snapshot: &Snapshot) -> Vec<IntentSignal>;
}
