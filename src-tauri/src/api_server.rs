use std::{
    collections::{BTreeMap, HashMap},
    env,
    net::SocketAddr,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{
        HeaderMap, StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    AppState,
    actions::ActionError,
    commands::{
        actions::{
            attach_existing_session as attach_existing_session_action,
            continue_existing_session as continue_existing_session_action,
            detach_existing_session as detach_existing_session_action,
            pause_existing_session as pause_existing_session_action,
            resume_existing_session as resume_existing_session_action,
        },
        dashboard::{
            DashboardSnapshot, build_fixture_dashboard_snapshot_with_audit,
            build_local_dashboard_snapshot_with_audit, build_local_indexed_sessions,
        },
        query::{
            ListSessionInventoryRequest, SearchSessionInventoryRequest, expand_session,
            get_session, list_sessions_with_request, search_sessions_with_request, view_session,
        },
    },
    discovery::DiscoveryContext,
    openapi::openapi_document,
    preferences::build_runtime_paths,
    storage::sqlite::open_database,
};

#[derive(Debug, Clone)]
struct ApiState {
    app: AppState,
    fixtures_path: Option<PathBuf>,
    audit_db_path: Option<PathBuf>,
    api_token: Option<String>,
    automation_tasks: Arc<Mutex<HashMap<String, AutomationTaskReceipt>>>,
    next_automation_task_id: Arc<AtomicU64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServeConfig {
    host: Option<String>,
    port: Option<u16>,
    fixtures_path: Option<PathBuf>,
    audit_db_path: Option<PathBuf>,
    api_token: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQuery {
    assistant: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    sort_by: Option<String>,
    descending: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchQuery {
    query: String,
    assistant: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    sort_by: Option<String>,
    descending: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: &'static str,
    app_name: &'static str,
    version: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContinueSessionRequest {
    prompt: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AutomationTaskRequest {
    kind: String,
    session_id: Option<String>,
    prompt: Option<String>,
    query: Option<String>,
    assistant: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    sort_by: Option<String>,
    descending: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AutomationTaskReceipt {
    task_id: String,
    kind: String,
    status: String,
    submitted_at: String,
    completed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub fn run(args: &[String]) -> Result<(), String> {
    let config = parse_config(args)?;
    let host = config.host.unwrap_or_else(|| "127.0.0.1".to_string());
    let port = config.port.unwrap_or(43210);
    let address = format!("{host}:{port}")
        .parse::<SocketAddr>()
        .map_err(|error| format!("invalid listen address: {error}"))?;

    let state = ApiState {
        app: AppState::default(),
        fixtures_path: config.fixtures_path,
        audit_db_path: config.audit_db_path,
        api_token: config.api_token,
        automation_tasks: Arc::new(Mutex::new(HashMap::new())),
        next_automation_task_id: Arc::new(AtomicU64::new(1)),
    };

    let runtime = tokio::runtime::Runtime::new().map_err(|error| error.to_string())?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(address)
            .await
            .map_err(|error| error.to_string())?;
        axum::serve(listener, router(state))
            .await
            .map_err(|error| error.to_string())
    })
}

fn router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/openapi.json", get(openapi))
        .route("/api/v1/automation/tasks", post(create_automation_task))
        .route(
            "/api/v1/automation/tasks/{taskId}",
            get(get_automation_task),
        )
        .route("/api/v1/sessions", get(list_sessions))
        .route("/api/v1/sessions/search", get(search_sessions))
        .route("/api/v1/sessions/{sessionId}", get(get_session_detail))
        .route(
            "/api/v1/sessions/{sessionId}/view",
            get(view_session_detail),
        )
        .route(
            "/api/v1/sessions/{sessionId}/expand",
            get(expand_session_detail),
        )
        .route(
            "/api/v1/sessions/{sessionId}/resume",
            post(resume_session_control),
        )
        .route(
            "/api/v1/sessions/{sessionId}/pause",
            post(pause_session_control),
        )
        .route(
            "/api/v1/sessions/{sessionId}/attach",
            post(attach_session_control),
        )
        .route(
            "/api/v1/sessions/{sessionId}/detach",
            post(detach_session_control),
        )
        .route(
            "/api/v1/sessions/{sessionId}/continue",
            post(continue_session_control),
        )
        .with_state(state)
}

async fn health(State(state): State<ApiState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        app_name: state.app.app_name,
        version: state.app.version,
    })
}

async fn metrics(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    authorize(&state, &headers)?;
    let snapshot = load_snapshot_data(&state)?;
    Ok((
        [(CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        render_prometheus_metrics(&snapshot),
    ))
}

async fn openapi(State(state): State<ApiState>) -> Json<Value> {
    Json(openapi_document(&state.app))
}

async fn create_automation_task(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(request): Json<AutomationTaskRequest>,
) -> Result<Json<AutomationTaskReceipt>, ApiError> {
    authorize(&state, &headers)?;

    let receipt = execute_automation_task(&state, request);
    state
        .automation_tasks
        .lock()
        .map_err(|_| ApiError::internal("automation task store is poisoned"))?
        .insert(receipt.task_id.clone(), receipt.clone());

    Ok(Json(receipt))
}

async fn get_automation_task(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(task_id): Path<String>,
) -> Result<Json<AutomationTaskReceipt>, ApiError> {
    authorize(&state, &headers)?;
    let receipt = state
        .automation_tasks
        .lock()
        .map_err(|_| ApiError::internal("automation task store is poisoned"))?
        .get(&task_id)
        .cloned()
        .ok_or_else(|| ApiError::not_found(format!("automation task not found: {task_id}")))?;

    Ok(Json(receipt))
}

async fn list_sessions(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, ApiError> {
    authorize(&state, &headers)?;
    let snapshot = load_snapshot_data(&state)?;
    let request = ListSessionInventoryRequest {
        assistant: query.assistant,
        limit: query.limit,
        offset: query.offset,
        sort_by: query.sort_by,
        descending: query.descending,
    };

    Ok(Json(list_sessions_with_request(&snapshot, Some(&request))))
}

async fn search_sessions(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Value>, ApiError> {
    authorize(&state, &headers)?;
    let snapshot = load_snapshot_data(&state)?;
    let request = SearchSessionInventoryRequest {
        query: query.query,
        assistant: query.assistant,
        limit: query.limit,
        offset: query.offset,
        sort_by: query.sort_by,
        descending: query.descending,
    };

    Ok(Json(search_sessions_with_request(&snapshot, &request)))
}

async fn get_session_detail(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorize(&state, &headers)?;
    let snapshot = load_snapshot_data(&state)?;
    get_session(&snapshot, &session_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("session not found: {session_id}")))
}

async fn view_session_detail(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorize(&state, &headers)?;
    let snapshot = load_snapshot_data(&state)?;
    view_session(&snapshot, &session_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("session not found: {session_id}")))
}

async fn expand_session_detail(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorize(&state, &headers)?;
    let snapshot = load_snapshot_data(&state)?;
    expand_session(&snapshot, &session_id)
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("session not found: {session_id}")))
}

async fn resume_session_control(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorize(&state, &headers)?;
    Ok(Json(execute_session_control_action(
        &state,
        &session_id,
        |session, actor, connection| resume_existing_session_action(session, actor, connection),
    )?))
}

async fn pause_session_control(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorize(&state, &headers)?;
    Ok(Json(execute_session_control_action(
        &state,
        &session_id,
        |session, actor, connection| pause_existing_session_action(session, actor, connection),
    )?))
}

async fn attach_session_control(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorize(&state, &headers)?;
    Ok(Json(execute_session_control_action(
        &state,
        &session_id,
        |session, actor, connection| attach_existing_session_action(session, actor, connection),
    )?))
}

async fn detach_session_control(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    authorize(&state, &headers)?;
    Ok(Json(execute_session_control_action(
        &state,
        &session_id,
        |session, actor, connection| detach_existing_session_action(session, actor, connection),
    )?))
}

async fn continue_session_control(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Json(request): Json<ContinueSessionRequest>,
) -> Result<Json<Value>, ApiError> {
    authorize(&state, &headers)?;
    Ok(Json(execute_session_control_action(
        &state,
        &session_id,
        |session, actor, connection| {
            continue_existing_session_action(session, &request.prompt, actor, connection)
        },
    )?))
}

fn load_snapshot_data(state: &ApiState) -> Result<DashboardSnapshot, ApiError> {
    if let Some(fixtures_path) = state.fixtures_path.as_ref() {
        let mut snapshot = build_fixture_dashboard_snapshot_with_audit(
            fixtures_path,
            state.audit_db_path.as_deref(),
        )
        .map_err(ApiError::internal)?;
        if let Ok(runtime) = build_runtime_paths() {
            snapshot.runtime = runtime.snapshot();
        }
        return Ok(snapshot);
    }

    let mut snapshot = build_local_dashboard_snapshot_with_audit(
        &build_discovery_context(),
        state.audit_db_path.as_deref(),
    )
    .map_err(ApiError::internal)?;
    if let Ok(mut runtime) = build_runtime_paths() {
        if let Some(custom_audit_db_path) = state.audit_db_path.as_ref() {
            runtime.audit_db_path = custom_audit_db_path.clone();
        }
        snapshot.runtime = runtime.snapshot();
    }
    Ok(snapshot)
}

fn build_discovery_context() -> DiscoveryContext {
    DiscoveryContext {
        home_dir: resolve_home_dir(),
        xdg_config_home: env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        xdg_data_home: env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        wsl_home_dir: env::var_os("OPEN_SESSION_MANAGER_WSL_HOME").map(PathBuf::from),
    }
}

fn execute_session_control_action<F>(
    state: &ApiState,
    session_id: &str,
    action: F,
) -> Result<Value, ApiError>
where
    F: FnOnce(
        &crate::domain::session::SessionRecord,
        &str,
        &rusqlite::Connection,
    ) -> crate::actions::ActionResult<
        crate::actions::session_control::SessionControlResult,
    >,
{
    if state.fixtures_path.is_some() {
        return Err(ApiError::bad_request(
            "session control API is unavailable while serving fixture snapshots".to_string(),
        ));
    }

    let context = build_discovery_context();
    let indexed = build_local_indexed_sessions(&context)
        .map_err(ApiError::internal)?
        .into_iter()
        .find(|indexed| indexed.session.session_id == session_id)
        .ok_or_else(|| ApiError::not_found(format!("session not found: {session_id}")))?;

    let audit_db_path = control_audit_db_path(state)?;
    let connection = open_database(&audit_db_path).map_err(ApiError::internal)?;
    action(&indexed.session, &resolve_actor(), &connection).map_err(map_action_error)?;

    let snapshot = build_local_dashboard_snapshot_with_audit(&context, Some(&audit_db_path))
        .map_err(ApiError::internal)?;
    get_session(&snapshot, session_id)
        .ok_or_else(|| ApiError::not_found(format!("session not found: {session_id}")))
}

fn resolve_home_dir() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("current dir resolves"))
}

fn resolve_actor() -> String {
    env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "api-server".to_string())
}

