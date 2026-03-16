use std::{
    collections::HashMap,
    fmt, fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use toml::Value as TomlValue;

use crate::domain::config::ConfigArtifact;

use super::{AssistantConfigAudit, RiskFlag, SecretMaterial};

pub type AuditResult<T> = Result<T, AuditError>;

#[derive(Debug)]
pub enum AuditError {
    Io(io::Error),
    Json(serde_json::Error),
    Json5(String),
    Toml(toml::de::Error),
    UnsupportedAssistant(String),
}

impl fmt::Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
            Self::Json5(error) => write!(f, "json5 error: {error}"),
            Self::Toml(error) => write!(f, "toml error: {error}"),
            Self::UnsupportedAssistant(assistant) => {
                write!(f, "unsupported assistant: {assistant}")
            }
        }
    }
}

impl std::error::Error for AuditError {}

impl From<io::Error> for AuditError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for AuditError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<toml::de::Error> for AuditError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigAuditTarget {
    pub assistant: String,
    pub scope: String,
    pub source_layer: String,
    pub path: PathBuf,
}

impl ConfigAuditTarget {
    pub fn new(
        assistant: impl Into<String>,
        scope: impl Into<String>,
        source_layer: impl Into<String>,
        path: PathBuf,
    ) -> Self {
        Self {
            assistant: assistant.into(),
            scope: scope.into(),
            source_layer: source_layer.into(),
            path,
        }
    }
}

pub fn audit_config(target: &ConfigAuditTarget) -> AuditResult<AssistantConfigAudit> {
    match target.assistant.as_str() {
        "codex" => audit_codex(target),
        "claude-code" => audit_claude_code(target),
        "opencode" => audit_opencode(target),
        "gemini-cli" => audit_gemini_cli(target),
        "openclaw" => audit_openclaw(target),
        assistant => Err(AuditError::UnsupportedAssistant(assistant.to_string())),
    }
}

fn audit_codex(target: &ConfigAuditTarget) -> AuditResult<AssistantConfigAudit> {
    let text = fs::read_to_string(&target.path)?;
    let parsed: TomlValue = toml::from_str(&text)?;
    let provider = parsed
        .get("model_provider")
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned);
    let model = parsed
        .get("model")
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned);
    let approval_policy = parsed
        .get("approval_policy")
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned);
    let sandbox_mode = parsed
        .get("sandbox_mode")
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned);

    let provider_table = provider.as_deref().and_then(|provider_id| {
        parsed
            .get("model_providers")
            .and_then(|value| value.get(provider_id))
    });
    let base_url = provider_table
        .and_then(|value| value.get("base_url"))
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned);
    let api_key = provider_table
        .and_then(|value| {
            value
                .get("api_key")
                .or_else(|| value.get("auth_token"))
                .or_else(|| value.get("api_token"))
        })
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned);

    let permissions_json = json!({
        "approval_policy": approval_policy,
        "sandbox_mode": sandbox_mode,
    })
    .to_string();

    let mcp_json = json!({}).to_string();
    let official_or_proxy = classify_endpoint(provider.as_deref(), base_url.as_deref());
    let risk_flags = codex_risk_flags(
        provider.as_deref(),
        base_url.as_deref(),
        approval_policy.as_deref(),
        sandbox_mode.as_deref(),
    );

    Ok(AssistantConfigAudit {
        config: build_config_artifact(
            target,
            provider.clone(),
            model,
            base_url.clone(),
            permissions_json,
            mcp_json,
        ),
        secrets: api_key
            .into_iter()
            .map(|value| SecretMaterial {
                provider: provider.clone().unwrap_or_else(|| "openai".to_string()),
                kind: "api_key".to_string(),
                location: provider_table_location(provider.as_deref(), "api_key"),
                source_type: "toml".to_string(),
                value,
                official_or_proxy: official_or_proxy.clone(),
                last_modified_at: file_modified_at(&target.path),
            })
            .collect(),
        risk_flags,
    })
}

