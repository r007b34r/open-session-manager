#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigArtifact {
    pub artifact_id: String,
    pub assistant: String,
    pub scope: String,
    pub path: String,
    pub source_layer: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub permissions_json: String,
    pub mcp_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialArtifact {
    pub artifact_id: String,
    pub provider: String,
    pub kind: String,
    pub location: String,
    pub source_type: String,
    pub masked_value: String,
    pub fingerprint: String,
    pub official_or_proxy: String,
    pub last_modified_at: Option<String>,
}
