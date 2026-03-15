use std::{collections::BTreeSet, fmt, fs, io, path::Path};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::{
    adapters::{
        claude_code::ClaudeCodeAdapter,
        codex::CodexAdapter,
        opencode::OpenCodeAdapter,
        traits::{AdapterError, SessionAdapter, collect_files},
    },
    audit::{
        config_audit::{AuditError, ConfigAuditTarget, audit_config},
        credential_audit::build_credential_artifacts,
    },
    commands::{
        configs::discover_known_config_targets,
        discovery::discover_known_session_roots,
    },
    discovery::{DiscoveryContext, KnownPath},
    domain::audit::AuditEvent,
    domain::session::SessionRecord,
    insights::{
        InsightInput, garbage::derive_garbage_score, progress::derive_progress_state,
        title::derive_title, value::derive_value_score,
    },
    storage::sqlite::{load_audit_events, open_database},
};

#[derive(Debug)]
pub enum SnapshotError {
    Adapter(AdapterError),
    Audit(AuditError),
    Io(io::Error),
    Json(serde_json::Error),
    Sql(rusqlite::Error),
    UnsupportedAssistant(String),
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adapter(error) => write!(f, "adapter error: {error}"),
            Self::Audit(error) => write!(f, "audit error: {error}"),
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
            Self::Sql(error) => write!(f, "sqlite error: {error}"),
            Self::UnsupportedAssistant(assistant) => {
                write!(f, "unsupported assistant: {assistant}")
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

impl From<AdapterError> for SnapshotError {
    fn from(value: AdapterError) -> Self {
        Self::Adapter(value)
    }
}

impl From<AuditError> for SnapshotError {
    fn from(value: AuditError) -> Self {
        Self::Audit(value)
    }
}

impl From<io::Error> for SnapshotError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for SnapshotError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<rusqlite::Error> for SnapshotError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sql(value)
    }
}

pub type SnapshotResult<T> = Result<T, SnapshotError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSnapshot {
    pub metrics: Vec<DashboardMetric>,
    pub sessions: Vec<SessionDetailRecord>,
    pub configs: Vec<ConfigRiskRecord>,
    pub audit_events: Vec<AuditEventRecord>,
}