fn audit_claude_code(target: &ConfigAuditTarget) -> AuditResult<AssistantConfigAudit> {
    let text = fs::read_to_string(&target.path)?;
    let parsed: Value = serde_json::from_str(&text)?;
    let env = parsed
        .get("env")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let base_url = env
        .get("ANTHROPIC_BASE_URL")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let auth_token = env
        .get("ANTHROPIC_AUTH_TOKEN")
        .or_else(|| env.get("ANTHROPIC_API_KEY"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let model = parsed
        .get("model")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let permissions = parsed
        .get("permissions")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let hooks = parsed
        .get("hooks")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));

    let risk_flags = claude_risk_flags(base_url.as_deref(), &permissions, &hooks);

    Ok(AssistantConfigAudit {
        config: build_config_artifact(
            target,
            Some("anthropic".to_string()),
            model,
            base_url.clone(),
            permissions.to_string(),
            json!({ "hooks": hooks }).to_string(),
        ),
        secrets: auth_token
            .into_iter()
            .map(|value| SecretMaterial {
                provider: "anthropic".to_string(),
                kind: "auth_token".to_string(),
                location: "env.ANTHROPIC_AUTH_TOKEN".to_string(),
                source_type: "json".to_string(),
                value,
                official_or_proxy: classify_endpoint(Some("anthropic"), base_url.as_deref()),
                last_modified_at: file_modified_at(&target.path),
            })
            .collect(),
        risk_flags,
    })
}

fn audit_opencode(target: &ConfigAuditTarget) -> AuditResult<AssistantConfigAudit> {
    let text = fs::read_to_string(&target.path)?;
    let parsed: Value = serde_json::from_str(&text)?;
    let model = parsed
        .get("model")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let provider_id = parsed
        .get("provider")
        .and_then(Value::as_object)
        .and_then(|providers| providers.keys().next().cloned())
        .or_else(|| model.as_deref().and_then(model_provider_from_selection));
    let provider_config = provider_id.as_deref().and_then(|provider| {
        parsed
            .get("provider")
            .and_then(Value::as_object)
            .and_then(|providers| providers.get(provider))
    });
    let options = provider_config
        .and_then(|provider| provider.get("options"))
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let base_url = options
        .get("baseURL")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let api_key = options
        .get("apiKey")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let permissions = parsed
        .get("permission")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let mcp = parsed
        .get("mcp")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));

    let provider_ref = provider_id.as_deref();
    let official_or_proxy = classify_endpoint(provider_ref, base_url.as_deref());
    let risk_flags = opencode_risk_flags(provider_ref, base_url.as_deref(), &permissions);

    Ok(AssistantConfigAudit {
        config: build_config_artifact(
            target,
            provider_id.clone(),
            model,
            base_url.clone(),
            permissions.to_string(),
            mcp.to_string(),
        ),
        secrets: api_key
            .into_iter()
            .map(|value| SecretMaterial {
                provider: provider_id
                    .clone()
                    .unwrap_or_else(|| "opencode".to_string()),
                kind: "api_key".to_string(),
                location: provider_option_location(provider_ref, "apiKey"),
                source_type: "json".to_string(),
                value,
                official_or_proxy: official_or_proxy.clone(),
                last_modified_at: file_modified_at(&target.path),
            })
            .collect(),
        risk_flags,
    })
}

