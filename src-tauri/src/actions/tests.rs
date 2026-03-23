use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

use rusqlite::Connection;
use serde_json::to_string_pretty;

use crate::{
    audit::{
        config_audit::{ConfigAuditTarget, audit_config},
        credential_audit::build_credential_artifacts,
    },
    domain::session::{SessionInsight, SessionRecord},
    storage::sqlite::bootstrap_database,
};

use super::{
    ActionError, AuditWriteRequest, QuarantineManifest, write_audit_event,
    config_writeback::{
        ConfigRollbackRequest, ConfigWritebackRequest, ConfigWritebackUpdate,
        rollback_config_writeback, write_config,
    },
    delete::{SoftDeleteRequest, soft_delete_session},
    export::{ExportRequest, export_session_markdown},
    git_control::{
        GitBranchSwitchRequest, GitCommitRequest, GitPushRequest, commit_project, push_project,
        switch_branch,
    },
    restore::restore_session,
    session_control::{SessionControlRequest, continue_session, resume_session},
};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

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

    let _guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("lock env guard");
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
    assert_eq!(event_count, 4);

    let event_types = query_event_types(&connection);
    assert!(event_types.contains(&"export_markdown".to_string()));
    assert!(event_types.contains(&"cleanup_checklist".to_string()));
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
    fs::write(
        &manifest_path,
        to_string_pretty(&manifest).expect("serialize manifest"),
    )
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
    fs::write(
        &manifest_path,
        to_string_pretty(&manifest).expect("serialize manifest"),
    )
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
    let info_path = storage_root
        .join("session")
        .join("info")
        .join("ses-demo.json");
    let message_dir = storage_root
        .join("session")
        .join("message")
        .join("ses-demo");
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
    assert!(exported.contains("## Session Handoff"));
    assert!(exported.contains("- Next focus: 确认 relay 风险"));
    assert!(exported.contains("- Open tasks: 1"));
    assert!(exported.contains("- Completed tasks: 1"));
    assert!(exported.contains("- Resume cue: 已完成风险初筛，建议先导出再隔离。"));
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
    assert!(exported.contains("## Session Handoff"));
    assert!(exported.contains("- Next focus: 清理过期 shell hook"));
    assert!(exported.contains("- Open tasks: 1"));
    assert!(exported.contains("- Completed tasks: 1"));
    assert!(exported.contains("- Resume cue: 已记录待办并继续审计。"));
}

#[test]
fn exports_markdown_without_todos_still_builds_session_handoff() {
    let sandbox = temp_root();
    let source_path = sandbox.join("sessions").join("codex-ses-handoff.jsonl");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(
        &source_path,
        concat!(
            "{\"timestamp\":\"2026-03-15T06:00:00Z\",\"type\":\"session_meta\",\"payload\":",
            "{\"id\":\"codex-handoff-1\",\"cwd\":\"C:\\\\Projects\\\\demo\"}}\n",
            "{\"timestamp\":\"2026-03-15T06:00:01Z\",\"type\":\"response_item\",\"payload\":",
            "{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",",
            "\"text\":\"Investigate why the cleanup queue keeps including archived sessions.\"}]}}\n",
            "{\"timestamp\":\"2026-03-15T06:00:04Z\",\"type\":\"response_item\",\"payload\":",
            "{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",",
            "\"text\":\"The stale filters are reading the wrong root and need a scoped fix.\"}]}}\n"
        ),
    )
    .expect("write codex session");

    let export_root = sandbox.join("exports");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "codex-handoff-1".to_string(),
        installation_id: None,
        assistant: "codex".to_string(),
        environment: "windows".to_string(),
        project_path: Some("C:/Projects/demo".to_string()),
        source_path: source_path.display().to_string(),
        started_at: Some("2026-03-15T06:00:00Z".to_string()),
        ended_at: Some("2026-03-15T06:00:04Z".to_string()),
        last_activity_at: Some("2026-03-15T06:00:04Z".to_string()),
        message_count: 4,
        tool_count: 0,
        status: "in_progress".to_string(),
        raw_format: "codex-jsonl".to_string(),
        content_hash: "handoff123".to_string(),
    };

    let insight = SessionInsight {
        session_id: session.session_id.clone(),
        title: "Audit stale cleanup queue".to_string(),
        topic_labels_json: r#"["cleanup","queue"]"#.to_string(),
        summary:
            "Scope the stale-session filter to the active discovery root before deleting anything."
                .to_string(),
        progress_state: "in_progress".to_string(),
        progress_percent: Some(45),
        value_score: 76,
        stale_score: 10,
        garbage_score: 14,
        risk_flags_json: r#"["review_before_cleanup"]"#.to_string(),
        confidence: 0.88,
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
    assert!(exported.contains("## Session Handoff"));
    assert!(exported.contains(
        "- Next focus: Scope the stale-session filter to the active discovery root before deleting anything."
    ));
    assert!(exported.contains("- Open tasks: 0"));
    assert!(exported.contains("- Completed tasks: 0"));
    assert!(exported.contains(
        "- Resume cue: The stale filters are reading the wrong root and need a scoped fix."
    ));
}

