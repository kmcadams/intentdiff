use crate::semantic::signal::IntentSignal;

use std::collections::HashSet;

#[derive(Debug)]
pub struct DiffResult {
    pub only_in_left: Vec<IntentSignal>,
    pub only_in_right: Vec<IntentSignal>,
}

pub fn diff_signals(left: &[IntentSignal], right: &[IntentSignal]) -> DiffResult {
    let left_set: HashSet<_> = left.iter().cloned().collect();
    let right_set: HashSet<_> = right.iter().cloned().collect();

    let only_in_left = left_set.difference(&right_set).cloned().collect();

    let only_in_right = right_set.difference(&left_set).cloned().collect();

    DiffResult {
        only_in_left,
        only_in_right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::semantic::signal::SignalStrength;

    fn signal(category: &str, description: &str, strength: SignalStrength) -> IntentSignal {
        IntentSignal {
            category: category.to_string(),
            description: description.to_string(),
            strength,
            source_path: "test.yaml".to_string(),
        }
    }

    #[test]
    fn identical_signals_produce_empty_diff() {
        let left = vec![signal("security", "tls enabled", SignalStrength::Critical)];
        let right = left.clone();

        let result = diff_signals(&left, &right);

        assert!(result.only_in_left.is_empty());
        assert!(result.only_in_right.is_empty());
    }

    #[test]
    fn left_only_signal_detected() {
        let left = vec![signal("security", "tls enabled", SignalStrength::Critical)];
        let right = vec![];

        let result = diff_signals(&left, &right);

        assert_eq!(result.only_in_left.len(), 1);
        assert!(result.only_in_right.is_empty());
    }

    #[test]
    fn severity_change_is_detected_as_difference() {
        let left = vec![signal("security", "tls enabled", SignalStrength::Critical)];
        let right = vec![signal("security", "tls enabled", SignalStrength::Warning)];

        let result = diff_signals(&left, &right);

        assert_eq!(result.only_in_left.len(), 1);
        assert_eq!(result.only_in_right.len(), 1);
    }
}
