use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
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

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures")
        .canonicalize()
        .expect("fixtures root resolves")
}

#[test]
fn snapshot_command_emits_real_dashboard_json_from_fixtures() {
    let output = Command::new(env!("CARGO_BIN_EXE_open-session-manager-core"))
        .args([
            "snapshot",
            "--fixtures",
            fixtures_root().to_str().expect("fixtures path as str"),
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

    let sessions = snapshot
        .get("sessions")
        .and_then(Value::as_array)
        .expect("sessions array exists");
    let configs = snapshot
        .get("configs")
        .and_then(Value::as_array)
        .expect("configs array exists");

    assert_eq!(sessions.len(), 3);
    assert_eq!(configs.len(), 3);
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

    restore_session(&manifest.manifest_path, "r007b34r", &connection)
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

    assert_eq!(audit_events.len(), 3);
    assert_eq!(
        audit_events[0]
            .get("type")
            .and_then(Value::as_str)
            .expect("event type exists"),
        "restore"
    );
    assert_eq!(
        audit_events[2]
            .get("type")
            .and_then(Value::as_str)
            .expect("event type exists"),
        "export_markdown"
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
