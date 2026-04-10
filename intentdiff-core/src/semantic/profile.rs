//! Profiles bundle rules and policy for a concrete use case.

use crate::engine::Engine;
use crate::policy::{DefaultPolicyEvaluator, PolicyEvaluator};
use crate::semantic::rule::Rule;
use crate::semantic::rules::{persistence::EmptyDirRule, transport::TlsEnabledRule};
use crate::BasicAnalyzer;

pub struct Profile {
    pub name: &'static str,
    rules: Vec<Box<dyn Rule>>,
    policy: Box<dyn PolicyEvaluator>,
}

impl Profile {
    pub fn k8s_web() -> Self {
        Self {
            name: "k8s-web",
            rules: vec![Box::new(EmptyDirRule), Box::new(TlsEnabledRule)],
            policy: Box::new(DefaultPolicyEvaluator),
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "k8s-web" => Some(Self::k8s_web()),
            _ => None,
        }
    }

    pub fn build_analyzer(self) -> BasicAnalyzer {
        BasicAnalyzer::new(self.rules)
    }

    pub fn build_engine(self) -> Engine {
        let analyzer = BasicAnalyzer::new(self.rules);
        Engine::new(Box::new(analyzer), self.policy)
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
