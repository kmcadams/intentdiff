#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuleId(pub &'static str);

impl RuleId {
    pub const PERSISTENCE_EMPTYDIR: RuleId = RuleId("persistence.emptydir");

    pub const TRANSPORT_TLS_ENABLED: RuleId = RuleId("transport.tls_enabled");
}
