use crate::{RuleId, SignalStrength, semantic::signal::IntentSignal};

use std::collections::HashMap;

#[derive(Debug)]
pub struct DiffResult {
    pub added: Vec<IntentSignal>,
    pub removed: Vec<IntentSignal>,
    pub severity_changed: Vec<SeverityChange>,
}

#[derive(Debug)]
pub struct SeverityChange {
    pub signal: IntentSignal,
    pub from: SignalStrength,
    pub to: SignalStrength,
}

pub fn diff_signals(left: &[IntentSignal], right: &[IntentSignal]) -> DiffResult {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut severity_changed = Vec::new();

    let left_map: HashMap<RuleId, &IntentSignal> = left.iter().map(|s| (s.rule_id, s)).collect();

    let right_map: HashMap<RuleId, &IntentSignal> = right.iter().map(|s| (s.rule_id, s)).collect();

    let all_rule_ids: std::collections::HashSet<_> =
        left_map.keys().chain(right_map.keys()).cloned().collect();

    for rule_id in all_rule_ids {
        match (left_map.get(&rule_id), right_map.get(&rule_id)) {
            (Some(left_signal), None) => {
                removed.push((*left_signal).clone());
            }
            (None, Some(right_signal)) => {
                added.push((*right_signal).clone());
            }
            (Some(left_signal), Some(right_signal)) => {
                if left_signal.strength != right_signal.strength {
                    severity_changed.push(SeverityChange {
                        signal: (*right_signal).clone(),
                        from: left_signal.strength,
                        to: right_signal.strength,
                    });
                }
            }
            (None, None) => unreachable!(),
        }
    }

    DiffResult {
        added,
        removed,
        severity_changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::semantic::rule_id::RuleId;
    use crate::semantic::signal::{SignalCategory, SignalStrength};

    fn signal(
        category: SignalCategory,
        description: &str,
        strength: SignalStrength,
    ) -> IntentSignal {
        IntentSignal {
            rule_id: RuleId("test"),
            category,
            description: description.to_string(),
            strength,
            source_path: "test.yaml".to_string(),
        }
    }

    #[test]
    fn identical_signals_produce_empty_diff() {
        let left = vec![signal(
            SignalCategory::Security,
            "tls enabled",
            SignalStrength::Critical,
        )];
        let right = left.clone();

        let result = diff_signals(&left, &right);

        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert!(result.severity_changed.is_empty());
    }

    #[test]
    fn left_only_signal_detected() {
        let left = vec![signal(
            SignalCategory::Security,
            "tls enabled",
            SignalStrength::Critical,
        )];
        let right = vec![];

        let result = diff_signals(&left, &right);
        assert_eq!(result.removed.len(), 1);
        assert!(result.added.is_empty());
        assert!(result.severity_changed.is_empty());
    }

    #[test]
    fn severity_change_is_detected_as_difference() {
        let left = vec![signal(
            SignalCategory::Security,
            "tls enabled",
            SignalStrength::Critical,
        )];
        let right = vec![signal(
            SignalCategory::Security,
            "tls enabled",
            SignalStrength::Warning,
        )];

        let result = diff_signals(&left, &right);

        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert_eq!(result.severity_changed.len(), 1);

        let change = &result.severity_changed[0];
        assert_eq!(change.from, SignalStrength::Critical);
        assert_eq!(change.to, SignalStrength::Warning);
    }
}
