use crate::semantic::rule_id::RuleId;
use crate::snapshot::ResourceRef;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum SignalStrength {
    Informational = 0,
    Warning = 1,
    Critical = 2,
}

impl SignalStrength {
    pub fn highest(signals: &[IntentSignal]) -> Option<Self> {
        signals.iter().map(|s| s.strength).max()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignalCategory {
    Security,
    Authentication,
    Transport,
    Persistence,
    NetworkExposure,
    Runtime,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntentSignal {
    pub rule_id: RuleId,
    pub resource: ResourceRef,
    pub category: SignalCategory,
    pub strength: SignalStrength,
    pub description: String,
    pub source_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::ResourceRef;

    #[test]
    fn severity_ordering_is_correct() {
        assert!(SignalStrength::Warning > SignalStrength::Informational);
        assert!(SignalStrength::Critical > SignalStrength::Warning);
        assert!(SignalStrength::Critical > SignalStrength::Informational);
        assert!(SignalStrength::Informational < SignalStrength::Warning);
        assert!(SignalStrength::Warning < SignalStrength::Critical);
    }

    #[test]
    fn highest_severity_is_detected() {
        let signals = vec![
            IntentSignal {
                rule_id: RuleId("test"),
                resource: ResourceRef {
                    document_index: 0,
                    kind: Some("Service".into()),
                    name: Some("api".into()),
                    namespace: Some("default".into()),
                },
                category: SignalCategory::Security,
                description: "test".into(),
                strength: SignalStrength::Informational,
                source_path: "x".into(),
            },
            IntentSignal {
                rule_id: RuleId("test2"),
                resource: ResourceRef {
                    document_index: 1,
                    kind: Some("Deployment".into()),
                    name: Some("api".into()),
                    namespace: Some("default".into()),
                },
                category: SignalCategory::Security,
                description: "test2".into(),
                strength: SignalStrength::Critical,
                source_path: "x".into(),
            },
        ];

        let highest = SignalStrength::highest(&signals);
        assert_eq!(highest, Some(SignalStrength::Critical));
    }
}
