use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use rusqlite::Connection;
use serde_json::to_string_pretty;

use crate::{
    domain::session::{SessionInsight, SessionRecord},
    storage::sqlite::bootstrap_database,
};

use super::{
    ActionError, QuarantineManifest,
    delete::{SoftDeleteRequest, soft_delete_session},
    export::{ExportRequest, export_session_markdown},
    restore::restore_session,
};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

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
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("export markdown");

    let exported = fs::read_to_string(&export_result.output_path).expect("read exported markdown");
    assert!(exported.contains("---"));
    assert!(exported.contains("title: \"整理 agent 会话\""));
    assert!(exported.contains("## Summary"));
    assert!(exported.contains("## Progress"));

    let manifest = soft_delete_session(&SoftDeleteRequest {
        session: &session,
        quarantine_root: &quarantine_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("soft delete session");

    assert!(!source_path.exists());
    assert!(manifest.quarantined_path.exists());
    assert!(manifest.manifest_path.exists());

    restore_session(
        &manifest.manifest_path,
        &quarantine_root,
        &[sandbox.join("sessions")],
        "r007b34r",
        &connection,
    )
        .expect("restore session");

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

#[test]
fn refuses_soft_delete_until_a_markdown_export_has_been_recorded() {
    let sandbox = temp_root();
    let source_path = sandbox.join("sessions").join("rollout-2026-03-15.jsonl");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(&source_path, "{\"type\":\"response_item\"}\n").expect("write source session");

    let quarantine_root = sandbox.join("quarantine");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "ses-needs-export".to_string(),
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

    let error = soft_delete_session(&SoftDeleteRequest {
        session: &session,
        quarantine_root: &quarantine_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect_err("soft delete should require an export first");

    match error {
        ActionError::Precondition(message) => {
            assert!(message.contains("export"));
        }
        other => panic!("unexpected error: {other}"),
    }

    assert!(source_path.exists());
    assert!(!quarantine_root.exists());
}

#[test]
fn sanitizes_session_ids_before_writing_export_and_quarantine_paths() {
    let sandbox = temp_root();
    let source_path = sandbox.join("sessions").join("rollout-2026-03-15.jsonl");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(&source_path, "{\"type\":\"response_item\"}\n").expect("write source session");

    let export_root = sandbox.join("exports");
    let quarantine_root = sandbox.join("quarantine");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: r"..\..\ses/escape".to_string(),
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
        title: "sanitize output paths".to_string(),
        topic_labels_json: r#"["cleanup"]"#.to_string(),
        summary: "Session ids must not escape the managed roots.".to_string(),
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
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("export markdown");

    let export_root_resolved = export_root.canonicalize().expect("export root resolves");
    let export_path_resolved = export_result
        .output_path
        .canonicalize()
        .expect("export path resolves");
    assert!(export_path_resolved.starts_with(&export_root_resolved));

    let manifest = soft_delete_session(&SoftDeleteRequest {
        session: &session,
        quarantine_root: &quarantine_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("soft delete session");

    let quarantine_root_resolved = quarantine_root
        .canonicalize()
        .expect("quarantine root resolves");
    let quarantined_path_resolved = manifest
        .quarantined_path
        .canonicalize()
        .expect("quarantined path resolves");
    assert!(quarantined_path_resolved.starts_with(&quarantine_root_resolved));
}

#[test]
fn refuses_restore_when_manifest_is_outside_the_managed_quarantine_root() {
    let sandbox = temp_root();
    let quarantine_root = sandbox.join("quarantine");
    let rogue_root = sandbox.join("rogue");
    let payload_path = rogue_root.join("payload").join("session.jsonl");
    let manifest_path = rogue_root.join("manifest.json");
    let restored_path = sandbox.join("sessions").join("restored.jsonl");

    fs::create_dir_all(payload_path.parent().expect("payload dir")).expect("create payload dir");
    fs::write(&payload_path, "{\"type\":\"response_item\"}\n").expect("write rogue payload");

    let manifest = QuarantineManifest {
        session_id: "rogue-session".to_string(),
        original_path: restored_path,
        quarantined_path: payload_path,
        manifest_path: manifest_path.clone(),
        deleted_at: "2026-03-15T10:00:00Z".to_string(),
        related_assets: Vec::new(),
    };
    fs::write(&manifest_path, to_string_pretty(&manifest).expect("serialize manifest"))
        .expect("write rogue manifest");

    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let error = restore_session(
        &manifest_path,
        &quarantine_root,
        &[sandbox.join("sessions")],
        "r007b34r",
        &connection,
    )
        .expect_err("restore should reject manifests outside the managed root");

    match error {
        ActionError::Precondition(message) => {
            assert!(message.contains("managed quarantine root"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn refuses_restore_when_original_path_is_outside_allowed_session_roots() {
    let sandbox = temp_root();
    let quarantine_root = sandbox.join("quarantine");
    let session_root = quarantine_root.join("managed-session");
    let payload_path = session_root.join("payload").join("session.jsonl");
    let manifest_path = session_root.join("manifest.json");
    let allowed_restore_root = sandbox.join("sessions");
    let escape_path = sandbox.join("outside").join("restored.jsonl");

    fs::create_dir_all(payload_path.parent().expect("payload dir")).expect("create payload dir");
    fs::write(&payload_path, "{\"type\":\"response_item\"}\n").expect("write payload");

    let manifest = QuarantineManifest {
        session_id: "managed-session".to_string(),
        original_path: escape_path,
        quarantined_path: payload_path,
        manifest_path: manifest_path.clone(),
        deleted_at: "2026-03-15T11:00:00Z".to_string(),
        related_assets: Vec::new(),
    };
    fs::write(&manifest_path, to_string_pretty(&manifest).expect("serialize manifest"))
        .expect("write manifest");

    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let error = restore_session(
        &manifest_path,
        &quarantine_root,
        &[allowed_restore_root],
        "r007b34r",
        &connection,
    )
    .expect_err("restore should reject paths outside allowed session roots");

    match error {
        ActionError::Precondition(message) => {
            assert!(message.contains("allowed session root"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn soft_delete_and_restore_opencode_session_bundle() {
    let sandbox = temp_root();
    let storage_root = sandbox.join("storage");
    let info_path = storage_root.join("session").join("info").join("ses-demo.json");
    let message_dir = storage_root.join("session").join("message").join("ses-demo");
    let part_dir = storage_root.join("session").join("part").join("ses-demo");
    let message_path = message_dir.join("msg-user.json");
    let part_path = part_dir.join("prt-user.json");

    fs::create_dir_all(info_path.parent().expect("info dir")).expect("create info dir");
    fs::create_dir_all(&message_dir).expect("create message dir");
    fs::create_dir_all(&part_dir).expect("create part dir");
    fs::write(&info_path, "{\"id\":\"ses-demo\"}\n").expect("write info");
    fs::write(&message_path, "{\"id\":\"msg-user\"}\n").expect("write message");
    fs::write(&part_path, "{\"id\":\"prt-user\"}\n").expect("write part");

    let export_root = sandbox.join("exports");
    let quarantine_root = sandbox.join("quarantine");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "ses-demo".to_string(),
        installation_id: None,
        assistant: "opencode".to_string(),
        environment: "linux".to_string(),
        project_path: Some("/home/max/project".to_string()),
        source_path: info_path.display().to_string(),
        started_at: Some("2026-03-15T08:00:00Z".to_string()),
        ended_at: Some("2026-03-15T08:05:00Z".to_string()),
        last_activity_at: Some("2026-03-15T08:05:00Z".to_string()),
        message_count: 2,
        tool_count: 0,
        status: "completed".to_string(),
        raw_format: "opencode-storage".to_string(),
        content_hash: "bundle123".to_string(),
    };

    let insight = SessionInsight {
        session_id: session.session_id.clone(),
        title: "restore opencode bundle".to_string(),
        topic_labels_json: r#"["cleanup"]"#.to_string(),
        summary: "OpenCode session bundles must move together.".to_string(),
        progress_state: "completed".to_string(),
        progress_percent: Some(100),
        value_score: 70,
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

    assert!(!info_path.exists());
    assert!(!message_dir.exists());
    assert!(!part_dir.exists());
    assert_eq!(manifest.related_assets.len(), 2);
    assert!(
        manifest
            .related_assets
            .iter()
            .any(|asset| asset.original_path == message_dir)
    );
    assert!(
        manifest
            .related_assets
            .iter()
            .any(|asset| asset.original_path == part_dir)
    );

    restore_session(
        &manifest.manifest_path,
        &quarantine_root,
        &[storage_root.join("session")],
        "r007b34r",
        &connection,
    )
    .expect("restore opencode bundle");

    assert!(info_path.exists());
    assert!(message_path.exists());
    assert!(part_path.exists());
}

#[test]
fn exports_markdown_with_yaml_safe_frontmatter_values() {
    let sandbox = temp_root();
    let source_path = sandbox.join("sessions").join("rollout-2026-03-15.jsonl");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(&source_path, "{\"type\":\"response_item\"}\n").expect("write source session");

    let export_root = sandbox.join("exports");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "ses:yaml\nunsafe".to_string(),
        installation_id: None,
        assistant: "codex".to_string(),
        environment: "windows".to_string(),
        project_path: Some("C:/Projects/demo:alpha".to_string()),
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
        title: "yaml: title\nwith \"quotes\"".to_string(),
        topic_labels_json: r#"["cleanup"]"#.to_string(),
        summary: "Summary body stays readable.".to_string(),
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
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("export markdown");

    let exported = fs::read_to_string(&export_result.output_path).expect("read exported markdown");
    assert!(exported.contains("title: \"yaml: title\\nwith \\\"quotes\\\"\""));
    assert!(exported.contains("session_id: \"ses:yaml\\nunsafe\""));
    assert!(exported.contains("project_path: \"C:/Projects/demo:alpha\""));
}

#[test]
fn exports_markdown_with_upstream_style_digest() {
    let sandbox = temp_root();
    let source_path = sandbox.join("sessions").join("claude-ses-2.jsonl");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(
        &source_path,
        concat!(
            "{\"type\":\"user\",\"sessionId\":\"claude-ses-2\",",
            "\"cwd\":\"C:/Projects/ops\",",
            "\"timestamp\":\"2026-03-15T07:00:00Z\",",
            "\"message\":{\"content\":\"梳理 relay 配置并决定是否清理\"},",
            "\"todos\":[",
            "{\"content\":\"导出高价值摘要\",\"status\":\"completed\"},",
            "{\"content\":\"确认 relay 风险\",\"status\":\"pending\"}",
            "]}\n",
            "{\"type\":\"assistant\",\"sessionId\":\"claude-ses-2\",",
            "\"timestamp\":\"2026-03-15T07:05:00Z\",",
            "\"message\":{\"content\":[",
            "{\"type\":\"text\",\"text\":\"已完成风险初筛，建议先导出再隔离。\"}",
            "]}}\n"
        ),
    )
    .expect("write claude session");

    let export_root = sandbox.join("exports");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "claude-ses-2".to_string(),
        installation_id: None,
        assistant: "claude-code".to_string(),
        environment: "windows".to_string(),
        project_path: Some("C:/Projects/ops".to_string()),
        source_path: source_path.display().to_string(),
        started_at: Some("2026-03-15T07:00:00Z".to_string()),
        ended_at: Some("2026-03-15T07:05:00Z".to_string()),
        last_activity_at: Some("2026-03-15T07:05:00Z".to_string()),
        message_count: 8,
        tool_count: 1,
        status: "completed".to_string(),
        raw_format: "claude-code-jsonl".to_string(),
        content_hash: "digest123".to_string(),
    };

    let insight = SessionInsight {
        session_id: session.session_id.clone(),
        title: "梳理 relay 清理策略".to_string(),
        topic_labels_json: r#"["relay","cleanup","ops"]"#.to_string(),
        summary: "风险已经初筛完成，下一步是导出要点并决定是否隔离。".to_string(),
        progress_state: "in_progress".to_string(),
        progress_percent: Some(65),
        value_score: 84,
        stale_score: 12,
        garbage_score: 18,
        risk_flags_json: r#"["stale_followup_needed","dangerous_permissions"]"#.to_string(),
        confidence: 0.92,
    };

    let export_result = export_session_markdown(&ExportRequest {
        session: &session,
        insight: &insight,
        output_root: &export_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("export markdown");

    let exported = fs::read_to_string(&export_result.output_path).expect("read exported markdown");
    assert!(exported.contains("## Cleanup Recommendation"));
    assert!(exported.contains("## Signals"));
    assert!(exported.contains("relay, cleanup, ops"));
    assert!(exported.contains("stale_followup_needed, dangerous_permissions"));
    assert!(exported.contains("## Todo Snapshot"));
    assert!(exported.contains("- [x] 导出高价值摘要"));
    assert!(exported.contains("- [ ] 确认 relay 风险"));
    assert!(exported.contains("## Transcript Highlights"));
    assert!(exported.contains("### User"));
    assert!(exported.contains("梳理 relay 配置并决定是否清理"));
    assert!(exported.contains("### Assistant"));
    assert!(exported.contains("已完成风险初筛，建议先导出再隔离。"));
}

#[test]
fn exports_markdown_with_claude_todowrite_digest() {
    let sandbox = temp_root();
    let source_path = sandbox.join("sessions").join("claude-ses-3.jsonl");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(
        &source_path,
        concat!(
            "{\"type\":\"user\",\"sessionId\":\"claude-ses-3\",",
            "\"cwd\":\"C:/Projects/ops\",",
            "\"timestamp\":\"2026-03-15T07:00:00Z\",",
            "\"message\":{\"content\":\"检查 relay 风险\"}}\n",
            "{\"type\":\"assistant\",\"sessionId\":\"claude-ses-3\",",
            "\"timestamp\":\"2026-03-15T07:05:00Z\",",
            "\"message\":{\"content\":[",
            "{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"TodoWrite\",\"input\":{\"todos\":[",
            "{\"content\":\"确认 relay override\",\"status\":\"completed\"},",
            "{\"content\":\"清理过期 shell hook\",\"status\":\"pending\"}",
            "]}}",
            ",{\"type\":\"text\",\"text\":\"已记录待办并继续审计。\"}",
            "]}}\n"
        ),
    )
    .expect("write claude session");

    let export_root = sandbox.join("exports");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "claude-ses-3".to_string(),
        installation_id: None,
        assistant: "claude-code".to_string(),
        environment: "windows".to_string(),
        project_path: Some("C:/Projects/ops".to_string()),
        source_path: source_path.display().to_string(),
        started_at: Some("2026-03-15T07:00:00Z".to_string()),
        ended_at: Some("2026-03-15T07:05:00Z".to_string()),
        last_activity_at: Some("2026-03-15T07:05:00Z".to_string()),
        message_count: 8,
        tool_count: 1,
        status: "completed".to_string(),
        raw_format: "claude-code-jsonl".to_string(),
        content_hash: "digest124".to_string(),
    };

    let insight = SessionInsight {
        session_id: session.session_id.clone(),
        title: "提取 TodoWrite 待办".to_string(),
        topic_labels_json: r#"["relay","todo"]"#.to_string(),
        summary: "待办来源于 Claude assistant 的 TodoWrite 工具调用。".to_string(),
        progress_state: "in_progress".to_string(),
        progress_percent: Some(50),
        value_score: 70,
        stale_score: 12,
        garbage_score: 18,
        risk_flags_json: r#"["dangerous_permissions"]"#.to_string(),
        confidence: 0.92,
    };

    let export_result = export_session_markdown(&ExportRequest {
        session: &session,
        insight: &insight,
        output_root: &export_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("export markdown");

    let exported = fs::read_to_string(&export_result.output_path).expect("read exported markdown");
    assert!(exported.contains("## Todo Snapshot"));
    assert!(exported.contains("- [x] 确认 relay override"));
    assert!(exported.contains("- [ ] 清理过期 shell hook"));
}

fn temp_root() -> PathBuf {
    let suffix = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "open-session-manager-actions-{}-{suffix}",
        std::process::id(),
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
