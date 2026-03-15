use std::{
    fs,
    path::{Path, PathBuf},
};

use rusqlite::Connection;
use serde_json::json;

use crate::{
    domain::session::{SessionInsight, SessionRecord},
    transcript::{TranscriptHighlight, TranscriptTodo, build_transcript_digest},
};

use super::{ActionResult, AuditWriteRequest, safe_managed_name, write_audit_event};

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
        .join(format!(
            "session-{}.md",
            safe_managed_name(&request.session.session_id)
        ));

    let markdown = build_markdown(request.session, request.insight);
    fs::write(&output_path, markdown)?;

    write_audit_event(
        request.connection,
        AuditWriteRequest {
            event_type: "export_markdown",
            target_type: "session",
            target_id: &request.session.session_id,
            actor: request.actor,
            before_state: Some(json!({ "source_path": request.session.source_path }).to_string()),
            after_state: Some(json!({ "output_path": output_path }).to_string()),
            result: "success",
        },
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
    let tags = parse_string_list(&insight.topic_labels_json);
    let risk_flags = parse_string_list(&insight.risk_flags_json);
    let transcript_digest = build_transcript_digest(session);
    let (recommendation_code, recommendation_reason) =
        derive_cleanup_recommendation(insight, &risk_flags);
    let tags_line = join_or_unknown(&tags);
    let risk_flags_line = join_or_unknown(&risk_flags);
    let todo_section = render_todo_section(&transcript_digest.todos);
    let transcript_section = render_transcript_highlights(&transcript_digest.highlights);

    // The richer section layout is a clean-room adoption of the export patterns
    // used in daaain/claude-code-log's markdown exporter and transcript parser.
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
## Cleanup Recommendation\n\
- Decision: `{recommendation_code}`\n\
- Reason: {recommendation_reason}\n\n\
## Progress\n\
- State: {progress_state}\n\
- Percent: {progress_percent}\n\
- Value score: {value_score}\n\
- Garbage score: {garbage_score}\n\n\
## Signals\n\
- Tags: {tags_line}\n\
- Risk flags: {risk_flags_line}\n\
- Confidence: {confidence:.2}\n\n\
{todo_section}\
{transcript_section}\
## Source\n\
- Transcript path: `{source_path}`\n\
- Raw format: `{raw_format}`\n",
        title = yaml_frontmatter_string(&insight.title),
        session_id = yaml_frontmatter_string(&session.session_id),
        assistant = yaml_frontmatter_string(&session.assistant),
        environment = yaml_frontmatter_string(&session.environment),
        project_path = yaml_frontmatter_string(project_path),
        last_activity = yaml_frontmatter_string(last_activity),
        summary = insight.summary,
        recommendation_code = recommendation_code,
        recommendation_reason = recommendation_reason,
        progress_state = insight.progress_state,
        progress_percent = progress_percent,
        value_score = insight.value_score,
        garbage_score = insight.garbage_score,
        tags_line = tags_line,
        risk_flags_line = risk_flags_line,
        confidence = insight.confidence,
        todo_section = todo_section,
        transcript_section = transcript_section,
        source_path = session.source_path,
        raw_format = session.raw_format,
    )
}

fn yaml_frontmatter_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t");

    format!("\"{escaped}\"")
}

fn parse_string_list(value: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(value).unwrap_or_default()
}

fn join_or_unknown(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn derive_cleanup_recommendation(
    insight: &SessionInsight,
    risk_flags: &[String],
) -> (&'static str, String) {
    if insight.garbage_score >= 70 && insight.value_score < 40 {
        return (
            "export_then_quarantine",
            "Low-value session with strong garbage signals. Keep the Markdown artifact, then isolate the raw transcript."
                .to_string(),
        );
    }

    if insight.value_score >= 70 && insight.progress_state != "completed" {
        return (
            "keep_and_resume",
            "High-value work is still in flight. Preserve the transcript and use the export as a follow-up brief."
                .to_string(),
        );
    }

    if insight.value_score >= 70 {
        return (
            "preserve_and_archive",
            "High-value work is already complete. Preserve the Markdown summary and archive the raw session when cleanup is needed."
                .to_string(),
        );
    }

    if !risk_flags.is_empty() {
        return (
            "review_before_cleanup",
            "Risk signals are present. Review the exported notes before moving the original session into quarantine."
                .to_string(),
        );
    }

    (
        "archive_after_export",
        "No major risks were detected. Export the useful parts, then archive or isolate the original session if space matters."
            .to_string(),
    )
}

fn render_todo_section(todos: &[TranscriptTodo]) -> String {
    if todos.is_empty() {
        return String::new();
    }

    let items = todos
        .iter()
        .map(|todo| {
            format!(
                "- [{}] {}",
                if todo.completed { "x" } else { " " },
                todo.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("## Todo Snapshot\n{items}\n\n")
}

fn render_transcript_highlights(highlights: &[TranscriptHighlight]) -> String {
    if highlights.is_empty() {
        return String::new();
    }

    let body = highlights
        .iter()
        .map(|highlight| format!("### {}\n{}\n", highlight.role, highlight.content))
        .collect::<Vec<_>>()
        .join("\n");

    format!("## Transcript Highlights\n{body}\n")
}
