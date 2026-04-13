#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuleId(pub &'static str);

impl RuleId {
    pub const AVAILABILITY_REPLICA_POSTURE: RuleId = RuleId("availability.replica_posture");
    pub const NETWORK_EXPOSURE_PUBLIC: RuleId = RuleId("network.exposure.public");
    pub const PERSISTENCE_EMPTYDIR: RuleId = RuleId("persistence.emptydir");
    pub const PERSISTENCE_STORAGE_MODE: RuleId = RuleId("persistence.storage_mode");
    pub const SECURITY_RUN_AS_NON_ROOT: RuleId = RuleId("security.run_as_non_root");

    pub const TRANSPORT_TLS_ENABLED: RuleId = RuleId("transport.tls_enabled");
}