fn audit_gemini_cli(target: &ConfigAuditTarget) -> AuditResult<AssistantConfigAudit> {
    let settings_text = fs::read_to_string(&target.path)?;
    let parsed: Value = serde_json::from_str(&settings_text)?;
    let env_path = target.path.parent().unwrap_or(&target.path).join(".env");
    let env = if env_path.exists() {
        parse_dotenv(&fs::read_to_string(&env_path)?)
    } else {
        HashMap::new()
    };

    let provider = Some("google".to_string());
    let model = env
        .get("GEMINI_MODEL")
        .cloned()
        .or_else(|| env.get("GOOGLE_GEMINI_MODEL").cloned());
    let base_url = env
        .get("GOOGLE_GEMINI_BASE_URL")
        .cloned()
        .or_else(|| env.get("GEMINI_BASE_URL").cloned());
    let secret = env
        .get("GEMINI_API_KEY")
        .cloned()
        .or_else(|| env.get("GOOGLE_API_KEY").cloned())
        .or_else(|| env.get("GOOGLE_API_TOKEN").cloned());
    let selected_type = parsed
        .get("security")
        .and_then(|value| value.get("auth"))
        .and_then(|value| value.get("selectedType"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let session_retention = parsed
        .get("general")
        .and_then(|value| value.get("sessionRetention"))
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let permissions_json = json!({
        "selectedType": selected_type,
        "sessionRetention": session_retention,
    })
    .to_string();
    let mcp = parsed
        .get("mcpServers")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let official_or_proxy = classify_endpoint(provider.as_deref(), base_url.as_deref());
    let risk_flags = gemini_risk_flags(
        base_url.as_deref(),
        selected_type.as_deref(),
        secret.is_some(),
    );

    Ok(AssistantConfigAudit {
        config: build_config_artifact(
            target,
            provider.clone(),
            model,
            base_url.clone(),
            permissions_json,
            mcp.to_string(),
        ),
        secrets: secret
            .into_iter()
            .map(|value| SecretMaterial {
                provider: "google".to_string(),
                kind: "api_key".to_string(),
                location: ".env.GEMINI_API_KEY".to_string(),
                source_type: "dotenv".to_string(),
                value,
                official_or_proxy: official_or_proxy.clone(),
                last_modified_at: file_modified_at(&env_path).or_else(|| file_modified_at(&target.path)),
            })
            .collect(),
        risk_flags,
    })
}

fn audit_openclaw(target: &ConfigAuditTarget) -> AuditResult<AssistantConfigAudit> {
    let text = fs::read_to_string(&target.path)?;
    let parsed: Value =
        json5::from_str(&text).map_err(|error| AuditError::Json5(error.to_string()))?;
    let provider_id = parsed
        .get("models")
        .and_then(|value| value.get("providers"))
        .and_then(Value::as_object)
        .and_then(|providers| providers.keys().next().cloned());
    let provider_config = provider_id.as_deref().and_then(|provider| {
        parsed
            .get("models")
            .and_then(|value| value.get("providers"))
            .and_then(Value::as_object)
            .and_then(|providers| providers.get(provider))
    });
    let base_url = provider_config
        .and_then(|value| value.get("baseUrl").or_else(|| value.get("base_url")))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let api_key = provider_config
        .and_then(|value| value.get("apiKey").or_else(|| value.get("api_key")))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let model = parsed
        .get("agents")
        .and_then(|value| value.get("defaults"))
        .and_then(|value| value.get("model"))
        .and_then(|value| value.get("primary"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            provider_config
                .and_then(|value| value.get("models"))
                .and_then(Value::as_array)
                .and_then(|models| models.first())
                .and_then(|model| model.get("id"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        });
    let tools = parsed
        .get("tools")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let env = parsed
        .get("env")
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));
    let permissions_json = json!({
        "tools": tools,
        "env": env,
    })
    .to_string();
    let official_or_proxy = classify_endpoint(provider_id.as_deref(), base_url.as_deref());
    let risk_flags = openclaw_risk_flags(provider_id.as_deref(), base_url.as_deref(), &tools);

    Ok(AssistantConfigAudit {
        config: build_config_artifact(
            target,
            provider_id.clone(),
            model,
            base_url.clone(),
            permissions_json,
            json!({}).to_string(),
        ),
        secrets: api_key
            .into_iter()
            .map(|value| SecretMaterial {
                provider: provider_id
                    .clone()
                    .unwrap_or_else(|| "openclaw".to_string()),
                kind: "api_key".to_string(),
                location: "models.providers.<id>.apiKey".to_string(),
                source_type: "json5".to_string(),
                value,
                official_or_proxy: official_or_proxy.clone(),
                last_modified_at: file_modified_at(&target.path),
            })
            .collect(),
        risk_flags,
    })
}

fn codex_risk_flags(
    provider: Option<&str>,
    base_url: Option<&str>,
    approval_policy: Option<&str>,
    sandbox_mode: Option<&str>,
) -> Vec<RiskFlag> {
    let mut flags = endpoint_risk_flags(provider, base_url);

    if approval_policy == Some("never") {
        flags.push(RiskFlag::new(
            "dangerous_approval_policy",
            "high",
            "Codex approval policy is set to never.",
        ));
    }

    if sandbox_mode == Some("danger-full-access") {
        flags.push(RiskFlag::new(
            "dangerous_sandbox",
            "high",
            "Codex sandbox mode allows unrestricted filesystem access.",
        ));
    }

    flags
}

fn claude_risk_flags(base_url: Option<&str>, permissions: &Value, hooks: &Value) -> Vec<RiskFlag> {
    let mut flags = endpoint_risk_flags(Some("anthropic"), base_url);

    let has_broad_permissions = permissions
        .get("allow")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(Value::as_str).any(|value| {
                value.starts_with("Bash(")
                    || value.starts_with("Write(")
                    || value.starts_with("Edit(")
            })
        })
        .unwrap_or(false)
        || matches!(
            permissions.get("defaultMode").and_then(Value::as_str),
            Some("acceptEdits" | "acceptAll")
        );

    if has_broad_permissions {
        flags.push(RiskFlag::new(
            "dangerous_permissions",
            "high",
            "Claude Code allows wide write or shell permissions.",
        ));
    }

    let has_shell_hook = hooks
        .as_object()
        .map(|map| !map.is_empty())
        .unwrap_or(false);
    if has_shell_hook {
        flags.push(RiskFlag::new(
            "shell_hook",
            "medium",
            "Claude Code hooks execute local commands on session events.",
        ));
    }

    flags
}

