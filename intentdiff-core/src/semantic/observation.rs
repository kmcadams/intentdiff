//! An observation is a detected fact emitted by a rule.

use crate::semantic::rule_id::RuleId;
use crate::snapshot::ResourceRef;
use crate::SignalCategory;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObservationValue {
    Bool(bool),
    Keyword(&'static str),
    Integer(i64),
    String(String),
}

impl fmt::Display for ObservationValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObservationValue::Bool(flag) => write!(f, "{flag}"),
            ObservationValue::Keyword(value) => f.write_str(value),
            ObservationValue::Integer(value) => write!(f, "{value}"),
            ObservationValue::String(value) => f.write_str(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntentObservation {
    pub rule_id: RuleId,
    pub rule_title: &'static str,
    pub rule_rationale: &'static str,
    pub resource: ResourceRef,
    pub category: SignalCategory,
    pub value: ObservationValue,
    pub description: String,
    pub source_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_value_display_is_human_readable() {
        assert_eq!(ObservationValue::Bool(true).to_string(), "true");
        assert_eq!(ObservationValue::Keyword("public").to_string(), "public");
        assert_eq!(ObservationValue::Integer(3).to_string(), "3");
        assert_eq!(
            ObservationValue::String("empty_dir".into()).to_string(),
            "empty_dir"
        );
    }
}
