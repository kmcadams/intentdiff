use crate::{
    RuleId,
    semantic::observation::IntentObservation,
    snapshot::ResourceMatchKey,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct DiffResult {
    pub added: Vec<IntentObservation>,
    pub removed: Vec<IntentObservation>,
    pub changed: Vec<ObservationChange>,
}

#[derive(Debug)]
pub struct ObservationChange {
    pub left: IntentObservation,
    pub right: IntentObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct SignalKey {
    rule_id: RuleId,
    resource: ResourceMatchKey,
}

pub fn diff_observations(left: &[IntentObservation], right: &[IntentObservation]) -> DiffResult {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    let left_map: HashMap<SignalKey, &IntentObservation> =
        left.iter().map(|s| (SignalKey::from(s), s)).collect();

    let right_map: HashMap<SignalKey, &IntentObservation> =
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
                if left_signal.value != right_signal.value {
                    changed.push(ObservationChange {
                        left: (*left_signal).clone(),
                        right: (*right_signal).clone(),
                    });
                }
            }
            (None, None) => unreachable!(),
        }
    }

    added.sort_by_key(|signal| SignalKey::from(signal));
    removed.sort_by_key(|signal| SignalKey::from(signal));
    changed.sort_by_key(|change| SignalKey::from(&change.right));

    DiffResult {
        added,
        removed,
        changed,
    }
}

impl From<&IntentObservation> for SignalKey {
    fn from(value: &IntentObservation) -> Self {
        Self {
            rule_id: value.rule_id,
            resource: ResourceMatchKey::from(&value.resource),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::semantic::rule_id::RuleId;
    use crate::semantic::{
        observation::{IntentObservation, ObservationValue},
        signal::SignalCategory,
    };
    use crate::snapshot::ResourceRef;

    fn observation(
        rule_id: RuleId,
        document_index: usize,
        category: SignalCategory,
        description: &str,
        value: ObservationValue,
    ) -> IntentObservation {
        IntentObservation {
            rule_id,
            resource: ResourceRef {
                document_index,
                kind: Some("Service".into()),
                name: Some(format!("resource-{document_index}")),
                namespace: Some("default".into()),
            },
            category,
            value,
            description: description.to_string(),
            source_path: "test.yaml".to_string(),
        }
    }

    #[test]
    fn identical_observations_produce_empty_diff() {
        let left = vec![observation(
            RuleId("test"),
            0,
            SignalCategory::Security,
            "tls enabled",
            ObservationValue::Bool(true),
        )];
        let right = left.clone();

        let result = diff_observations(&left, &right);

        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert!(result.changed.is_empty());
    }

    #[test]
    fn left_only_observation_detected() {
        let left = vec![observation(
            RuleId("test"),
            0,
            SignalCategory::Security,
            "tls enabled",
            ObservationValue::Bool(true),
        )];
        let right = vec![];

        let result = diff_observations(&left, &right);
        assert_eq!(result.removed.len(), 1);
        assert!(result.added.is_empty());
        assert!(result.changed.is_empty());
    }

    #[test]
    fn value_change_is_detected_as_difference() {
        let left = vec![observation(
            RuleId("test"),
            0,
            SignalCategory::Security,
            "tls enabled",
            ObservationValue::Bool(true),
        )];
        let right = vec![observation(
            RuleId("test"),
            0,
            SignalCategory::Security,
            "tls disabled",
            ObservationValue::Bool(false),
        )];

        let result = diff_observations(&left, &right);

        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert_eq!(result.changed.len(), 1);

        let change = &result.changed[0];
        assert_eq!(change.left.value, ObservationValue::Bool(true));
        assert_eq!(change.right.value, ObservationValue::Bool(false));
    }

    #[test]
    fn same_rule_for_multiple_resources_does_not_collapse() {
        let left = vec![
            observation(
                RuleId("test"),
                0,
                SignalCategory::Security,
                "api enables tls",
                ObservationValue::Bool(true),
            ),
            observation(
                RuleId("test"),
                1,
                SignalCategory::Security,
                "admin enables tls",
                ObservationValue::Bool(true),
            ),
        ];
        let right = vec![observation(
            RuleId("test"),
            0,
            SignalCategory::Security,
            "api enables tls",
            ObservationValue::Bool(true),
        )];

        let result = diff_observations(&left, &right);

        assert_eq!(result.removed.len(), 1);
        assert_eq!(result.removed[0].resource.document_index, 1);
    }

    #[test]
    fn named_resources_match_even_when_document_order_differs() {
        let left = vec![observation(
            RuleId("test"),
            0,
            SignalCategory::Security,
            "api enables tls",
            ObservationValue::Bool(true),
        )];
        let right = vec![IntentObservation {
            rule_id: RuleId("test"),
            resource: ResourceRef {
                document_index: 9,
                kind: Some("Service".into()),
                name: Some("resource-0".into()),
                namespace: Some("default".into()),
            },
            category: SignalCategory::Security,
            value: ObservationValue::Bool(false),
            description: "api disables tls".into(),
            source_path: "test.yaml".into(),
        }];

        let result = diff_observations(&left, &right);

        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
        assert_eq!(result.changed.len(), 1);
    }
}
