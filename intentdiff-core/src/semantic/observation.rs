use crate::semantic::rule_id::RuleId;
use crate::snapshot::ResourceRef;
use crate::SignalCategory;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObservationValue {
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntentObservation {
    pub rule_id: RuleId,
    pub resource: ResourceRef,
    pub category: SignalCategory,
    pub value: ObservationValue,
    pub description: String,
    pub source_path: String,
}
