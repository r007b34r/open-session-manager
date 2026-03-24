use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::domain::session::SessionRecord;

use super::traits::{AdapterError, AdapterResult, SessionAdapter, collect_files, hash_file};

#[derive(Debug, Clone, Default)]
pub(crate) struct RooTaskMetadata {
    pub model: Option<String>,
    pub agent: Option<String>,
    pub project_path: Option<String>,
}

pub struct RooCodeAdapter;

impl SessionAdapter for RooCodeAdapter {
    fn assistant_name(&self) -> &'static str {
        "roo-code"
    }

    fn discover_session_files(&self, root: &Path) -> AdapterResult<Vec<PathBuf>> {
        collect_files(root, &|path| {
            path.file_name().and_then(|value| value.to_str()) == Some("ui_messages.json")
        })
    }

    fn parse_session(&self, source: &Path) -> AdapterResult<SessionRecord> {
        let entries = read_roo_entries(source)?;
        if entries.is_empty() {
            return Err(AdapterError::InvalidSession(format!(
                "missing roo entries in {}",
                source.display()
            )));
        }

        let metadata = roo_task_metadata(source);
        let session_id = source
            .parent()
            .and_then(|value| value.file_name())
            .and_then(|value| value.to_str())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                AdapterError::InvalidSession(format!(
                    "missing task id in {}",
                    source.display()
                ))
            })?
            .to_string();
        let started_at = entries.iter().find_map(roo_timestamp);
        let last_activity_at = entries.iter().rev().find_map(roo_timestamp);
        let message_count = entries
            .iter()
            .filter(|entry| roo_visible_message(entry).is_some())
            .count() as u32;
        let tool_count = entries
            .iter()
            .filter(|entry| roo_say(entry).is_some_and(|say| say.contains("tool")))
            .count() as u32;

        Ok(SessionRecord {
            session_id,
            installation_id: None,
            assistant: self.assistant_name().to_string(),
            environment: "windows".to_string(),
            project_path: metadata.project_path,
            source_path: source.display().to_string(),
            started_at: started_at.clone(),
            ended_at: last_activity_at.clone(),
            last_activity_at,
            message_count,
            tool_count,
            status: "available".to_string(),
            raw_format: "roo-code-ui-messages".to_string(),
            content_hash: hash_file(source)?,
        })
    }
}

pub(crate) fn read_roo_entries(source: &Path) -> AdapterResult<Vec<Value>> {
    let parsed: Value = serde_json::from_slice(&fs::read(source)?)?;
    parsed.as_array().cloned().ok_or_else(|| {
        AdapterError::InvalidSession(format!("invalid roo ui messages in {}", source.display()))
    })
}

pub(crate) fn roo_task_metadata(source: &Path) -> RooTaskMetadata {
    let history_path = source
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("api_conversation_history.json");
    let Ok(content) = fs::read_to_string(history_path) else {
        return RooTaskMetadata::default();
    };

    let mut metadata = RooTaskMetadata::default();
    let mut offset = 0_usize;
    const START: &str = "<environment_details>";
    const END: &str = "</environment_details>";

    while let Some(start_rel) = content[offset..].find(START) {
        let start_index = offset + start_rel + START.len();
        let Some(end_rel) = content[start_index..].find(END) else {
            break;
        };
        let end_index = start_index + end_rel;
        let block = &content[start_index..end_index];

        metadata.model = extract_tag_value(block, "model").or(metadata.model);
        metadata.agent = extract_tag_value(block, "slug")
            .or_else(|| extract_tag_value(block, "name"))
            .or(metadata.agent);
        metadata.project_path = extract_tag_value(block, "workspace")
            .or_else(|| extract_tag_value(block, "cwd"))
            .or_else(|| extract_tag_value(block, "projectPath"))
            .or(metadata.project_path);

        offset = end_index + END.len();
    }

    metadata
}

pub(crate) fn roo_say(value: &Value) -> Option<&str> {
    value.get("say").and_then(Value::as_str)
}

pub(crate) fn roo_timestamp(value: &Value) -> Option<String> {
    value.get("ts")
        .or_else(|| value.get("timestamp"))
        .and_then(stringish)
}

pub(crate) fn roo_visible_message(value: &Value) -> Option<(&'static str, String)> {
    let say = roo_say(value)?;
    let text = value
        .get("text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())?
        .to_string();

    if say.contains("user") {
        Some(("user", text))
    } else if say.contains("assistant") {
        Some(("assistant", text))
    } else {
        None
    }
}

pub(crate) fn roo_api_req_payload(value: &Value) -> Option<Value> {
    if value.get("type").and_then(Value::as_str) != Some("say")
        || roo_say(value) != Some("api_req_started")
    {
        return None;
    }

    let text = value.get("text").and_then(Value::as_str)?;
    serde_json::from_str(text).ok()
}

fn extract_tag_value(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start_index = block.find(&open)? + open.len();
    let rest = &block[start_index..];
    let end_rel = rest.find(&close)?;
    let value = rest[..end_rel].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn stringish(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .or_else(|| value.as_i64().map(|value| value.to_string()))
        .or_else(|| value.as_u64().map(|value| value.to_string()))
        .or_else(|| value.as_f64().map(|value| value.to_string()))
}