#[derive(Debug, Serialize)]
pub struct DashboardMetric {
    pub label: String,
    pub value: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetailRecord {
    pub session_id: String,
    pub title: String,
    pub assistant: String,
    pub progress_state: String,
    pub progress_percent: u8,
    pub last_activity_at: String,
    pub environment: String,
    pub value_score: u8,
    pub summary: String,
    pub project_path: String,
    pub source_path: String,
    pub tags: Vec<String>,
    pub risk_flags: Vec<String>,
    pub key_artifacts: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRiskRecord {
    pub artifact_id: String,
    pub assistant: String,
    pub scope: String,
    pub path: String,
    pub provider: String,
    pub base_url: String,
    pub masked_secret: String,
    pub official_or_proxy: String,
    pub risks: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEventRecord {
    pub event_id: String,
    pub r#type: String,
    pub target: String,
    pub actor: String,
    pub created_at: String,
    pub result: String,
    pub detail: String,
}

#[derive(Debug, Default)]
struct SessionNarrative {
    first_user_goal: Option<String>,
    last_assistant_message: Option<String>,
    error_count: u32,
}

pub fn build_fixture_dashboard_snapshot(fixtures_root: &Path) -> SnapshotResult<DashboardSnapshot> {
    build_fixture_dashboard_snapshot_with_audit(fixtures_root, None)
}

pub fn build_fixture_dashboard_snapshot_with_audit(
    fixtures_root: &Path,
    audit_db_path: Option<&Path>,
) -> SnapshotResult<DashboardSnapshot> {
    let session_roots = vec![
        KnownPath::new("codex", "session", "windows", fixtures_root.join("codex")),
        KnownPath::new(
            "claude-code",
            "session",
            "windows",
            fixtures_root.join("claude"),
        ),
        KnownPath::new("opencode", "session", "linux", fixtures_root.join("opencode")),
    ];

    let config_targets = vec![
        ConfigAuditTarget::new(
            "codex",
            "global",
            "fixtures",
            fixtures_root.join("configs").join("codex").join("config.toml"),
        ),
        ConfigAuditTarget::new(
            "claude-code",
            "global",
            "fixtures",
            fixtures_root.join("configs").join("claude").join("settings.json"),
        ),
        ConfigAuditTarget::new(
            "opencode",
            "global",
            "fixtures",
            fixtures_root.join("configs").join("opencode").join("opencode.json"),
        ),
    ];

    build_snapshot(session_roots, config_targets, audit_db_path)
}

pub fn build_local_dashboard_snapshot(
    context: &DiscoveryContext,
) -> SnapshotResult<DashboardSnapshot> {
    build_local_dashboard_snapshot_with_audit(context, None)
}

pub fn build_local_dashboard_snapshot_with_audit(
    context: &DiscoveryContext,
    audit_db_path: Option<&Path>,
) -> SnapshotResult<DashboardSnapshot> {
    build_snapshot(
        discover_known_session_roots(context),
        discover_known_config_targets(context),
        audit_db_path,
    )
}

fn build_snapshot(
    session_roots: Vec<KnownPath>,
    config_targets: Vec<ConfigAuditTarget>,
    audit_db_path: Option<&Path>,
) -> SnapshotResult<DashboardSnapshot> {
    let sessions = build_session_records(&session_roots)?;
    let configs = build_config_records(&config_targets)?;
    let audit_events = build_audit_records(audit_db_path)?;

    Ok(DashboardSnapshot {
        metrics: build_metrics(&sessions, &configs),
        sessions,
        configs,
        audit_events,
    })
}

fn build_session_records(session_roots: &[KnownPath]) -> SnapshotResult<Vec<SessionDetailRecord>> {
    let mut sessions = Vec::new();

    for root in session_roots {
        if !root.path.exists() {
            continue;
        }

        let adapter = session_adapter(&root.assistant)?;
        let discovered = adapter.discover_session_files(&root.path)?;

        for session_file in discovered {
            let mut session = adapter.parse_session(&session_file)?;
            session.environment = root.environment.clone();
            let narrative = extract_session_narrative(&session)?;
            sessions.push(build_session_detail(session, narrative));
        }
    }

    Ok(sessions)
}

fn build_session_detail(
    session: SessionRecord,
    narrative: SessionNarrative,
) -> SessionDetailRecord {
    let input = InsightInput {
        first_user_goal: narrative.first_user_goal.as_deref(),
        last_assistant_message: narrative.last_assistant_message.as_deref(),
        message_count: session.message_count,
        tool_count: session.tool_count,
        error_count: narrative.error_count,
        last_activity_at: session.last_activity_at.as_deref(),
    };
    let title = derive_title(&input);
    let (progress_state, progress_percent) = derive_progress_state(&input);
    let value_score = derive_value_score(&input);
    let garbage_score = derive_garbage_score(&input);
    let stale_score = derive_stale_score(session.last_activity_at.as_deref());
    let summary = narrative
        .last_assistant_message
        .clone()
        .or_else(|| narrative.first_user_goal.clone())
        .unwrap_or_else(|| format!("Indexed transcript from {}", session.source_path));
    let tags = build_tags(&session, &title);
    let risk_flags = build_session_risk_flags(
        progress_state,
        narrative.error_count,
        stale_score,
        garbage_score,
    );
    let key_artifacts = build_session_key_artifacts(&session, &narrative);
    let project_path = session
        .project_path
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let source_path = session.source_path.clone();

    SessionDetailRecord {
        session_id: session.session_id,
        title: title.clone(),
        assistant: session.assistant,
        progress_state: progress_state.to_string(),
        progress_percent: progress_percent.unwrap_or(0),
        last_activity_at: session.last_activity_at.unwrap_or_else(|| "unknown".to_string()),
        environment: session.environment,
        value_score,
        summary,
        project_path,
        source_path,
        tags,
        risk_flags,
        key_artifacts,
    }
}

fn build_tags(session: &SessionRecord, title: &str) -> Vec<String> {
    let mut tags = BTreeSet::new();
    tags.insert(session.assistant.clone());
    tags.insert(session.environment.clone());

    if let Some(project) = session.project_path.as_deref().and_then(last_path_segment) {
        tags.insert(project.to_string());
    }

    for token in title
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| token.len() >= 4)
        .take(2)
    {
        tags.insert(token.to_ascii_lowercase());
    }

    tags.into_iter().collect()
}

fn build_session_risk_flags(
    progress_state: &str,
    error_count: u32,
    stale_score: u8,
    garbage_score: u8,
) -> Vec<String> {
    let mut flags = Vec::new();

    if progress_state == "blocked" {
        flags.push("blocked_session".to_string());
    }

    if error_count > 0 {
        flags.push("error_detected".to_string());
    }

    if stale_score >= 70 {
        flags.push("stale_session".to_string());
    }

    if garbage_score >= 70 {
        flags.push("likely_garbage".to_string());
    }

    flags
}

fn build_session_key_artifacts(
    session: &SessionRecord,
    narrative: &SessionNarrative,
) -> Vec<String> {
    let mut artifacts = Vec::new();

    if let Some(goal) = &narrative.first_user_goal {
        artifacts.push(format!("First goal: {goal}"));
    }

    if let Some(message) = &narrative.last_assistant_message {
        artifacts.push(format!("Latest assistant note: {message}"));
    }

    artifacts.push(format!("Transcript path: {}", session.source_path));
    artifacts.push(format!("Messages indexed: {}", session.message_count));

    if session.tool_count > 0 {
        artifacts.push(format!("Tool calls indexed: {}", session.tool_count));
    }

    artifacts
}

fn build_config_records(
    config_targets: &[ConfigAuditTarget],
) -> SnapshotResult<Vec<ConfigRiskRecord>> {
    let mut configs = Vec::new();

    for target in config_targets {
        if !target.path.exists() {
            continue;
        }

        let audit = audit_config(target)?;
        let credentials = build_credential_artifacts(&audit.secrets);
        let primary_credential = credentials.first();
        let provider = audit
            .config
            .provider
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let base_url = audit
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "not_configured".to_string());

        configs.push(ConfigRiskRecord {
            artifact_id: audit.config.artifact_id,
            assistant: audit.config.assistant,
            scope: normalize_scope(&audit.config.scope),
            path: audit.config.path,
            provider,
            base_url,
            masked_secret: primary_credential
                .map(|credential| credential.masked_value.clone())
                .unwrap_or_else(|| "not_detected".to_string()),
            official_or_proxy: primary_credential
                .map(|credential| credential.official_or_proxy.clone())
                .unwrap_or_else(|| infer_proxy_mode(&audit.risk_flags)),
            risks: audit
                .risk_flags
                .into_iter()
                .map(|risk| risk.code)
                .collect::<Vec<_>>(),
        });
    }

    Ok(configs)
}

fn build_metrics(
    sessions: &[SessionDetailRecord],
    configs: &[ConfigRiskRecord],
) -> Vec<DashboardMetric> {
    let high_value_count = sessions.iter().filter(|session| session.value_score >= 70).count();
    let risky_config_count = configs
        .iter()
        .filter(|config| config.official_or_proxy == "proxy" || !config.risks.is_empty())
        .count();
    let reclaim_bytes = sessions
        .iter()
        .filter(|session| session.risk_flags.iter().any(|flag| flag == "likely_garbage"))
        .filter_map(|session| fs::metadata(&session.source_path).ok())
        .map(|metadata| metadata.len())
        .sum::<u64>();

    vec![
        DashboardMetric {
            label: "indexed_sessions".to_string(),
            value: sessions.len().to_string(),
            note: "across_windows_linux_and_wsl_surfaces".to_string(),
        },
        DashboardMetric {
            label: "high_value_candidates".to_string(),
            value: high_value_count.to_string(),
            note: "worth_exporting_before_cleanup".to_string(),
        },
        DashboardMetric {
            label: "risky_configs".to_string(),
            value: risky_config_count.to_string(),
            note: "relay_wide_permissions_or_shell_hooks".to_string(),
        },
        DashboardMetric {
            label: "cold_storage_saved".to_string(),
            value: format_bytes(reclaim_bytes),
            note: "potential_reclaim_from_soft_delete_queue".to_string(),
        },
    ]
}

fn build_audit_records(audit_db_path: Option<&Path>) -> SnapshotResult<Vec<AuditEventRecord>> {
    let Some(audit_db_path) = audit_db_path else {
        return Ok(Vec::new());
    };

    if !audit_db_path.exists() {
        return Ok(Vec::new());
    }

    let connection = open_database(audit_db_path)?;
    let events = load_audit_events(&connection, 100)?;

    Ok(events.into_iter().map(build_audit_record).collect())
}

fn build_audit_record(event: AuditEvent) -> AuditEventRecord {
    let detail = summarize_audit_event(&event);

    AuditEventRecord {
        event_id: event.event_id.clone(),
        r#type: event.event_type.clone(),
        target: event.target_id.clone(),
        actor: event.actor,
        created_at: event.created_at,
        result: event.result.clone(),
        detail,
    }
}

fn summarize_audit_event(event: &AuditEvent) -> String {
    match event.event_type.as_str() {
        "export_markdown" => {
            format!("Exported Markdown artifact for {}.", event.target_id)
        }
        "soft_delete" => {
            format!("Moved {} into the quarantine queue.", event.target_id)
        }
        "restore" => {
            format!("Restored {} from quarantine.", event.target_id)
        }
        _ => format!("Recorded {} for {}.", event.event_type, event.target_id),
    }
}

fn extract_session_narrative(session: &SessionRecord) -> SnapshotResult<SessionNarrative> {
    match session.assistant.as_str() {
        "codex" => extract_codex_narrative(Path::new(&session.source_path)),
        "claude-code" => extract_claude_narrative(Path::new(&session.source_path)),
        "opencode" => extract_opencode_narrative(Path::new(&session.source_path)),
        assistant => Err(SnapshotError::UnsupportedAssistant(assistant.to_string())),
    }
}

fn extract_codex_narrative(source: &Path) -> SnapshotResult<SessionNarrative> {
    let lines = read_jsonl(source)?;
    let mut narrative = SessionNarrative::default();

    for line in lines {
        if line.get("type").and_then(Value::as_str) != Some("response_item") {
            continue;
        }

        let payload = line.get("payload").unwrap_or(&Value::Null);
        if payload.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }

        let role = payload.get("role").and_then(Value::as_str);
        let message = extract_text_array(payload.get("content"));

        match role {
            Some("user") if narrative.first_user_goal.is_none() => {
                narrative.first_user_goal = message;
            }
            Some("assistant") => {
                if let Some(message) = message {
                    if looks_like_error_message(&message) {
                        narrative.error_count += 1;
                    }
                    narrative.last_assistant_message = Some(message);
                }
            }
            _ => {}
        }
    }

