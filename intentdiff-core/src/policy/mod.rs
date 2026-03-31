use crate::{DiffResult, RuleId, SignalStrength};

#[derive(Debug)]
pub struct PolicyReport {
    pub findings: Vec<PolicyFinding>,
}

#[derive(Debug)]
pub struct PolicyFinding {
    pub rule_id: RuleId,
    pub severity: SignalStrength,
    pub message: String,
}

pub trait PolicyEvaluator {
    fn evaluate(&self, diff: &DiffResult) -> PolicyReport;
}

pub struct DefaultPolicyEvaluator;

impl PolicyEvaluator for DefaultPolicyEvaluator {
    fn evaluate(&self, diff: &DiffResult) -> PolicyReport {
        let mut findings = Vec::new();

        for change in &diff.changed {
            findings.push(PolicyFinding {
                rule_id: change.right.rule_id,
                severity: severity_for_rule(change.right.rule_id),
                message: format!(
                    "{} differs for {}: {} -> {}",
                    rule_label(change.right.rule_id),
                    change.right.resource,
                    change.left.value,
                    change.right.value,
                ),
            });
        }

        for observation in &diff.added {
            findings.push(PolicyFinding {
                rule_id: observation.rule_id,
                severity: severity_for_rule(observation.rule_id),
                message: format!(
                    "{} is only present in {} for {}: {}",
                    rule_label(observation.rule_id),
                    observation.source_path,
                    observation.resource,
                    observation.value,
                ),
            });
        }

        for observation in &diff.removed {
            findings.push(PolicyFinding {
                rule_id: observation.rule_id,
                severity: severity_for_rule(observation.rule_id),
                message: format!(
                    "{} is only present in {} for {}: {}",
                    rule_label(observation.rule_id),
                    observation.source_path,
                    observation.resource,
                    observation.value,
                ),
            });
        }

        findings.sort_by_key(|finding| std::cmp::Reverse(finding.severity));

        PolicyReport { findings }
    }
}

impl PolicyReport {
    pub fn highest_severity(&self) -> Option<SignalStrength> {
        self.findings.iter().map(|finding| finding.severity).max()
    }

    pub fn meets_or_exceeds(&self, threshold: SignalStrength) -> bool {
        self.highest_severity()
            .is_some_and(|severity| severity >= threshold)
    }
}

fn severity_for_rule(rule_id: RuleId) -> SignalStrength {
    match rule_id {
        RuleId::TRANSPORT_TLS_ENABLED => SignalStrength::Critical,
        RuleId::PERSISTENCE_EMPTYDIR => SignalStrength::Warning,
        _ => SignalStrength::Informational,
    }
}

fn rule_label(rule_id: RuleId) -> &'static str {
    match rule_id {
        RuleId::TRANSPORT_TLS_ENABLED => "TLS behavior",
        RuleId::PERSISTENCE_EMPTYDIR => "emptyDir usage",
        _ => rule_id.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ObservationValue,
        RuleId,
        diff::diff_observations,
        semantic::{observation::IntentObservation, signal::SignalCategory},
        snapshot::ResourceRef,
    };

    fn observation(
        rule_id: RuleId,
        source_path: &str,
        value: ObservationValue,
    ) -> IntentObservation {
        IntentObservation {
            rule_id,
            resource: ResourceRef {
                document_index: 0,
                kind: Some("Service".into()),
                name: Some("api".into()),
                namespace: Some("default".into()),
            },
            category: SignalCategory::Transport,
            value,
            description: "test".into(),
            source_path: source_path.into(),
        }
    }

    #[test]
    fn changed_tls_observation_is_classified_as_critical() {
        let left = vec![observation(
            RuleId::TRANSPORT_TLS_ENABLED,
            "dev.yaml",
            ObservationValue::Bool(false),
        )];
        let right = vec![observation(
            RuleId::TRANSPORT_TLS_ENABLED,
            "prod.yaml",
            ObservationValue::Bool(true),
        )];

        let diff = diff_observations(&left, &right);
        let report = DefaultPolicyEvaluator.evaluate(&diff);

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, SignalStrength::Critical);
        assert!(report.findings[0].message.contains("false -> true"));
    }

    #[test]
    fn highest_severity_respects_thresholds() {
        let report = PolicyReport {
            findings: vec![PolicyFinding {
                rule_id: RuleId::PERSISTENCE_EMPTYDIR,
                severity: SignalStrength::Warning,
                message: "test".into(),
            }],
        };

        assert!(report.meets_or_exceeds(SignalStrength::Informational));
        assert!(report.meets_or_exceeds(SignalStrength::Warning));
        assert!(!report.meets_or_exceeds(SignalStrength::Critical));
    }
}
