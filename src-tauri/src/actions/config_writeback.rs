use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::audit::{
    AssistantConfigAudit,
    config_audit::{ConfigAuditTarget, audit_config},
};

use super::{
    ActionError, ActionResult, AuditWriteRequest, ensure_managed_path, safe_managed_name,
    write_audit_event,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigWritebackUpdate {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub secret: Option<String>,
}

pub struct ConfigWritebackRequest<'a> {
    pub target: &'a ConfigAuditTarget,
    pub update: &'a ConfigWritebackUpdate,
    pub backup_root: &'a Path,
    pub actor: &'a str,
    pub connection: &'a Connection,
}

pub struct ConfigRollbackRequest<'a> {
    pub manifest_path: &'a Path,
    pub backup_root: &'a Path,
    pub actor: &'a str,
    pub connection: &'a Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigBackupAsset {
    pub original_path: PathBuf,
    pub backup_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigBackupManifest {
    pub backup_id: String,
    pub artifact_id: String,
    pub assistant: String,
    pub scope: String,
    pub source_layer: String,
    pub target_path: PathBuf,
    pub manifest_path: PathBuf,
    pub created_at: String,
    pub assets: Vec<ConfigBackupAsset>,
}

pub struct ConfigWritebackResult {
    pub manifest_path: PathBuf,
}

pub fn write_config(request: &ConfigWritebackRequest<'_>) -> ActionResult<ConfigWritebackResult> {
    validate_update(request.target, request.update)?;
    let before_audit = audit_config(request.target)?;
    let manifest = create_backup_manifest(request.target, request.backup_root)?;

    if let Err(error) = apply_update(request.target, request.update) {
        restore_backup_assets(&manifest.assets)?;
        return Err(error);
    }

    let after_audit = match audit_config(request.target) {
        Ok(audit) => audit,
        Err(error) => {
            restore_backup_assets(&manifest.assets)?;
            return Err(error.into());
        }
    };

    write_audit_event(
        request.connection,
        AuditWriteRequest {
            event_type: "config_writeback",
            target_type: "config",
            target_id: &before_audit.config.artifact_id,
            actor: request.actor,
            before_state: Some(audit_state_json(&before_audit)),
            after_state: Some(audit_state_json_with_manifest(
                &after_audit,
                &manifest.manifest_path,
            )),
            result: "success",
        },
    )?;

    Ok(ConfigWritebackResult {
        manifest_path: manifest.manifest_path,
    })
}

pub fn rollback_config_writeback(request: &ConfigRollbackRequest<'_>) -> ActionResult<()> {
    let manifest_path = ensure_managed_path(
        request.manifest_path,
        request.backup_root,
        "config rollback manifest",
    )?;
    let manifest: ConfigBackupManifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    let target = ConfigAuditTarget::new(
        manifest.assistant.clone(),
        manifest.scope.clone(),
        manifest.source_layer.clone(),
        manifest.target_path.clone(),
    );
    let before_audit = audit_config(&target)?;

    restore_backup_assets(&manifest.assets)?;

    let after_audit = audit_config(&target)?;
    write_audit_event(
        request.connection,
        AuditWriteRequest {
            event_type: "config_rollback",
            target_type: "config",
            target_id: &manifest.artifact_id,
            actor: request.actor,
            before_state: Some(audit_state_json(&before_audit)),
            after_state: Some(audit_state_json_with_manifest(
                &after_audit,
                &manifest.manifest_path,
            )),
            result: "success",
        },
    )?;

    Ok(())
}

fn validate_update(target: &ConfigAuditTarget, update: &ConfigWritebackUpdate) -> ActionResult<()> {
    for value in [&update.provider, &update.model, &update.secret] {
        if let Some(value) = value
            && value.trim().is_empty()
        {
            return Err(ActionError::Precondition(
                "config writeback fields must not be empty".to_string(),
            ));
        }
    }

    if let Some(base_url) = &update.base_url
        && !(base_url.starts_with("https://") || base_url.starts_with("http://"))
    {
        return Err(ActionError::Precondition(format!(
            "config writeback requires an absolute http(s) base URL for {}",
            target.assistant
        )));
    }

    if matches!(
        target.assistant.as_str(),
        "github-copilot-cli" | "gemini-cli"
    ) && let Some(provider) = &update.provider
        && provider
            != if target.assistant == "github-copilot-cli" {
                "github"
            } else {
                "google"
            }
    {
        return Err(ActionError::Precondition(format!(
            "{} provider is fixed and cannot be changed safely yet",
            target.assistant
        )));
    }

    Ok(())
}