    Ok(narrative)
}

fn extract_claude_narrative(source: &Path) -> SnapshotResult<SessionNarrative> {
    let lines = read_jsonl(source)?;
    let mut narrative = SessionNarrative::default();

    for line in lines {
        match line.get("type").and_then(Value::as_str) {
            Some("user") if narrative.first_user_goal.is_none() => {
                narrative.first_user_goal = line
                    .get("message")
                    .and_then(|message| message.get("content"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
            }
            Some("assistant") => {
                if let Some(message) = line
                    .get("message")
                    .and_then(|message| message.get("content"))
                    .and_then(Value::as_array)
                    .and_then(|parts| {
                        parts.iter().find_map(|part| {
                            part.get("text")
                                .and_then(Value::as_str)
                                .map(ToOwned::to_owned)
                        })
                    })
                {
                    if looks_like_error_message(&message) {
                        narrative.error_count += 1;
                    }
                    narrative.last_assistant_message = Some(message);
                }
            }
            _ => {}
        }
    }

    Ok(narrative)
}

fn extract_opencode_narrative(source: &Path) -> SnapshotResult<SessionNarrative> {
    let session_info: Value = serde_json::from_slice(&fs::read(source)?)?;
    let session_id = session_info
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let storage_root = source
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| SnapshotError::Io(io::Error::other("invalid opencode session path")))?;
    let message_dir = storage_root
        .join("session")
        .join("message")
        .join(session_id);
    let part_dir = storage_root.join("session").join("part").join(session_id);
    let mut message_files = collect_files(&message_dir, &|path| {
        path.extension().and_then(|value| value.to_str()) == Some("json")
    })?;
    message_files.sort();
    let mut narrative = SessionNarrative {
        first_user_goal: session_info
            .get("title")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        ..Default::default()
    };

    for message_file in message_files {
        let message: Value = serde_json::from_slice(&fs::read(&message_file)?)?;
        let message_id = message
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let message_part_dir = part_dir.join(message_id);
        let texts = collect_opencode_part_texts(&message_part_dir)?;
        let joined = texts.join(" ").trim().to_string();

        if joined.is_empty() {
            continue;
        }

        match message.get("role").and_then(Value::as_str) {
            Some("user") if narrative.first_user_goal.is_none() => {
                narrative.first_user_goal = Some(joined);
            }
            Some("assistant") => {
                if looks_like_error_message(&joined) {
                    narrative.error_count += 1;
                }
                narrative.last_assistant_message = Some(joined);
            }
            _ => {}
        }
    }

    Ok(narrative)
}

fn collect_opencode_part_texts(part_dir: &Path) -> SnapshotResult<Vec<String>> {
    let mut files = collect_files(part_dir, &|path| {
        path.extension().and_then(|value| value.to_str()) == Some("json")
    })?;
    files.sort();

    let mut texts = Vec::new();
    for file in files {
        let part: Value = serde_json::from_slice(&fs::read(file)?)?;
        if part.get("type").and_then(Value::as_str) == Some("text") {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                texts.push(text.to_string());
            }
        }
    }

    Ok(texts)
}

