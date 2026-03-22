use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::domain::session::SessionRecord;

use super::traits::{AdapterError, AdapterResult, SessionAdapter, collect_files, hash_file};

pub struct OpenClawAdapter;

impl SessionAdapter for OpenClawAdapter {
    fn assistant_name(&self) -> &'static str {
        "openclaw"
    }

    fn discover_session_files(&self, root: &Path) -> AdapterResult<Vec<PathBuf>> {
        collect_files(root, &|path| {
            path.extension().and_then(|value| value.to_str()) == Some("jsonl")
                && !path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| name.ends_with(".jsonl.lock"))
                && path
                    .components()
                    .any(|component| component.as_os_str() == "agents")
                && path
                    .components()
                    .any(|component| component.as_os_str() == "sessions")
        })
    }

    fn parse_session(&self, source: &Path) -> AdapterResult<SessionRecord> {
        let reader = BufReader::new(File::open(source)?);

        let mut session_id = None;
        let mut started_at = None;
        let mut last_activity_at = None;
        let mut project_path = None;
        let mut message_count = 0_u32;
        let mut tool_calls = BTreeSet::new();

        for line in reader.lines() {
            let line = line?;
            let parsed: Value = serde_json::from_str(&line)?;

            if let Some(timestamp) = parsed.get("timestamp").and_then(Value::as_str) {
                if started_at.is_none() {
                    started_at = Some(timestamp.to_string());
                }
                last_activity_at = Some(timestamp.to_string());
            }

            match openclaw_kind(&parsed) {
                Some("session") => {
                    session_id = parsed
                        .get("id")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned);
                    project_path = parsed
                        .get("cwd")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned);
                }
                Some("message") => {
                    let Some(message) = parsed.get("message") else {
                        continue;
                    };
                    match openclaw_role(message) {
                        Some("user" | "assistant") => {
                            if openclaw_text(message).is_some() {
                                message_count += 1;
                            }
                            if openclaw_role(message) == Some("assistant") {
                                for tool_call in openclaw_tool_calls(message) {
                                    if let Some(tool_call_id) =
                                        tool_call.get("id").and_then(Value::as_str)
                                    {
                                        tool_calls.insert(tool_call_id.to_string());
                                    }
                                }
                            }
                        }
                        Some("toolresult") => {
                            if let Some(tool_call_id) = message
                                .get("toolCallId")
                                .or_else(|| message.get("tool_call_id"))
                                .and_then(Value::as_str)
                            {
                                tool_calls.insert(tool_call_id.to_string());
                            }
                        }
                        _ => {}
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
            raw_format: "openclaw-jsonl".to_string(),
            content_hash: hash_file(source)?,
        })
    }
}

pub(crate) fn openclaw_kind(value: &Value) -> Option<&str> {
    value
        .get("type")
        .and_then(Value::as_str)
        .map(|kind| match kind {
            "model_change" => "modelchange",
            "thinking_level_change" => "thinkinglevelchange",
            other => other,
        })
}

pub(crate) fn openclaw_role(value: &Value) -> Option<&str> {
    value
        .get("role")
        .and_then(Value::as_str)
        .map(|role| match role {
            "toolResult" => "toolresult",
            other => other,
        })
}

pub(crate) fn openclaw_text(value: &Value) -> Option<String> {
    let content = value.get("content")?;

    if let Some(text) = content
        .as_str()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Some(text.to_string());
    }

    content
        .as_array()
        .map(|blocks| {
            blocks
                .iter()
                .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|block| block.get("text").and_then(Value::as_str))
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|text| !text.is_empty())
}

pub(crate) fn openclaw_tool_calls(value: &Value) -> Vec<&Value> {
    value
        .get("content")
        .and_then(Value::as_array)
        .map(|blocks| {
            blocks
                .iter()
                .filter(|block| block.get("type").and_then(Value::as_str) == Some("toolCall"))
                .collect()
        })
        .unwrap_or_default()
}
