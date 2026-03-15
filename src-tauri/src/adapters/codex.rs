use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::domain::session::SessionRecord;

use super::traits::{AdapterError, AdapterResult, SessionAdapter, collect_files, hash_file};

pub struct CodexAdapter;

impl SessionAdapter for CodexAdapter {
    fn assistant_name(&self) -> &'static str {
        "codex"
    }

    fn discover_session_files(&self, root: &Path) -> AdapterResult<Vec<PathBuf>> {
        collect_files(root, &|path| {
            path.extension().and_then(|value| value.to_str()) == Some("jsonl")
                && path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| name.starts_with("rollout-"))
        })
    }

    fn parse_session(&self, source: &Path) -> AdapterResult<SessionRecord> {
        let reader = BufReader::new(File::open(source)?);

        let mut session_id = None;
        let mut started_at = None;
        let mut last_activity_at = None;
        let mut project_path = None;
        let mut message_count = 0_u32;
        let tool_count = 0_u32;

        for line in reader.lines() {
            let line = line?;
            let parsed: Value = serde_json::from_str(&line)?;

            if let Some(timestamp) = parsed.get("timestamp").and_then(Value::as_str) {
                if started_at.is_none() {
                    started_at = Some(timestamp.to_string());
                }
                last_activity_at = Some(timestamp.to_string());
            }

            match parsed.get("type").and_then(Value::as_str) {
                Some("session_meta") => {
                    let payload = parsed.get("payload").unwrap_or(&Value::Null);
                    session_id = payload
                        .get("id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .or(session_id);
                    project_path = payload
                        .get("cwd")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .or(project_path);
                }
                Some("response_item") => {
                    let payload = parsed.get("payload").unwrap_or(&Value::Null);
                    if payload.get("type").and_then(Value::as_str) == Some("message") {
                        let role = payload.get("role").and_then(Value::as_str);
                        if matches!(role, Some("user" | "assistant")) {
                            message_count += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        let session_id = session_id.ok_or_else(|| {
            AdapterError::InvalidSession(format!("missing session id in {}", source.display()))
        })?;

        Ok(SessionRecord {
            session_id,
            installation_id: None,
            assistant: self.assistant_name().to_string(),
            environment: "windows".to_string(),
            project_path,
            source_path: source.display().to_string(),
            started_at: started_at.clone(),
            ended_at: last_activity_at.clone(),
            last_activity_at,
            message_count,
            tool_count,
            status: "available".to_string(),
            raw_format: "codex-jsonl".to_string(),
            content_hash: hash_file(source)?,
        })
    }
}
