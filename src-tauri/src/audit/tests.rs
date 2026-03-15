use std::path::PathBuf;

use super::{
    config_audit::{ConfigAuditTarget, audit_config},
    credential_audit::build_credential_artifacts,
    redaction::{fingerprint_secret, mask_secret},
};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures/configs")
        .canonicalize()
        .expect("fixtures root resolves")
}

#[test]
fn audits_codex_and_claude_configs_with_masked_credentials_and_risk_flags() {
    let codex = audit_config(&ConfigAuditTarget::new(
        "codex",
        "user",
        "global",
        fixtures_root().join("codex").join("config.toml"),
    ))
    .expect("codex config parses");

    assert_eq!(codex.config.assistant, "codex");
    assert_eq!(codex.config.provider.as_deref(), Some("cch"));
    assert_eq!(
        codex.config.base_url.as_deref(),
        Some("https://relay.cch.example/v1")
    );
    assert!(has_flag(&codex.risk_flags, "dangerous_approval_policy"));
    assert!(has_flag(&codex.risk_flags, "dangerous_sandbox"));
    assert!(has_flag(&codex.risk_flags, "third_party_provider"));
    assert!(has_flag(&codex.risk_flags, "third_party_base_url"));

    let codex_credentials = build_credential_artifacts(&codex.secrets);
    assert_eq!(codex_credentials.len(), 1);
    assert_eq!(codex_credentials[0].provider, "cch");
    assert_eq!(codex_credentials[0].official_or_proxy, "proxy");
    assert!(codex_credentials[0].masked_value.starts_with("***"));
    assert!(codex_credentials[0].fingerprint.starts_with("sha256:"));
    assert!(!codex_credentials[0].masked_value.contains("123456789"));

    let claude = audit_config(&ConfigAuditTarget::new(
        "claude-code",
        "user",
        "global",
        fixtures_root().join("claude").join("settings.json"),
    ))
    .expect("claude config parses");

    assert_eq!(claude.config.assistant, "claude-code");
    assert_eq!(claude.config.provider.as_deref(), Some("anthropic"));
    assert_eq!(
        claude.config.base_url.as_deref(),
        Some("https://relay.anthropic-proxy.example/v1")
    );
    assert!(has_flag(&claude.risk_flags, "dangerous_permissions"));
    assert!(has_flag(&claude.risk_flags, "shell_hook"));
    assert!(claude.config.permissions_json.contains("Bash(*)"));

    let claude_credentials = build_credential_artifacts(&claude.secrets);
    assert_eq!(claude_credentials.len(), 1);
    assert_eq!(claude_credentials[0].provider, "anthropic");
    assert_eq!(claude_credentials[0].official_or_proxy, "proxy");
    assert!(claude_credentials[0].masked_value.starts_with("***"));
}

#[test]
fn audits_opencode_provider_options_and_masks_api_keys() {
    let opencode = audit_config(&ConfigAuditTarget::new(
        "opencode",
        "user",
        "global",
        fixtures_root().join("opencode").join("opencode.json"),
    ))
    .expect("opencode config parses");

    assert_eq!(opencode.config.assistant, "opencode");
    assert_eq!(opencode.config.provider.as_deref(), Some("openrouter"));
    assert_eq!(
        opencode.config.model.as_deref(),
        Some("openrouter/anthropic/claude-sonnet-4")
    );
    assert!(has_flag(&opencode.risk_flags, "third_party_provider"));
    assert!(has_flag(&opencode.risk_flags, "dangerous_permissions"));
    assert!(opencode.config.mcp_json.contains("filesystem"));

    let credentials = build_credential_artifacts(&opencode.secrets);
    assert_eq!(credentials.len(), 1);
    assert_eq!(credentials[0].provider, "openrouter");
    assert_eq!(credentials[0].official_or_proxy, "proxy");
    assert!(credentials[0].masked_value.starts_with("***"));
    assert!(credentials[0].fingerprint.starts_with("sha256:"));
}

#[test]
fn redacts_secrets_without_exposing_plaintext() {
    let secret = "sk-test-1234567890";

    assert_eq!(mask_secret(secret), "***7890");
    assert_eq!(fingerprint_secret(secret).len(), 19);
}

fn has_flag<T>(flags: &[T], code: &str) -> bool
where
    T: std::borrow::Borrow<super::RiskFlag>,
{
    flags.iter().any(|flag| flag.borrow().code == code)
}
