use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::domain::session::SessionRecord;

use super::traits::{AdapterError, AdapterResult, SessionAdapter, collect_files, hash_file};

pub struct ClaudeCodeAdapter;

impl SessionAdapter for ClaudeCodeAdapter {
    fn assistant_name(&self) -> &'static str {
        "claude-code"
    }

    fn discover_session_files(&self, root: &Path) -> AdapterResult<Vec<PathBuf>> {
        collect_files(root, &|path| path.extension().and_then(|value| value.to_str()) == Some("jsonl"))
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

            session_id = parsed
                .get("sessionId")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or(session_id);
            project_path = parsed
                .get("cwd")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or(project_path);

            if matches!(
                parsed.get("type").and_then(Value::as_str),
                Some("user" | "assistant")
            ) {
                message_count += 1;
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
            raw_format: "claude-code-jsonl".to_string(),
            content_hash: hash_file(source)?,
        })
    }
}
