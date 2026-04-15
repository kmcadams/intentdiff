use crate::semantic::observation::ObservationValue;
use crate::semantic::rule::{Rule, RuleMeta, RuleObservation};
use crate::snapshot::SnapshotDocument;
use crate::{RuleId, SignalCategory};
use serde_yaml::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthMode {
    None,
    Basic,
    Oidc,
    Unknown,
}

impl From<AuthMode> for ObservationValue {
    fn from(value: AuthMode) -> Self {
        match value {
            AuthMode::None => ObservationValue::Keyword("none"),
            AuthMode::Basic => ObservationValue::Keyword("basic"),
            AuthMode::Oidc => ObservationValue::Keyword("oidc"),
            AuthMode::Unknown => ObservationValue::Keyword("unknown"),
        }
    }
}

pub struct AuthModeRule;

impl Rule for AuthModeRule {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: RuleId::AUTH_MODE,
            category: SignalCategory::Authentication,
            title: "Authentication mode",
            rationale: "Authentication drift changes who can access the workload and how identity is enforced.",
        }
    }

    fn evaluate(&self, document: &SnapshotDocument) -> Option<RuleObservation> {
        let containers = workload_containers(document)?;
        let mode = detect_auth_mode(containers);

        Some(RuleObservation {
            value: mode.into(),
            description: format!(
                "{} uses {} authentication",
                document.display_name(),
                match mode {
                    AuthMode::None => "no",
                    AuthMode::Basic => "basic",
                    AuthMode::Oidc => "oidc",
                    AuthMode::Unknown => "unknown",
                }
            ),
        })
    }
}

fn workload_containers<'a>(document: &'a SnapshotDocument) -> Option<&'a [Value]> {
    if document.is_kind("Pod") {
        return document.sequence_at_path(&["spec", "containers"]);
    }

    match document.kind() {
        Some("Deployment" | "StatefulSet" | "DaemonSet" | "ReplicaSet" | "Job") => {
            document.sequence_at_path(&["spec", "template", "spec", "containers"])
        }
        Some("CronJob") => {
            document.sequence_at_path(&["spec", "jobTemplate", "spec", "template", "spec", "containers"])
        }
        _ => None,
    }
}

fn detect_auth_mode(containers: &[Value]) -> AuthMode {
    let mut saw_auth_hint = false;

    for container in containers {
        let Some(env_items) = env_items(container) else {
            continue;
        };

        for env_item in env_items {
            let Some((name, value)) = env_name_and_value(env_item) else {
                continue;
            };

            let name_upper = name.to_ascii_uppercase();
            let value_lower = value.map(|v| v.to_ascii_lowercase());

            if name_upper.starts_with("OIDC_")
                || value_lower
                    .as_deref()
                    .is_some_and(|v| v.contains("oidc") || v.contains("openid"))
            {
                return AuthMode::Oidc;
            }

            if name_upper == "AUTH_MODE" || name_upper == "AUTH_PROVIDER" {
                saw_auth_hint = true;

                if value_lower
                    .as_deref()
                    .is_some_and(|v| v.contains("oidc") || v.contains("openid"))
                {
                    return AuthMode::Oidc;
                }

                if value_lower.as_deref().is_some_and(|v| v.contains("basic")) {
                    return AuthMode::Basic;
                }

                if value_lower.as_deref().is_some_and(|v| v.contains("none")) {
                    return AuthMode::None;
                }
            }
        }
    }

    if saw_auth_hint {
        AuthMode::Unknown
    } else {
        AuthMode::None
    }
}

fn env_items(container: &Value) -> Option<&[Value]> {
    match container {
        Value::Mapping(mapping) => match mapping.get(Value::String("env".into())) {
            Some(Value::Sequence(items)) => Some(items.as_slice()),
            _ => None,
        },
        _ => None,
    }
}

fn env_name_and_value(env_item: &Value) -> Option<(&str, Option<&str>)> {
    let Value::Mapping(mapping) = env_item else {
        return None;
    };

    let name = mapping.get(Value::String("name".into()))?.as_str()?;
    let value = mapping.get(Value::String("value".into())).and_then(Value::as_str);

    Some((name, value))
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
    fn defaults_to_none_without_auth_hints() {
        let document = first_document(
            "kind: Deployment\nmetadata:\n  name: web\nspec:\n  template:\n    spec:\n      containers:\n        - name: web\n          image: nginx\n",
        );

        let observation = AuthModeRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("none"));
    }

    #[test]
    fn detects_basic_auth_from_auth_mode_env() {
        let document = first_document(
            "kind: Deployment\nmetadata:\n  name: web\nspec:\n  template:\n    spec:\n      containers:\n        - name: web\n          image: nginx\n          env:\n            - name: AUTH_MODE\n              value: basic\n",
        );

        let observation = AuthModeRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("basic"));
    }

    #[test]
    fn detects_oidc_from_oidc_envs() {
        let document = first_document(
            "kind: Deployment\nmetadata:\n  name: web\nspec:\n  template:\n    spec:\n      containers:\n        - name: web\n          image: nginx\n          env:\n            - name: OIDC_ISSUER_URL\n              value: https://issuer.example.com\n            - name: OIDC_CLIENT_ID\n              value: web\n",
        );

        let observation = AuthModeRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("oidc"));
    }

    #[test]
    fn emits_unknown_for_unrecognized_auth_mode() {
        let document = first_document(
            "kind: Deployment\nmetadata:\n  name: web\nspec:\n  template:\n    spec:\n      containers:\n        - name: web\n          image: nginx\n          env:\n            - name: AUTH_MODE\n              value: kerberos\n",
        );

        let observation = AuthModeRule.evaluate(&document).expect("rule should match");

        assert_eq!(observation.value, ObservationValue::Keyword("unknown"));
    }
}
