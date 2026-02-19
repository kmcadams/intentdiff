use crate::BasicAnalyzer;
use crate::semantic::rule::Rule;
use crate::semantic::rules::{persistence::EmptyDirRule, transport::TlsEnabledRule};

pub struct Profile {
    pub name: &'static str,
    rules: Vec<Box<dyn Rule>>,
}

impl Profile {
    pub fn k8s_web() -> Self {
        Self {
            name: "k8s-web",
            rules: vec![Box::new(EmptyDirRule), Box::new(TlsEnabledRule)],
        }
    }

    pub fn build_analyzer(self) -> BasicAnalyzer {
        BasicAnalyzer::new(self.rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k8s_web_profile_contains_expected_rules() {
        let profile = Profile::k8s_web();
        assert_eq!(profile.rules.len(), 2);
    }
}
