pub mod config_audit;
pub mod credential_audit;
pub mod redaction;

use crate::domain::config::ConfigArtifact;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiskFlag {
    pub code: String,
    pub severity: String,
    pub message: String,
}

impl RiskFlag {
    pub fn new(
        code: impl Into<String>,
        severity: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: severity.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretMaterial {
    pub provider: String,
    pub kind: String,
    pub location: String,
    pub source_type: String,
    pub value: String,
    pub official_or_proxy: String,
    pub last_modified_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantConfigAudit {
    pub config: ConfigArtifact,
    pub secrets: Vec<SecretMaterial>,
    pub risk_flags: Vec<RiskFlag>,
}

#[cfg(test)]
mod tests;
