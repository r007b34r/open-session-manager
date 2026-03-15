use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::domain::session::SessionRecord;

use super::traits::{AdapterError, AdapterResult, SessionAdapter, collect_files, hash_file};

pub struct CopilotCliAdapter;

impl SessionAdapter for CopilotCliAdapter {
    fn assistant_name(&self) -> &'static str {
        "github-copilot-cli"
    }

    fn discover_session_files(&self, root: &Path) -> AdapterResult<Vec<PathBuf>> {
        collect_files(root, &|path| {
            path.extension().and_then(|value| value.to_str()) == Some("jsonl")
                && path
                    .components()
                    .any(|component| component.as_os_str() == "session-state")
        })
    }

    fn parse_session(&self, source: &Path) -> AdapterResult<SessionRecord> {
        let reader = BufReader::new(File::open(source)?);

        let mut session_id = source
            .file_stem()
            .and_then(|value| value.to_str())
            .map(ToOwned::to_owned);
        let mut started_at = None;
        let mut last_activity_at = None;
        let mut project_path = None;
        let mut message_count = 0_u32;
        let mut tool_calls = BTreeSet::new();

        for line in reader.lines() {
            let line = line?;
            let parsed: Value = serde_json::from_str(&line)?;
            let data = parsed.get("data").unwrap_or(&Value::Null);

            if let Some(timestamp) = parsed.get("timestamp").and_then(Value::as_str) {
                if started_at.is_none() {
                    started_at = Some(timestamp.to_string());
                }
                last_activity_at = Some(timestamp.to_string());
            }

            match parsed.get("type").and_then(Value::as_str) {
                Some("session.start") => {
                    session_id = data
                        .get("sessionId")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .or(session_id);
                }
                Some("session.info") => {
                    project_path = data
                        .get("message")
                        .and_then(Value::as_str)
                        .and_then(parse_folder_trust_path)
                        .or(project_path);
                }
                Some("user.message") => {
                    if data
                        .get("content")
                        .and_then(Value::as_str)
                        .is_some_and(|content| !content.trim().is_empty())
                    {
                        message_count += 1;
                    }
                }
                Some("assistant.message") => {
                    if data
                        .get("content")
                        .and_then(Value::as_str)
                        .is_some_and(|content| !content.trim().is_empty())
                    {
                        message_count += 1;
                    }
                    for request in copilot_tool_requests(data) {
                        if let Some(tool_call_id) =
                            request.get("toolCallId").and_then(Value::as_str)
                        {
                            tool_calls.insert(tool_call_id.to_string());
                        }
                    }
                }
                Some("tool.execution_start" | "tool.execution_complete") => {
                    if let Some(tool_call_id) = data.get("toolCallId").and_then(Value::as_str) {
                        tool_calls.insert(tool_call_id.to_string());
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
            tool_count: tool_calls.len() as u32,
            status: "available".to_string(),
            raw_format: "github-copilot-cli-jsonl".to_string(),
            content_hash: hash_file(source)?,
        })
    }
}

pub(crate) fn parse_folder_trust_path(message: &str) -> Option<String> {
    let message = message.trim();
    let rest = message.strip_prefix("Folder ")?;
    let path = rest.split(" has been").next()?.trim();
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

pub(crate) fn copilot_tool_requests(data: &Value) -> Vec<&Value> {
    data.get("toolRequests")
        .and_then(Value::as_array)
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}