fn read_jsonl(path: &Path) -> SnapshotResult<Vec<Value>> {
    fs::read_to_string(path)?
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn extract_text_array(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_array)
        .map(|parts| {
            parts.iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|text| !text.trim().is_empty())
}

fn session_adapter(assistant: &str) -> SnapshotResult<Box<dyn SessionAdapter>> {
    match assistant {
        "codex" => Ok(Box::new(CodexAdapter)),
        "claude-code" => Ok(Box::new(ClaudeCodeAdapter)),
        "opencode" => Ok(Box::new(OpenCodeAdapter)),
        assistant => Err(SnapshotError::UnsupportedAssistant(assistant.to_string())),
    }
}

fn normalize_scope(scope: &str) -> String {
    match scope {
        "user" => "global".to_string(),
        value => value.to_string(),
    }
}

fn infer_proxy_mode(risks: &[crate::audit::RiskFlag]) -> String {
    if risks.iter().any(|risk| {
        matches!(
            risk.code.as_str(),
            "third_party_provider" | "third_party_base_url"
        )
    }) {
        "proxy".to_string()
    } else {
        "official".to_string()
    }
}

fn last_path_segment(path: &str) -> Option<&str> {
    path.rsplit(['\\', '/']).find(|segment| !segment.is_empty())
}

fn derive_stale_score(last_activity_at: Option<&str>) -> u8 {
    let Some(timestamp) = parse_timestamp(last_activity_at) else {
        return 100;
    };

    let days = Utc::now().signed_duration_since(timestamp).num_days();
    if days >= 90 {
        100
    } else if days >= 30 {
        75
    } else if days >= 14 {
        40
    } else {
        10
    }
}

fn parse_timestamp(value: Option<&str>) -> Option<DateTime<Utc>> {
    let value = value?;
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(parsed.with_timezone(&Utc));
    }

    let milliseconds = value.parse::<i64>().ok()?;
    DateTime::from_timestamp_millis(milliseconds)
}

fn looks_like_error_message(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.starts_with("fatal:")
        || lowered.starts_with("error:")
        || lowered.contains(" exception")
        || lowered.contains(" failed")
        || lowered.contains("missing configuration")
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    match bytes as f64 {
        value if value >= GIB => format!("{:.1} GB", value / GIB),
        value if value >= MIB => format!("{:.1} MB", value / MIB),
        value if value >= KIB => format!("{:.1} KB", value / KIB),
        value => format!("{value:.0} B"),
    }
}