fn create_backup_manifest(
    target: &ConfigAuditTarget,
    backup_root: &Path,
) -> ActionResult<ConfigBackupManifest> {
    let created_at = Utc::now().to_rfc3339();
    let backup_id = format!(
        "{}-{}",
        safe_managed_name(&target.assistant),
        Utc::now().format("%Y%m%d%H%M%S")
    );
    let bundle_root = backup_root.join(&backup_id);
    let payload_root = bundle_root.join("payload");
    let manifest_path = bundle_root.join("manifest.json");
    let assets = backup_source_paths(target)?
        .into_iter()
        .enumerate()
        .map(|(index, source_path)| {
            let file_name = source_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("config");
            let backup_path = payload_root.join(format!("{index:02}-{file_name}"));

            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(&source_path, &backup_path)?;
            Ok(ConfigBackupAsset {
                original_path: source_path,
                backup_path,
            })
        })
        .collect::<ActionResult<Vec<_>>>()?;
    let before_audit = audit_config(target)?;
    let manifest = ConfigBackupManifest {
        backup_id,
        artifact_id: before_audit.config.artifact_id,
        assistant: target.assistant.clone(),
        scope: target.scope.clone(),
        source_layer: target.source_layer.clone(),
        target_path: target.path.clone(),
        manifest_path: manifest_path.clone(),
        created_at,
        assets,
    };

    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    Ok(manifest)
}

fn backup_source_paths(target: &ConfigAuditTarget) -> ActionResult<Vec<PathBuf>> {
    let mut paths = match target.assistant.as_str() {
        "github-copilot-cli" => vec![resolve_copilot_write_path(&target.path)],
        "factory-droid" => vec![resolve_factory_write_path(&target.path)],
        _ => vec![target.path.clone()],
    };

    if matches!(target.assistant.as_str(), "gemini-cli" | "qwen-cli") {
        let env_path = target.path.parent().unwrap_or(&target.path).join(".env");
        if env_path.exists() {
            paths.push(env_path);
        }
    }

    paths.sort();
    paths.dedup();

    if paths.iter().any(|path| !path.exists()) {
        return Err(ActionError::Precondition(format!(
            "config writeback backup source is missing for {}",
            target.assistant
        )));
    }

    Ok(paths)
}

fn apply_update(target: &ConfigAuditTarget, update: &ConfigWritebackUpdate) -> ActionResult<()> {
    match target.assistant.as_str() {
        "github-copilot-cli" => apply_copilot_update(target, update),
        "factory-droid" => apply_factory_update(target, update),
        "gemini-cli" => apply_gemini_update(target, update),
        "qwen-cli" => apply_qwen_update(target, update),
        "openclaw" => apply_openclaw_update(target, update),
        assistant => Err(ActionError::Precondition(format!(
            "config writeback is not supported yet for {assistant}"
        ))),
    }
}

fn apply_copilot_update(
    target: &ConfigAuditTarget,
    update: &ConfigWritebackUpdate,
) -> ActionResult<()> {
    let write_path = resolve_copilot_write_path(&target.path);
    let mut parsed = load_json_file(&write_path)?;

    if let Some(provider) = &update.provider {
        parsed["provider"] = Value::String(provider.clone());
    }
    if let Some(model) = &update.model {
        parsed["model"] = Value::String(model.clone());
    }
    if let Some(base_url) = &update.base_url {
        if parsed
            .get("githubEnterprise")
            .and_then(Value::as_object)
            .is_some()
        {
            parsed["githubEnterprise"]["uri"] = Value::String(base_url.clone());
        } else {
            parsed["baseUrl"] = Value::String(base_url.clone());
        }
    }
    if let Some(secret) = &update.secret {
        parsed["authToken"] = Value::String(secret.clone());
    }

    write_json_file(&write_path, &parsed)
}

fn apply_factory_update(
    target: &ConfigAuditTarget,
    update: &ConfigWritebackUpdate,
) -> ActionResult<()> {
    let write_path = resolve_factory_write_path(&target.path);
    let mut parsed = load_json_file(&write_path)?;

    if let Some(provider) = &update.provider {
        parsed["provider"] = Value::String(provider.clone());
    }
    if let Some(model) = &update.model {
        parsed["defaultModel"] = Value::String(model.clone());
    }
    if let Some(base_url) = &update.base_url {
        parsed["baseUrl"] = Value::String(base_url.clone());
    }
    if let Some(secret) = &update.secret {
        parsed["apiKey"] = Value::String(secret.clone());
    }

    write_json_file(&write_path, &parsed)
}

