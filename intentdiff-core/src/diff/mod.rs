use crate::{RuleId, SignalStrength, semantic::signal::IntentSignal, snapshot::ResourceRef};
use std::collections::{HashMap, HashSet};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct SignalKey {
    rule_id: RuleId,
    resource: ResourceRef,
}

pub fn diff_signals(left: &[IntentSignal], right: &[IntentSignal]) -> DiffResult {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut severity_changed = Vec::new();

    let left_map: HashMap<SignalKey, &IntentSignal> =
        left.iter().map(|s| (SignalKey::from(s), s)).collect();

    let right_map: HashMap<SignalKey, &IntentSignal> =
        right.iter().map(|s| (SignalKey::from(s), s)).collect();

    let all_signal_keys: HashSet<_> = left_map.keys().chain(right_map.keys()).cloned().collect();

    for signal_key in all_signal_keys {
        match (left_map.get(&signal_key), right_map.get(&signal_key)) {
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

    added.sort_by_key(|signal| SignalKey::from(signal));
    removed.sort_by_key(|signal| SignalKey::from(signal));
    severity_changed.sort_by_key(|change| SignalKey::from(&change.signal));

    DiffResult {
        added,
        removed,
        severity_changed,
    }
}

impl From<&IntentSignal> for SignalKey {
    fn from(value: &IntentSignal) -> Self {
        Self {
            rule_id: value.rule_id,
            resource: value.resource.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::semantic::rule_id::RuleId;
    use crate::semantic::signal::{SignalCategory, SignalStrength};
    use crate::snapshot::ResourceRef;

    fn signal(
        rule_id: RuleId,
        document_index: usize,
        category: SignalCategory,
        description: &str,
        strength: SignalStrength,
    ) -> IntentSignal {
        IntentSignal {
            rule_id,
            resource: ResourceRef {
                document_index,
                kind: Some("Service".into()),
                name: Some(format!("resource-{document_index}")),
                namespace: Some("default".into()),
            },
            category,
            description: description.to_string(),
            strength,
            source_path: "test.yaml".to_string(),
        }
    }

    #[test]
    fn identical_signals_produce_empty_diff() {
        let left = vec![signal(
            RuleId("test"),
            0,
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
            RuleId("test"),
            0,
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
            RuleId("test"),
            0,
            SignalCategory::Security,
            "tls enabled",
            SignalStrength::Critical,
        )];
        let right = vec![signal(
            RuleId("test"),
            0,
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

    #[test]
    fn same_rule_for_multiple_resources_does_not_collapse() {
        let left = vec![
            signal(
                RuleId("test"),
                0,
                SignalCategory::Security,
                "api enables tls",
                SignalStrength::Informational,
            ),
            signal(
                RuleId("test"),
                1,
                SignalCategory::Security,
                "admin enables tls",
                SignalStrength::Informational,
            ),
        ];
        let right = vec![signal(
            RuleId("test"),
            0,
            SignalCategory::Security,
            "api enables tls",
            SignalStrength::Informational,
        )];

        let result = diff_signals(&left, &right);

        assert_eq!(result.removed.len(), 1);
        assert_eq!(result.removed[0].resource.document_index, 1);
    }
}
