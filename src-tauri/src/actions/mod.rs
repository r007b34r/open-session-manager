pub mod delete;
pub mod export;
pub mod restore;

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::domain::audit::AuditEvent;

pub type ActionResult<T> = Result<T, ActionError>;

#[derive(Debug)]
pub enum ActionError {
    Io(io::Error),
    Sql(rusqlite::Error),
    Json(serde_json::Error),
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Sql(error) => write!(f, "sqlite error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuarantineManifest {
    pub session_id: String,
    pub original_path: PathBuf,
    pub quarantined_path: PathBuf,
    pub manifest_path: PathBuf,
    pub deleted_at: String,
}

pub fn write_audit_event(
    connection: &Connection,
    event_type: &str,
    target_type: &str,
    target_id: &str,
    actor: &str,
    before_state: Option<String>,
    after_state: Option<String>,
    result: &str,
) -> ActionResult<AuditEvent> {
    let created_at = Utc::now().to_rfc3339();
    let event_id = audit_event_id(event_type, target_type, target_id, &created_at);
    let event = AuditEvent {
        event_id,
        event_type: event_type.to_string(),
        target_type: target_type.to_string(),
        target_id: target_id.to_string(),
        actor: actor.to_string(),
        created_at,
        before_state,
        after_state,
        result: result.to_string(),
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

#[cfg(test)]
mod tests;
