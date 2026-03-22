use std::{
    env, fs,
    path::PathBuf,
    process::Command,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

use open_session_manager_core::{
    actions::{
        delete::{SoftDeleteRequest, soft_delete_session},
        export::{ExportRequest, export_session_markdown},
        restore::restore_session,
    },
    domain::session::{SessionInsight, SessionRecord},
    storage::sqlite::bootstrap_database,
};
use rusqlite::Connection;
use serde_json::Value;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures")
        .canonicalize()
        .expect("fixtures root resolves")
}

#[test]
fn snapshot_command_emits_real_dashboard_json_from_fixtures() {
    let output = run_fixture_command(["snapshot"]);

    assert!(
        output.status.success(),
        "snapshot command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let snapshot: Value =
        serde_json::from_slice(&output.stdout).expect("snapshot command prints json");

    let sessions = snapshot
        .get("sessions")
        .and_then(Value::as_array)
        .expect("sessions array exists");
    let configs = snapshot
        .get("configs")
        .and_then(Value::as_array)
        .expect("configs array exists");
    let usage_overview = snapshot
        .get("usageOverview")
        .expect("usage overview exists");

    assert_eq!(sessions.len(), 8);
    assert_eq!(configs.len(), 7);
    assert_eq!(
        sessions[0]
            .get("title")
            .and_then(Value::as_str)
            .expect("title exists"),
        "整理本地 agent 会话"
    );
    assert_eq!(
        sessions[0]
            .get("summary")
            .and_then(Value::as_str)
            .expect("summary exists"),
        "先扫描 Codex 和 Claude 的 transcript 目录。"
    );
    assert_eq!(
        configs[0]
            .get("maskedSecret")
            .and_then(Value::as_str)
            .expect("masked secret exists"),
        "***6789"
    );
    assert!(
        configs.iter().any(|config| {
            config.get("assistant").and_then(Value::as_str) == Some("gemini-cli")
                && config.get("model").and_then(Value::as_str)
                    == Some("gemini-2.5-pro-preview-06-05")
        }),
        "fixture snapshot should include Gemini config preview"
    );
    assert!(
        configs.iter().any(|config| {
            config.get("assistant").and_then(Value::as_str) == Some("openclaw")
                && config.get("model").and_then(Value::as_str)
                    == Some("openrouter/anthropic/claude-sonnet-4")
        }),
        "fixture snapshot should include OpenClaw config preview"
    );
    assert!(
        configs.iter().any(|config| {
            config.get("assistant").and_then(Value::as_str) == Some("github-copilot-cli")
                && config.get("model").and_then(Value::as_str) == Some("gpt-5")
                && config
                    .get("risks")
                    .and_then(Value::as_array)
                    .is_some_and(|risks| {
                        risks
                            .iter()
                            .any(|risk| risk.as_str() == Some("dangerous_permissions"))
                    })
        }),
        "fixture snapshot should include Copilot config governance data"
    );
    assert!(
        configs.iter().any(|config| {
            config.get("assistant").and_then(Value::as_str) == Some("factory-droid")
                && config.get("model").and_then(Value::as_str)
                    == Some("openrouter/anthropic/claude-sonnet-4")
                && config.get("maskedSecret").and_then(Value::as_str) == Some("***7890")
        }),
        "fixture snapshot should include Factory Droid config governance data"
    );

    let assistants = sessions
        .iter()
        .filter_map(|session| session.get("assistant").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(assistants.contains(&"gemini-cli"));
    assert!(assistants.contains(&"github-copilot-cli"));
    assert!(assistants.contains(&"factory-droid"));
    assert!(assistants.contains(&"openclaw"));
    assert_eq!(
        usage_overview
            .get("totals")
            .and_then(|totals| totals.get("sessionsWithUsage"))
            .and_then(Value::as_u64),
        Some(5)
    );
    assert_eq!(
        usage_overview
            .get("assistants")
            .and_then(Value::as_array)
            .and_then(|assistants| {
                assistants.iter().find(|assistant| {
                    assistant.get("assistant").and_then(Value::as_str) == Some("claude-code")
                })
            })
            .and_then(|assistant| assistant.get("costSource"))
            .and_then(Value::as_str),
        Some("estimated")
    );
    assert!(
        sessions.iter().any(|session| {
            session.get("sessionId").and_then(Value::as_str) == Some("openclaw-ses-1")
                && session
                    .get("usage")
                    .and_then(|usage| usage.get("costUsd"))
                    .and_then(Value::as_f64)
                    == Some(0.02)
        }),
        "fixture snapshot should expose session usage payloads"
    );
    let usage_timeline = snapshot
        .get("usageTimeline")
        .and_then(Value::as_array)
        .expect("usage timeline exists");
    assert_eq!(usage_timeline.len(), 2);
    let timeline_2025 = usage_timeline
        .iter()
        .find(|entry| entry.get("date").and_then(Value::as_str) == Some("2025-03-15"))
        .expect("2025 timeline exists");
    let timeline_2026 = usage_timeline
        .iter()
        .find(|entry| entry.get("date").and_then(Value::as_str) == Some("2026-03-15"))
        .expect("2026 timeline exists");
    assert_eq!(
        timeline_2025
            .get("sessionsWithUsage")
            .and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        timeline_2025.get("costUsd").and_then(Value::as_f64),
        Some(0.02)
    );
    assert_eq!(
        timeline_2025.get("costSource").and_then(Value::as_str),
        Some("reported")
    );
    assert_eq!(
        timeline_2026
            .get("sessionsWithUsage")
            .and_then(Value::as_u64),
        Some(4)
    );
    assert!(timeline_2026.get("costUsd").is_none());
    assert_eq!(
        timeline_2026.get("costSource").and_then(Value::as_str),
        Some("unknown")
    );
}

#[test]
fn list_command_emits_session_inventory() {
    let output = run_fixture_command(["list"]);

    assert!(
        output.status.success(),
        "list command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_slice(&output.stdout).expect("list command prints json");
    let sessions = payload
        .get("sessions")
        .and_then(Value::as_array)
        .expect("sessions array exists");

    assert_eq!(sessions.len(), 8);
    assert_eq!(
        sessions[0]
            .get("sessionId")
            .and_then(Value::as_str)
            .expect("session id exists"),
        "codex-ses-1"
    );
    assert_eq!(
        sessions[1]
            .get("title")
            .and_then(Value::as_str)
            .expect("title exists"),
        "扫描 Claude transcripts"
    );
}

#[test]
fn search_command_returns_ranked_hits() {
    let output = run_fixture_command(["search", "--query", "Claude"]);

    assert!(
        output.status.success(),
        "search command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("search command prints json");
    let hits = payload
        .get("hits")
        .and_then(Value::as_array)
        .expect("hits array exists");

    assert_eq!(
        payload
            .get("query")
            .and_then(Value::as_str)
            .expect("query exists"),
        "Claude"
    );
    assert!(
        hits.len() >= 2,
        "expected at least two Claude-related hits, got {hits:?}"
    );
    assert_eq!(
        hits[0]
            .get("sessionId")
            .and_then(Value::as_str)
            .expect("session id exists"),
        "claude-ses-1"
    );
    assert!(
        hits[0]
            .get("snippet")
            .and_then(Value::as_str)
            .is_some_and(|snippet| snippet.contains("Claude")),
        "top hit should preserve a readable snippet"
    );
    assert!(
        hits[0]
            .get("matchReasons")
            .and_then(Value::as_array)
            .is_some_and(|reasons| reasons.iter().any(|reason| reason.as_str() == Some("title"))),
        "top hit should explain that the title matched"
    );
}

#[test]
fn get_command_returns_full_session_detail() {
    let output = run_fixture_command(["get", "--session", "claude-ses-1"]);

    assert!(
        output.status.success(),
        "get command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let session: Value =
        serde_json::from_slice(&output.stdout).expect("get command prints json");

    assert_eq!(
        session
            .get("assistant")
            .and_then(Value::as_str)
            .expect("assistant exists"),
        "claude-code"
    );
    assert_eq!(
        session
            .get("summary")
            .and_then(Value::as_str)
            .expect("summary exists"),
        "已定位项目目录并准备索引。"
    );
    assert_eq!(
        session
            .get("todoItems")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(2)
    );
}

#[test]
fn view_command_renders_markdown_summary() {
    let output = run_fixture_command(["view", "--session", "claude-ses-1"]);

    assert!(
        output.status.success(),
        "view command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("view command prints json");
    let content = payload
        .get("content")
        .and_then(Value::as_str)
        .expect("markdown content exists");

    assert_eq!(
        payload
            .get("sessionId")
            .and_then(Value::as_str)
            .expect("session id exists"),
        "claude-ses-1"
    );
    assert!(content.contains("# 扫描 Claude transcripts"));
    assert!(content.contains("## Summary"));
    assert!(content.contains("## Open Todos"));
}

#[test]
fn expand_command_returns_context_bundle() {
    let output = run_fixture_command(["expand", "--session", "claude-ses-1"]);

    assert!(
        output.status.success(),
        "expand command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value =
        serde_json::from_slice(&output.stdout).expect("expand command prints json");

    assert_eq!(
        payload
            .get("session")
            .and_then(|session| session.get("sessionId"))
            .and_then(Value::as_str)
            .expect("session id exists"),
        "claude-ses-1"
    );
    assert!(
        payload
            .get("relatedConfigs")
            .and_then(Value::as_array)
            .is_some_and(|configs| {
                configs.iter().any(|config| {
                    config.get("assistant").and_then(Value::as_str) == Some("claude-code")
                })
            }),
        "expand should surface related config context"
    );
    assert_eq!(
        payload
            .get("transcriptHighlights")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        payload
            .get("todoItems")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(2)
    );
}

#[test]
fn robot_json_mode_emits_compact_cli_payloads() {
    let list = run_fixture_command(["list", "--json"]);

    assert!(
        list.status.success(),
        "list --json command failed: {}",
        String::from_utf8_lossy(&list.stderr)
    );
    let list_stdout = String::from_utf8(list.stdout).expect("list stdout is utf8");
    assert!(
        !list_stdout.contains("\n  "),
        "robot json mode should emit compact json without pretty indentation"
    );
    let list_payload: Value =
        serde_json::from_str(&list_stdout).expect("list --json prints valid json");
    assert_eq!(
        list_payload
            .get("sessions")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(8)
    );

    let view = run_fixture_command(["view", "--session", "claude-ses-1", "--json"]);

    assert!(
        view.status.success(),
        "view --json command failed: {}",
        String::from_utf8_lossy(&view.stderr)
    );
    let view_stdout = String::from_utf8(view.stdout).expect("view stdout is utf8");
    assert!(
        !view_stdout.contains("\n  "),
        "view --json should emit compact json envelope"
    );
    let view_payload: Value =
        serde_json::from_str(&view_stdout).expect("view --json prints valid json");
    assert_eq!(
        view_payload
            .get("sessionId")
            .and_then(Value::as_str)
            .expect("session id exists"),
        "claude-ses-1"
    );
    assert!(
        view_payload
            .get("content")
            .and_then(Value::as_str)
            .is_some_and(|content| content.contains("# 扫描 Claude transcripts"))
    );
}

#[test]
fn snapshot_command_skips_invalid_local_sessions_without_stderr_noise() {
    let sandbox = temp_root();
    let home_dir = sandbox.join("home");
    let codex_root = home_dir.join(".codex").join("sessions");
    let claude_root = home_dir.join(".claude").join("projects");

    fs::create_dir_all(&codex_root).expect("create codex root");
    fs::create_dir_all(&claude_root).expect("create claude root");

    let codex_fixture =
        fixtures_root().join("codex/2026/03/15/rollout-2026-03-15T12-00-00-codex-ses-1.jsonl");
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

    let output = Command::new(env!("CARGO_BIN_EXE_open-session-manager-core"))
        .env("HOME", &home_dir)
        .env("USERPROFILE", &home_dir)
        .args(["snapshot"])
        .output()
        .expect("snapshot command runs");

    assert!(
        output.status.success(),
        "snapshot command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "desktop-facing snapshot should not emit recoverable parse noise: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let snapshot: Value =
        serde_json::from_slice(&output.stdout).expect("snapshot command prints json");
    let sessions = snapshot
        .get("sessions")
        .and_then(Value::as_array)
        .expect("sessions array exists");
    let doctor_findings = snapshot
        .get("doctorFindings")
        .and_then(Value::as_array)
        .expect("doctor findings array exists");

    assert_eq!(sessions.len(), 1);
    assert_eq!(
        sessions[0]
            .get("sessionId")
            .and_then(Value::as_str)
            .expect("session id exists"),
        "codex-ses-1"
    );
    assert_eq!(doctor_findings.len(), 1);
    assert_eq!(
        doctor_findings[0]
            .get("code")
            .and_then(Value::as_str)
            .expect("code exists"),
        "malformed_session_skipped"
    );
    assert_eq!(
        doctor_findings[0]
            .get("severity")
            .and_then(Value::as_str)
            .expect("severity exists"),
        "warn"
    );
    assert!(
        doctor_findings[0]
            .get("detail")
            .and_then(Value::as_str)
            .is_some_and(|detail| detail.contains("broken-session.jsonl")),
        "doctor finding should preserve the skipped file path"
    );
}

#[test]
fn snapshot_command_includes_persisted_audit_history() {
    let sandbox = temp_root();
    let source_path = sandbox.join("sessions").join("rollout-2026-03-15.jsonl");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(&source_path, "{\"type\":\"response_item\"}\n").expect("write source session");

    let export_root = sandbox.join("exports");
    let quarantine_root = sandbox.join("quarantine");
    let audit_db_path = sandbox.join("audit.db");
    let connection = Connection::open(&audit_db_path).expect("open persistent sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "ses-persisted-1".to_string(),
        installation_id: None,
        assistant: "codex".to_string(),
        environment: "windows".to_string(),
        project_path: Some(r"C:\Projects\demo".to_string()),
        source_path: source_path.display().to_string(),
        started_at: Some("2026-03-15T05:00:00Z".to_string()),
        ended_at: Some("2026-03-15T05:15:00Z".to_string()),
        last_activity_at: Some("2026-03-15T05:15:00Z".to_string()),
        message_count: 10,
        tool_count: 4,
        status: "completed".to_string(),
        raw_format: "codex-jsonl".to_string(),
        content_hash: "abc123".to_string(),
    };

    let insight = SessionInsight {
        session_id: session.session_id.clone(),
        title: "整理 agent 会话".to_string(),
        topic_labels_json: r#"["cleanup","governance"]"#.to_string(),
        summary: "已扫描本地会话并生成清理建议。".to_string(),
        progress_state: "completed".to_string(),
        progress_percent: Some(100),
        value_score: 88,
        stale_score: 12,
        garbage_score: 18,
        risk_flags_json: "[]".to_string(),
        confidence: 0.92,
    };

    export_session_markdown(&ExportRequest {
        session: &session,
        insight: &insight,
        output_root: &export_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("export markdown");

    let manifest = soft_delete_session(&SoftDeleteRequest {
        session: &session,
        quarantine_root: &quarantine_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("soft delete session");

    restore_session(
        &manifest.manifest_path,
        &quarantine_root,
        &[sandbox.join("sessions")],
        "r007b34r",
        &connection,
    )
    .expect("restore session");
    drop(connection);

    let output = Command::new(env!("CARGO_BIN_EXE_open-session-manager-core"))
        .args([
            "snapshot",
            "--fixtures",
            fixtures_root().to_str().expect("fixtures path as str"),
            "--audit-db",
            audit_db_path.to_str().expect("audit db path as str"),
        ])
        .output()
        .expect("snapshot command runs");

    assert!(
        output.status.success(),
        "snapshot command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let snapshot: Value =
        serde_json::from_slice(&output.stdout).expect("snapshot command prints json");
    let audit_events = snapshot
        .get("auditEvents")
        .and_then(Value::as_array)
        .expect("audit events array exists");

    assert_eq!(audit_events.len(), 4);

    let audit_types = audit_events
        .iter()
        .filter_map(|event| event.get("type").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert!(audit_types.contains(&"restore"));
    assert!(audit_types.contains(&"soft_delete"));
    assert!(audit_types.contains(&"cleanup_checklist"));
    assert!(audit_types.contains(&"export_markdown"));
}

#[test]
fn snapshot_command_exposes_session_control_state() {
    let sandbox = temp_root();
    let home_dir = sandbox.join("home");
    let codex_root = home_dir.join(".codex").join("sessions");
    let bin_dir = sandbox.join("bin");

    fs::create_dir_all(&codex_root).expect("create codex root");
    fs::create_dir_all(&bin_dir).expect("create fake bin dir");
    fs::copy(
        fixtures_root().join("codex/2026/03/15/rollout-2026-03-15T12-00-00-codex-ses-1.jsonl"),
        codex_root.join("rollout-2026-03-15.jsonl"),
    )
    .expect("copy codex fixture");
    write_fake_codex_executable(&bin_dir);

    let output = with_path_prefix(&bin_dir, || {
        Command::new(env!("CARGO_BIN_EXE_open-session-manager-core"))
            .env("HOME", &home_dir)
            .env("USERPROFILE", &home_dir)
            .args(["snapshot"])
            .output()
            .expect("snapshot command runs")
    });

    assert!(
        output.status.success(),
        "snapshot command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let snapshot: Value =
        serde_json::from_slice(&output.stdout).expect("snapshot command prints json");
    let control = snapshot
        .get("sessions")
        .and_then(Value::as_array)
        .and_then(|sessions| {
            sessions.iter().find(|session| {
                session.get("sessionId").and_then(Value::as_str) == Some("codex-ses-1")
            })
        })
        .and_then(|session| session.get("sessionControl"))
        .expect("snapshot should expose session control");

    assert_eq!(
        control.get("supported").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        control.get("available").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        control.get("controller").and_then(Value::as_str),
        Some("codex")
    );
}

#[test]
fn doctor_command_reports_malformed_local_sessions() {
    let sandbox = temp_root();
    let home_dir = sandbox.join("home");
    let codex_root = home_dir.join(".codex").join("sessions");
    let claude_root = home_dir.join(".claude").join("projects");

    fs::create_dir_all(&codex_root).expect("create codex root");
    fs::create_dir_all(&claude_root).expect("create claude root");

    let codex_fixture =
        fixtures_root().join("codex/2026/03/15/rollout-2026-03-15T12-00-00-codex-ses-1.jsonl");
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

    let output = Command::new(env!("CARGO_BIN_EXE_open-session-manager-core"))
        .env("HOME", &home_dir)
        .env("USERPROFILE", &home_dir)
        .args(["doctor"])
        .output()
        .expect("doctor command runs");

    assert!(
        output.status.success(),
        "doctor command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "doctor command should report via stdout JSON instead of stderr noise: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: Value = serde_json::from_slice(&output.stdout).expect("doctor command prints json");
    let findings = report
        .get("findings")
        .and_then(Value::as_array)
        .expect("doctor findings array exists");

    assert_eq!(
        report
            .get("status")
            .and_then(Value::as_str)
            .expect("status exists"),
        "warn"
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings[0]
            .get("code")
            .and_then(Value::as_str)
            .expect("code exists"),
        "malformed_session_skipped"
    );
}

#[test]
fn doctor_command_reports_unknown_session_candidates() {
    let sandbox = temp_root();
    let home_dir = sandbox.join("home");
    let factory_root = home_dir.join(".factory").join("projects").join("project-a");

    fs::create_dir_all(&factory_root).expect("create factory root");
    fs::write(
        factory_root.join("mystery.jsonl"),
        concat!(
            "{\"type\":\"heartbeat\",\"timestamp\":\"2026-03-15T10:00:00Z\"}\n",
            "{\"type\":\"state_dump\",\"payload\":{\"phase\":\"unknown\"}}\n"
        ),
    )
    .expect("write unknown factory transcript");

    let output = Command::new(env!("CARGO_BIN_EXE_open-session-manager-core"))
        .env("HOME", &home_dir)
        .env("USERPROFILE", &home_dir)
        .args(["doctor"])
        .output()
        .expect("doctor command runs");

    assert!(
        output.status.success(),
        "doctor command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: Value = serde_json::from_slice(&output.stdout).expect("doctor command prints json");
    let findings = report
        .get("findings")
        .and_then(Value::as_array)
        .expect("doctor findings array exists");
    let finding = findings
        .iter()
        .find(|entry| {
            entry.get("code").and_then(Value::as_str) == Some("unknown_session_candidate")
        })
        .expect("doctor should report unknown session candidate");

    assert_eq!(
        finding
            .get("assistant")
            .and_then(Value::as_str)
            .expect("assistant exists"),
        "factory-droid"
    );
    assert!(
        finding
            .get("path")
            .and_then(Value::as_str)
            .is_some_and(|path| path.ends_with("mystery.jsonl")),
        "doctor finding should preserve the unknown file path"
    );
}

fn temp_root() -> PathBuf {
    let suffix = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "open-session-manager-cli-snapshot-{}-{suffix}",
        std::process::id(),
    ));

    if root.exists() {
        fs::remove_dir_all(&root).expect("reset temp root");
    }

    fs::create_dir_all(&root).expect("create temp root");
    root
}

fn with_path_prefix<T>(bin_dir: &std::path::Path, action: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("lock env guard");

    let original_path = env::var_os("PATH");
    let joined = match &original_path {
        Some(path) => {
            env::join_paths([bin_dir.to_path_buf(), PathBuf::from(path)]).expect("join PATH")
        }
        None => env::join_paths([bin_dir.to_path_buf()]).expect("set PATH"),
    };
    unsafe {
        env::set_var("PATH", joined);
    }

    let result = action();

    match original_path {
        Some(path) => unsafe {
            env::set_var("PATH", path);
        },
        None => unsafe {
            env::remove_var("PATH");
        },
    }

    result
}

fn write_fake_codex_executable(bin_dir: &std::path::Path) {
    if cfg!(windows) {
        fs::write(bin_dir.join("codex.cmd"), "@echo off\r\necho ok\r\n").expect("write fake codex");
        return;
    }

    let script_path = bin_dir.join("codex");
    fs::write(&script_path, "#!/bin/sh\nprintf 'ok\\n'\n").expect("write fake codex");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
            .expect("chmod fake codex");
    }
}

fn run_fixture_command<const N: usize>(args: [&str; N]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_open-session-manager-core"));
    let fixtures_root = fixtures_root();

    command.args(args);
    command.args([
        "--fixtures",
        fixtures_root.to_str().expect("fixtures path as str"),
    ]);

    command.output().expect("fixture command runs")
}
