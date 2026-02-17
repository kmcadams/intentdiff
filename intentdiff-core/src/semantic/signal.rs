use std::hash::{Hash, Hasher};

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

#[derive(Debug, Clone, Eq)]
pub struct IntentSignal {
    pub category: SignalCategory,
    pub strength: SignalStrength,
    pub description: String,
    pub source_path: String,
}

impl PartialEq for IntentSignal {
    fn eq(&self, other: &Self) -> bool {
        self.category == other.category
            && self.strength == other.strength
            && self.description == other.description
            && self.source_path == other.source_path
    }
}

impl Hash for IntentSignal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.category.hash(state);
        self.description.hash(state);
        self.source_path.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                category: SignalCategory::Security,
                description: "test".into(),
                strength: SignalStrength::Informational,
                source_path: "x".into(),
            },
            IntentSignal {
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
