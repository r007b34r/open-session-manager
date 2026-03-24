use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::domain::session::SessionRecord;

use super::traits::{AdapterError, AdapterResult, SessionAdapter, collect_files, hash_file};

pub struct QwenCliAdapter;

impl SessionAdapter for QwenCliAdapter {
    fn assistant_name(&self) -> &'static str {
        "qwen-cli"
    }

    fn discover_session_files(&self, root: &Path) -> AdapterResult<Vec<PathBuf>> {
        collect_files(root, &|path| {
            path.extension().and_then(|value| value.to_str()) == Some("jsonl")
                && path
                    .components()
                    .any(|component| component.as_os_str() == "chats")
        })
    }

    fn parse_session(&self, source: &Path) -> AdapterResult<SessionRecord> {
        let lines = read_qwen_lines(source)?;
        if lines.is_empty() {
            return Err(AdapterError::InvalidSession(format!(
                "missing session lines in {}",
                source.display()
            )));
        }

        let session_id = lines
            .iter()
            .find_map(|line| qwen_session_id(line).map(ToOwned::to_owned))
            .unwrap_or_else(|| fallback_session_id(source));
        let project_path = lines
            .iter()
            .find_map(|line| qwen_project_path(line).map(ToOwned::to_owned))
            .or_else(|| path_project_path(source));
        let started_at = lines.iter().find_map(qwen_timestamp);
        let last_activity_at = lines.iter().rev().find_map(qwen_timestamp);
        let message_count = lines
            .iter()
            .filter(|line| {
                qwen_role(line).is_some_and(|role| matches!(role, "user" | "assistant"))
                    && qwen_text(line).is_some()
            })
            .count() as u32;
        let tool_count = lines.iter().map(|line| qwen_tool_calls(line).len() as u32).sum();

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
            raw_format: "qwen-cli-jsonl".to_string(),
            content_hash: hash_file(source)?,
        })
    }
}

pub(crate) fn read_qwen_lines(source: &Path) -> AdapterResult<Vec<Value>> {
    let reader = BufReader::new(File::open(source)?);
    let mut lines = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
            lines.push(parsed);
        }
    }

    Ok(lines)
}

pub(crate) fn qwen_role(value: &Value) -> Option<&str> {
    let role = value
        .get("type")
        .or_else(|| value.get("role"))
        .or_else(|| value.get("message").and_then(|message| message.get("role")))
        .and_then(Value::as_str)?;

    if role.eq_ignore_ascii_case("human") {
        Some("user")
    } else if role.eq_ignore_ascii_case("model") {
        Some("assistant")
    } else {
        Some(role)
    }
}

pub(crate) fn qwen_text(value: &Value) -> Option<String> {
    [
        value.get("content"),
        value.get("text"),
        value.get("message").and_then(|message| message.get("content")),
        value.get("message").and_then(|message| message.get("text")),
        value.get("parts"),
        value.get("message").and_then(|message| message.get("parts")),
    ]
    .into_iter()
    .flatten()
    .find_map(extract_qwen_text_value)
}

pub(crate) fn qwen_timestamp(value: &Value) -> Option<String> {
    value
        .get("timestamp")
        .or_else(|| value.get("createdAt"))
        .or_else(|| value.get("created_at"))
        .or_else(|| value.get("message").and_then(|message| message.get("timestamp")))
        .and_then(stringish)
}

pub(crate) fn qwen_session_id(value: &Value) -> Option<&str> {
    value.get("sessionId")
        .or_else(|| value.get("session_id"))
        .or_else(|| value.get("message").and_then(|message| message.get("sessionId")))
        .or_else(|| value.get("message").and_then(|message| message.get("session_id")))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

pub(crate) fn qwen_project_path(value: &Value) -> Option<&str> {
    value
        .get("cwd")
        .or_else(|| value.get("projectPath"))
        .or_else(|| value.get("project_path"))
        .or_else(|| value.get("workspace"))
        .or_else(|| value.get("message").and_then(|message| message.get("cwd")))
        .or_else(|| {
            value
                .get("message")
                .and_then(|message| message.get("projectPath"))
        })
        .or_else(|| value.get("message").and_then(|message| message.get("workspace")))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
}

pub(crate) fn qwen_usage_metadata(value: &Value) -> Option<&Value> {
    value
        .get("usageMetadata")
        .or_else(|| value.get("usage_metadata"))
        .or_else(|| {
            value
                .get("message")
                .and_then(|message| message.get("usageMetadata"))
        })
        .or_else(|| {
            value
                .get("message")
                .and_then(|message| message.get("usage_metadata"))
        })
}

pub(crate) fn qwen_tool_calls(value: &Value) -> Vec<&Value> {
    value
        .get("toolCalls")
        .or_else(|| value.get("tool_calls"))
        .or_else(|| value.get("message").and_then(|message| message.get("toolCalls")))
        .or_else(|| value.get("message").and_then(|message| message.get("tool_calls")))
        .and_then(Value::as_array)
        .map(|calls| calls.iter().collect())
        .unwrap_or_default()
}

fn extract_qwen_text_value(value: &Value) -> Option<String> {
    if let Some(text) = value.as_str().map(str::trim).filter(|text| !text.is_empty()) {
        return Some(text.to_string());
    }

    if let Some(text) = value
        .get("text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Some(text.to_string());
    }

    value
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter_map(extract_qwen_text_value)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn fallback_session_id(source: &Path) -> String {
    let file_name = source
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown");
    let project_name = source
        .parent()
        .and_then(Path::parent)
        .and_then(|value| value.file_name())
        .and_then(|value| value.to_str())
        .unwrap_or("unknown");
    format!("{project_name}-{file_name}")
}

fn path_project_path(source: &Path) -> Option<String> {
    source
        .parent()
        .and_then(Path::parent)
        .and_then(|value| value.file_name())
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
}

fn stringish(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .or_else(|| value.as_i64().map(|value| value.to_string()))
        .or_else(|| value.as_u64().map(|value| value.to_string()))
        .or_else(|| value.as_f64().map(|value| value.to_string()))
}
