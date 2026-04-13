//! Rules detect facts. They inspect a document, emit an observation value, and
//! provide metadata that later layers can use for reporting.

use crate::semantic::rule_id::RuleId;
use crate::semantic::observation::ObservationValue;
use crate::snapshot::SnapshotDocument;
use crate::SignalCategory;

pub trait Rule {
    fn meta(&self) -> RuleMeta;
    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation>;
}

pub struct RuleMeta {
    pub id: RuleId,
    pub category: SignalCategory,
    pub title: &'static str,
    pub rationale: &'static str,
}

pub struct RuleObservation {
    pub value: ObservationValue,
    pub description: String,
}
