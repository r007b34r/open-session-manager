use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    domain::session::SessionRecord,
    storage::sqlite::{
        SessionControlEventRow, SessionControlStateRow, insert_session_control_event,
        load_session_control_state, upsert_session_control_state,
    },
};

use super::{ActionError, ActionResult, AuditWriteRequest, write_audit_event};

const DEFAULT_RESUME_PROMPT: &str =
    "Resume this session and reply with a one-line status summary ending with READY.";
const ATTACH_RESPONSE: &str = "Session attached for follow-up prompts.";
const DETACH_RESPONSE: &str = "Session detached from follow-up prompts.";
const CONTINUE_COOLDOWN_SECONDS: i64 = 10;

pub struct SessionControlRequest<'a> {
    pub session: &'a SessionRecord,
    pub actor: &'a str,
    pub connection: &'a Connection,
    pub prompt: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionControlResult {
    pub session_id: String,
    pub controller: String,
    pub command: String,
    pub prompt: String,
    pub response: String,
    pub attached: bool,
}

#[derive(Debug, Clone)]
struct ResolvedController {
    controller: &'static str,
    command: String,
    working_dir: PathBuf,
}

pub fn resume_session(request: &SessionControlRequest<'_>) -> ActionResult<SessionControlResult> {
    execute_session_control(
        request,
        "resume",
        request.prompt.unwrap_or(DEFAULT_RESUME_PROMPT),
        "session_resume",
    )
}

pub fn continue_session(request: &SessionControlRequest<'_>) -> ActionResult<SessionControlResult> {
    let Some(prompt) = request.prompt.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(ActionError::Precondition(
            "continue prompt must not be empty".to_string(),
        ));
    };

    execute_session_control(request, "continue", prompt, "session_continue")
}

pub fn attach_session(request: &SessionControlRequest<'_>) -> ActionResult<SessionControlResult> {
    update_session_attachment(request, true, "session_attach", ATTACH_RESPONSE)
}

pub fn detach_session(request: &SessionControlRequest<'_>) -> ActionResult<SessionControlResult> {
    update_session_attachment(request, false, "session_detach", DETACH_RESPONSE)
}

fn execute_session_control(
    request: &SessionControlRequest<'_>,
    operation: &str,
    prompt: &str,
    audit_event_type: &str,
) -> ActionResult<SessionControlResult> {
    let controller = resolve_controller(request.session)?;
    if !command_is_available(&controller.command) {
        return Err(ActionError::Execution(format!(
            "assistant command is not available on PATH: {}",
            controller.command
        )));
    }

    let mut state = load_session_control_state(request.connection, &request.session.session_id)?
        .unwrap_or_else(|| default_session_control_state(request.session, controller.controller));

    if operation == "continue" && !state.attached {
        return Err(ActionError::Precondition(
            "continue requires an attached session; attach or resume it first".to_string(),
        ));
    }

    if operation == "continue" {
        ensure_session_is_ready_for_continue(request.session, &state)?;
        ensure_continue_cooldown_elapsed(&state)?;
    }

    let prepared = match controller.controller {
        "codex" => build_codex_command(&controller, request.session, prompt),
        "claude-code" => build_claude_command(&controller, request.session, prompt),
        other => {
            return Err(ActionError::Precondition(format!(
                "session control is not supported for {other}"
            )));
        }
    }?;
    let response = prepared.execute()?;
    let now = Utc::now().to_rfc3339();
    let rendered_command = prepared.render();

    state.assistant = request.session.assistant.clone();
    state.controller = controller.controller.to_string();
    state.available = true;
    state.attached = true;
    state.last_command = Some(rendered_command.clone());
    state.last_prompt = Some(prompt.to_string());
    state.last_response = Some(response.clone());
    state.last_error = None;
    if operation == "resume" {
        state.last_resumed_at = Some(now.clone());
    } else {
        state.last_continued_at = Some(now.clone());
    }

    upsert_session_control_state(request.connection, &state)?;
    insert_session_control_event(
        request.connection,
        &SessionControlEventRow {
            event_id: session_control_event_id(&request.session.session_id, operation, &now),
            session_id: request.session.session_id.clone(),
            operation: operation.to_string(),
            created_at: now.clone(),
            prompt: Some(prompt.to_string()),
            response: Some(response.clone()),
            result: "success".to_string(),
            error_message: None,
            command: Some(rendered_command.clone()),
        },
    )?;
    write_audit_event(
        request.connection,
        AuditWriteRequest {
            event_type: audit_event_type,
            target_type: "session",
            target_id: &request.session.session_id,
            actor: request.actor,
            before_state: None,
            after_state: Some(
                json!({
                    "controller": controller.controller,
                    "command": rendered_command,
                    "prompt": prompt,
                    "response": response,
                })
                .to_string(),
            ),
            result: "success",
        },
    )?;

    Ok(SessionControlResult {
        session_id: request.session.session_id.clone(),
        controller: controller.controller.to_string(),
        command: controller.command,
        prompt: prompt.to_string(),
        response,
        attached: true,
    })
}

