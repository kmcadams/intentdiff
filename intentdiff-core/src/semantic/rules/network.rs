use crate::semantic::observation::ObservationValue;
use crate::semantic::rule::{Rule, RuleMeta, RuleObservation};
use crate::snapshot::SnapshotDocument;
use crate::{RuleId, SignalCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExposurePosture {
    Internal,
    Public,
}

impl From<ExposurePosture> for ObservationValue {
    fn from(value: ExposurePosture) -> Self {
        match value {
            ExposurePosture::Internal => ObservationValue::Keyword("internal"),
            ExposurePosture::Public => ObservationValue::Keyword("public"),
        }
    }
}

pub struct PublicExposureRule;

impl Rule for PublicExposureRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::NETWORK_EXPOSURE_PUBLIC,
            category: SignalCategory::NetworkExposure,
            title: "Public exposure",
            rationale: "Exposure drift changes whether a workload is reachable from outside the cluster boundary.",
        }
    }

    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation> {
        let posture = if document.is_kind("Ingress") {
            Some(ExposurePosture::Public)
        } else if document.is_kind("Service") {
            match document.string_at_path(&["spec", "type"]).unwrap_or("ClusterIP") {
                "NodePort" | "LoadBalancer" | "ExternalName" => Some(ExposurePosture::Public),
                _ => Some(ExposurePosture::Internal),
            }
        } else {
            None
        }?;

        Some(RuleObservation {
            value: posture.into(),
            description: match posture {
                ExposurePosture::Internal => {
                    format!("{} is only internally exposed", document.display_name())
                }
                ExposurePosture::Public => {
                    format!("{} is publicly exposed", document.display_name())
                }
            },
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
    fn service_cluster_ip_is_internal() {
        let document = first_document("kind: Service\nmetadata:\n  name: web\nspec:\n  type: ClusterIP\n");
        let observation = PublicExposureRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("internal"));
    }

    #[test]
    fn service_load_balancer_is_public() {
        let document = first_document("kind: Service\nmetadata:\n  name: web\nspec:\n  type: LoadBalancer\n");
        let observation = PublicExposureRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("public"));
    }

    #[test]
    fn ingress_is_public() {
        let document = first_document("kind: Ingress\nmetadata:\n  name: edge\n");
        let observation = PublicExposureRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("public"));
    }
}