fn apply_gemini_update(
    target: &ConfigAuditTarget,
    update: &ConfigWritebackUpdate,
) -> ActionResult<()> {
    let settings = load_json_file(&target.path)?;
    let env_path = target.path.parent().unwrap_or(&target.path).join(".env");
    let mut env_map = if env_path.exists() {
        parse_dotenv(&fs::read_to_string(&env_path)?)
    } else {
        BTreeMap::new()
    };

    if let Some(model) = &update.model {
        env_map.insert("GEMINI_MODEL".to_string(), model.clone());
    }
    if let Some(base_url) = &update.base_url {
        env_map.insert("GOOGLE_GEMINI_BASE_URL".to_string(), base_url.clone());
    }
    if let Some(secret) = &update.secret {
        env_map.insert("GEMINI_API_KEY".to_string(), secret.clone());
    }

    if settings.get("security").is_none() {
        return Err(ActionError::Precondition(
            "gemini settings.json is missing the security block".to_string(),
        ));
    }

    fs::write(&env_path, serialize_dotenv(&env_map))?;
    Ok(())
}

fn apply_openclaw_update(
    target: &ConfigAuditTarget,
    update: &ConfigWritebackUpdate,
) -> ActionResult<()> {
    let content = fs::read_to_string(&target.path)?;
    let mut parsed: Value =
        json5::from_str(&content).map_err(|error| ActionError::Precondition(error.to_string()))?;
    let current_provider = parsed
        .get("models")
        .and_then(|value| value.get("providers"))
        .and_then(Value::as_object)
        .and_then(|providers| providers.keys().next().cloned())
        .ok_or_else(|| {
            ActionError::Precondition("openclaw config does not expose a provider".to_string())
        })?;

    if let Some(provider) = &update.provider
        && provider != &current_provider
    {
        return Err(ActionError::Precondition(
            "openclaw provider renames are not supported safely yet".to_string(),
        ));
    }

    let providers = parsed
        .get_mut("models")
        .and_then(|value| value.get_mut("providers"))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| {
            ActionError::Precondition("openclaw config providers block is missing".to_string())
        })?;
    let provider_config = providers.get_mut(&current_provider).ok_or_else(|| {
        ActionError::Precondition("openclaw provider config is missing".to_string())
    })?;
    let provider_object = provider_config.as_object_mut().ok_or_else(|| {
        ActionError::Precondition("openclaw provider config must be an object".to_string())
    })?;

    if let Some(base_url) = &update.base_url {
        provider_object.insert("baseUrl".to_string(), Value::String(base_url.clone()));
    }
    if let Some(secret) = &update.secret {
        provider_object.insert("apiKey".to_string(), Value::String(secret.clone()));
    }
    if let Some(model) = &update.model {
        parsed["agents"]["defaults"]["model"]["primary"] = Value::String(model.clone());
    }

    write_json_file(&target.path, &parsed)
}

