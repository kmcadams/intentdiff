use crate::semantic::rule_id::RuleId;
use crate::{IntentSignal, Snapshot};

pub trait Rule {
    fn id(&self) -> RuleId;
    fn evaluate(&self, snapshot: &Snapshot) -> Vec<IntentSignal>;
}