fn opencode_risk_flags(
    provider: Option<&str>,
    base_url: Option<&str>,
    permissions: &Value,
) -> Vec<RiskFlag> {
    let mut flags = endpoint_risk_flags(provider, base_url);

    let has_dangerous_permissions = permissions
        .as_object()
        .map(|entries| {
            entries.iter().any(|(key, value)| {
                matches!(
                    key.as_str(),
                    "bash" | "edit" | "patch" | "multiedit" | "write"
                ) && value.as_str() == Some("allow")
            })
        })
        .unwrap_or(false);

    if has_dangerous_permissions {
        flags.push(RiskFlag::new(
            "dangerous_permissions",
            "high",
            "OpenCode permission policy allows write or shell tools without extra gating.",
        ));
    }

    flags
}

fn gemini_risk_flags(
    base_url: Option<&str>,
    selected_type: Option<&str>,
    has_secret: bool,
) -> Vec<RiskFlag> {
    let mut flags = endpoint_risk_flags(Some("google"), base_url);

    if selected_type != Some("oauth-personal") && !has_secret {
        flags.push(RiskFlag::new(
            "missing_primary_secret",
            "high",
            "Gemini CLI is not in OAuth mode and no API key was detected.",
        ));
    }

    flags
}

fn openclaw_risk_flags(
    provider: Option<&str>,
    base_url: Option<&str>,
    tools: &Value,
) -> Vec<RiskFlag> {
    let mut flags = endpoint_risk_flags(provider, base_url);

    let has_dangerous_permissions = tools
        .get("profile")
        .and_then(Value::as_str)
        .is_some_and(|profile| profile == "full")
        || tools
            .get("allow")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|value| matches!(value, "bash" | "write" | "edit" | "patch"))
            })
            .unwrap_or(false);

    if has_dangerous_permissions {
        flags.push(RiskFlag::new(
            "dangerous_permissions",
            "high",
            "OpenClaw tools policy allows broad shell or write access.",
        ));
    }

    flags
}

