use crate::semantic::observation::ObservationValue;
use crate::semantic::rule::{Rule, RuleMeta, RuleObservation};
use crate::snapshot::SnapshotDocument;
use crate::{RuleId, SignalCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplicaPosture {
    ScaledToZero,
    SingleReplica,
    Replicated,
}

impl From<ReplicaPosture> for ObservationValue {
    fn from(value: ReplicaPosture) -> Self {
        match value {
            ReplicaPosture::ScaledToZero => ObservationValue::Keyword("scaled_to_zero"),
            ReplicaPosture::SingleReplica => ObservationValue::Keyword("single_replica"),
            ReplicaPosture::Replicated => ObservationValue::Keyword("replicated"),
        }
    }
}

pub struct ReplicaPostureRule;

impl Rule for ReplicaPostureRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::AVAILABILITY_REPLICA_POSTURE,
            category: SignalCategory::Availability,
            title: "Replica posture",
            rationale: "Replica drift changes baseline availability and failure tolerance.",
        }
    }

    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation> {
        if !matches!(document.kind(), Some("Deployment" | "StatefulSet" | "ReplicaSet")) {
            return None;
        }

        let replicas = document.integer_at_path(&["spec", "replicas"]).unwrap_or(1);
        let posture = match replicas {
            i if i <= 0 => ReplicaPosture::ScaledToZero,
            1 => ReplicaPosture::SingleReplica,
            _ => ReplicaPosture::Replicated,
        };

        Some(RuleObservation {
            value: posture.into(),
            description: format!("{} has a {posture_text} posture", document.display_name(), posture_text = match posture {
                ReplicaPosture::ScaledToZero => "scaled_to_zero",
                ReplicaPosture::SingleReplica => "single_replica",
                ReplicaPosture::Replicated => "replicated",
            }),
        })
    }
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
    fn missing_replicas_defaults_to_single() {
        let document = first_document("kind: Deployment\nmetadata:\n  name: web\nspec:\n  template:\n    spec:\n      containers: []\n");
        let observation = ReplicaPostureRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("single_replica"));
    }

    #[test]
    fn multiple_replicas_are_replicated() {
        let document = first_document("kind: Deployment\nmetadata:\n  name: web\nspec:\n  replicas: 3\n  template:\n    spec:\n      containers: []\n");
        let observation = ReplicaPostureRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("replicated"));
    }
}
