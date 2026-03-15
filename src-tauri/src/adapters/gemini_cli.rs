use std::{fs, path::{Path, PathBuf}};

use serde_json::Value;

use crate::domain::session::SessionRecord;

use super::traits::{AdapterError, AdapterResult, SessionAdapter, collect_files, hash_file};

pub struct GeminiCliAdapter;

impl SessionAdapter for GeminiCliAdapter {
    fn assistant_name(&self) -> &'static str {
        "gemini-cli"
    }

    fn discover_session_files(&self, root: &Path) -> AdapterResult<Vec<PathBuf>> {
        collect_files(root, &|path| {
            path.extension().and_then(|value| value.to_str()) == Some("json")
                && path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| name.starts_with("session-"))
        })
    }

    fn parse_session(&self, source: &Path) -> AdapterResult<SessionRecord> {
        let parsed: Value = serde_json::from_slice(&fs::read(source)?)?;
        let messages = gemini_messages(&parsed);

        let session_id = parsed
            .get("sessionId")
            .or_else(|| parsed.get("session_id"))
            .or_else(|| parsed.get("id"))
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AdapterError::InvalidSession(format!("missing session id in {}", source.display()))
            })?
            .to_string();
        let project_path = parsed
            .get("cwd")
            .or_else(|| parsed.get("projectPath"))
            .or_else(|| parsed.get("projectRoot"))
            .or_else(|| parsed.get("project_root"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| gemini_project_path(&messages));
        let started_at = parsed
            .get("startTime")
            .or_else(|| parsed.get("start_time"))
            .and_then(stringish)
            .or_else(|| messages.iter().find_map(|message| gemini_timestamp(message)));
        let last_activity_at = parsed
            .get("lastUpdated")
            .or_else(|| parsed.get("last_updated"))
            .and_then(stringish)
            .or_else(|| {
                messages
                    .iter()
                    .rev()
                    .find_map(|message| gemini_timestamp(message))
            });
        let message_count = messages
            .iter()
            .filter(|message| {
                gemini_role(message).is_some_and(|role| matches!(role, "user" | "assistant"))
                    && gemini_text(message).is_some()
            })
            .count() as u32;
        let tool_count = messages
            .iter()
            .map(|message| gemini_tool_calls(message).len() as u32)
            .sum();

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
            raw_format: "gemini-cli-json".to_string(),
            content_hash: hash_file(source)?,
        })
    }
}

pub(crate) fn gemini_messages(parsed: &Value) -> Vec<&Value> {
    if let Some(messages) = parsed.as_array() {
        return messages.iter().collect();
    }

    parsed
        .get("messages")
        .or_else(|| parsed.get("history"))
        .or_else(|| parsed.get("items"))
        .and_then(Value::as_array)
        .map(|messages| messages.iter().collect())
        .unwrap_or_default()
}

pub(crate) fn gemini_role(message: &Value) -> Option<&str> {
    let role = message
        .get("type")
        .or_else(|| message.get("role"))
        .and_then(Value::as_str)?;

    if role.eq_ignore_ascii_case("human") {
        Some("user")
    } else if role.eq_ignore_ascii_case("model") || role.eq_ignore_ascii_case("gemini") {
        Some("assistant")
    } else {
        Some(role)
    }
}

pub(crate) fn gemini_text(message: &Value) -> Option<String> {
    if let Some(content) = message.get("content").and_then(Value::as_str) {
        let content = content.trim();
        if !content.is_empty() {
            return Some(content.to_string());
        }
    }

    if let Some(text) = message.get("text").and_then(Value::as_str) {
        let text = text.trim();
        if !text.is_empty() {
            return Some(text.to_string());
        }
    }

    for key in ["content", "parts"] {
        let parts = message.get(key).and_then(Value::as_array);
        if let Some(parts) = parts {
            let joined = parts
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join(" ");

            if !joined.is_empty() {
                return Some(joined);
            }
        }
    }

    None
}

pub(crate) fn gemini_timestamp(message: &Value) -> Option<String> {
    message
        .get("timestamp")
        .or_else(|| message.get("ts"))
        .or_else(|| message.get("time"))
        .or_else(|| message.get("created_at"))
        .and_then(stringish)
}

pub(crate) fn gemini_tool_calls(message: &Value) -> Vec<&Value> {
    message
        .get("toolCalls")
        .or_else(|| message.get("tool_calls"))
        .and_then(Value::as_array)
        .map(|calls| calls.iter().collect())
        .unwrap_or_default()
}

fn gemini_project_path(messages: &[&Value]) -> Option<String> {
    messages.iter().find_map(|message| {
        message
            .get("cwd")
            .or_else(|| message.get("projectPath"))
            .or_else(|| message.get("projectRoot"))
            .or_else(|| message.get("project_root"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    })
}

fn stringish(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .or_else(|| value.as_i64().map(|value| value.to_string()))
        .or_else(|| value.as_u64().map(|value| value.to_string()))
        .or_else(|| value.as_f64().map(|value| value.to_string()))
}
