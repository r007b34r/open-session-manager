use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    env, fmt, fs, io,
    path::{Path, PathBuf},
    process::Command,
    time::UNIX_EPOCH,
};

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    adapters::{
        claude_code::ClaudeCodeAdapter,
        codex::CodexAdapter,
        copilot_cli::CopilotCliAdapter,
        factory_droid::{
            DroidDialect, FactoryDroidAdapter, detect_droid_dialect, normalize_droid_kind,
        },
        gemini_cli::{
            GeminiCliAdapter, gemini_messages, gemini_role, gemini_text, gemini_tool_calls,
        },
        openclaw::{OpenClawAdapter, openclaw_kind, openclaw_role, openclaw_text},
        opencode::OpenCodeAdapter,
        traits::{AdapterError, SessionAdapter, collect_files},
    },
    audit::{
        config_audit::{AuditError, ConfigAuditTarget, audit_config},
        credential_audit::build_credential_artifacts,
    },
    commands::{configs::discover_known_config_targets, discovery::discover_known_session_roots},
    discovery::{DiscoveryContext, KnownPath},
    domain::{
        audit::AuditEvent,
        session::{SessionInsight, SessionRecord},
    },
    insights::{
        InsightInput, garbage::derive_garbage_score, progress::derive_progress_state,
        title::derive_title, value::derive_value_score,
    },
    preferences::RuntimeSnapshot,
    session_text::normalize_session_text,
    storage::sqlite::{
        SessionIndexCacheRow, SessionIndexRunRecord, delete_session_index_cache_rows,
        insert_session_index_run, list_session_index_cache_paths, load_audit_events,
        load_session_control_state, load_session_index_cache_row, open_database,
        upsert_session_index_cache_row,
    },
    transcript::{TranscriptHighlight, TranscriptTodo, build_transcript_digest},
    usage::{
        SessionUsageRecord, UsageOverviewRecord, UsageTimelineRecord, build_usage_overview,
        build_usage_timeline, extract_session_usage,
    },
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
    pub git_projects: Vec<GitProjectRecord>,
    pub doctor_findings: Vec<DoctorFindingRecord>,
    pub audit_events: Vec<AuditEventRecord>,
    pub usage_overview: UsageOverviewRecord,
    pub usage_timeline: Vec<UsageTimelineRecord>,
    pub runtime: RuntimeSnapshot,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DashboardMetric {
    pub label: String,
    pub value: String,
    pub note: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub transcript_highlights: Vec<TranscriptHighlight>,
    pub todo_items: Vec<TranscriptTodo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<SessionUsageRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_control: Option<SessionControlRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionControlRecord {
    pub supported: bool,
    pub available: bool,
    pub controller: String,
    pub command: String,
    pub attached: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_response: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_resumed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_continued_at: Option<String>,
}

#[derive(Debug)]
pub struct IndexedSession {
    pub session: SessionRecord,
    pub insight: SessionInsight,
    pub detail: SessionDetailRecord,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRiskRecord {
    pub artifact_id: String,
    pub assistant: String,
    pub scope: String,
    pub path: String,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub base_url: String,
    pub masked_secret: String,
    pub official_or_proxy: String,
    pub risks: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<ConfigMcpServerRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigMcpServerRecord {
    pub server_id: String,
    pub name: String,
    pub enabled: bool,
    pub status: String,
    pub transport: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub config_json: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitProjectRecord {
    pub project_path: String,
    pub repo_root: String,
    pub branch: String,
    pub status: String,
    pub session_count: usize,
    pub dirty: bool,
    pub staged_changes: u32,
    pub unstaged_changes: u32,
    pub untracked_files: u32,
    pub ahead: u32,
    pub behind: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit_at: Option<String>,
    #[serde(default)]
    pub recent_commits: Vec<GitCommitRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCommitRecord {
    pub sha: String,
    pub summary: String,
    pub author: String,
    pub authored_at: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quarantined_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DoctorFindingRecord {
    pub code: String,
    pub severity: String,
    pub assistant: String,
    pub path: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    pub status: String,
    pub findings: Vec<DoctorFindingRecord>,
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
        KnownPath::new(
            "opencode",
            "session",
            "linux",
            fixtures_root.join("opencode"),
        ),
        KnownPath::new(
            "gemini-cli",
            "session",
            "windows",
            fixtures_root.join("gemini").join("tmp"),
        ),
        KnownPath::new(
            "github-copilot-cli",
            "session",
            "windows",
            fixtures_root.join("copilot"),
        ),
        KnownPath::new(
            "factory-droid",
            "session",
            "windows",
            fixtures_root.join("factory"),
        ),
        KnownPath::new(
            "openclaw",
            "session",
            "windows",
            fixtures_root.join("openclaw"),
        ),
    ];

    let config_targets = vec![
        ConfigAuditTarget::new(
            "codex",
            "global",
            "fixtures",
            fixtures_root
                .join("configs")
                .join("codex")
                .join("config.toml"),
        ),
        ConfigAuditTarget::new(
            "claude-code",
            "global",
            "fixtures",
            fixtures_root
                .join("configs")
                .join("claude")
                .join("settings.json"),
        ),
        ConfigAuditTarget::new(
            "opencode",
            "global",
            "fixtures",
            fixtures_root
                .join("configs")
                .join("opencode")
                .join("opencode.json"),
        ),
        ConfigAuditTarget::new(
            "gemini-cli",
            "global",
            "fixtures",
            fixtures_root
                .join("configs")
                .join("gemini")
                .join("settings.json"),
        ),
        ConfigAuditTarget::new(
            "github-copilot-cli",
            "global",
            "fixtures",
            fixtures_root
                .join("configs")
                .join("copilot")
                .join("config.json"),
        ),
        ConfigAuditTarget::new(
            "factory-droid",
            "global",
            "fixtures",
            fixtures_root
                .join("configs")
                .join("factory")
                .join("settings.json"),
        ),
        ConfigAuditTarget::new(
            "openclaw",
            "global",
            "fixtures",
            fixtures_root
                .join("configs")
                .join("openclaw")
                .join("openclaw.json"),
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

pub fn build_local_indexed_sessions(
    context: &DiscoveryContext,
) -> SnapshotResult<Vec<IndexedSession>> {
    Ok(build_indexed_sessions_with_findings(discover_known_session_roots(context), None)?.sessions)
}

pub fn find_local_config_target(
    context: &DiscoveryContext,
    artifact_id: &str,
) -> SnapshotResult<Option<ConfigAuditTarget>> {
    let session_roots = discover_known_session_roots(context);
    let sessions = build_session_records(&session_roots)?;
    let derived_project_targets = derive_project_config_targets(&sessions);
    let merged_config_targets = merge_config_targets(
        discover_known_config_targets(context),
        derived_project_targets,
    );

    for target in merged_config_targets {
        if !target.path.exists() {
            continue;
        }

        let audit = audit_config(&target)?;
        if audit.config.artifact_id == artifact_id {
            return Ok(Some(target));
        }
    }

    Ok(None)
}

pub fn build_local_doctor_report(context: &DiscoveryContext) -> SnapshotResult<DoctorReport> {
    let findings =
        build_indexed_sessions_with_findings(discover_known_session_roots(context), None)?
            .doctor_findings;

    Ok(DoctorReport {
        status: doctor_status(&findings).to_string(),
        findings,
    })
}

fn build_git_project_records(sessions: &[SessionDetailRecord]) -> Vec<GitProjectRecord> {
    let mut projects = BTreeMap::<String, GitProjectRecord>::new();

    for session in sessions {
        let project_path = session.project_path.trim();
        if project_path.is_empty() || project_path.eq_ignore_ascii_case("unknown") {
            continue;
        }

        let Some(inspected) = inspect_git_project(Path::new(project_path)) else {
            continue;
        };

        let entry = projects
            .entry(inspected.repo_root.clone())
            .or_insert_with(|| GitProjectRecord {
                project_path: project_path.to_string(),
                repo_root: inspected.repo_root.clone(),
                branch: inspected.branch.clone(),
                status: inspected.status.clone(),
                session_count: 0,
                dirty: inspected.dirty,
                staged_changes: inspected.staged_changes,
                unstaged_changes: inspected.unstaged_changes,
                untracked_files: inspected.untracked_files,
                ahead: inspected.ahead,
                behind: inspected.behind,
                last_commit_summary: inspected.last_commit_summary.clone(),
                last_commit_at: inspected.last_commit_at.clone(),
                recent_commits: inspected.recent_commits.clone(),
            });

        entry.session_count += 1;
    }

    let mut values = projects.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| {
        let dirty_delta = right.dirty.cmp(&left.dirty);
        if dirty_delta != std::cmp::Ordering::Equal {
            return dirty_delta;
        }

        let session_delta = right.session_count.cmp(&left.session_count);
        if session_delta != std::cmp::Ordering::Equal {
            return session_delta;
        }

        left.repo_root.cmp(&right.repo_root)
    });
    values
}

struct GitProjectInspection {
    repo_root: String,
    branch: String,
    status: String,
    dirty: bool,
    staged_changes: u32,
    unstaged_changes: u32,
    untracked_files: u32,
    ahead: u32,
    behind: u32,
    last_commit_summary: Option<String>,
    last_commit_at: Option<String>,
    recent_commits: Vec<GitCommitRecord>,
}

fn inspect_git_project(project_path: &Path) -> Option<GitProjectInspection> {
    if !project_path.exists() {
        return None;
    }

    let repo_root = normalize_git_path(&run_git_command(
        project_path,
        &["rev-parse", "--show-toplevel"],
    )?);
    let status_output = run_git_command(project_path, &["status", "--porcelain", "--branch"])?;
    let parsed_status = parse_git_status(&status_output);
    let recent_commits = parse_git_commits(
        &run_git_command(
            project_path,
            &["log", "-n", "5", "--pretty=format:%H%x1f%s%x1f%an%x1f%cI"],
        )
        .unwrap_or_default(),
    );
    let last_commit_summary = recent_commits.first().map(|commit| commit.summary.clone());
    let last_commit_at = recent_commits
        .first()
        .map(|commit| commit.authored_at.clone());

    Some(GitProjectInspection {
        repo_root,
        branch: parsed_status.branch,
        status: parsed_status.status,
        dirty: parsed_status.dirty,
        staged_changes: parsed_status.staged_changes,
        unstaged_changes: parsed_status.unstaged_changes,
        untracked_files: parsed_status.untracked_files,
        ahead: parsed_status.ahead,
        behind: parsed_status.behind,
        last_commit_summary,
        last_commit_at,
        recent_commits,
    })
}

fn run_git_command(project_path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(project_path)
        .args(args)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn normalize_git_path(value: &str) -> String {
    PathBuf::from(value.trim()).display().to_string()
}

struct ParsedGitStatus {
    branch: String,
    status: String,
    dirty: bool,
    staged_changes: u32,
    unstaged_changes: u32,
    untracked_files: u32,
    ahead: u32,
    behind: u32,
}

fn parse_git_status(output: &str) -> ParsedGitStatus {
    let mut branch = "unknown".to_string();
    let mut staged_changes = 0;
    let mut unstaged_changes = 0;
    let mut untracked_files = 0;
    let mut ahead = 0;
    let mut behind = 0;

    for (index, line) in output.lines().enumerate() {
        if index == 0 && line.starts_with("## ") {
            let header = line.trim_start_matches("## ");
            if let Some((branch_part, _)) = header.split_once("...") {
                branch = branch_part.trim().to_string();
            } else {
                branch = header
                    .split_whitespace()
                    .next()
                    .unwrap_or("unknown")
                    .to_string();
            }

            if let Some((_, bracketed)) = header.split_once('[') {
                let bracketed = bracketed.trim_end_matches(']');
                for part in bracketed.split(',') {
                    let trimmed = part.trim();
                    if let Some(value) = trimmed.strip_prefix("ahead ") {
                        ahead = value.parse::<u32>().unwrap_or(0);
                    }
                    if let Some(value) = trimmed.strip_prefix("behind ") {
                        behind = value.parse::<u32>().unwrap_or(0);
                    }
                }
            }

            continue;
        }

        let bytes = line.as_bytes();
        if bytes.len() < 2 {
            continue;
        }

        let left = bytes[0] as char;
        let right = bytes[1] as char;

        if left == '?' && right == '?' {
            untracked_files += 1;
            continue;
        }

        if left != ' ' {
            staged_changes += 1;
        }

        if right != ' ' {
            unstaged_changes += 1;
        }
    }

    let dirty = staged_changes > 0 || unstaged_changes > 0 || untracked_files > 0;
    let status = if dirty {
        "dirty".to_string()
    } else if ahead > 0 || behind > 0 {
        "diverged".to_string()
    } else {
        "clean".to_string()
    };

    ParsedGitStatus {
        branch,
        status,
        dirty,
        staged_changes,
        unstaged_changes,
        untracked_files,
        ahead,
        behind,
    }
}

fn parse_git_commits(output: &str) -> Vec<GitCommitRecord> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\u{1f}');
            Some(GitCommitRecord {
                sha: parts.next()?.trim().to_string(),
                summary: parts.next()?.trim().to_string(),
                author: parts.next()?.trim().to_string(),
                authored_at: parts.next()?.trim().to_string(),
            })
        })
        .collect()
}

fn build_snapshot(
    session_roots: Vec<KnownPath>,
    config_targets: Vec<ConfigAuditTarget>,
    audit_db_path: Option<&Path>,
) -> SnapshotResult<DashboardSnapshot> {
    let connection = match audit_db_path {
        Some(path) => Some(open_database(path)?),
        None => None,
    };
    let IndexedSessionBuildResult {
        sessions: indexed_sessions,
        doctor_findings,
    } = build_indexed_sessions_with_findings(session_roots, connection.as_ref())?;
    let sessions = indexed_sessions
        .into_iter()
        .map(|indexed| {
            let mut detail = indexed.detail;
            detail.session_control = Some(build_session_control_record(
                &indexed.session,
                connection.as_ref(),
            ));
            detail
        })
        .collect::<Vec<_>>();
    let derived_project_targets = derive_project_config_targets(&sessions);
    let merged_config_targets = merge_config_targets(config_targets, derived_project_targets);
    let configs = build_config_records(&merged_config_targets)?;
    let audit_events = build_audit_records(connection.as_ref())?;
    let usage_overview = build_usage_overview(
        sessions
            .iter()
            .map(|session| (session.assistant.clone(), session.usage.clone())),
    );
    let usage_timeline = build_usage_timeline(sessions.iter().map(|session| {
        (
            Some(session.last_activity_at.clone()),
            session.usage.clone(),
        )
    }));
    let git_projects = build_git_project_records(&sessions);

    Ok(DashboardSnapshot {
        metrics: build_metrics(&sessions, &configs),
        sessions,
        configs,
        git_projects,
        doctor_findings,
        audit_events,
        usage_overview,
        usage_timeline,
        runtime: RuntimeSnapshot::default(),
    })
}

fn build_session_records(session_roots: &[KnownPath]) -> SnapshotResult<Vec<SessionDetailRecord>> {
    Ok(
        build_indexed_sessions_with_findings(session_roots.to_vec(), None)?
            .sessions
            .into_iter()
            .map(|indexed| indexed.detail)
            .collect(),
    )
}

#[derive(Debug, Default)]
struct IndexedBuildStats {
    discovered_files: i64,
    cache_hits: i64,
    cache_misses: i64,
    reindexed_files: i64,
    stale_deleted: i64,
}

struct IndexedSessionBuildResult {
    sessions: Vec<IndexedSession>,
    doctor_findings: Vec<DoctorFindingRecord>,
}

fn build_indexed_sessions_with_findings(
    session_roots: Vec<KnownPath>,
    connection: Option<&Connection>,
) -> SnapshotResult<IndexedSessionBuildResult> {
    let mut sessions = Vec::new();
    let mut doctor_findings = Vec::new();
    let mut observed_source_paths = HashSet::new();
    let mut stats = IndexedBuildStats::default();
    let started_at = Utc::now().to_rfc3339();

    for root in &session_roots {
        if !root.path.exists() {
            continue;
        }

        let adapter = session_adapter(&root.assistant)?;
        let discovered = adapter.discover_session_files(&root.path)?;
        let discovered_paths = discovered.iter().cloned().collect::<HashSet<_>>();
        doctor_findings.extend(build_unknown_session_candidate_findings(
            root,
            &discovered_paths,
        )?);

        for session_file in discovered {
            stats.discovered_files += 1;
            observed_source_paths.insert(session_file.display().to_string());

            if let Some(cached) = try_load_cached_indexed_session(connection, root, &session_file)?
            {
                stats.cache_hits += 1;
                sessions.push(cached);
                continue;
            }

            stats.cache_misses += 1;

            match build_indexed_session_from_file(adapter.as_ref(), root, &session_file) {
                Ok(indexed) => {
                    stats.reindexed_files += 1;
                    persist_cached_indexed_session(connection, &session_file, &indexed)?;
                    sessions.push(indexed);
                }
                Err(error) if is_recoverable_session_file_error(&error) => {
                    doctor_findings.push(build_recoverable_session_file_finding(
                        root,
                        &session_file,
                        &error,
                    ));
                    emit_recoverable_session_file_warning(root, &session_file, &error);
                }
                Err(error) => return Err(error),
            }
        }
    }

    if let Some(connection) = connection {
        stats.stale_deleted =
            prune_stale_cached_sessions(connection, &observed_source_paths)? as i64;
        persist_index_run_stats(connection, &started_at, &stats)?;
    }

    Ok(IndexedSessionBuildResult {
        sessions,
        doctor_findings,
    })
}

fn build_indexed_session_from_file(
    adapter: &dyn SessionAdapter,
    root: &KnownPath,
    session_file: &Path,
) -> SnapshotResult<IndexedSession> {
    let mut session = adapter.parse_session(session_file)?;
    session.environment = root.environment.clone();
    let narrative = extract_session_narrative(&session)?;
    build_indexed_session(session, narrative)
}

fn is_recoverable_session_file_error(error: &SnapshotError) -> bool {
    matches!(
        error,
        SnapshotError::Adapter(AdapterError::InvalidSession(_))
            | SnapshotError::Adapter(AdapterError::Json(_))
            | SnapshotError::Adapter(AdapterError::Io(_))
            | SnapshotError::Json(_)
            | SnapshotError::Io(_)
    )
}

fn emit_recoverable_session_file_warning(
    root: &KnownPath,
    session_file: &Path,
    error: &SnapshotError,
) {
    if env::var_os("OPEN_SESSION_MANAGER_VERBOSE_DISCOVERY").is_none() {
        return;
    }

    eprintln!(
        "skipping malformed session file for {}: {} ({error})",
        root.assistant,
        session_file.display()
    );
}

fn build_recoverable_session_file_finding(
    root: &KnownPath,
    session_file: &Path,
    error: &SnapshotError,
) -> DoctorFindingRecord {
    DoctorFindingRecord {
        code: "malformed_session_skipped".to_string(),
        severity: "warn".to_string(),
        assistant: root.assistant.clone(),
        path: session_file.display().to_string(),
        detail: format!(
            "Skipped malformed session file for {} because it could not be parsed safely ({error}).",
            root.assistant
        ),
    }
}

fn build_unknown_session_candidate_findings(
    root: &KnownPath,
    discovered_paths: &HashSet<PathBuf>,
) -> SnapshotResult<Vec<DoctorFindingRecord>> {
    Ok(collect_unknown_session_candidate_files(root)?
        .into_iter()
        .filter(|path| !discovered_paths.contains(path))
        .map(|path| DoctorFindingRecord {
            code: "unknown_session_candidate".to_string(),
            severity: "warn".to_string(),
            assistant: root.assistant.clone(),
            path: path.display().to_string(),
            detail: format!(
                "Found a session-like file under the known {} root, but no supported parser recognized its format yet.",
                root.assistant
            ),
        })
        .collect())
}

fn collect_unknown_session_candidate_files(root: &KnownPath) -> SnapshotResult<Vec<PathBuf>> {
    match root.assistant.as_str() {
        "factory-droid" => collect_files(&root.path, &|path| {
            path.extension().and_then(|value| value.to_str()) == Some("jsonl")
        })
        .map_err(Into::into),
        "openclaw" => collect_files(&root.path, &|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.ends_with(".jsonl.lock"))
                && path
                    .components()
                    .any(|component| component.as_os_str() == "agents")
                && path
                    .components()
                    .any(|component| component.as_os_str() == "sessions")
        })
        .map_err(Into::into),
        _ => Ok(Vec::new()),
    }
}

fn doctor_status(findings: &[DoctorFindingRecord]) -> &str {
    if findings.iter().any(|finding| finding.severity == "warn") {
        "warn"
    } else {
        "ok"
    }
}

fn build_indexed_session(
    session: SessionRecord,
    narrative: SessionNarrative,
) -> SnapshotResult<IndexedSession> {
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
    let transcript_digest = build_transcript_digest(&session);
    let project_path = session
        .project_path
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let source_path = session.source_path.clone();
    let last_activity_at = session
        .last_activity_at
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let confidence = derive_confidence(&narrative);
    let progress_state = progress_state.to_string();
    let usage = extract_session_usage(&session);
    let detail = SessionDetailRecord {
        session_id: session.session_id.clone(),
        title: title.clone(),
        assistant: session.assistant.clone(),
        progress_state: progress_state.clone(),
        progress_percent: progress_percent.unwrap_or(0),
        last_activity_at,
        environment: session.environment.clone(),
        value_score,
        summary,
        project_path,
        source_path,
        tags: tags.clone(),
        risk_flags: risk_flags.clone(),
        key_artifacts,
        transcript_highlights: transcript_digest.highlights.clone(),
        todo_items: transcript_digest.todos.clone(),
        usage,
        session_control: None,
    };
    let insight = SessionInsight {
        session_id: session.session_id.clone(),
        title,
        topic_labels_json: serde_json::to_string(&tags)?,
        summary: detail.summary.clone(),
        progress_state,
        progress_percent,
        value_score,
        stale_score,
        garbage_score,
        risk_flags_json: serde_json::to_string(&risk_flags)?,
        confidence,
    };

    Ok(IndexedSession {
        session,
        insight,
        detail,
    })
}

fn try_load_cached_indexed_session(
    connection: Option<&Connection>,
    root: &KnownPath,
    session_file: &Path,
) -> SnapshotResult<Option<IndexedSession>> {
    let Some(connection) = connection else {
        return Ok(None);
    };

    let metadata = read_session_file_metadata(session_file)?;
    let source_path = session_file.display().to_string();
    let Some(cached) = load_session_index_cache_row(connection, &source_path)? else {
        return Ok(None);
    };

    if cached.assistant != root.assistant
        || cached.environment != root.environment
        || cached.source_size != metadata.0
        || cached.source_modified_at != metadata.1
    {
        return Ok(None);
    }

    let Ok(session) = serde_json::from_str::<SessionRecord>(&cached.session_json) else {
        return Ok(None);
    };
    let Ok(insight) = serde_json::from_str::<SessionInsight>(&cached.insight_json) else {
        return Ok(None);
    };
    let Ok(detail) = serde_json::from_str::<SessionDetailRecord>(&cached.detail_json) else {
        return Ok(None);
    };

    Ok(Some(IndexedSession {
        session,
        insight,
        detail,
    }))
}

fn persist_cached_indexed_session(
    connection: Option<&Connection>,
    session_file: &Path,
    indexed: &IndexedSession,
) -> SnapshotResult<()> {
    let Some(connection) = connection else {
        return Ok(());
    };

    let metadata = read_session_file_metadata(session_file)?;
    let row = SessionIndexCacheRow {
        source_path: session_file.display().to_string(),
        assistant: indexed.session.assistant.clone(),
        environment: indexed.session.environment.clone(),
        source_size: metadata.0,
        source_modified_at: metadata.1,
        session_id: indexed.session.session_id.clone(),
        session_json: serde_json::to_string(&indexed.session)?,
        insight_json: serde_json::to_string(&indexed.insight)?,
        detail_json: serde_json::to_string(&indexed.detail)?,
        indexed_at: Utc::now().to_rfc3339(),
    };

    upsert_session_index_cache_row(connection, &row)?;
    Ok(())
}

fn prune_stale_cached_sessions(
    connection: &Connection,
    observed_source_paths: &HashSet<String>,
) -> SnapshotResult<usize> {
    let stale_paths = list_session_index_cache_paths(connection)?
        .into_iter()
        .filter(|source_path| !observed_source_paths.contains(source_path))
        .collect::<Vec<_>>();

    delete_session_index_cache_rows(connection, &stale_paths).map_err(Into::into)
}

fn persist_index_run_stats(
    connection: &Connection,
    started_at: &str,
    stats: &IndexedBuildStats,
) -> SnapshotResult<()> {
    let finished_at = Utc::now().to_rfc3339();
    let payload = format!(
        "{started_at}:{}:{}:{}:{}",
        stats.discovered_files, stats.cache_hits, stats.cache_misses, stats.reindexed_files
    );
    let digest = Sha256::digest(payload.as_bytes());
    let run_id = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    insert_session_index_run(
        connection,
        &SessionIndexRunRecord {
            run_id,
            started_at: started_at.to_string(),
            finished_at,
            discovered_files: stats.discovered_files,
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
            reindexed_files: stats.reindexed_files,
            stale_deleted: stats.stale_deleted,
        },
    )?;

    Ok(())
}

fn read_session_file_metadata(session_file: &Path) -> SnapshotResult<(i64, i64)> {
    let metadata = fs::metadata(session_file)?;
    let source_size = metadata.len().min(i64::MAX as u64) as i64;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default();

    Ok((source_size, modified))
}

fn build_session_control_record(
    session: &SessionRecord,
    connection: Option<&Connection>,
) -> SessionControlRecord {
    let (supported, controller, command) = match session.assistant.as_str() {
        "codex" => (
            true,
            "codex".to_string(),
            env::var("OPEN_SESSION_MANAGER_CODEX_COMMAND").unwrap_or_else(|_| "codex".to_string()),
        ),
        "claude-code" => (
            true,
            "claude-code".to_string(),
            env::var("OPEN_SESSION_MANAGER_CLAUDE_CODE_COMMAND")
                .unwrap_or_else(|_| "claude".to_string()),
        ),
        assistant => (false, assistant.to_string(), String::new()),
    };
    let available = supported && dashboard_command_is_available(&command);
    let persisted = connection
        .and_then(|db| load_session_control_state(db, &session.session_id).ok())
        .flatten();

    SessionControlRecord {
        supported,
        available,
        controller,
        command,
        attached: persisted.as_ref().is_some_and(|state| state.attached),
        last_command: persisted
            .as_ref()
            .and_then(|state| state.last_command.clone()),
        last_prompt: persisted
            .as_ref()
            .and_then(|state| state.last_prompt.clone()),
        last_response: persisted
            .as_ref()
            .and_then(|state| state.last_response.clone()),
        last_error: persisted
            .as_ref()
            .and_then(|state| state.last_error.clone()),
        last_resumed_at: persisted
            .as_ref()
            .and_then(|state| state.last_resumed_at.clone()),
        last_continued_at: persisted
            .as_ref()
            .and_then(|state| state.last_continued_at.clone()),
    }
}

fn dashboard_command_is_available(command: &str) -> bool {
    if command.trim().is_empty() {
        return false;
    }

    if command.contains(std::path::MAIN_SEPARATOR)
        || command.contains('/')
        || command.contains('\\')
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
        if cfg!(windows) && dir.join(command).exists() {
            return true;
        }

        path_exts
            .iter()
            .map(|ext| dir.join(format!("{command}{ext}")))
            .any(|candidate| candidate.exists())
    })
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
        let artifact_id = audit.config.artifact_id.clone();

        configs.push(ConfigRiskRecord {
            artifact_id: artifact_id.clone(),
            assistant: audit.config.assistant,
            scope: normalize_scope(&audit.config.scope),
            path: audit.config.path,
            provider,
            model: audit.config.model,
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
            mcp_servers: parse_config_mcp_servers(&artifact_id, &audit.config.mcp_json),
        });
    }

    Ok(configs)
}