fn update_session_attachment(
    request: &SessionControlRequest<'_>,
    attached: bool,
    audit_event_type: &str,
    response: &str,
) -> ActionResult<SessionControlResult> {
    let controller = resolve_controller(request.session)?;
    if !command_is_available(&controller.command) {
        return Err(ActionError::Execution(format!(
            "assistant command is not available on PATH: {}",
            controller.command
        )));
    }

    let now = Utc::now().to_rfc3339();
    let operation = if attached { "attach" } else { "detach" };
    let rendered_command = format!("osm {operation} {}", request.session.session_id);
    let mut state = load_session_control_state(request.connection, &request.session.session_id)?
        .unwrap_or_else(|| default_session_control_state(request.session, controller.controller));

    state.assistant = request.session.assistant.clone();
    state.controller = controller.controller.to_string();
    state.available = true;
    state.attached = attached;
    state.last_command = Some(rendered_command.clone());
    state.last_response = Some(response.to_string());
    state.last_error = None;
    if attached {
        state.last_resumed_at = Some(now.clone());
    }

    upsert_session_control_state(request.connection, &state)?;
    insert_session_control_event(
        request.connection,
        &SessionControlEventRow {
            event_id: session_control_event_id(&request.session.session_id, operation, &now),
            session_id: request.session.session_id.clone(),
            operation: operation.to_string(),
            created_at: now.clone(),
            prompt: None,
            response: Some(response.to_string()),
            result: "success".to_string(),
            error_message: None,
            command: Some(rendered_command.clone()),
        },
    )?;
    write_audit_event(
        request.connection,
        AuditWriteRequest {
            event_type: audit_event_type,
            target_type: "session",
            target_id: &request.session.session_id,
            actor: request.actor,
            before_state: None,
            after_state: Some(
                json!({
                    "controller": controller.controller,
                    "command": rendered_command,
                    "response": response,
                    "attached": attached,
                })
                .to_string(),
            ),
            result: "success",
        },
    )?;

    Ok(SessionControlResult {
        session_id: request.session.session_id.clone(),
        controller: controller.controller.to_string(),
        command: controller.command,
        prompt: String::new(),
        response: response.to_string(),
        attached,
    })
}

fn resolve_controller(session: &SessionRecord) -> ActionResult<ResolvedController> {
    let working_dir = session_working_dir(session);

    match session.assistant.as_str() {
        "codex" => Ok(ResolvedController {
            controller: "codex",
            command: env::var("OPEN_SESSION_MANAGER_CODEX_COMMAND")
                .unwrap_or_else(|_| "codex".to_string()),
            working_dir,
        }),
        "claude-code" => Ok(ResolvedController {
            controller: "claude-code",
            command: env::var("OPEN_SESSION_MANAGER_CLAUDE_CODE_COMMAND")
                .unwrap_or_else(|_| "claude".to_string()),
            working_dir,
        }),
        assistant => Err(ActionError::Precondition(format!(
            "session control is not supported for assistant `{assistant}`"
        ))),
    }
}

fn default_session_control_state(
    session: &SessionRecord,
    controller: &str,
) -> SessionControlStateRow {
    SessionControlStateRow {
        session_id: session.session_id.clone(),
        assistant: session.assistant.clone(),
        controller: controller.to_string(),
        available: true,
        attached: false,
        last_command: None,
        last_prompt: None,
        last_response: None,
        last_error: None,
        last_resumed_at: None,
        last_continued_at: None,
    }
}

fn ensure_session_is_ready_for_continue(
    session: &SessionRecord,
    state: &SessionControlStateRow,
) -> ActionResult<()> {
    if session_is_busy_for_continue(session, state) {
        return Err(ActionError::Precondition(
            "continue is blocked while the session is busy; wait for READY or idle before sending another prompt"
                .to_string(),
        ));
    }

    Ok(())
}

fn ensure_continue_cooldown_elapsed(state: &SessionControlStateRow) -> ActionResult<()> {
    let Some(last_continued_at) = state.last_continued_at.as_deref() else {
        return Ok(());
    };

    let Some(last_continued_at) = parse_control_timestamp(last_continued_at) else {
        return Ok(());
    };

    if Utc::now()
        .signed_duration_since(last_continued_at)
        .num_seconds()
        < CONTINUE_COOLDOWN_SECONDS
    {
        return Err(ActionError::Precondition(
            "continue is cooling down; wait a moment before sending another prompt"
                .to_string(),
        ));
    }

    Ok(())
}