#[test]
fn exports_cleanup_checklist_and_runs_session_end_hook_when_present() {
    let sandbox = temp_root();
    let source_path = sandbox.join("sessions").join("codex-ses-cleanup.jsonl");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(&source_path, "{\"type\":\"response_item\"}\n").expect("write source session");

    let project_root = sandbox.join("project");
    let hook_path = write_session_end_hook(&project_root);
    let export_root = sandbox.join("exports");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "codex-cleanup-hook".to_string(),
        installation_id: None,
        assistant: "codex".to_string(),
        environment: "windows".to_string(),
        project_path: Some(project_root.display().to_string()),
        source_path: source_path.display().to_string(),
        started_at: Some("2026-03-16T06:00:00Z".to_string()),
        ended_at: Some("2026-03-16T06:02:00Z".to_string()),
        last_activity_at: Some("2026-03-16T06:02:00Z".to_string()),
        message_count: 2,
        tool_count: 0,
        status: "completed".to_string(),
        raw_format: "codex-jsonl".to_string(),
        content_hash: "cleanup-hook-123".to_string(),
    };

    let insight = SessionInsight {
        session_id: session.session_id.clone(),
        title: "Prepare cleanup checklist".to_string(),
        topic_labels_json: r#"["cleanup","hook"]"#.to_string(),
        summary: "Export the session, run the end hook, and persist the cleanup checklist."
            .to_string(),
        progress_state: "completed".to_string(),
        progress_percent: Some(100),
        value_score: 82,
        stale_score: 8,
        garbage_score: 20,
        risk_flags_json: r#"["review_before_cleanup"]"#.to_string(),
        confidence: 0.91,
    };

    let export_result = export_session_markdown(&ExportRequest {
        session: &session,
        insight: &insight,
        output_root: &export_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("export markdown");

    let checklist_path = export_root.join("cleanup-codex-cleanup-hook.json");
    let hook_report_path = export_root.join("session-end-codex-cleanup-hook.log");

    assert!(export_result.output_path.exists());
    assert!(checklist_path.exists());
    assert!(hook_report_path.exists());

    let checklist = serde_json::from_str::<serde_json::Value>(
        &fs::read_to_string(&checklist_path).expect("read cleanup checklist"),
    )
    .expect("parse cleanup checklist");

    assert_eq!(
        checklist.get("sessionId").and_then(serde_json::Value::as_str),
        Some("codex-cleanup-hook")
    );
    assert_eq!(
        checklist
            .get("exportPath")
            .and_then(serde_json::Value::as_str),
        Some(export_result.output_path.display().to_string().as_str())
    );
    assert_eq!(
        checklist
            .get("sessionEndHook")
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str),
        Some("success")
    );
    assert_eq!(
        checklist
            .get("sessionEndHook")
            .and_then(|value| value.get("scriptPath"))
            .and_then(serde_json::Value::as_str),
        Some(hook_path.display().to_string().as_str())
    );

    let hook_report = fs::read_to_string(&hook_report_path).expect("read hook report");
    assert!(hook_report.contains("hook saw session codex-cleanup-hook"));
    assert!(hook_report.contains(checklist_path.display().to_string().as_str()));
    assert!(hook_report.contains(export_result.output_path.display().to_string().as_str()));

    let event_types = query_event_types(&connection);
    assert!(event_types.contains(&"export_markdown".to_string()));
    assert!(event_types.contains(&"cleanup_checklist".to_string()));
    assert!(event_types.contains(&"session_end_hook".to_string()));
}

