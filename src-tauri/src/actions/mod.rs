pub mod config_writeback;
pub mod delete;
pub mod export;
pub mod restore;
pub mod session_control;

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{audit::config_audit::AuditError, domain::audit::AuditEvent};

pub type ActionResult<T> = Result<T, ActionError>;

#[derive(Debug)]
pub enum ActionError {
    Io(io::Error),
    Sql(rusqlite::Error),
    Json(serde_json::Error),
    Audit(AuditError),
    Execution(String),
    Precondition(String),
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Sql(error) => write!(f, "sqlite error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
            Self::Audit(error) => write!(f, "audit error: {error}"),
            Self::Execution(message) => write!(f, "execution error: {message}"),
            Self::Precondition(message) => write!(f, "precondition failed: {message}"),
        }
    }
}

impl std::error::Error for ActionError {}

impl From<io::Error> for ActionError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<rusqlite::Error> for ActionError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sql(value)
    }
}

impl From<serde_json::Error> for ActionError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<AuditError> for ActionError {
    fn from(value: AuditError) -> Self {
        Self::Audit(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuarantineAsset {
    pub original_path: PathBuf,
    pub quarantined_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuarantineManifest {
    pub session_id: String,
    pub original_path: PathBuf,
    pub quarantined_path: PathBuf,
    pub manifest_path: PathBuf,
    pub deleted_at: String,
    #[serde(default)]
    pub related_assets: Vec<QuarantineAsset>,
}

pub struct AuditWriteRequest<'a> {
    pub event_type: &'a str,
    pub target_type: &'a str,
    pub target_id: &'a str,
    pub actor: &'a str,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub result: &'a str,
}

pub fn write_audit_event(
    connection: &Connection,
    request: AuditWriteRequest<'_>,
) -> ActionResult<AuditEvent> {
    let created_at = Utc::now().to_rfc3339();
    let event_id = audit_event_id(
        request.event_type,
        request.target_type,
        request.target_id,
        &created_at,
    );
    let event = AuditEvent {
        event_id,
        event_type: request.event_type.to_string(),
        target_type: request.target_type.to_string(),
        target_id: request.target_id.to_string(),
        actor: request.actor.to_string(),
        created_at,
        before_state: request.before_state,
        after_state: request.after_state,
        result: request.result.to_string(),
        error_message: None,
    };

    connection.execute(
        "INSERT INTO audit_events (
            event_id,
            event_type,
            target_type,
            target_id,
            actor,
            created_at,
            before_state,
            after_state,
            result,
            error_message
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            event.event_id,
            event.event_type,
            event.target_type,
            event.target_id,
            event.actor,
            event.created_at,
            event.before_state,
            event.after_state,
            event.result,
            event.error_message,
        ],
    )?;

    Ok(event)
}

pub fn safe_managed_name(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();

    if sanitized.is_empty() {
        "session".to_string()
    } else {
        sanitized
    }
}

pub fn ensure_managed_path(path: &Path, managed_root: &Path, label: &str) -> ActionResult<PathBuf> {
    let managed_root = managed_root.canonicalize().map_err(|_| {
        ActionError::Precondition(format!(
            "{label} must stay inside the managed quarantine root"
        ))
    })?;
    let resolved = path.canonicalize().map_err(|_| {
        ActionError::Precondition(format!(
            "{label} must stay inside the managed quarantine root"
        ))
    })?;

    if resolved.starts_with(&managed_root) {
        Ok(resolved)
    } else {
        Err(ActionError::Precondition(format!(
            "{label} must stay inside the managed quarantine root"
        )))
    }
}

pub fn ensure_allowed_restore_path(
    path: &Path,
    allowed_roots: &[PathBuf],
    quarantine_root: &Path,
    label: &str,
) -> ActionResult<PathBuf> {
    if !path.is_absolute() {
        return Err(ActionError::Precondition(format!(
            "{label} must be an absolute path inside an allowed session root"
        )));
    }

    let normalized_target = normalize_lexical_path(path);
    let normalized_quarantine_root = normalize_lexical_path(quarantine_root);

    if normalized_target.starts_with(&normalized_quarantine_root) {
        return Err(ActionError::Precondition(format!(
            "{label} must not point back into the managed quarantine root"
        )));
    }

    let is_allowed = allowed_roots
        .iter()
        .map(|root| normalize_lexical_path(root))
        .any(|root| normalized_target.starts_with(&root));

    if is_allowed {
        Ok(normalized_target)
    } else {
        Err(ActionError::Precondition(format!(
            "{label} must stay inside an allowed session root"
        )))
    }
}

pub fn has_successful_markdown_export(
    connection: &Connection,
    session_id: &str,
) -> ActionResult<bool> {
    let exists = has_successful_session_event(connection, session_id, "export_markdown")?;

    Ok(exists != 0)
}

pub fn has_successful_cleanup_checklist(
    connection: &Connection,
    session_id: &str,
) -> ActionResult<bool> {
    let exists = has_successful_session_event(connection, session_id, "cleanup_checklist")?;

    Ok(exists != 0)
}

pub fn latest_session_end_hook_failed(
    connection: &Connection,
    session_id: &str,
) -> ActionResult<bool> {
    let latest_result = connection.query_row(
        "SELECT result
         FROM audit_events
         WHERE event_type = 'session_end_hook'
           AND target_type = 'session'
           AND target_id = ?1
         ORDER BY created_at DESC
         LIMIT 1",
        [session_id],
        |row| row.get::<_, String>(0),
    );

    match latest_result {
        Ok(result) => Ok(result == "failed"),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
        Err(error) => Err(ActionError::Sql(error)),
    }
}

pub fn move_path(source: &Path, destination: &Path) -> ActionResult<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::rename(source, destination) {
        Ok(()) => Ok(()),
        Err(_) if source.is_dir() => {
            copy_directory(source, destination)?;
            fs::remove_dir_all(source)?;
            Ok(())
        }
        Err(_) => {
            fs::copy(source, destination)?;
            fs::remove_file(source)?;
            Ok(())
        }
    }
}

pub fn remove_empty_parents(start: &Path, stop_at: &Path) -> ActionResult<()> {
    let mut current = start.parent();
    while let Some(path) = current {
        if path == stop_at || !path.starts_with(stop_at) {
            break;
        }

        if fs::read_dir(path)?.next().is_some() {
            break;
        }

        fs::remove_dir(path)?;
        current = path.parent();
    }

    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> ActionResult<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry.path();
        let next_destination = destination.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_directory(&entry_path, &next_destination)?;
        } else {
            if let Some(parent) = next_destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&entry_path, &next_destination)?;
        }
    }

    Ok(())
}

fn audit_event_id(
    event_type: &str,
    target_type: &str,
    target_id: &str,
    created_at: &str,
) -> String {
    let payload = format!("{event_type}:{target_type}:{target_id}:{created_at}");
    let digest = Sha256::digest(payload.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn normalize_lexical_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            std::path::Component::RootDir => {
                normalized.push(component.as_os_str());
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(segment) => normalized.push(segment),
        }
    }

    normalized
}

fn has_successful_session_event(
    connection: &Connection,
    session_id: &str,
    event_type: &str,
) -> ActionResult<i64> {
    connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1
                FROM audit_events
                WHERE event_type = ?1
                  AND target_type = 'session'
                  AND target_id = ?2
                  AND result = 'success'
            )",
            params![event_type, session_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(Into::into)
}

#[cfg(test)]
mod tests;