fn session_is_busy_for_continue(
    session: &SessionRecord,
    state: &SessionControlStateRow,
) -> bool {
    if !state.attached {
        return false;
    }

    if session.ended_at.is_some() || matches_terminal_status(&session.status) {
        return false;
    }

    if state
        .last_error
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        return false;
    }

    if state
        .last_response
        .as_deref()
        .is_some_and(looks_like_ready_response)
    {
        return false;
    }

    true
}

fn parse_control_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.with_timezone(&Utc))
}

fn looks_like_ready_response(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("ready")
        || lowered.contains("awaiting")
        || lowered.contains("waiting for")
}

fn matches_terminal_status(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "completed" | "done" | "finished" | "failed" | "exited" | "stopped"
    )
}

fn session_working_dir(session: &SessionRecord) -> PathBuf {
    if let Some(project_path) = session
        .project_path
        .as_deref()
        .filter(|value| !value.trim().is_empty() && *value != "unknown")
    {
        let path = PathBuf::from(project_path);
        if path.exists() {
            return path;
        }
    }

    Path::new(&session.source_path)
        .parent()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn command_is_available(command: &str) -> bool {
    if command.contains(std::path::MAIN_SEPARATOR) || command.contains('/') || command.contains('\\')
    {
        return Path::new(command).exists();
    }

    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    let path_exts = if cfg!(windows) {
        env::var_os("PATHEXT")
            .and_then(|value| value.into_string().ok())
            .map(|value| value.split(';').map(|entry| entry.to_string()).collect())
            .unwrap_or_else(|| {
                vec![
                    ".COM".to_string(),
                    ".EXE".to_string(),
                    ".BAT".to_string(),
                    ".CMD".to_string(),
                ]
            })
    } else {
        vec![String::new()]
    };

    env::split_paths(&path_var).any(|dir| {
        if cfg!(windows) {
            if dir.join(command).exists() {
                return true;
            }
        }

        path_exts
            .iter()
            .map(|ext| dir.join(format!("{command}{ext}")))
            .any(|candidate| candidate.exists())
    })
}

#[derive(Debug)]
struct PreparedCommand {
    command: String,
    args: Vec<String>,
    working_dir: PathBuf,
    output_file: Option<PathBuf>,
}

impl PreparedCommand {
    fn execute(&self) -> ActionResult<String> {
        let output = Command::new(&self.command)
            .args(&self.args)
            .current_dir(&self.working_dir)
            .output()
            .map_err(|error| ActionError::Execution(error.to_string()))?;

        if !output.status.success() {
            return Err(ActionError::Execution(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        if let Some(path) = &self.output_file {
            let content = fs::read_to_string(path)?;
            if !content.trim().is_empty() {
                return Ok(content.trim().to_string());
            }
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            Ok("OK".to_string())
        } else {
            Ok(stdout)
        }
    }

    fn render(&self) -> String {
        std::iter::once(self.command.clone())
            .chain(self.args.iter().cloned())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn build_codex_command(
    controller: &ResolvedController,
    session: &SessionRecord,
    prompt: &str,
) -> ActionResult<PreparedCommand> {
    let output_file = env::temp_dir().join(format!(
        "osm-codex-session-control-{}.txt",
        session.session_id
    ));

    Ok(PreparedCommand {
        command: controller.command.clone(),
        args: vec![
            "-C".to_string(),
            controller.working_dir.display().to_string(),
            "exec".to_string(),
            "resume".to_string(),
            session.session_id.clone(),
            prompt.to_string(),
            "--skip-git-repo-check".to_string(),
            "-o".to_string(),
            output_file.display().to_string(),
        ],
        working_dir: controller.working_dir.clone(),
        output_file: Some(output_file),
    })
}

fn build_claude_command(
    controller: &ResolvedController,
    session: &SessionRecord,
    prompt: &str,
) -> ActionResult<PreparedCommand> {
    Ok(PreparedCommand {
        command: controller.command.clone(),
        args: vec![
            "-p".to_string(),
            "-r".to_string(),
            session.session_id.clone(),
            prompt.to_string(),
        ],
        working_dir: controller.working_dir.clone(),
        output_file: None,
    })
}

fn session_control_event_id(session_id: &str, operation: &str, created_at: &str) -> String {
    let payload = format!("{session_id}:{operation}:{created_at}");
    let digest = Sha256::digest(payload.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