fn apply_qwen_update(
    target: &ConfigAuditTarget,
    update: &ConfigWritebackUpdate,
) -> ActionResult<()> {
    let mut parsed = load_json_file(&target.path)?;
    let current_audit = audit_config(target)?;
    let current_provider = current_audit
        .config
        .provider
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    if let Some(provider) = &update.provider
        && provider != &current_provider
    {
        return Err(ActionError::Precondition(
            "qwen-cli provider renames are not supported safely yet".to_string(),
        ));
    }

    let selected_type = parsed
        .get("security")
        .and_then(|value| value.get("auth"))
        .and_then(|value| value.get("selectedType"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let desired_model = update
        .model
        .as_deref()
        .or(current_audit.config.model.as_deref());

    if let Some(model) = &update.model {
        parsed["model"]["name"] = Value::String(model.clone());
    }

    let provider_env_key = qwen_provider_env_key(&parsed, selected_type.as_deref(), desired_model);

    if let Some(base_url) = &update.base_url {
        if !update_qwen_provider_field(
            &mut parsed,
            selected_type.as_deref(),
            desired_model,
            "baseUrl",
            Value::String(base_url.clone()),
        ) {
            parsed["security"]["auth"]["baseUrl"] = Value::String(base_url.clone());
        }
    }

    if let Some(secret) = &update.secret {
        if let Some(env_key) = provider_env_key {
            let env_path = target.path.parent().unwrap_or(&target.path).join(".env");
            if env_path.exists() {
                let mut env_map = parse_dotenv(&fs::read_to_string(&env_path)?);
                env_map.insert(env_key, secret.clone());
                fs::write(&env_path, serialize_dotenv(&env_map))?;
            } else {
                parsed["settings"]["env"][env_key] = Value::String(secret.clone());
            }
        } else {
            parsed["security"]["auth"]["apiKey"] = Value::String(secret.clone());
        }
    }

    write_json_file(&target.path, &parsed)
}

fn load_json_file(path: &Path) -> ActionResult<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn write_json_file(path: &Path, value: &Value) -> ActionResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

fn restore_backup_assets(assets: &[ConfigBackupAsset]) -> ActionResult<()> {
    for asset in assets {
        if let Some(parent) = asset.original_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&asset.backup_path, &asset.original_path)?;
    }

    Ok(())
}

fn parse_dotenv(content: &str) -> BTreeMap<String, String> {
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

fn serialize_dotenv(values: &BTreeMap<String, String>) -> String {
    let mut lines = values
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>();
    lines.push(String::new());
    lines.join("\n")
}

fn resolve_copilot_write_path(path: &Path) -> PathBuf {
    match path.file_name().and_then(|value| value.to_str()) {
        Some("settings.json") => {
            let local_path = path.with_file_name("settings.local.json");
            if local_path.exists() {
                local_path
            } else {
                path.to_path_buf()
            }
        }
        _ => path.to_path_buf(),
    }
}

fn resolve_factory_write_path(path: &Path) -> PathBuf {
    match path.file_name().and_then(|value| value.to_str()) {
        Some("settings.json") => {
            let local_path = path.with_file_name("settings.local.json");
            if local_path.exists() {
                local_path
            } else {
                path.to_path_buf()
            }
        }
        _ => path.to_path_buf(),
    }
}

fn qwen_provider_env_key(
    parsed: &Value,
    selected_type: Option<&str>,
    model: Option<&str>,
) -> Option<String> {
    qwen_provider_entry(parsed, selected_type, model)
        .and_then(|entry| entry.get("envKey"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn qwen_provider_entry<'a>(
    parsed: &'a Value,
    selected_type: Option<&str>,
    model: Option<&str>,
) -> Option<&'a Value> {
    let selected_type = selected_type?;
    let providers = parsed
        .get("modelProviders")
        .and_then(|value| value.get(selected_type))
        .and_then(Value::as_array)?;

    if let Some(model) = model
        && let Some(entry) = providers
            .iter()
            .find(|entry| entry.get("id").and_then(Value::as_str) == Some(model))
    {
        return Some(entry);
    }

    providers.first()
}

fn update_qwen_provider_field(
    parsed: &mut Value,
    selected_type: Option<&str>,
    model: Option<&str>,
    field: &str,
    next_value: Value,
) -> bool {
    let Some(selected_type) = selected_type else {
        return false;
    };
    let Some(providers) = parsed
        .get_mut("modelProviders")
        .and_then(|value| value.get_mut(selected_type))
        .and_then(Value::as_array_mut)
    else {
        return false;
    };

    let index = if let Some(model) = model {
        providers
            .iter()
            .position(|entry| entry.get("id").and_then(Value::as_str) == Some(model))
            .unwrap_or(0)
    } else {
        0
    };

    let Some(entry) = providers.get_mut(index).and_then(Value::as_object_mut) else {
        return false;
    };
    entry.insert(field.to_string(), next_value);
    true
}

fn audit_state_json(audit: &AssistantConfigAudit) -> String {
    audit_state_value(audit).to_string()
}

fn audit_state_json_with_manifest(audit: &AssistantConfigAudit, manifest_path: &Path) -> String {
    let mut state = audit_state_value(audit);
    if let Some(object) = state.as_object_mut() {
        object.insert(
            "manifest_path".to_string(),
            Value::String(manifest_path.display().to_string()),
        );
    }

    state.to_string()
}

fn audit_state_value(audit: &AssistantConfigAudit) -> Value {
    json!({
        "artifactId": audit.config.artifact_id,
        "assistant": audit.config.assistant,
        "scope": audit.config.scope,
        "path": audit.config.path,
        "provider": audit.config.provider,
        "model": audit.config.model,
        "baseUrl": audit.config.base_url,
        "riskFlags": audit.risk_flags.iter().map(|flag| flag.code.clone()).collect::<Vec<_>>(),
        "secretCount": audit.secrets.len(),
    })
}