fn parse_config(args: &[String]) -> Result<ServeConfig, String> {
    let host = parse_flag_value(args, "--host").map(ToString::to_string);
    let port = parse_flag_value(args, "--port")
        .map(|value| value.parse::<u16>().map_err(|error| error.to_string()))
        .transpose()?;
    let fixtures_path = parse_flag_value(args, "--fixtures").map(PathBuf::from);
    let audit_db_path = parse_flag_value(args, "--audit-db").map(PathBuf::from);
    let api_token = parse_flag_value(args, "--api-token")
        .map(ToString::to_string)
        .or_else(|| env::var("OPEN_SESSION_MANAGER_API_TOKEN").ok());

    Ok(ServeConfig {
        host,
        port,
        fixtures_path,
        audit_db_path,
        api_token,
    })
}

fn parse_flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find_map(|window| (window[0] == flag).then_some(window[1].as_str()))
}

fn control_audit_db_path(state: &ApiState) -> Result<PathBuf, ApiError> {
    let mut runtime_paths = build_runtime_paths().map_err(ApiError::internal)?;
    if let Some(custom_audit_db_path) = state.audit_db_path.as_ref() {
        runtime_paths.audit_db_path = custom_audit_db_path.clone();
    }

    Ok(runtime_paths.audit_db_path)
}