fn endpoint_risk_flags(provider: Option<&str>, base_url: Option<&str>) -> Vec<RiskFlag> {
    let mut flags = Vec::new();

    if let Some(provider) = provider
        && !is_official_provider(provider)
    {
        flags.push(RiskFlag::new(
            "third_party_provider",
            "medium",
            format!("Provider `{provider}` is a third-party relay or gateway."),
        ));
    }

    if let Some(base_url) = base_url
        && !is_official_base_url(provider, base_url)
    {
        flags.push(RiskFlag::new(
            "third_party_base_url",
            "high",
            format!("Base URL `{base_url}` is not an official endpoint."),
        ));
    }

    flags
}

fn build_config_artifact(
    target: &ConfigAuditTarget,
    provider: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
    permissions_json: String,
    mcp_json: String,
) -> ConfigArtifact {
    ConfigArtifact {
        artifact_id: artifact_id(target),
        assistant: target.assistant.clone(),
        scope: target.scope.clone(),
        path: target.path.display().to_string(),
        source_layer: target.source_layer.clone(),
        provider,
        model,
        base_url,
        permissions_json,
        mcp_json,
    }
}

fn artifact_id(target: &ConfigAuditTarget) -> String {
    let payload = format!(
        "{}:{}:{}:{}",
        target.assistant,
        target.scope,
        target.source_layer,
        target.path.display()
    );
    let digest = Sha256::digest(payload.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn file_modified_at(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    Some(system_time_to_rfc3339(modified))
}

fn system_time_to_rfc3339(value: SystemTime) -> String {
    let datetime: DateTime<Utc> = value.into();
    datetime.to_rfc3339()
}

fn classify_endpoint(provider: Option<&str>, base_url: Option<&str>) -> String {
    if is_official_provider(provider.unwrap_or_default())
        && base_url.is_none_or(|base_url| is_official_base_url(provider, base_url))
    {
        "official".to_string()
    } else {
        "proxy".to_string()
    }
}

fn is_official_provider(provider: &str) -> bool {
    matches!(provider, "openai" | "anthropic" | "opencode" | "google")
}

fn is_official_base_url(provider: Option<&str>, base_url: &str) -> bool {
    let Some(host) = extract_host(base_url) else {
        return false;
    };

    match provider {
        Some("openai") => host.ends_with("openai.com"),
        Some("anthropic") => host.ends_with("anthropic.com"),
        Some("google") => host.ends_with("googleapis.com") || host.ends_with("google.com"),
        Some("opencode") => host.ends_with("opencode.ai"),
        Some("openrouter") => host.ends_with("openrouter.ai"),
        _ => false,
    }
}

fn extract_host(base_url: &str) -> Option<&str> {
    let without_scheme = base_url.split("://").nth(1).unwrap_or(base_url);
    let host_port = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(without_scheme);
    let host = host_port.split(':').next().unwrap_or(host_port).trim();

    if host.is_empty() { None } else { Some(host) }
}

fn model_provider_from_selection(value: &str) -> Option<String> {
    value.split('/').next().map(ToOwned::to_owned)
}

fn provider_table_location(provider: Option<&str>, field: &str) -> String {
    let provider = provider.unwrap_or("default");
    format!("model_providers.{provider}.{field}")
}

fn provider_option_location(provider: Option<&str>, field: &str) -> String {
    let provider = provider.unwrap_or("default");
    format!("provider.{provider}.options.{field}")
}

fn parse_dotenv(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }

            let (key, value) = trimmed.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}
