use crate::semantic::rule_id::RuleId;
use crate::{SignalCategory, SignalStrength, Snapshot};

pub trait Rule {
    // fn id(&self) -> RuleId;
    fn meta(&self) -> RuleMeta;
    fn evaluate(&self, snapshot: &Snapshot) -> bool;
}

pub struct RuleMeta {
    pub id: RuleId,
    pub category: SignalCategory,
    pub default_severity: SignalStrength,
}