#[test]
fn soft_delete_requires_cleanup_checklist_before_quarantine() {
    let sandbox = temp_root();
    let source_path = sandbox.join("sessions").join("rollout-2026-03-16.jsonl");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(&source_path, "{\"type\":\"response_item\"}\n").expect("write source session");

    let export_root = sandbox.join("exports");
    let quarantine_root = sandbox.join("quarantine");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "ses-cleanup-precondition".to_string(),
        installation_id: None,
        assistant: "codex".to_string(),
        environment: "windows".to_string(),
        project_path: Some(r"C:\Projects\demo".to_string()),
        source_path: source_path.display().to_string(),
        started_at: Some("2026-03-16T05:00:00Z".to_string()),
        ended_at: Some("2026-03-16T05:15:00Z".to_string()),
        last_activity_at: Some("2026-03-16T05:15:00Z".to_string()),
        message_count: 10,
        tool_count: 4,
        status: "completed".to_string(),
        raw_format: "codex-jsonl".to_string(),
        content_hash: "cleanup-precondition-123".to_string(),
    };

    let export_path = export_root.join("session-ses-cleanup-precondition.md");
    write_audit_event(
        &connection,
        AuditWriteRequest {
            event_type: "export_markdown",
            target_type: "session",
            target_id: &session.session_id,
            actor: "r007b34r",
            before_state: Some(
                serde_json::json!({ "source_path": session.source_path.clone() }).to_string(),
            ),
            after_state: Some(serde_json::json!({ "output_path": export_path }).to_string()),
            result: "success",
        },
    )
    .expect("record export event");

    let error = soft_delete_session(&SoftDeleteRequest {
        session: &session,
        quarantine_root: &quarantine_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect_err("soft delete should require a cleanup checklist first");

    match error {
        ActionError::Precondition(message) => {
            assert!(message.contains("cleanup checklist"));
        }
        other => panic!("unexpected error: {other}"),
    }

    assert!(source_path.exists());
    assert!(!quarantine_root.exists());
}

#[test]
fn writes_back_and_rolls_back_copilot_config_with_backup_and_audit() {
    let sandbox = temp_root();
    let fixtures_root = config_fixtures_root();
    let config_root = sandbox.join(".copilot");
    let backup_root = sandbox.join("config-backups");
    let target = ConfigAuditTarget::new(
        "github-copilot-cli",
        "user",
        "global",
        config_root.join("config.json"),
    );
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    fs::create_dir_all(&config_root).expect("create config root");
    fs::copy(
        fixtures_root.join("copilot").join("config.json"),
        config_root.join("config.json"),
    )
    .expect("copy copilot config");
    fs::copy(
        fixtures_root.join("copilot").join("mcp-config.json"),
        config_root.join("mcp-config.json"),
    )
    .expect("copy copilot mcp config");

    let result = write_config(&ConfigWritebackRequest {
        target: &target,
        update: &ConfigWritebackUpdate {
            provider: Some("github".to_string()),
            model: Some("gpt-5-mini".to_string()),
            base_url: Some("https://github.com/api/copilot".to_string()),
            secret: Some("ghu_new_secret_123454321".to_string()),
        },
        backup_root: &backup_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("write copilot config");

    let after = audit_config(&target).expect("re-audit copilot config");
    let credentials = build_credential_artifacts(&after.secrets);

    assert!(result.manifest_path.exists());
    assert_eq!(after.config.model.as_deref(), Some("gpt-5-mini"));
    assert_eq!(
        after.config.base_url.as_deref(),
        Some("https://github.com/api/copilot")
    );
    assert!(!has_flag(&after.risk_flags, "third_party_base_url"));
    assert_eq!(credentials[0].masked_value, "***4321");

    rollback_config_writeback(&ConfigRollbackRequest {
        manifest_path: &result.manifest_path,
        backup_root: &backup_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("rollback copilot config");

    let restored = audit_config(&target).expect("restore copilot config");
    let restored_credentials = build_credential_artifacts(&restored.secrets);
    let event_types = query_event_types(&connection);

    assert_eq!(restored.config.model.as_deref(), Some("gpt-5"));
    assert_eq!(
        restored.config.base_url.as_deref(),
        Some("https://copilot.enterprise-relay.example")
    );
    assert_eq!(restored_credentials[0].masked_value, "***7890");
    assert!(event_types.contains(&"config_writeback".to_string()));
    assert!(event_types.contains(&"config_rollback".to_string()));
}

#[test]
fn writes_back_and_rolls_back_factory_config_with_backup_and_audit() {
    let sandbox = temp_root();
    let fixtures_root = config_fixtures_root();
    let config_root = sandbox.join(".factory");
    let backup_root = sandbox.join("config-backups");
    let target = ConfigAuditTarget::new(
        "factory-droid",
        "user",
        "global",
        config_root.join("settings.json"),
    );
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    fs::create_dir_all(&config_root).expect("create config root");
    fs::copy(
        fixtures_root.join("factory").join("settings.json"),
        config_root.join("settings.json"),
    )
    .expect("copy factory config");
    fs::copy(
        fixtures_root.join("factory").join("settings.local.json"),
        config_root.join("settings.local.json"),
    )
    .expect("copy factory local config");

    let result = write_config(&ConfigWritebackRequest {
        target: &target,
        update: &ConfigWritebackUpdate {
            provider: Some("openai".to_string()),
            model: Some("gpt-5-mini".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            secret: Some("sk-factory-new-123454321".to_string()),
        },
        backup_root: &backup_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("write factory config");

    let after = audit_config(&target).expect("re-audit factory config");
    let credentials = build_credential_artifacts(&after.secrets);

    assert!(result.manifest_path.exists());
    assert_eq!(after.config.provider.as_deref(), Some("openai"));
    assert_eq!(after.config.model.as_deref(), Some("gpt-5-mini"));
    assert_eq!(
        after.config.base_url.as_deref(),
        Some("https://api.openai.com/v1")
    );
    assert!(!has_flag(&after.risk_flags, "third_party_provider"));
    assert!(!has_flag(&after.risk_flags, "third_party_base_url"));
    assert_eq!(credentials[0].masked_value, "***4321");

    rollback_config_writeback(&ConfigRollbackRequest {
        manifest_path: &result.manifest_path,
        backup_root: &backup_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("rollback factory config");

    let restored = audit_config(&target).expect("restore factory config");
    let restored_credentials = build_credential_artifacts(&restored.secrets);

    assert_eq!(restored.config.provider.as_deref(), Some("openrouter"));
    assert_eq!(
        restored.config.base_url.as_deref(),
        Some("https://factory-relay.example/v1")
    );
    assert_eq!(restored_credentials[0].masked_value, "***7890");
}

#[test]
fn writes_back_and_rolls_back_gemini_config_with_backup_and_audit() {
    let sandbox = temp_root();
    let fixtures_root = config_fixtures_root();
    let config_root = sandbox.join(".gemini");
    let backup_root = sandbox.join("config-backups");
    let target = ConfigAuditTarget::new(
        "gemini-cli",
        "user",
        "global",
        config_root.join("settings.json"),
    );
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    fs::create_dir_all(&config_root).expect("create config root");
    fs::copy(
        fixtures_root.join("gemini").join("settings.json"),
        config_root.join("settings.json"),
    )
    .expect("copy gemini config");
    fs::copy(
        fixtures_root.join("gemini").join(".env"),
        config_root.join(".env"),
    )
    .expect("copy gemini env");

    let result = write_config(&ConfigWritebackRequest {
        target: &target,
        update: &ConfigWritebackUpdate {
            provider: Some("google".to_string()),
            model: Some("gemini-2.5-pro".to_string()),
            base_url: Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
            secret: Some("AIzaSyNewGemini123454321".to_string()),
        },
        backup_root: &backup_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("write gemini config");

    let after = audit_config(&target).expect("re-audit gemini config");
    let credentials = build_credential_artifacts(&after.secrets);

    assert!(result.manifest_path.exists());
    assert_eq!(after.config.model.as_deref(), Some("gemini-2.5-pro"));
    assert_eq!(
        after.config.base_url.as_deref(),
        Some("https://generativelanguage.googleapis.com/v1beta")
    );
    assert!(!has_flag(&after.risk_flags, "third_party_base_url"));
    assert_eq!(credentials[0].masked_value, "***4321");

    rollback_config_writeback(&ConfigRollbackRequest {
        manifest_path: &result.manifest_path,
        backup_root: &backup_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("rollback gemini config");

    let restored = audit_config(&target).expect("restore gemini config");
    let restored_credentials = build_credential_artifacts(&restored.secrets);

    assert_eq!(
        restored.config.model.as_deref(),
        Some("gemini-2.5-pro-preview-06-05")
    );
    assert_eq!(
        restored.config.base_url.as_deref(),
        Some("https://gateway.gemini-proxy.example/v1beta")
    );
    assert_eq!(restored_credentials[0].masked_value, "***4321");
}

#[test]
fn writes_back_and_rolls_back_openclaw_config_with_backup_and_audit() {
    let sandbox = temp_root();
    let fixtures_root = config_fixtures_root();
    let config_root = sandbox.join(".openclaw");
    let backup_root = sandbox.join("config-backups");
    let target = ConfigAuditTarget::new(
        "openclaw",
        "user",
        "global",
        config_root.join("openclaw.json"),
    );
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    fs::create_dir_all(&config_root).expect("create config root");
    fs::copy(
        fixtures_root.join("openclaw").join("openclaw.json"),
        config_root.join("openclaw.json"),
    )
    .expect("copy openclaw config");

    let result = write_config(&ConfigWritebackRequest {
        target: &target,
        update: &ConfigWritebackUpdate {
            provider: Some("openrouter".to_string()),
            model: Some("openrouter/openai/gpt-5-mini".to_string()),
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            secret: Some("sk-openclaw-new-123454321".to_string()),
        },
        backup_root: &backup_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("write openclaw config");

    let after = audit_config(&target).expect("re-audit openclaw config");
    let credentials = build_credential_artifacts(&after.secrets);

    assert!(result.manifest_path.exists());
    assert_eq!(
        after.config.model.as_deref(),
        Some("openrouter/openai/gpt-5-mini")
    );
    assert_eq!(
        after.config.base_url.as_deref(),
        Some("https://openrouter.ai/api/v1")
    );
    assert_eq!(credentials[0].masked_value, "***4321");

    rollback_config_writeback(&ConfigRollbackRequest {
        manifest_path: &result.manifest_path,
        backup_root: &backup_root,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("rollback openclaw config");

    let restored = audit_config(&target).expect("restore openclaw config");
    let restored_credentials = build_credential_artifacts(&restored.secrets);

    assert_eq!(
        restored.config.model.as_deref(),
        Some("openrouter/anthropic/claude-sonnet-4")
    );
    assert_eq!(
        restored.config.base_url.as_deref(),
        Some("https://openrouter.ai/api/v1")
    );
    assert_eq!(restored_credentials[0].masked_value, "***7890");
}

#[test]
fn resumes_supported_session_and_records_control_state() {
    let sandbox = temp_root();
    let bin_dir = sandbox.join("bin");
    let log_path = sandbox.join("codex.log");
    let project_root = sandbox.join("project");
    let source_path = sandbox.join("sessions").join("codex-session.jsonl");

    fs::create_dir_all(&bin_dir).expect("create bin dir");
    fs::create_dir_all(&project_root).expect("create project dir");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(&source_path, "{\"type\":\"response_item\"}\n").expect("write source session");
    write_fake_codex_executable(&bin_dir, &log_path);

    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "codex-ses-1".to_string(),
        installation_id: None,
        assistant: "codex".to_string(),
        environment: "windows".to_string(),
        project_path: Some(project_root.display().to_string()),
        source_path: source_path.display().to_string(),
        started_at: Some("2026-03-15T05:00:00Z".to_string()),
        ended_at: None,
        last_activity_at: Some("2026-03-15T05:15:00Z".to_string()),
        message_count: 10,
        tool_count: 4,
        status: "running".to_string(),
        raw_format: "codex-jsonl".to_string(),
        content_hash: "abc123".to_string(),
    };

    with_path_prefix(&bin_dir, || {
        unsafe {
            env::set_var(
                "OPEN_SESSION_MANAGER_CODEX_COMMAND",
                fake_command_path(&bin_dir, "codex"),
            );
        }
        let result = resume_session(&SessionControlRequest {
            session: &session,
            actor: "r007b34r",
            connection: &connection,
            prompt: None,
        })
        .expect("resume session");

        assert!(result.response.contains("READY"));
        unsafe {
            env::remove_var("OPEN_SESSION_MANAGER_CODEX_COMMAND");
        }
    });

    let (attached, last_command): (i64, String) = connection
        .query_row(
            "SELECT attached, last_command
             FROM session_control_state
             WHERE session_id = ?1",
            [session.session_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("control state should be persisted");

    assert_eq!(attached, 1);
    assert!(last_command.contains("exec resume"));
    assert!(last_command.contains("codex-ses-1"));
    assert!(query_event_types(&connection).contains(&"session_resume".to_string()));
    assert!(
        fs::read_to_string(&log_path)
            .expect("read codex log")
            .contains("exec resume"),
        "fake codex command should have been invoked"
    );
}

#[test]
fn continues_attached_session_and_persists_audit_event() {
    let sandbox = temp_root();
    let bin_dir = sandbox.join("bin");
    let log_path = sandbox.join("claude.log");
    let project_root = sandbox.join("project");
    let source_path = sandbox.join("sessions").join("claude-session.jsonl");

    fs::create_dir_all(&bin_dir).expect("create bin dir");
    fs::create_dir_all(&project_root).expect("create project dir");
    fs::create_dir_all(source_path.parent().expect("session dir")).expect("create session dir");
    fs::write(&source_path, "{\"type\":\"assistant\"}\n").expect("write source session");
    write_fake_claude_executable(&bin_dir, &log_path);

    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    let session = SessionRecord {
        session_id: "claude-ses-1".to_string(),
        installation_id: None,
        assistant: "claude-code".to_string(),
        environment: "windows".to_string(),
        project_path: Some(project_root.display().to_string()),
        source_path: source_path.display().to_string(),
        started_at: Some("2026-03-15T05:00:00Z".to_string()),
        ended_at: None,
        last_activity_at: Some("2026-03-15T05:15:00Z".to_string()),
        message_count: 18,
        tool_count: 7,
        status: "running".to_string(),
        raw_format: "claude-jsonl".to_string(),
        content_hash: "def456".to_string(),
    };

    with_path_prefix(&bin_dir, || {
        unsafe {
            env::set_var(
                "OPEN_SESSION_MANAGER_CLAUDE_CODE_COMMAND",
                fake_command_path(&bin_dir, "claude"),
            );
        }
        continue_session(&SessionControlRequest {
            session: &session,
            actor: "r007b34r",
            connection: &connection,
            prompt: Some("Continue with the next verification step."),
        })
        .expect("continue session");
        unsafe {
            env::remove_var("OPEN_SESSION_MANAGER_CLAUDE_CODE_COMMAND");
        }
    });

    let (attached, last_prompt, last_response): (i64, String, String) = connection
        .query_row(
            "SELECT attached, last_prompt, last_response
             FROM session_control_state
             WHERE session_id = ?1",
            [session.session_id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("control state should be persisted");

    assert_eq!(attached, 1);
    assert_eq!(last_prompt, "Continue with the next verification step.");
    assert!(last_response.contains("READY"));
    assert!(query_event_types(&connection).contains(&"session_continue".to_string()));
    assert!(
        fs::read_to_string(&log_path)
            .expect("read claude log")
            .contains("-r claude-ses-1"),
        "fake claude command should receive resume args"
    );
}

#[test]
fn commits_git_project_and_records_audit_event() {
    let sandbox = temp_root();
    let repo_root = sandbox.join("git-commit");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    init_git_repo(&repo_root);
    fs::write(repo_root.join("README.md"), "# updated\n").expect("write dirty readme");
    fs::write(repo_root.join("notes.txt"), "new file\n").expect("write new file");

    let result = commit_project(&GitCommitRequest {
        repo_root: &repo_root,
        message: "feat: capture git controls",
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("commit git project");

    assert_eq!(result.branch, "main");
    assert_eq!(result.summary, "feat: capture git controls");
    assert_eq!(
        git(&repo_root, &["log", "-1", "--pretty=%s"]),
        "feat: capture git controls"
    );
    assert!(query_event_types(&connection).contains(&"git_commit".to_string()));
}

#[test]
fn switches_git_branch_with_dirty_worktree_guardrail() {
    let sandbox = temp_root();
    let repo_root = sandbox.join("git-switch");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    init_git_repo(&repo_root);
    fs::write(repo_root.join("README.md"), "# dirty\n").expect("write dirty readme");

    let error = switch_branch(&GitBranchSwitchRequest {
        repo_root: &repo_root,
        branch: "feature/git-panel",
        actor: "r007b34r",
        connection: &connection,
    })
    .expect_err("dirty worktree should block branch switch");

    match error {
        ActionError::Precondition(message) => {
            assert!(message.contains("clean"));
        }
        other => panic!("unexpected error: {other}"),
    }

    git(&repo_root, &["add", "."]);
    git(&repo_root, &["commit", "-m", "chore: clean before switch"]);

    let result = switch_branch(&GitBranchSwitchRequest {
        repo_root: &repo_root,
        branch: "feature/git-panel",
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("switch branch");

    assert_eq!(result.branch, "feature/git-panel");
    assert_eq!(git(&repo_root, &["branch", "--show-current"]), "feature/git-panel");
    assert!(query_event_types(&connection).contains(&"git_branch_switch".to_string()));
}

#[test]
fn pushes_git_project_to_upstream_and_records_audit_event() {
    let sandbox = temp_root();
    let remote_root = sandbox.join("remote.git");
    let repo_root = sandbox.join("git-push");
    let connection = Connection::open_in_memory().expect("open sqlite");
    bootstrap_database(&connection).expect("bootstrap schema");

    init_bare_git_repo(&remote_root);
    init_git_repo(&repo_root);
    git(
        &repo_root,
        &["remote", "add", "origin", remote_root.to_str().expect("remote path")],
    );
    git(&repo_root, &["push", "-u", "origin", "main"]);

    fs::write(repo_root.join("README.md"), "# pushed\n").expect("write dirty readme");
    commit_project(&GitCommitRequest {
        repo_root: &repo_root,
        message: "feat: push git controls",
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("create outgoing commit");

    let result = push_project(&GitPushRequest {
        repo_root: &repo_root,
        remote: None,
        actor: "r007b34r",
        connection: &connection,
    })
    .expect("push git project");

    assert_eq!(result.branch, "main");
    assert!(result.output.contains("origin"));
    assert_eq!(
        git(&repo_root, &["log", "-1", "--pretty=%s"]),
        git_bare(&remote_root, &["log", "-1", "--pretty=%s", "main"])
    );
    assert!(query_event_types(&connection).contains(&"git_push".to_string()));
}

fn config_fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures/configs")
        .canonicalize()
        .expect("config fixtures root resolves")
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

fn init_git_repo(repo_root: &Path) {
    fs::create_dir_all(repo_root).expect("create repo root");
    git(repo_root, &["init"]);
    git(repo_root, &["branch", "-M", "main"]);
    git(repo_root, &["config", "user.name", "OSM Test"]);
    git(repo_root, &["config", "user.email", "osm-test@example.com"]);
    fs::write(repo_root.join("README.md"), "# seed\n").expect("write readme");
    git(repo_root, &["add", "."]);
    git(repo_root, &["commit", "-m", "chore: seed repo"]);
}

fn init_bare_git_repo(repo_root: &Path) {
    fs::create_dir_all(repo_root).expect("create bare repo root");
    git(repo_root, &["init", "--bare"]);
}

fn git(repo_root: &Path, args: &[&str]) -> String {
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

fn git_bare(repo_root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("--git-dir")
        .arg(repo_root)
        .args(args)
        .output()
        .expect("bare git command should run");

    if !output.status.success() {
        panic!(
            "git --git-dir {:?} {:?} failed: {}",
            repo_root,
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn with_path_prefix<T>(bin_dir: &Path, action: impl FnOnce() -> T) -> T {
    let _guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("lock env guard");

    let original_path = env::var_os("PATH");
    let joined = match &original_path {
        Some(path) => env::join_paths(
            std::iter::once(bin_dir.to_path_buf()).chain(env::split_paths(path)),
        )
        .expect("join PATH"),
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

fn write_fake_codex_executable(bin_dir: &Path, log_path: &Path) {
    if cfg!(windows) {
        let script_path = bin_dir.join("codex.cmd");
        fs::write(
            &script_path,
            format!(
                concat!(
                    "@echo off\r\n",
                    "setlocal EnableDelayedExpansion\r\n",
                    "echo %*>>\"{}\"\r\n",
                    "set \"out=\"\r\n",
                    ":next\r\n",
                    "if \"%~1\"==\"\" goto done\r\n",
                    "if \"%~1\"==\"-o\" (\r\n",
                    "  set \"out=%~2\"\r\n",
                    "  shift\r\n",
                    ")\r\n",
                    "shift\r\n",
                    "goto next\r\n",
                    ":done\r\n",
                    "if not \"!out!\"==\"\" (\r\n",
                    "  >\"!out!\" echo READY from fake codex\r\n",
                    ")\r\n",
                    "echo ok\r\n"
                ),
                log_path.display()
            ),
        )
        .expect("write fake codex");
        return;
    }

    let script_path = bin_dir.join("codex");
    fs::write(
        &script_path,
        format!(
            concat!(
                "#!/bin/sh\n",
                "printf '%s\\n' \"$*\" >> '{}'\n",
                "out=''\n",
                "while [ \"$#\" -gt 0 ]; do\n",
                "  if [ \"$1\" = '-o' ]; then\n",
                "    out=\"$2\"\n",
                "    shift 2\n",
                "    continue\n",
                "  fi\n",
                "  shift\n",
                "done\n",
                "if [ -n \"$out\" ]; then\n",
                "  printf 'READY from fake codex\\n' > \"$out\"\n",
                "fi\n",
                "printf 'ok\\n'\n"
            ),
            log_path.display()
        ),
    )
    .expect("write fake codex");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
            .expect("chmod fake codex");
    }
}

fn write_fake_claude_executable(bin_dir: &Path, log_path: &Path) {
    if cfg!(windows) {
        let script_path = bin_dir.join("claude.cmd");
        fs::write(
            &script_path,
            format!(
                concat!(
                    "@echo off\r\n",
                    "echo %*>>\"{}\"\r\n",
                    "echo READY from fake claude\r\n"
                ),
                log_path.display()
            ),
        )
        .expect("write fake claude");
        return;
    }

    let script_path = bin_dir.join("claude");
    fs::write(
        &script_path,
        format!(
            concat!(
                "#!/bin/sh\n",
                "printf '%s\\n' \"$*\" >> '{}'\n",
                "printf 'READY from fake claude\\n'\n"
            ),
            log_path.display()
        ),
    )
    .expect("write fake claude");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
            .expect("chmod fake claude");
    }
}

fn fake_command_path(bin_dir: &Path, name: &str) -> PathBuf {
    if cfg!(windows) {
        bin_dir.join(format!("{name}.cmd"))
    } else {
        bin_dir.join(name)
    }
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

fn has_flag<T>(flags: &[T], code: &str) -> bool
where
    T: std::borrow::Borrow<crate::audit::RiskFlag>,
{
    flags.iter().any(|flag| flag.borrow().code == code)
}

fn write_session_end_hook(project_root: &Path) -> PathBuf {
    let hooks_dir = project_root.join(".open-session-manager").join("hooks");
    fs::create_dir_all(&hooks_dir).expect("create session-end hooks dir");

    if cfg!(windows) {
        let hook_path = hooks_dir.join("session-end.ps1");
        fs::write(
            &hook_path,
            concat!(
                "param([string]$ChecklistPath, [string]$ExportPath)\n",
                "Write-Output \"hook saw session $env:OSM_SESSION_ID\"\n",
                "Write-Output \"checklist=$ChecklistPath\"\n",
                "Write-Output \"export=$ExportPath\"\n",
            ),
        )
        .expect("write powershell session-end hook");
        return hook_path;
    }

    let hook_path = hooks_dir.join("session-end.sh");
    fs::write(
        &hook_path,
        concat!(
            "#!/bin/sh\n",
            "printf 'hook saw session %s\\n' \"$OSM_SESSION_ID\"\n",
            "printf 'checklist=%s\\n' \"$1\"\n",
            "printf 'export=%s\\n' \"$2\"\n",
        ),
    )
    .expect("write shell session-end hook");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))
            .expect("mark shell hook executable");
    }

    hook_path
}
