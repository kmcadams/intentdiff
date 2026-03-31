pub mod diff;
pub mod engine;
pub mod policy;
pub mod report;
pub mod semantic;
pub mod snapshot;

pub use diff::{DiffResult, diff_observations};
pub use engine::{AnalysisResult, Engine};
pub use policy::{DefaultPolicyEvaluator, PolicyEvaluator, PolicyFinding, PolicyReport};
pub use report::{render_markdown, render_terminal};
pub use semantic::{
    BasicAnalyzer, SemanticAnalyzer,
    observation::{IntentObservation, ObservationValue},
    profile::Profile,
    rule_id::RuleId,
    signal::{SignalCategory, SignalStrength},
};
pub use snapshot::{ResourceMatchKey, ResourceRef, Snapshot, SnapshotDocument, SnapshotParseError};
