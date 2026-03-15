use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::domain::session::SessionRecord;

use super::traits::{AdapterError, AdapterResult, SessionAdapter, collect_files, hash_file};

pub struct FactoryDroidAdapter;

impl SessionAdapter for FactoryDroidAdapter {
    fn assistant_name(&self) -> &'static str {
        "factory-droid"
    }

    fn discover_session_files(&self, root: &Path) -> AdapterResult<Vec<PathBuf>> {
        let mut files = collect_files(root, &|path| {
            path.extension().and_then(|value| value.to_str()) == Some("jsonl")
        })?;
        files.retain(|path| looks_like_droid_file(path).unwrap_or(false));
        Ok(files)
    }

    fn parse_session(&self, source: &Path) -> AdapterResult<SessionRecord> {
        match detect_droid_dialect(source)? {
            DroidDialect::SessionStore => parse_session_store(source, self.assistant_name()),
            DroidDialect::StreamJson => parse_stream_json(source, self.assistant_name()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DroidDialect {
    SessionStore,
    StreamJson,
}

pub(crate) fn detect_droid_dialect(source: &Path) -> AdapterResult<DroidDialect> {
    let reader = BufReader::new(File::open(source)?);

    for line in reader.lines().take(10) {
        let line = line?;
        let parsed: Value = serde_json::from_str(&line)?;
        let Some(raw_kind) = parsed.get("type").and_then(Value::as_str) else {
            continue;
        };
        let kind = normalize_droid_kind(raw_kind);

        if kind == "sessionstart" {
            return Ok(DroidDialect::SessionStore);
        }

        if kind == "message" && parsed.get("message").is_some() {
            return Ok(DroidDialect::SessionStore);
        }

        if matches!(
            kind.as_str(),
            "system" | "message" | "toolcall" | "toolresult" | "completion" | "error"
        ) {
            return Ok(DroidDialect::StreamJson);
        }
    }

    Err(AdapterError::InvalidSession(format!(
        "unrecognized factory droid transcript in {}",
        source.display()
    )))
}

pub(crate) fn looks_like_droid_file(source: &Path) -> AdapterResult<bool> {
    detect_droid_dialect(source).map(|_| true).or_else(|error| match error {
        AdapterError::InvalidSession(_) => Ok(false),
        other => Err(other),
    })
}

pub(crate) fn normalize_droid_kind(raw: &str) -> String {
    raw.trim()
        .to_ascii_lowercase()
        .replace(['_', '-'], "")
}

fn parse_session_store(source: &Path, assistant: &str) -> AdapterResult<SessionRecord> {
    let reader = BufReader::new(File::open(source)?);

    let mut session_id = None;
    let mut started_at = None;
    let mut last_activity_at = None;
    let mut project_path = None;
    let mut message_count = 0_u32;
    let mut tool_count = 0_u32;

    for line in reader.lines() {
        let line = line?;
        let parsed: Value = serde_json::from_str(&line)?;
        let kind = parsed
            .get("type")
            .and_then(Value::as_str)
            .map(normalize_droid_kind)
            .unwrap_or_default();

        if let Some(timestamp) = parsed.get("timestamp").and_then(Value::as_str) {
            if started_at.is_none() {
                started_at = Some(timestamp.to_string());
            }
            last_activity_at = Some(timestamp.to_string());
        }

        if kind == "sessionstart" {
            session_id = parsed.get("id").and_then(Value::as_str).map(ToOwned::to_owned);
            project_path = parsed.get("cwd").and_then(Value::as_str).map(ToOwned::to_owned);
            continue;
        }

        if kind != "message" {
            continue;
        }

        let Some(message) = parsed.get("message") else {
            continue;
        };
        let parts = message
            .get("content")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let has_text = parts.iter().any(|part| {
            part.get("type")
                .and_then(Value::as_str)
                .is_some_and(|kind| normalize_droid_kind(kind) == "text")
                && part
                    .get("text")
                    .and_then(Value::as_str)
                    .is_some_and(|text| !text.trim().is_empty())
        });

        if has_text {
            message_count += 1;
        }

        tool_count += parts
            .iter()
            .filter(|part| {
                part.get("type")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| normalize_droid_kind(kind) == "tooluse")
            })
            .count() as u32;
    }

    let session_id = session_id.ok_or_else(|| {
        AdapterError::InvalidSession(format!("missing session id in {}", source.display()))
    })?;

    Ok(SessionRecord {
        session_id,
        installation_id: None,
        assistant: assistant.to_string(),
        environment: "windows".to_string(),
        project_path,
        source_path: source.display().to_string(),
        started_at: started_at.clone(),
        ended_at: last_activity_at.clone(),
        last_activity_at,
        message_count,
        tool_count,
        status: "available".to_string(),
        raw_format: "factory-droid-session-store".to_string(),
        content_hash: hash_file(source)?,
    })
}

fn parse_stream_json(source: &Path, assistant: &str) -> AdapterResult<SessionRecord> {
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
        let kind = parsed
            .get("type")
            .and_then(Value::as_str)
            .map(normalize_droid_kind)
            .unwrap_or_default();

        session_id = parsed
            .get("session_id")
            .or_else(|| parsed.get("sessionId"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or(session_id);

        if let Some(timestamp) = parsed.get("timestamp").and_then(Value::as_str) {
            if started_at.is_none() {
                started_at = Some(timestamp.to_string());
            }
            last_activity_at = Some(timestamp.to_string());
        }

        if project_path.is_none() {
            project_path = parsed
                .get("cwd")
                .or_else(|| parsed.get("workingDirectory"))
                .or_else(|| parsed.get("working_directory"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
        }

        match kind.as_str() {
            "message" => {
                let role = parsed.get("role").and_then(Value::as_str).unwrap_or_default();
                let content = parsed
                    .get("content")
                    .or_else(|| parsed.get("text"))
                    .or_else(|| parsed.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or_default();

                if matches!(role, "user" | "assistant") && !content.trim().is_empty() {
                    message_count += 1;
                }
            }
            "completion" => {
                if parsed
                    .get("finalText")
                    .or_else(|| parsed.get("final"))
                    .and_then(Value::as_str)
                    .is_some_and(|text| !text.trim().is_empty())
                {
                    message_count += 1;
                }
            }
            "toolcall" => {
                if let Some(tool_call_id) = droid_tool_call_id(&parsed) {
                    tool_calls.insert(tool_call_id.to_string());
                }
            }
            "toolresult" => {
                if let Some(tool_call_id) = droid_tool_call_id(&parsed) {
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
        assistant: assistant.to_string(),
        environment: "windows".to_string(),
        project_path,
        source_path: source.display().to_string(),
        started_at: started_at.clone(),
        ended_at: last_activity_at.clone(),
        last_activity_at,
        message_count,
        tool_count: tool_calls.len() as u32,
        status: "available".to_string(),
        raw_format: "factory-droid-stream-json".to_string(),
        content_hash: hash_file(source)?,
    })
}

pub(crate) fn droid_tool_call_id(parsed: &Value) -> Option<&str> {
    parsed
        .get("toolCallId")
        .or_else(|| parsed.get("tool_call_id"))
        .or_else(|| parsed.get("id"))
        .and_then(Value::as_str)
}
