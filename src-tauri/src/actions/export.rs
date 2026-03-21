use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use rusqlite::Connection;
use serde::Serialize;
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
    pub cleanup_checklist_path: PathBuf,
    pub session_end_hook_report_path: Option<PathBuf>,
}

pub fn export_session_markdown(request: &ExportRequest<'_>) -> ActionResult<ExportResult> {
    fs::create_dir_all(request.output_root)?;

    let output_path = request.output_root.join(format!(
        "session-{}.md",
        safe_managed_name(&request.session.session_id)
    ));

    let markdown = build_markdown(request.session, request.insight);
    fs::write(&output_path, markdown)?;
    let cleanup_result =
        write_cleanup_checklist(request.session, request.insight, request.output_root, &output_path)?;

    write_audit_event(
        request.connection,
        AuditWriteRequest {
            event_type: "export_markdown",
            target_type: "session",
            target_id: &request.session.session_id,
            actor: request.actor,
            before_state: Some(json!({ "source_path": request.session.source_path }).to_string()),
            after_state: Some(
                json!({
                    "output_path": output_path,
                    "checklist_path": cleanup_result.checklist_path,
                    "hook_report_path": cleanup_result.hook.report_path,
                })
                .to_string(),
            ),
            result: "success",
        },
    )?;

    write_audit_event(
        request.connection,
        AuditWriteRequest {
            event_type: "cleanup_checklist",
            target_type: "session",
            target_id: &request.session.session_id,
            actor: request.actor,
            before_state: None,
            after_state: Some(
                json!({
                    "checklist_path": cleanup_result.checklist_path,
                    "ready_for_delete": cleanup_result.ready_for_delete,
                    "hook_status": cleanup_result.hook.status,
                    "hook_report_path": cleanup_result.hook.report_path,
                })
                .to_string(),
            ),
            result: "success",
        },
    )?;

    if cleanup_result.hook.status != "not_configured" {
        write_audit_event(
            request.connection,
            AuditWriteRequest {
                event_type: "session_end_hook",
                target_type: "session",
                target_id: &request.session.session_id,
                actor: request.actor,
                before_state: None,
                after_state: Some(
                    json!({
                        "script_path": cleanup_result.hook.script_path,
                        "hook_report_path": cleanup_result.hook.report_path,
                        "summary": cleanup_result.hook.summary,
                    })
                    .to_string(),
                ),
                result: cleanup_result.hook.result_label(),
            },
        )?;
    }

    Ok(ExportResult {
        output_path,
        cleanup_checklist_path: cleanup_result.checklist_path,
        session_end_hook_report_path: cleanup_result.hook.report_path.clone(),
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupChecklist {
    session_id: String,
    assistant: String,
    export_path: String,
    project_path: String,
    recommendation_code: String,
    recommendation_reason: String,
    progress_state: String,
    open_tasks: usize,
    completed_tasks: usize,
    risk_flags: Vec<String>,
    warnings: Vec<String>,
    ready_for_delete: bool,
    session_end_hook: SessionEndHookResult,
    generated_at: String,
}

#[derive(Debug, Clone)]
struct CleanupChecklistWriteResult {
    checklist_path: PathBuf,
    ready_for_delete: bool,
    hook: SessionEndHookResult,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionEndHookResult {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    script_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_path: Option<PathBuf>,
    summary: String,
}

impl SessionEndHookResult {
    fn result_label(&self) -> &'static str {
        if self.status == "failed" {
            "failed"
        } else {
            "success"
        }
    }
}

fn write_cleanup_checklist(
    session: &SessionRecord,
    insight: &SessionInsight,
    output_root: &Path,
    export_path: &Path,
) -> ActionResult<CleanupChecklistWriteResult> {
    let safe_session_id = safe_managed_name(&session.session_id);
    let checklist_path = output_root.join(format!("cleanup-{safe_session_id}.json"));
    let digest = build_transcript_digest(session);
    let risk_flags = parse_string_list(&insight.risk_flags_json);
    let warnings = build_cleanup_warnings(insight, &risk_flags, &digest.todos);
    let (recommendation_code, recommendation_reason) =
        derive_cleanup_recommendation(insight, &risk_flags);

    let mut checklist = CleanupChecklist {
        session_id: session.session_id.clone(),
        assistant: session.assistant.clone(),
        export_path: export_path.display().to_string(),
        project_path: session
            .project_path
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        recommendation_code: recommendation_code.to_string(),
        recommendation_reason,
        progress_state: insight.progress_state.clone(),
        open_tasks: digest.todos.iter().filter(|todo| !todo.completed).count(),
        completed_tasks: digest.todos.iter().filter(|todo| todo.completed).count(),
        risk_flags,
        warnings,
        ready_for_delete: true,
        session_end_hook: SessionEndHookResult {
            status: "not_configured".to_string(),
            script_path: None,
            report_path: None,
            summary: "No session-end hook was configured for this project.".to_string(),
        },
        generated_at: chrono::Utc::now().to_rfc3339(),
    };

    fs::write(&checklist_path, serde_json::to_string_pretty(&checklist)?)?;

    if let Some(hook_path) = resolve_session_end_hook(session) {
        let hook_result = run_session_end_hook(session, &hook_path, &checklist_path, export_path);
        checklist.ready_for_delete = hook_result.status != "failed";
        checklist.session_end_hook = hook_result.clone();
    }

    fs::write(&checklist_path, serde_json::to_string_pretty(&checklist)?)?;

    Ok(CleanupChecklistWriteResult {
        checklist_path,
        ready_for_delete: checklist.ready_for_delete,
        hook: checklist.session_end_hook,
    })
}

fn build_cleanup_warnings(
    insight: &SessionInsight,
    risk_flags: &[String],
    todos: &[TranscriptTodo],
) -> Vec<String> {
    let mut warnings = Vec::new();

    let open_tasks = todos.iter().filter(|todo| !todo.completed).count();
    if open_tasks > 0 {
        warnings.push(format!("{open_tasks} open tasks still remain in the transcript."));
    }

    if insight.progress_state != "completed" {
        warnings.push(format!(
            "Session progress is still {}, so quarantine should only happen after review.",
            insight.progress_state
        ));
    }

    if !risk_flags.is_empty() {
        warnings.push(format!(
            "Risk flags still need review before cleanup: {}.",
            risk_flags.join(", ")
        ));
    }

    warnings
}

fn resolve_session_end_hook(session: &SessionRecord) -> Option<PathBuf> {
    let project_root = session.project_path.as_deref()?.trim();
    if project_root.is_empty() || project_root.eq_ignore_ascii_case("unknown") {
        return None;
    }

    let project_root = Path::new(project_root);
    let candidates = [
        project_root
            .join(".open-session-manager")
            .join("hooks")
            .join("session-end.ps1"),
        project_root
            .join(".open-session-manager")
            .join("hooks")
            .join("session-end.sh"),
        project_root.join(".osm").join("hooks").join("session-end.ps1"),
        project_root.join(".osm").join("hooks").join("session-end.sh"),
    ];

    candidates.into_iter().find(|path| path.is_file())
}

fn run_session_end_hook(
    session: &SessionRecord,
    hook_path: &Path,
    checklist_path: &Path,
    export_path: &Path,
) -> SessionEndHookResult {
    let report_path = export_path.parent().map(|parent| {
        parent.join(format!(
            "session-end-{}.log",
            safe_managed_name(&session.session_id)
        ))
    });

    let mut command = match hook_path.extension().and_then(|value| value.to_str()) {
        Some("ps1") if cfg!(windows) => {
            let mut command = Command::new("powershell");
            command.arg("-ExecutionPolicy").arg("Bypass").arg("-File").arg(hook_path);
            command
        }
        Some("ps1") => {
            let mut command = Command::new("pwsh");
            command.arg("-File").arg(hook_path);
            command
        }
        _ => {
            let mut command = Command::new("sh");
            command.arg(hook_path);
            command
        }
    };

    command
        .arg(checklist_path)
        .arg(export_path)
        .env("OSM_SESSION_ID", &session.session_id)
        .env("OSM_SESSION_ASSISTANT", &session.assistant)
        .env("OSM_SESSION_SOURCE_PATH", &session.source_path);

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            let message = format!("failed to launch session-end hook: {error}");
            if let Some(report_path) = &report_path {
                let _ = fs::write(report_path, &message);
            }
            return SessionEndHookResult {
                status: "failed".to_string(),
                script_path: Some(hook_path.display().to_string()),
                report_path,
                summary: message,
            };
        }
    };

    let mut report = String::new();
    if !output.stdout.is_empty() {
        report.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        if !report.is_empty() && !report.ends_with('\n') {
            report.push('\n');
        }
        report.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    if let Some(report_path) = &report_path {
        let _ = fs::write(report_path, &report);
    }

    let status = if output.status.success() {
        "success"
    } else {
        "failed"
    };

    let summary = if output.status.success() {
        format!("session-end hook completed for {}.", session.session_id)
    } else {
        format!(
            "session-end hook failed for {} with exit code {:?}.",
            session.session_id,
            output.status.code()
        )
    };

    SessionEndHookResult {
        status: status.to_string(),
        script_path: Some(hook_path.display().to_string()),
        report_path,
        summary,
    }
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
    let handoff_section = render_session_handoff(&transcript_digest, &insight.summary);
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
{handoff_section}\
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
        handoff_section = handoff_section,
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

fn render_session_handoff(digest: &crate::transcript::TranscriptDigest, summary: &str) -> String {
    let next_focus = digest
        .todos
        .iter()
        .find(|todo| !todo.completed)
        .map(|todo| todo.content.as_str())
        .unwrap_or(summary);
    let resume_cue = digest
        .highlights
        .iter()
        .rev()
        .find(|highlight| highlight.role == "Assistant")
        .or_else(|| digest.highlights.last())
        .map(|highlight| highlight.content.as_str())
        .unwrap_or(summary);
    let open_tasks = digest.todos.iter().filter(|todo| !todo.completed).count();
    let completed_tasks = digest.todos.iter().filter(|todo| todo.completed).count();

    format!(
        "## Session Handoff\n\
- Next focus: {}\n\
- Open tasks: {}\n\
- Completed tasks: {}\n\
- Resume cue: {}\n\n",
        compact_markdown_line(next_focus),
        open_tasks,
        completed_tasks,
        compact_markdown_line(resume_cue),
    )
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

fn compact_markdown_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
