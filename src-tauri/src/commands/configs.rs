use crate::{
    audit::config_audit::ConfigAuditTarget, commands::discovery::discover_known_roots,
    discovery::DiscoveryContext,
};

pub fn discover_known_config_targets(context: &DiscoveryContext) -> Vec<ConfigAuditTarget> {
    discover_known_roots(context)
        .into_iter()
        .filter(|path| path.kind == "config")
        .map(|path| ConfigAuditTarget::new(path.assistant, "user", path.environment, path.path))
        .collect()
}