fn authorize(state: &ApiState, headers: &HeaderMap) -> Result<(), ApiError> {
    let Some(expected_token) = state.api_token.as_deref() else {
        return Ok(());
    };

    let provided = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    if provided == Some(expected_token) {
        return Ok(());
    }

    Err(ApiError {
        status: StatusCode::UNAUTHORIZED,
        message: "missing or invalid bearer token".to_string(),
    })
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn internal(message: impl ToString) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.to_string(),
        }
    }

    fn not_found(message: impl ToString) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    fn bad_request(message: impl ToString) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.to_string(),
        }
    }

    fn conflict(message: impl ToString) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(json!({
                "error": self.message
            })),
        )
            .into_response()
    }
}

fn map_action_error(error: ActionError) -> ApiError {
    match error {
        ActionError::Precondition(message) => ApiError::conflict(message),
        ActionError::Execution(message) => ApiError::conflict(message),
        other => ApiError::internal(other),
    }
}

fn render_prometheus_metrics(snapshot: &DashboardSnapshot) -> String {
    let mut body = String::new();

    let session_count = snapshot.sessions.len() as u64;
    let config_count = snapshot.configs.len() as u64;
    let git_project_count = snapshot.git_projects.len() as u64;
    let doctor_finding_count = snapshot.doctor_findings.len() as u64;
    let audit_event_count = snapshot.audit_events.len() as u64;
    let session_control_supported_count = snapshot
        .sessions
        .iter()
        .filter(|session| {
            session
                .session_control
                .as_ref()
                .is_some_and(|control| control.supported)
        })
        .count() as u64;
    let session_control_available_count = snapshot
        .sessions
        .iter()
        .filter(|session| {
            session
                .session_control
                .as_ref()
                .is_some_and(|control| control.available)
        })
        .count() as u64;

    append_metric_help(
        &mut body,
        "osm_sessions_total",
        "Total discovered sessions.",
    );
    append_metric(&mut body, "osm_sessions_total", session_count);

    append_metric_help(
        &mut body,
        "osm_session_control_supported_total",
        "Total sessions with a supported control adapter.",
    );
    append_metric(
        &mut body,
        "osm_session_control_supported_total",
        session_control_supported_count,
    );

    append_metric_help(
        &mut body,
        "osm_session_control_available_total",
        "Total sessions with a control command available on this host.",
    );
    append_metric(
        &mut body,
        "osm_session_control_available_total",
        session_control_available_count,
    );

    append_metric_help(
        &mut body,
        "osm_configs_total",
        "Total discovered config artifacts.",
    );
    append_metric(&mut body, "osm_configs_total", config_count);

    append_metric_help(
        &mut body,
        "osm_git_projects_total",
        "Total git projects in snapshot.",
    );
    append_metric(&mut body, "osm_git_projects_total", git_project_count);

    append_metric_help(
        &mut body,
        "osm_doctor_findings_total",
        "Total doctor findings in snapshot.",
    );
    append_metric(&mut body, "osm_doctor_findings_total", doctor_finding_count);

    append_metric_help(
        &mut body,
        "osm_audit_events_total",
        "Total audit events included in snapshot.",
    );
    append_metric(&mut body, "osm_audit_events_total", audit_event_count);

    append_labeled_counts(
        &mut body,
        "osm_sessions_by_assistant",
        "Discovered sessions grouped by assistant.",
        snapshot
            .sessions
            .iter()
            .map(|session| session.assistant.as_str()),
    );
    append_labeled_counts(
        &mut body,
        "osm_configs_by_assistant",
        "Discovered configs grouped by assistant.",
        snapshot
            .configs
            .iter()
            .map(|config| config.assistant.as_str()),
    );

    body
}

