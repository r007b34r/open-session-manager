use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::domain::session::SessionRecord;

use super::traits::{AdapterError, AdapterResult, SessionAdapter, collect_files, hash_file};

pub struct OpenCodeAdapter;

impl SessionAdapter for OpenCodeAdapter {
    fn assistant_name(&self) -> &'static str {
        "opencode"
    }

    fn discover_session_files(&self, root: &Path) -> AdapterResult<Vec<PathBuf>> {
        collect_files(root, &|path| {
            path.extension().and_then(|value| value.to_str()) == Some("json")
                && path
                    .components()
                    .any(|component| component.as_os_str() == "storage")
                && path
                    .components()
                    .any(|component| component.as_os_str() == "info")
        })
    }

    fn parse_session(&self, source: &Path) -> AdapterResult<SessionRecord> {
        let session_info: Value = serde_json::from_slice(&fs::read(source)?)?;
        let session_id = session_info
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AdapterError::InvalidSession(format!("missing session id in {}", source.display()))
            })?
            .to_string();

        let storage_root = source
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or_else(|| {
                AdapterError::InvalidSession("invalid opencode storage path".to_string())
            })?;

        let message_dir = storage_root
            .join("session")
            .join("message")
            .join(&session_id);
        let part_dir = storage_root.join("session").join("part").join(&session_id);

        let message_files = collect_files(&message_dir, &|path| {
            path.extension().and_then(|value| value.to_str()) == Some("json")
        })?;

        let mut last_activity_at = session_info
            .get("time")
            .and_then(|value| value.get("updated"))
            .and_then(Value::as_i64)
            .map(|value| value.to_string());
        let mut tool_count = 0_u32;

        for message_file in &message_files {
            let message: Value = serde_json::from_slice(&fs::read(message_file)?)?;
            let message_id = message
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| AdapterError::InvalidSession("message missing id".to_string()))?;

            let message_part_dir = part_dir.join(message_id);
            let part_files = collect_files(&message_part_dir, &|path| {
                path.extension().and_then(|value| value.to_str()) == Some("json")
            })?;

            for part_file in part_files {
                let part: Value = serde_json::from_slice(&fs::read(part_file)?)?;
                if part.get("type").and_then(Value::as_str) == Some("tool") {
                    tool_count += 1;
                }
            }

            if let Some(timestamp) = message
                .get("time")
                .and_then(|value| value.get("completed").or_else(|| value.get("created")))
                .and_then(Value::as_i64)
            {
                last_activity_at = Some(timestamp.to_string());
            }
        }

        let started_at = session_info
            .get("time")
            .and_then(|value| value.get("created"))
            .and_then(Value::as_i64)
            .map(|value| value.to_string());

        Ok(SessionRecord {
            session_id,
            installation_id: None,
            assistant: self.assistant_name().to_string(),
            environment: "linux".to_string(),
            project_path: session_info
                .get("directory")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            source_path: source.display().to_string(),
            started_at: started_at.clone(),
            ended_at: last_activity_at.clone(),
            last_activity_at,
            message_count: message_files.len() as u32,
            tool_count,
            status: "available".to_string(),
            raw_format: "opencode-storage".to_string(),
            content_hash: hash_file(source)?,
        })
    }
}
