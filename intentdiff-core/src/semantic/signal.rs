//! Shared semantic types. Rules emit observations categorized by domain, while
//! policy assigns a severity to the resulting drift.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum Severity {
    Informational = 0,
    Warning = 1,
    Critical = 2,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Severity::Critical => "CRITICAL",
            Severity::Warning => "WARNING",
            Severity::Informational => "INFORMATIONAL",
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
        assert!(Severity::Warning > Severity::Informational);
        assert!(Severity::Critical > Severity::Warning);
        assert!(Severity::Critical > Severity::Informational);
        assert!(Severity::Informational < Severity::Warning);
        assert!(Severity::Warning < Severity::Critical);
    }

    #[test]
    fn display_formats_uppercase_labels() {
        assert_eq!(Severity::Critical.to_string(), "CRITICAL");
        assert_eq!(Severity::Warning.to_string(), "WARNING");
        assert_eq!(Severity::Informational.to_string(), "INFORMATIONAL");
    }
}
