use crate::semantic::observation::ObservationValue;
use crate::semantic::rule::{Rule, RuleMeta, RuleObservation};
use crate::snapshot::SnapshotDocument;
use crate::{RuleId, SignalCategory};
use serde_yaml::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NonRootPosture {
    Enforced,
    NotEnforced,
    Partial,
    Unknown,
}

impl From<NonRootPosture> for ObservationValue {
    fn from(value: NonRootPosture) -> Self {
        match value {
            NonRootPosture::Enforced => ObservationValue::Keyword("enforced"),
            NonRootPosture::NotEnforced => ObservationValue::Keyword("not_enforced"),
            NonRootPosture::Partial => ObservationValue::Keyword("partial"),
            NonRootPosture::Unknown => ObservationValue::Keyword("unknown"),
        }
    }
}

pub struct RunAsNonRootRule;

impl Rule for RunAsNonRootRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::SECURITY_RUN_AS_NON_ROOT,
            category: SignalCategory::Security,
            title: "Run as non-root posture",
        }
    }

    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation> {
        let pod_spec = workload_pod_spec(document)?;
        let posture = run_as_non_root_posture(pod_spec);

        Some(RuleObservation {
            value: posture.into(),
            description: format!(
                "{} has a {} run-as-non-root posture",
                document.display_name(),
                match posture {
                    NonRootPosture::Enforced => "enforced",
                    NonRootPosture::NotEnforced => "not_enforced",
                    NonRootPosture::Partial => "partial",
                    NonRootPosture::Unknown => "unknown",
                }
            ),
        })
    }
}

fn workload_pod_spec<'a>(document: &'a SnapshotDocument) -> Option<&'a Value> {
    if document.is_kind("Pod") {
        return document.value_at_path(&["spec"]);
    }

    match document.kind() {
        Some("Deployment" | "StatefulSet" | "DaemonSet" | "ReplicaSet" | "Job") => {
            document.value_at_path(&["spec", "template", "spec"])
        }
        Some("CronJob") => document.value_at_path(&["spec", "jobTemplate", "spec", "template", "spec"]),
        _ => None,
    }
}

fn run_as_non_root_posture(pod_spec: &Value) -> NonRootPosture {
    if bool_in_mapping_path(pod_spec, &["securityContext", "runAsNonRoot"]) == Some(true) {
        return NonRootPosture::Enforced;
    }

    let Some(containers) = sequence_in_mapping_path(pod_spec, &["containers"]) else {
        return NonRootPosture::Unknown;
    };

    let mut has_true = false;
    let mut has_false = false;
    let mut has_missing = false;

    for container in containers {
        match bool_in_mapping_path(container, &["securityContext", "runAsNonRoot"]) {
            Some(true) => has_true = true,
            Some(false) => has_false = true,
            None => has_missing = true,
        }
    }

    match (has_true, has_false, has_missing) {
        (true, false, false) => NonRootPosture::Enforced,
        (false, true, false) => NonRootPosture::NotEnforced,
        (false, false, true) => NonRootPosture::Unknown,
        _ => NonRootPosture::Partial,
    }
}

fn sequence_in_mapping_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a [Value]> {
    match value_in_mapping_path(value, path) {
        Some(Value::Sequence(items)) => Some(items.as_slice()),
        _ => None,
    }
}

fn bool_in_mapping_path(value: &Value, path: &[&str]) -> Option<bool> {
    value_in_mapping_path(value, path).and_then(Value::as_bool)
}

fn value_in_mapping_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;

    for segment in path {
        current = match current {
            Value::Mapping(mapping) => mapping.get(Value::String((*segment).to_owned()))?,
            _ => return None,
        };
    }

    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Snapshot, SnapshotDocument};

    fn first_document(content: &str) -> SnapshotDocument {
        Snapshot::new("test.yaml".into(), content.into())
            .expect("snapshot should parse")
            .documents()[0]
            .clone()
    }

    #[test]
    fn pod_level_non_root_is_enforced() {
        let document = first_document(
            "kind: Deployment\nmetadata:\n  name: web\nspec:\n  template:\n    spec:\n      securityContext:\n        runAsNonRoot: true\n      containers:\n        - name: web\n          image: nginx\n",
        );
        let observation = RunAsNonRootRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("enforced"));
    }

    #[test]
    fn mixed_container_settings_are_partial() {
        let document = first_document(
            "kind: Deployment\nmetadata:\n  name: web\nspec:\n  template:\n    spec:\n      containers:\n        - name: web\n          image: nginx\n          securityContext:\n            runAsNonRoot: true\n        - name: sidecar\n          image: busybox\n",
        );
        let observation = RunAsNonRootRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("partial"));
    }
}
