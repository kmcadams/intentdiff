use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum SignalStrength {
    Informational = 0,
    Warning = 1,
    Critical = 2,
}

impl fmt::Display for SignalStrength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            SignalStrength::Critical => "CRITICAL",
            SignalStrength::Warning => "WARNING",
            SignalStrength::Informational => "INFORMATIONAL",
        };

        f.write_str(label)
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
    fn display_formats_uppercase_labels() {
        assert_eq!(SignalStrength::Critical.to_string(), "CRITICAL");
        assert_eq!(SignalStrength::Warning.to_string(), "WARNING");
        assert_eq!(SignalStrength::Informational.to_string(), "INFORMATIONAL");
    }
}
