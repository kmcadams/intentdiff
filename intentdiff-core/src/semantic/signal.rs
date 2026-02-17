use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignalStrength {
    Informational,
    Warning,
    Critical,
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
