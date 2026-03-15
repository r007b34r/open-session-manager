use std::{
    fs,
    path::{Path, PathBuf},
};

use rusqlite::Connection;
use serde_json::json;

use crate::domain::session::{SessionInsight, SessionRecord};

use super::{ActionResult, write_audit_event};

pub struct ExportRequest<'a> {
    pub session: &'a SessionRecord,
    pub insight: &'a SessionInsight,
    pub output_root: &'a Path,
    pub actor: &'a str,
    pub connection: &'a Connection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportResult {
    pub output_path: PathBuf,
}

pub fn export_session_markdown(request: &ExportRequest<'_>) -> ActionResult<ExportResult> {
    fs::create_dir_all(request.output_root)?;

    let output_path = request
        .output_root
        .join(format!("session-{}.md", request.session.session_id));

    let markdown = build_markdown(request.session, request.insight);
    fs::write(&output_path, markdown)?;

    write_audit_event(
        request.connection,
        "export_markdown",
        "session",
        &request.session.session_id,
        request.actor,
        Some(json!({ "source_path": request.session.source_path }).to_string()),
        Some(json!({ "output_path": output_path }).to_string()),
        "success",
    )?;

    Ok(ExportResult { output_path })
}

fn build_markdown(session: &SessionRecord, insight: &SessionInsight) -> String {
    let project_path = session.project_path.as_deref().unwrap_or("unknown");
    let last_activity = session.last_activity_at.as_deref().unwrap_or("unknown");
    let progress_percent = insight
        .progress_percent
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    format!(
        "---\n\
title: {title}\n\
session_id: {session_id}\n\
assistant: {assistant}\n\
environment: {environment}\n\
project_path: {project_path}\n\
last_activity_at: {last_activity}\n\
---\n\n\
# {title}\n\n\
## Summary\n\
{summary}\n\n\
## Progress\n\
- State: {progress_state}\n\
- Percent: {progress_percent}\n\
- Value score: {value_score}\n\
- Garbage score: {garbage_score}\n\n\
## Source\n\
- Transcript path: `{source_path}`\n\
- Raw format: `{raw_format}`\n",
        title = insight.title,
        session_id = session.session_id,
        assistant = session.assistant,
        environment = session.environment,
        project_path = project_path,
        last_activity = last_activity,
        summary = insight.summary,
        progress_state = insight.progress_state,
        progress_percent = progress_percent,
        value_score = insight.value_score,
        garbage_score = insight.garbage_score,
        source_path = session.source_path,
        raw_format = session.raw_format,
    )
}
