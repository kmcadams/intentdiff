use crate::{AnalysisResult, PolicyFinding, Severity, policy::PolicyReport};

pub fn render_terminal(result: &AnalysisResult) -> String {
    if result.policy.findings.is_empty() {
        return "No intent drift detected".to_string();
    }

    let mut output = String::from("Intent drift detected\n");
    append_findings(&mut output, &result.policy, false);
    output
}

pub fn render_markdown(result: &AnalysisResult) -> String {
    if result.policy.findings.is_empty() {
        return "No intent drift detected".to_string();
    }

    let mut output = String::from("## Intent Drift Detected\n");
    append_findings(&mut output, &result.policy, true);
    output
}

fn append_findings(output: &mut String, report: &PolicyReport, markdown: bool) {
    for severity in [
        Severity::Critical,
        Severity::Warning,
        Severity::Informational,
    ] {
        let findings: Vec<&PolicyFinding> = report
            .findings
            .iter()
            .filter(|finding| finding.severity == severity)
            .collect();

        if findings.is_empty() {
            continue;
        }

        if markdown {
            output.push_str(&format!("\n### {severity}\n"));
        } else {
            output.push_str(&format!("\n{severity}:\n"));
        }

        for finding in findings {
            output.push_str(&format!("- {}\n", finding.message));
            if markdown {
                output.push_str(&format!("  Why: {}\n", finding.rationale));
            } else {
                output.push_str(&format!("  why: {}\n", finding.rationale));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DiffResult, PolicyFinding, RuleId, policy::PolicyReport};

    fn result_with(severity: Severity) -> AnalysisResult {
        AnalysisResult {
            diff: DiffResult {
                added: vec![],
                removed: vec![],
                changed: vec![],
            },
            policy: PolicyReport {
                findings: vec![PolicyFinding {
                    rule_id: RuleId::TRANSPORT_TLS_ENABLED,
                    severity,
                    message: "TLS differs".into(),
                    rationale: "TLS drift changes whether edge traffic is encrypted in transit.",
                }],
            },
        }
    }

    #[test]
    fn terminal_renderer_groups_findings_by_severity() {
        let rendered = render_terminal(&result_with(Severity::Critical));

        assert!(rendered.contains("Intent drift detected"));
        assert!(rendered.contains("CRITICAL:"));
        assert!(rendered.contains("TLS differs"));
        assert!(rendered.contains("why: TLS drift"));
    }

    #[test]
    fn markdown_renderer_uses_markdown_headings() {
        let rendered = render_markdown(&result_with(Severity::Warning));

        assert!(rendered.contains("## Intent Drift Detected"));
        assert!(rendered.contains("### WARNING"));
        assert!(rendered.contains("Why: TLS drift"));
    }
}