fn parse_config_mcp_servers(
    config_artifact_id: &str,
    mcp_json: &str,
) -> Vec<ConfigMcpServerRecord> {
    let Ok(parsed) = serde_json::from_str::<Value>(mcp_json) else {
        return Vec::new();
    };

    let Some(servers) = extract_mcp_server_map(&parsed) else {
        return Vec::new();
    };

    let mut entries = servers
        .iter()
        .filter_map(|(name, raw)| {
            let raw_object = raw.as_object()?;
            let enabled = raw_object
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let command = raw_object
                .get("command")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let args = raw_object
                .get("args")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let url = raw_object
                .get("url")
                .or_else(|| raw_object.get("serverUrl"))
                .or_else(|| raw_object.get("endpoint"))
                .or_else(|| raw_object.get("baseUrl"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let transport = infer_mcp_transport(raw_object, command.as_deref(), url.as_deref());
            let status = infer_mcp_status(enabled, command.as_deref(), url.as_deref());

            Some(ConfigMcpServerRecord {
                server_id: format!("{config_artifact_id}:{name}"),
                name: name.clone(),
                enabled,
                status,
                transport,
                command,
                args,
                url,
                config_json: serde_json::to_string_pretty(raw).unwrap_or_else(|_| raw.to_string()),
            })
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| left.name.cmp(&right.name));
    entries
}

fn extract_mcp_server_map(value: &Value) -> Option<&serde_json::Map<String, Value>> {
    let object = value.as_object()?;

    if let Some(servers) = object.get("servers").and_then(Value::as_object) {
        return Some(servers);
    }

    if let Some(servers) = object.get("mcpServers").and_then(Value::as_object) {
        return Some(servers);
    }

    Some(object)
}

fn infer_mcp_transport(
    raw: &serde_json::Map<String, Value>,
    command: Option<&str>,
    url: Option<&str>,
) -> String {
    if let Some(transport) = raw.get("transport").and_then(Value::as_str) {
        return normalize_mcp_transport(transport);
    }

    if command.is_some() {
        return "stdio".to_string();
    }

    if let Some(url) = url {
        let lowered = url.to_ascii_lowercase();
        if lowered.contains("/sse") {
            return "sse".to_string();
        }

        return "http".to_string();
    }

    "embedded".to_string()
}

fn normalize_mcp_transport(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "stdio" => "stdio".to_string(),
        "sse" => "sse".to_string(),
        "http" | "streamable-http" | "streamable_http" => "http".to_string(),
        "embedded" | "local" => "embedded".to_string(),
        _ => value.trim().to_ascii_lowercase(),
    }
}

fn infer_mcp_status(enabled: bool, command: Option<&str>, url: Option<&str>) -> String {
    if !enabled {
        return "disabled".to_string();
    }

    if command.is_some() || url.is_some() {
        return "configured".to_string();
    }

    "enabled".to_string()
}

fn derive_project_config_targets(sessions: &[SessionDetailRecord]) -> Vec<ConfigAuditTarget> {
    let mut targets = Vec::new();

    for session in sessions {
        let project_path = normalize_project_path(&session.project_path);
        let Some(project_path) = project_path else {
            continue;
        };

        match session.assistant.as_str() {
            "github-copilot-cli" => {
                let copilot_dir = project_path.join(".github").join("copilot");
                let local_path = copilot_dir.join("settings.local.json");
                let base_path = copilot_dir.join("settings.json");
                let target_path = if local_path.exists() {
                    local_path
                } else {
                    base_path
                };
                targets.push(ConfigAuditTarget::new(
                    "github-copilot-cli",
                    "project",
                    session.environment.clone(),
                    target_path,
                ));
            }
            "factory-droid" => {
                let factory_dir = project_path.join(".factory");
                let local_path = factory_dir.join("settings.local.json");
                let base_path = factory_dir.join("settings.json");
                let target_path = if local_path.exists() {
                    local_path
                } else {
                    base_path
                };
                targets.push(ConfigAuditTarget::new(
                    "factory-droid",
                    "project",
                    session.environment.clone(),
                    target_path,
                ));
            }
            _ => {}
        }
    }

    targets
}

fn merge_config_targets(
    base_targets: Vec<ConfigAuditTarget>,
    derived_targets: Vec<ConfigAuditTarget>,
) -> Vec<ConfigAuditTarget> {
    let mut seen = HashSet::new();
    let mut merged = Vec::new();

    for target in base_targets.into_iter().chain(derived_targets) {
        let key = format!(
            "{}:{}:{}:{}",
            target.assistant,
            target.scope,
            target.source_layer,
            target.path.display()
        );
        if seen.insert(key) {
            merged.push(target);
        }
    }

    merged
}

fn build_metrics(
    sessions: &[SessionDetailRecord],
    configs: &[ConfigRiskRecord],
) -> Vec<DashboardMetric> {
    let high_value_count = sessions
        .iter()
        .filter(|session| session.value_score >= 70)
        .count();
    let risky_config_count = configs
        .iter()
        .filter(|config| config.official_or_proxy == "proxy" || !config.risks.is_empty())
        .count();
    let reclaim_bytes = sessions
        .iter()
        .filter(|session| {
            session
                .risk_flags
                .iter()
                .any(|flag| flag == "likely_garbage")
        })
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

fn build_audit_records(connection: Option<&Connection>) -> SnapshotResult<Vec<AuditEventRecord>> {
    let Some(connection) = connection else {
        return Ok(Vec::new());
    };
    let events = load_audit_events(connection, 100)?;

    Ok(events.into_iter().map(build_audit_record).collect())
}

fn build_audit_record(event: AuditEvent) -> AuditEventRecord {
    let detail = summarize_audit_event(&event);
    let paths = parse_audit_paths(&event.after_state);

    AuditEventRecord {
        event_id: event.event_id.clone(),
        r#type: event.event_type.clone(),
        target: event.target_id.clone(),
        actor: event.actor,
        created_at: event.created_at,
        result: event.result.clone(),
        detail,
        output_path: paths.output_path,
        quarantined_path: paths.quarantined_path,
        manifest_path: paths.manifest_path,
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

#[derive(Default)]
struct AuditEventPaths {
    output_path: Option<String>,
    quarantined_path: Option<String>,
    manifest_path: Option<String>,
}

fn parse_audit_paths(after_state: &Option<String>) -> AuditEventPaths {
    let Some(after_state) = after_state else {
        return AuditEventPaths::default();
    };

    let Ok(parsed) = serde_json::from_str::<Value>(after_state) else {
        return AuditEventPaths::default();
    };

    AuditEventPaths {
        output_path: parsed
            .get("output_path")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        quarantined_path: parsed
            .get("quarantined_path")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        manifest_path: parsed
            .get("manifest_path")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    }
}

fn extract_session_narrative(session: &SessionRecord) -> SnapshotResult<SessionNarrative> {
    match session.assistant.as_str() {
        "codex" => extract_codex_narrative(Path::new(&session.source_path)),
        "claude-code" => extract_claude_narrative(Path::new(&session.source_path)),
        "opencode" => extract_opencode_narrative(Path::new(&session.source_path)),
        "gemini-cli" => extract_gemini_narrative(Path::new(&session.source_path)),
        "github-copilot-cli" => extract_copilot_narrative(Path::new(&session.source_path)),
        "factory-droid" => extract_factory_droid_narrative(Path::new(&session.source_path)),
        "openclaw" => extract_openclaw_narrative(Path::new(&session.source_path)),
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
        let message = extract_text_array(payload.get("content"))
            .and_then(|value| normalize_session_text(&value));

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
                    .and_then(extract_claude_message_text)
                    .and_then(|value| normalize_session_text(&value));
            }
            Some("assistant") => {
                if let Some(message) = line
                    .get("message")
                    .and_then(|message| message.get("content"))
                    .and_then(extract_claude_message_text)
                    .and_then(|value| normalize_session_text(&value))
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
    let mut messages = Vec::new();
    let mut narrative = SessionNarrative {
        first_user_goal: session_info
            .get("title")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        ..Default::default()
    };

    for message_file in message_files {
        let message: Value = serde_json::from_slice(&fs::read(&message_file)?)?;
        messages.push((opencode_created_at(&message), message));
    }
    messages.sort_by_key(|(created_at, _)| *created_at);

    for (_, message) in messages {
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

fn extract_gemini_narrative(source: &Path) -> SnapshotResult<SessionNarrative> {
    let parsed: Value = serde_json::from_slice(&fs::read(source)?)?;
    let messages = gemini_messages(&parsed);
    let mut narrative = SessionNarrative::default();

    for message in messages {
        let text = gemini_text(message);
        match gemini_role(message) {
            Some("user") if narrative.first_user_goal.is_none() => {
                narrative.first_user_goal = text;
            }
            Some("assistant") => {
                if let Some(text) = text {
                    if looks_like_error_message(&text) {
                        narrative.error_count += 1;
                    }
                    narrative.last_assistant_message = Some(text);
                }
            }
            _ => {}
        }

        for tool_call in gemini_tool_calls(message) {
            if let Some(output) = tool_call
                .get("resultDisplay")
                .or_else(|| tool_call.get("output"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|output| !output.is_empty())
            {
                if looks_like_error_message(output) {
                    narrative.error_count += 1;
                }
                narrative.last_assistant_message = Some(output.to_string());
            }
        }
    }

    Ok(narrative)
}

fn extract_copilot_narrative(source: &Path) -> SnapshotResult<SessionNarrative> {
    let lines = read_jsonl(source)?;
    let mut narrative = SessionNarrative::default();

    for line in lines {
        let data = line.get("data").unwrap_or(&Value::Null);

        match line.get("type").and_then(Value::as_str) {
            Some("user.message") if narrative.first_user_goal.is_none() => {
                narrative.first_user_goal = data
                    .get("content")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty())
                    .map(ToOwned::to_owned);
            }
            Some("assistant.message") => {
                if let Some(content) = data
                    .get("content")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty())
                {
                    if looks_like_error_message(content) {
                        narrative.error_count += 1;
                    }
                    narrative.last_assistant_message = Some(content.to_string());
                }
            }
            Some("tool.execution_complete") => {
                let success = data.get("success").and_then(Value::as_bool).unwrap_or(true);
                let output = data
                    .get("result")
                    .and_then(|result| result.get("content"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty());

                if !success {
                    narrative.error_count += 1;
                }

                if let Some(output) = output
                    && (!success || looks_like_error_message(output))
                {
                    narrative.error_count += 1;
                }
            }
            _ => {}
        }
    }

    Ok(narrative)
}

fn extract_factory_droid_narrative(source: &Path) -> SnapshotResult<SessionNarrative> {
    match detect_droid_dialect(source)? {
        DroidDialect::SessionStore => extract_factory_droid_session_store_narrative(source),
        DroidDialect::StreamJson => extract_factory_droid_stream_narrative(source),
    }
}

fn extract_factory_droid_session_store_narrative(
    source: &Path,
) -> SnapshotResult<SessionNarrative> {
    let lines = read_jsonl(source)?;
    let mut narrative = SessionNarrative::default();

    for line in lines {
        if line
            .get("type")
            .and_then(Value::as_str)
            .map(normalize_droid_kind)
            .as_deref()
            != Some("message")
        {
            continue;
        }

        let Some(message) = line.get("message") else {
            continue;
        };
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let parts = message
            .get("content")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let text = parts
            .iter()
            .filter(|part| {
                part.get("type")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| normalize_droid_kind(kind) == "text")
            })
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        if role == "user" && narrative.first_user_goal.is_none() && !text.is_empty() {
            narrative.first_user_goal = Some(text.clone());
        }

        if role == "assistant" {
            if !text.is_empty() {
                if looks_like_error_message(&text) {
                    narrative.error_count += 1;
                }
                narrative.last_assistant_message = Some(text);
            }

            for part in &parts {
                if part
                    .get("type")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| normalize_droid_kind(kind) == "toolresult")
                    && let Some(content) = part.get("content").and_then(Value::as_str)
                    && looks_like_error_message(content)
                {
                    narrative.error_count += 1;
                }
            }
        }
    }

    Ok(narrative)
}

fn extract_factory_droid_stream_narrative(source: &Path) -> SnapshotResult<SessionNarrative> {
    let lines = read_jsonl(source)?;
    let mut narrative = SessionNarrative::default();

    for line in lines {
        match line
            .get("type")
            .and_then(Value::as_str)
            .map(normalize_droid_kind)
            .as_deref()
        {
            Some("message") => {
                let role = line.get("role").and_then(Value::as_str).unwrap_or_default();
                let content = line
                    .get("content")
                    .or_else(|| line.get("text"))
                    .or_else(|| line.get("message"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty());

                match (role, content) {
                    ("user", Some(content)) if narrative.first_user_goal.is_none() => {
                        narrative.first_user_goal = Some(content.to_string());
                    }
                    ("assistant", Some(content)) => {
                        if looks_like_error_message(content) {
                            narrative.error_count += 1;
                        }
                        narrative.last_assistant_message = Some(content.to_string());
                    }
                    _ => {}
                }
            }
            Some("completion") => {
                if let Some(content) = line
                    .get("finalText")
                    .or_else(|| line.get("final"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty())
                {
                    if looks_like_error_message(content) {
                        narrative.error_count += 1;
                    }
                    narrative.last_assistant_message = Some(content.to_string());
                }
            }
            Some("toolresult") => {
                let value = line.get("value").unwrap_or(&Value::Null);
                let exit_code = value
                    .get("exitCode")
                    .or_else(|| value.get("exit_code"))
                    .and_then(Value::as_i64)
                    .unwrap_or(0);
                let is_error = line
                    .get("isError")
                    .or_else(|| line.get("is_error"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false);

                if is_error || exit_code != 0 {
                    narrative.error_count += 1;
                }
            }
            Some("error") => narrative.error_count += 1,
            _ => {}
        }
    }

    Ok(narrative)
}

fn extract_openclaw_narrative(source: &Path) -> SnapshotResult<SessionNarrative> {
    let lines = read_jsonl(source)?;
    let mut narrative = SessionNarrative::default();

    for line in lines {
        if openclaw_kind(&line) != Some("message") {
            continue;
        }

        let Some(message) = line.get("message") else {
            continue;
        };

        match openclaw_role(message) {
            Some("user") if narrative.first_user_goal.is_none() => {
                narrative.first_user_goal = openclaw_text(message);
            }
            Some("assistant") => {
                if let Some(content) = openclaw_text(message) {
                    if looks_like_error_message(&content) {
                        narrative.error_count += 1;
                    }
                    narrative.last_assistant_message = Some(content);
                }
            }
            Some("toolresult") => {
                if let Some(content) = openclaw_text(message)
                    && looks_like_error_message(&content)
                {
                    narrative.error_count += 1;
                }
            }
            _ => {}
        }
    }

    Ok(narrative)
}

fn opencode_created_at(message: &Value) -> i64 {
    message
        .get("time")
        .and_then(|time| time.get("created"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

fn collect_opencode_part_texts(part_dir: &Path) -> SnapshotResult<Vec<String>> {
    let mut files = collect_files(part_dir, &|path| {
        path.extension().and_then(|value| value.to_str()) == Some("json")
    })?;
    files.sort();

    let mut texts = Vec::new();
    for file in files {
        let part: Value = serde_json::from_slice(&fs::read(file)?)?;
        if part.get("type").and_then(Value::as_str) == Some("text")
            && let Some(text) = part.get("text").and_then(Value::as_str)
        {
            texts.push(text.to_string());
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
            parts
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|text| !text.trim().is_empty())
}

fn extract_claude_message_text(content: &Value) -> Option<String> {
    match content {
        Value::String(value) => Some(value.to_string()),
        Value::Array(parts) => {
            let joined = parts
                .iter()
                .filter_map(|part| match part.get("type").and_then(Value::as_str) {
                    Some("text" | "input_text" | "output_text") => {
                        part.get("text").and_then(Value::as_str)
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ");
            (!joined.trim().is_empty()).then_some(joined)
        }
        _ => None,
    }
}

fn session_adapter(assistant: &str) -> SnapshotResult<Box<dyn SessionAdapter>> {
    match assistant {
        "codex" => Ok(Box::new(CodexAdapter)),
        "claude-code" => Ok(Box::new(ClaudeCodeAdapter)),
        "opencode" => Ok(Box::new(OpenCodeAdapter)),
        "gemini-cli" => Ok(Box::new(GeminiCliAdapter)),
        "github-copilot-cli" => Ok(Box::new(CopilotCliAdapter)),
        "factory-droid" => Ok(Box::new(FactoryDroidAdapter)),
        "openclaw" => Ok(Box::new(OpenClawAdapter)),
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

fn derive_confidence(narrative: &SessionNarrative) -> f32 {
    match (
        narrative.first_user_goal.is_some(),
        narrative.last_assistant_message.is_some(),
    ) {
        (true, true) => 0.92,
        (true, false) | (false, true) => 0.78,
        (false, false) => 0.6,
    }
}

fn normalize_project_path(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unknown") {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process::Command,
        sync::atomic::{AtomicU64, Ordering},
        thread,
        time::Duration,
    };

    use rusqlite::Connection;
    use serde_json::Value;

    use crate::discovery::DiscoveryContext;

    use super::{
        build_fixture_dashboard_snapshot, build_local_dashboard_snapshot,
        build_local_dashboard_snapshot_with_audit,
    };

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn local_snapshot_skips_invalid_session_files_and_keeps_valid_sessions() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");
        let codex_root = home_dir.join(".codex").join("sessions");
        let claude_root = home_dir.join(".claude").join("projects");

        fs::create_dir_all(&codex_root).expect("create codex root");
        fs::create_dir_all(&claude_root).expect("create claude root");

        let codex_fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../tests/fixtures/codex/2026/03/15/rollout-2026-03-15T12-00-00-codex-ses-1.jsonl",
        );
        fs::copy(&codex_fixture, codex_root.join("rollout-2026-03-15.jsonl"))
            .expect("copy codex fixture");

        fs::write(
            claude_root.join("broken-session.jsonl"),
            concat!(
                "{\"type\":\"user\",\"timestamp\":\"2026-03-15T10:00:00Z\",",
                "\"cwd\":\"C:/Projects/broken-session\",",
                "\"message\":{\"content\":\"collect local sessions\"}}\n"
            ),
        )
        .expect("write invalid claude session");

        let snapshot = build_local_dashboard_snapshot(&DiscoveryContext {
            home_dir,
            xdg_config_home: None,
            xdg_data_home: None,
            wsl_home_dir: None,
        })
        .expect("snapshot should skip malformed session files");

        assert_eq!(snapshot.sessions.len(), 1);
        assert_eq!(snapshot.sessions[0].session_id, "codex-ses-1");
    }

    #[test]
    fn fixture_snapshot_includes_transcript_digest() {
        let snapshot =
            build_fixture_dashboard_snapshot(&fixtures_root()).expect("fixture snapshot builds");

        let claude_session = snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "claude-ses-1")
            .expect("claude fixture session exists");
        let opencode_session = snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "ses_demo")
            .expect("opencode fixture session exists");
        let gemini_session = snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "gemini-ses-1")
            .expect("gemini fixture session exists");
        let copilot_session = snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "copilot-ses-1")
            .expect("copilot fixture session exists");
        let droid_session = snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "droid-session-1")
            .expect("droid session-store fixture session exists");
        let droid_stream = snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "droid-stream-1")
            .expect("droid stream-json fixture session exists");
        let openclaw_session = snapshot
            .sessions
            .iter()
            .find(|session| session.session_id == "openclaw-ses-1")
            .expect("openclaw fixture session exists");
        let gemini_config = snapshot
            .configs
            .iter()
            .find(|config| config.assistant == "gemini-cli")
            .expect("gemini config fixture exists");
        let openclaw_config = snapshot
            .configs
            .iter()
            .find(|config| config.assistant == "openclaw")
            .expect("openclaw config fixture exists");

        assert_eq!(claude_session.transcript_highlights[0].role, "User");
        assert!(
            claude_session.transcript_highlights[0]
                .content
                .contains("扫描 Claude transcripts")
        );
        assert_eq!(claude_session.todo_items.len(), 2);
        assert!(claude_session.todo_items[0].completed);
        assert_eq!(opencode_session.transcript_highlights[1].role, "Assistant");
        assert!(
            gemini_session
                .transcript_highlights
                .iter()
                .any(|highlight| { highlight.content.contains("Audit Gemini session retention") })
        );
        assert!(
            copilot_session
                .transcript_highlights
                .iter()
                .any(|highlight| { highlight.content.contains("stale branch") })
        );
        assert!(
            droid_session
                .transcript_highlights
                .iter()
                .any(|highlight| { highlight.content.contains("stale session") })
        );
        assert!(
            droid_stream
                .transcript_highlights
                .iter()
                .any(|highlight| { highlight.content.contains("No dirty files were detected") })
        );
        assert!(
            openclaw_session
                .transcript_highlights
                .iter()
                .any(|highlight| {
                    highlight
                        .content
                        .contains("Review OpenClaw transcripts and flag cleanup candidates")
                })
        );
        assert_eq!(gemini_config.provider, "google");
        assert_eq!(openclaw_config.provider, "openrouter");
    }

    #[test]
    fn fixture_snapshot_includes_usage_analytics() {
        let snapshot =
            build_fixture_dashboard_snapshot(&fixtures_root()).expect("fixture snapshot builds");
        let serialized = serde_json::to_value(&snapshot).expect("snapshot serializes");
        let usage_overview = serialized
            .get("usageOverview")
            .expect("usage overview should be present");
        let usage_timeline = serialized
            .get("usageTimeline")
            .and_then(serde_json::Value::as_array)
            .expect("usage timeline should be present");
        let assistants = usage_overview
            .get("assistants")
            .and_then(serde_json::Value::as_array)
            .expect("assistant usage list exists");
        let opencode_usage = assistants
            .iter()
            .find(|assistant| {
                assistant
                    .get("assistant")
                    .and_then(serde_json::Value::as_str)
                    == Some("opencode")
            })
            .expect("opencode usage aggregate exists");
        let session_usage = serialized
            .get("sessions")
            .and_then(serde_json::Value::as_array)
            .expect("sessions array exists")
            .iter()
            .find(|session| {
                session.get("sessionId").and_then(serde_json::Value::as_str) == Some("ses_demo")
            })
            .and_then(|session| session.get("usage"))
            .expect("opencode session usage should be present");

        assert_eq!(
            usage_overview
                .get("totals")
                .and_then(|totals| totals.get("sessionsWithUsage"))
                .and_then(serde_json::Value::as_u64),
            Some(5)
        );
        assert_eq!(
            opencode_usage
                .get("totalTokens")
                .and_then(serde_json::Value::as_u64),
            Some(210)
        );
        assert_eq!(
            opencode_usage
                .get("costUsd")
                .and_then(serde_json::Value::as_f64),
            Some(0.02)
        );
        assert_eq!(
            opencode_usage
                .get("costSource")
                .and_then(serde_json::Value::as_str),
            Some("reported")
        );
        assert_eq!(
            session_usage
                .get("model")
                .and_then(serde_json::Value::as_str),
            Some("gpt-5")
        );
        assert_eq!(
            session_usage
                .get("cacheReadTokens")
                .and_then(serde_json::Value::as_u64),
            Some(0)
        );
        assert_eq!(
            session_usage
                .get("totalTokens")
                .and_then(serde_json::Value::as_u64),
            Some(210)
        );
        let timeline_2025 = usage_timeline
            .iter()
            .find(|entry| {
                entry.get("date").and_then(serde_json::Value::as_str) == Some("2025-03-15")
            })
            .expect("2025 timeline bucket exists");
        let timeline_2026 = usage_timeline
            .iter()
            .find(|entry| {
                entry.get("date").and_then(serde_json::Value::as_str) == Some("2026-03-15")
            })
            .expect("2026 timeline bucket exists");
        assert_eq!(
            timeline_2025
                .get("sessionsWithUsage")
                .and_then(serde_json::Value::as_u64),
            Some(1)
        );
        assert_eq!(
            timeline_2025
                .get("totalTokens")
                .and_then(serde_json::Value::as_u64),
            Some(210)
        );
        assert_eq!(
            timeline_2025
                .get("costUsd")
                .and_then(serde_json::Value::as_f64),
            Some(0.02)
        );
        assert_eq!(
            timeline_2025
                .get("costSource")
                .and_then(serde_json::Value::as_str),
            Some("reported")
        );
        assert_eq!(
            timeline_2026
                .get("sessionsWithUsage")
                .and_then(serde_json::Value::as_u64),
            Some(4)
        );
        assert_eq!(
            timeline_2026
                .get("totalTokens")
                .and_then(serde_json::Value::as_u64),
            Some(115092)
        );
        assert!(timeline_2026.get("costUsd").is_none());
        assert_eq!(
            timeline_2026
                .get("costSource")
                .and_then(serde_json::Value::as_str),
            Some("unknown")
        );
    }

    #[test]
    fn local_snapshot_ignores_codex_scaffolding_when_deriving_title_and_highlights() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");
        let codex_dir = home_dir
            .join(".codex")
            .join("sessions")
            .join("2026")
            .join("03")
            .join("16");

        fs::create_dir_all(&codex_dir).expect("create codex dir");
        fs::write(
            codex_dir.join("rollout-2026-03-16T12-00-00-codex-real.jsonl"),
            concat!(
                "{\"timestamp\":\"2026-03-16T12:00:00Z\",\"type\":\"session_meta\",\"payload\":",
                "{\"id\":\"codex-real-1\",\"cwd\":\"C:\\\\Projects\\\\osm\"}}\n",
                "{\"timestamp\":\"2026-03-16T12:00:01Z\",\"type\":\"response_item\",\"payload\":",
                "{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",",
                "\"text\":\"# AGENTS.md instructions for C:\\\\Users\\\\Max\\n\\n<INSTRUCTIONS>### Available skills</INSTRUCTIONS>\"}]}}\n",
                "{\"timestamp\":\"2026-03-16T12:00:02Z\",\"type\":\"response_item\",\"payload\":",
                "{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",",
                "\"text\":\"<environment_context> <cwd>C:\\\\Projects\\\\osm</cwd> <shell>powershell</shell> </environment_context>\"}]}}\n",
                "{\"timestamp\":\"2026-03-16T12:00:03Z\",\"type\":\"response_item\",\"payload\":",
                "{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",",
                "\"text\":\"Fix why the session list keeps showing the first row and won't surface the real topic.\"}]}}\n",
                "{\"timestamp\":\"2026-03-16T12:00:05Z\",\"type\":\"response_item\",\"payload\":",
                "{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",",
                "\"text\":\"I traced it to scaffolding text polluting the session title and highlights.\"}]}}\n"
            ),
        )
        .expect("write codex session");

        let snapshot = build_local_dashboard_snapshot(&DiscoveryContext {
            home_dir,
            xdg_config_home: None,
            xdg_data_home: None,
            wsl_home_dir: None,
        })
        .expect("snapshot should build");

        assert_eq!(snapshot.sessions.len(), 1);
        assert!(
            snapshot.sessions[0]
                .title
                .starts_with("Fix why the session list keeps showing the first row")
        );
        assert!(
            !snapshot.sessions[0]
                .title
                .contains("AGENTS.md instructions")
        );
        assert_eq!(
            snapshot.sessions[0].transcript_highlights[0].content,
            "Fix why the session list keeps showing the first row and won't surface the real topic."
        );
        assert!(
            !snapshot.sessions[0]
                .transcript_highlights
                .iter()
                .any(|highlight| highlight.content.contains("AGENTS.md instructions"))
        );
    }

    #[test]
    fn local_snapshot_discovers_copilot_and_factory_configs() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");
        let copilot_dir = home_dir.join(".copilot");
        let factory_dir = home_dir.join(".factory");

        fs::create_dir_all(&copilot_dir).expect("create copilot dir");
        fs::create_dir_all(&factory_dir).expect("create factory dir");

        fs::copy(
            fixtures_root().join("configs/copilot/config.json"),
            copilot_dir.join("config.json"),
        )
        .expect("copy copilot config");
        fs::copy(
            fixtures_root().join("configs/copilot/mcp-config.json"),
            copilot_dir.join("mcp-config.json"),
        )
        .expect("copy copilot mcp config");
        fs::copy(
            fixtures_root().join("configs/factory/settings.json"),
            factory_dir.join("settings.json"),
        )
        .expect("copy factory config");
        fs::copy(
            fixtures_root().join("configs/factory/settings.local.json"),
            factory_dir.join("settings.local.json"),
        )
        .expect("copy factory local config");

        let snapshot = build_local_dashboard_snapshot(&DiscoveryContext {
            home_dir,
            xdg_config_home: None,
            xdg_data_home: None,
            wsl_home_dir: None,
        })
        .expect("snapshot should build");

        let copilot_config = snapshot
            .configs
            .iter()
            .find(|config| config.assistant == "github-copilot-cli")
            .expect("copilot config should be discovered");
        let factory_config = snapshot
            .configs
            .iter()
            .find(|config| config.assistant == "factory-droid")
            .expect("factory config should be discovered");

        assert_eq!(copilot_config.model.as_deref(), Some("gpt-5"));
        assert!(
            copilot_config
                .risks
                .iter()
                .any(|risk| risk == "dangerous_permissions")
        );
        assert_eq!(
            factory_config.model.as_deref(),
            Some("openrouter/anthropic/claude-sonnet-4")
        );
        assert_eq!(factory_config.masked_secret, "***7890");
    }

    #[test]
    fn local_snapshot_discovers_project_level_copilot_and_factory_configs() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");
        let copilot_session_root = home_dir.join(".copilot").join("session-state");
        let factory_session_root = home_dir.join(".factory").join("sessions").join("project-a");
        let copilot_project = sandbox.join("projects").join("copilot-project");
        let factory_project = sandbox.join("projects").join("factory-project");
        let copilot_config_dir = copilot_project.join(".github").join("copilot");
        let factory_config_dir = factory_project.join(".factory");
        let copilot_project_json = serde_json::to_string(&copilot_project.display().to_string())
            .expect("serialize copilot project path");
        let factory_project_json = serde_json::to_string(&factory_project.display().to_string())
            .expect("serialize factory project path");

        fs::create_dir_all(&copilot_session_root).expect("create copilot session root");
        fs::create_dir_all(&factory_session_root).expect("create factory session root");
        fs::create_dir_all(&copilot_config_dir).expect("create copilot config dir");
        fs::create_dir_all(&factory_config_dir).expect("create factory config dir");

        fs::copy(
            fixtures_root().join("configs/copilot/project/.github/copilot/settings.json"),
            copilot_config_dir.join("settings.json"),
        )
        .expect("copy copilot project settings");
        fs::copy(
            fixtures_root().join("configs/copilot/project/.github/copilot/settings.local.json"),
            copilot_config_dir.join("settings.local.json"),
        )
        .expect("copy copilot project local settings");
        fs::copy(
            fixtures_root().join("configs/factory/project/.factory/settings.json"),
            factory_config_dir.join("settings.json"),
        )
        .expect("copy factory project settings");
        fs::copy(
            fixtures_root().join("configs/factory/project/.factory/settings.local.json"),
            factory_config_dir.join("settings.local.json"),
        )
        .expect("copy factory project local settings");

        fs::write(
            copilot_session_root.join("copilot-project.jsonl"),
            format!(
                concat!(
                    "{{\"type\":\"session.start\",\"data\":{{\"sessionId\":\"copilot-project-1\"}},",
                    "\"timestamp\":\"2026-03-16T10:00:00.000Z\",\"id\":\"evt-1\"}}\n",
                    "{{\"type\":\"session.info\",\"data\":{{\"infoType\":\"folder_trust\",",
                    "\"message\":\"Folder {} has been added to trusted folders.\"}},",
                    "\"timestamp\":\"2026-03-16T10:00:01.000Z\",\"id\":\"evt-2\"}}\n",
                    "{{\"type\":\"user.message\",\"data\":{{\"content\":\"Audit project-level Copilot config.\"}},",
                    "\"timestamp\":\"2026-03-16T10:00:02.000Z\",\"id\":\"evt-3\"}}\n"
                ),
                copilot_project_json.trim_matches('"')
            ),
        )
        .expect("write copilot project session");

        fs::write(
            factory_session_root.join("droid-project.jsonl"),
            format!(
                concat!(
                    "{{\"type\":\"session_start\",\"id\":\"factory-project-1\",\"cwd\":\"{}\",",
                    "\"timestamp\":\"2026-03-16T11:00:00.000Z\",\"title\":\"Factory project config\"}}\n",
                    "{{\"type\":\"message\",\"timestamp\":\"2026-03-16T11:00:03.000Z\",",
                    "\"id\":\"msg-user-1\",\"message\":{{\"role\":\"user\",\"content\":[{{\"type\":\"text\",",
                    "\"text\":\"Audit project-level Factory config.\"}}]}}}}\n"
                ),
                factory_project_json.trim_matches('"')
            ),
        )
        .expect("write factory project session");

        let snapshot = build_local_dashboard_snapshot(&DiscoveryContext {
            home_dir,
            xdg_config_home: None,
            xdg_data_home: None,
            wsl_home_dir: None,
        })
        .expect("snapshot should build");

        let copilot_config = snapshot
            .configs
            .iter()
            .find(|config| config.assistant == "github-copilot-cli" && config.scope == "project")
            .expect("copilot project config should be discovered");
        let factory_config = snapshot
            .configs
            .iter()
            .find(|config| config.assistant == "factory-droid" && config.scope == "project")
            .expect("factory project config should be discovered");

        assert!(copilot_config.path.ends_with("settings.local.json"));
        assert_eq!(copilot_config.model.as_deref(), Some("gpt-5"));
        assert!(
            copilot_config
                .risks
                .iter()
                .any(|risk| risk == "dangerous_permissions")
        );
        assert!(factory_config.path.ends_with("settings.local.json"));
        assert_eq!(
            factory_config.model.as_deref(),
            Some("openrouter/anthropic/claude-sonnet-4")
        );
        assert_eq!(factory_config.masked_secret, "***7890");
    }

    #[test]
    fn local_snapshot_reuses_cached_index_for_unchanged_sessions() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");
        let codex_root = home_dir.join(".codex").join("sessions");
        let audit_db_path = sandbox.join("audit").join("audit.db");

        fs::create_dir_all(&codex_root).expect("create codex root");
        fs::create_dir_all(audit_db_path.parent().expect("audit dir")).expect("create audit dir");
        fs::copy(
            fixtures_root().join("codex/2026/03/15/rollout-2026-03-15T12-00-00-codex-ses-1.jsonl"),
            codex_root.join("rollout-2026-03-15.jsonl"),
        )
        .expect("copy codex fixture");

        let context = DiscoveryContext {
            home_dir,
            xdg_config_home: None,
            xdg_data_home: None,
            wsl_home_dir: None,
        };

        build_local_dashboard_snapshot_with_audit(&context, Some(&audit_db_path))
            .expect("first snapshot should build");
        build_local_dashboard_snapshot_with_audit(&context, Some(&audit_db_path))
            .expect("second snapshot should build");

        let connection = Connection::open(&audit_db_path).expect("open audit db");
        let latest_run = connection
            .query_row(
                "SELECT discovered_files, cache_hits, reindexed_files
                 FROM session_index_runs
                 ORDER BY started_at DESC
                 LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .expect("index run stats should be persisted");

        assert_eq!(latest_run.0, 1);
        assert_eq!(latest_run.1, 1);
        assert_eq!(latest_run.2, 0);
    }

    #[test]
    fn local_snapshot_reindexes_only_changed_sessions_incrementally() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");
        let codex_root = home_dir.join(".codex").join("sessions");
        let claude_root = home_dir
            .join(".claude")
            .join("projects")
            .join("C--Projects-Claude-Demo");
        let audit_db_path = sandbox.join("audit").join("audit.db");

        fs::create_dir_all(&codex_root).expect("create codex root");
        fs::create_dir_all(&claude_root).expect("create claude root");
        fs::create_dir_all(audit_db_path.parent().expect("audit dir")).expect("create audit dir");
        let codex_target = codex_root.join("rollout-2026-03-15.jsonl");
        let claude_target = claude_root.join("claude-ses-1.jsonl");
        fs::copy(
            fixtures_root().join("codex/2026/03/15/rollout-2026-03-15T12-00-00-codex-ses-1.jsonl"),
            &codex_target,
        )
        .expect("copy codex fixture");
        fs::copy(
            fixtures_root().join("claude/projects/C--Projects-Claude-Demo/claude-ses-1.jsonl"),
            &claude_target,
        )
        .expect("copy claude fixture");

        let context = DiscoveryContext {
            home_dir,
            xdg_config_home: None,
            xdg_data_home: None,
            wsl_home_dir: None,
        };

        build_local_dashboard_snapshot_with_audit(&context, Some(&audit_db_path))
            .expect("first snapshot should build");

        thread::sleep(Duration::from_millis(20));
        fs::write(
            &codex_target,
            concat!(
                "{\"timestamp\":\"2026-03-15T04:00:00.000Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"codex-ses-1\",\"timestamp\":\"2026-03-15T04:00:00.000Z\",\"cwd\":\"C:\\\\Projects\\\\demo\",\"originator\":\"codex_cli_rs\",\"cli_version\":\"0.97.0\",\"source\":\"cli\"}}\n",
                "{\"timestamp\":\"2026-03-15T04:00:03.000Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"整理本地 agent 会话\"}]}}\n",
                "{\"timestamp\":\"2026-03-15T04:00:06.000Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"先扫描 Codex 和 Claude 的 transcript 目录。\"}]}}\n",
                "{\"timestamp\":\"2026-03-15T04:00:09.000Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"补一条新的缓存命中验证消息。\"}]}}\n"
            ),
        )
        .expect("mutate codex session");

        build_local_dashboard_snapshot_with_audit(&context, Some(&audit_db_path))
            .expect("second snapshot should build");

        let connection = Connection::open(&audit_db_path).expect("open audit db");
        let latest_run = connection
            .query_row(
                "SELECT discovered_files, cache_hits, reindexed_files
                 FROM session_index_runs
                 ORDER BY started_at DESC
                 LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .expect("index run stats should be persisted");

        assert_eq!(latest_run.0, 2);
        assert_eq!(latest_run.1, 1);
        assert_eq!(latest_run.2, 1);
    }

    #[test]
    fn local_snapshot_emits_git_project_status_and_recent_commits() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");
        let repo_root = sandbox.join("repos").join("git-demo");
        let codex_root = home_dir.join(".codex").join("sessions").join("2026").join("03");

        fs::create_dir_all(&repo_root).expect("create repo root");
        fs::create_dir_all(&codex_root).expect("create codex root");

        git(&repo_root, &["init"]);
        git(&repo_root, &["branch", "-M", "main"]);
        git(&repo_root, &["config", "user.name", "OSM Test"]);
        git(&repo_root, &["config", "user.email", "osm-test@example.com"]);

        fs::write(repo_root.join("README.md"), "# git demo\n").expect("seed readme");
        git(&repo_root, &["add", "."]);
        git(&repo_root, &["commit", "-m", "feat: seed git dashboard"]);
        fs::write(repo_root.join("notes.txt"), "dirty working tree\n").expect("seed dirty file");

        let escaped_repo_path = repo_root.display().to_string().replace('\\', "\\\\");
        fs::write(
            codex_root.join("rollout-2026-03-23.jsonl"),
            format!(
                concat!(
                    "{{\"timestamp\":\"2026-03-23T04:00:00.000Z\",\"type\":\"session_meta\",",
                    "\"payload\":{{\"id\":\"codex-git-001\",\"timestamp\":\"2026-03-23T04:00:00.000Z\",",
                    "\"cwd\":\"{}\",\"originator\":\"codex_cli_rs\",",
                    "\"cli_version\":\"0.97.0\",\"source\":\"cli\"}}}}\n",
                    "{{\"timestamp\":\"2026-03-23T04:00:03.000Z\",\"type\":\"response_item\",",
                    "\"payload\":{{\"type\":\"message\",\"role\":\"user\",\"content\":[{{\"type\":\"input_text\",",
                    "\"text\":\"Inspect git governance state\"}}]}}}}\n",
                    "{{\"timestamp\":\"2026-03-23T04:00:06.000Z\",\"type\":\"response_item\",",
                    "\"payload\":{{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{{\"type\":\"output_text\",",
                    "\"text\":\"Git snapshot is pending.\"}}]}}}}\n"
                ),
                escaped_repo_path
            ),
        )
        .expect("write codex session");

        let snapshot = build_local_dashboard_snapshot(&DiscoveryContext {
            home_dir,
            xdg_config_home: None,
            xdg_data_home: None,
            wsl_home_dir: None,
        })
        .expect("snapshot should build");

        let serialized = serde_json::to_value(&snapshot).expect("serialize snapshot");
        let git_projects = serialized
            .get("gitProjects")
            .and_then(Value::as_array)
            .expect("git projects should be serialized");
        let project = git_projects
            .iter()
            .find(|record| {
                record
                    .get("repoRoot")
                    .and_then(Value::as_str)
                    .is_some_and(|path| path.contains("git-demo"))
            })
            .expect("repo record should exist");

        assert_eq!(
            project.get("branch").and_then(Value::as_str),
            Some("main")
        );
        assert_eq!(
            project.get("status").and_then(Value::as_str),
            Some("dirty")
        );
        assert_eq!(
            project.get("sessionCount").and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            project.get("untrackedFiles").and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(
            project
                .get("recentCommits")
                .and_then(Value::as_array)
                .and_then(|commits| commits.first())
                .and_then(|commit| commit.get("summary"))
                .and_then(Value::as_str),
            Some("feat: seed git dashboard")
        );
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures")
            .canonicalize()
            .expect("fixtures root resolves")
    }

    fn temp_root() -> PathBuf {
        let suffix = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "open-session-manager-dashboard-tests-{}-{suffix}",
            std::process::id(),
        ));

        if root.exists() {
            fs::remove_dir_all(&root).expect("reset temp root");
        }

        fs::create_dir_all(&root).expect("create temp root");
        root
    }

    fn git(repo_root: &std::path::Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_root)
            .output()
            .expect("git command should run");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}
