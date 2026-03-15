use std::{fs, path::PathBuf};

use rusqlite::Connection;

use crate::{
    domain::session::{SessionInsight, SessionRecord},
    storage::sqlite::bootstrap_database,
};

use super::{
    delete::{SoftDeleteRequest, soft_delete_session},
    export::{ExportRequest, export_session_markdown},
    restore::restore_session,
};

#[test]
fn exports_soft_deletes_restores_and_audits_session() {
    let sandbox = temp_root();
    let source_path = sandbox.join("sessions").join("rollout-2026-03-15.jsonl");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(&source_path, "{\"type\":\"response_item\"}\n").expect("write source session");

    let export_root = sandbox.join("exports");
    let quarantine_root = sandbox.join("quarantine");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "ses-archive-1".to_string(),
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

    let export_result = export_session_markdown(&ExportRequest {
        session: &session,
        insight: &insight,
        output_root: &export_root,
        actor: "Max",
        connection: &connection,
    })
    .expect("export markdown");

    let exported = fs::read_to_string(&export_result.output_path).expect("read exported markdown");
    assert!(exported.contains("---"));
    assert!(exported.contains("title: 整理 agent 会话"));
    assert!(exported.contains("## Summary"));
    assert!(exported.contains("## Progress"));

    let manifest = soft_delete_session(&SoftDeleteRequest {
        session: &session,
        quarantine_root: &quarantine_root,
        actor: "Max",
        connection: &connection,
    })
    .expect("soft delete session");

    assert!(!source_path.exists());
    assert!(manifest.quarantined_path.exists());
    assert!(manifest.manifest_path.exists());

    restore_session(&manifest.manifest_path, "Max", &connection).expect("restore session");

    assert!(source_path.exists());
    assert!(!manifest.quarantined_path.exists());

    let event_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM audit_events", [], |row| row.get(0))
        .expect("count audit events");
    assert_eq!(event_count, 3);

    let event_types = query_event_types(&connection);
    assert!(event_types.contains(&"export_markdown".to_string()));
    assert!(event_types.contains(&"soft_delete".to_string()));
    assert!(event_types.contains(&"restore".to_string()));

    fs::remove_dir_all(&sandbox).expect("cleanup sandbox");
}

fn temp_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "agent-session-governance-actions-{}",
        std::process::id()
    ));

    if root.exists() {
        fs::remove_dir_all(&root).expect("reset temp root");
    }

    fs::create_dir_all(&root).expect("create temp root");
    root
}

fn query_event_types(connection: &Connection) -> Vec<String> {
    let mut statement = connection
        .prepare("SELECT event_type FROM audit_events ORDER BY created_at")
        .expect("prepare audit query");
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("read audit rows");

    rows.collect::<Result<Vec<_>, _>>()
        .expect("collect event types")
}