fn append_metric_help(body: &mut String, name: &str, description: &str) {
    body.push_str("# HELP ");
    body.push_str(name);
    body.push(' ');
    body.push_str(description);
    body.push('\n');
    body.push_str("# TYPE ");
    body.push_str(name);
    body.push_str(" gauge\n");
}

fn append_metric(body: &mut String, name: &str, value: u64) {
    body.push_str(name);
    body.push(' ');
    body.push_str(&value.to_string());
    body.push('\n');
}

fn append_labeled_counts<'a>(
    body: &mut String,
    name: &str,
    description: &str,
    values: impl Iterator<Item = &'a str>,
) {
    let mut counts = BTreeMap::<String, u64>::new();
    for value in values {
        *counts.entry(value.to_string()).or_default() += 1;
    }

    append_metric_help(body, name, description);
    for (label, value) in counts {
        body.push_str(name);
        body.push_str("{assistant=\"");
        body.push_str(&escape_prometheus_label(&label));
        body.push_str("\"} ");
        body.push_str(&value.to_string());
        body.push('\n');
    }
}

fn escape_prometheus_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('"', "\\\"")
}

fn execute_automation_task(
    state: &ApiState,
    request: AutomationTaskRequest,
) -> AutomationTaskReceipt {
    let task_id = format!(
        "task-{:06}",
        state
            .next_automation_task_id
            .fetch_add(1, Ordering::Relaxed)
    );
    let submitted_at = Utc::now().to_rfc3339();
    let kind = request.kind.clone();

    let execution = match kind.as_str() {
        "snapshot.refresh" => load_snapshot_data(state)
            .and_then(|snapshot| serde_json::to_value(snapshot).map_err(ApiError::internal)),
        "sessions.search" => {
            let Some(query) = request.query else {
                return AutomationTaskReceipt {
                    task_id,
                    kind,
                    status: "failed".to_string(),
                    submitted_at: submitted_at.clone(),
                    completed_at: submitted_at,
                    result: None,
                    error: Some("automation task `sessions.search` requires `query`".to_string()),
                };
            };

            load_snapshot_data(state).map(|snapshot| {
                search_sessions_with_request(
                    &snapshot,
                    &SearchSessionInventoryRequest {
                        query,
                        assistant: request.assistant,
                        limit: request.limit,
                        offset: request.offset,
                        sort_by: request.sort_by,
                        descending: request.descending,
                    },
                )
            })
        }
        "sessions.resume" => {
            let Some(session_id) = request.session_id else {
                return AutomationTaskReceipt {
                    task_id,
                    kind,
                    status: "failed".to_string(),
                    submitted_at: submitted_at.clone(),
                    completed_at: submitted_at,
                    result: None,
                    error: Some(
                        "automation task `sessions.resume` requires `sessionId`".to_string(),
                    ),
                };
            };

            execute_session_control_action(state, &session_id, |session, actor, connection| {
                resume_existing_session_action(session, actor, connection)
            })
        }
        "sessions.continue" => {
            let Some(session_id) = request.session_id else {
                return AutomationTaskReceipt {
                    task_id,
                    kind,
                    status: "failed".to_string(),
                    submitted_at: submitted_at.clone(),
                    completed_at: submitted_at,
                    result: None,
                    error: Some(
                        "automation task `sessions.continue` requires `sessionId`".to_string(),
                    ),
                };
            };
            let Some(prompt) = request.prompt else {
                return AutomationTaskReceipt {
                    task_id,
                    kind,
                    status: "failed".to_string(),
                    submitted_at: submitted_at.clone(),
                    completed_at: submitted_at,
                    result: None,
                    error: Some(
                        "automation task `sessions.continue` requires `prompt`".to_string(),
                    ),
                };
            };

            execute_session_control_action(state, &session_id, |session, actor, connection| {
                continue_existing_session_action(session, &prompt, actor, connection)
            })
        }
        other => Err(ApiError::bad_request(format!(
            "unsupported automation task kind: {other}"
        ))),
    };

    let completed_at = Utc::now().to_rfc3339();
    match execution {
        Ok(result) => AutomationTaskReceipt {
            task_id,
            kind,
            status: "completed".to_string(),
            submitted_at,
            completed_at,
            result: Some(result),
            error: None,
        },
        Err(error) => AutomationTaskReceipt {
            task_id,
            kind,
            status: "failed".to_string(),
            submitted_at,
            completed_at,
            result: None,
            error: Some(error.message),
        },
    }
}
