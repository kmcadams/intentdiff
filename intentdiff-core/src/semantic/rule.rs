use crate::semantic::rule_id::RuleId;
use crate::snapshot::SnapshotDocument;
use crate::{SignalCategory, SignalStrength};

pub trait Rule {
    fn meta(&self) -> RuleMeta;
    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleMatch>;
}

pub struct RuleMeta {
    pub id: RuleId,
    pub category: SignalCategory,
    pub default_severity: SignalStrength,
}

pub struct RuleMatch {
    pub strength: SignalStrength,
    pub description: String,
}
