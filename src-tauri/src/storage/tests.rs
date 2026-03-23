use rusqlite::Connection;

use super::sqlite::{
    SessionControlStateRow, bootstrap_database, load_session_control_state, table_names,
    upsert_session_control_state,
};

#[test]
fn bootstrap_creates_expected_tables() {
    let connection = Connection::open_in_memory().expect("in-memory sqlite opens");

    bootstrap_database(&connection).expect("schema bootstraps successfully");

    let tables = table_names(&connection).expect("table names can be listed");

    for expected in [
        "installations",
        "sessions",
        "session_insights",
        "config_artifacts",
        "credential_artifacts",
        "audit_events",
        "session_index_cache",
        "session_index_runs",
        "session_control_state",
        "session_control_events",
    ] {
        assert!(
            tables.iter().any(|table| table == expected),
            "expected table `{expected}` in {tables:?}"
        );
    }
}

#[test]
fn persists_extended_session_control_state() {
    let connection = Connection::open_in_memory().expect("in-memory sqlite opens");

    bootstrap_database(&connection).expect("schema bootstraps successfully");
    upsert_session_control_state(
        &connection,
        &SessionControlStateRow {
            session_id: "ses-control-extended".to_string(),
            assistant: "codex".to_string(),
            controller: "codex".to_string(),
            available: true,
            attached: true,
            paused: true,
            last_command: Some("codex resume ses-control-extended".to_string()),
            last_prompt: Some("resume".to_string()),
            last_response: Some("READY".to_string()),
            last_error: None,
            last_resumed_at: Some("2026-03-23T12:00:00Z".to_string()),
            last_continued_at: Some("2026-03-23T12:01:00Z".to_string()),
            paused_at: Some("2026-03-23T12:02:00Z".to_string()),
            process_state: Some("paused".to_string()),
            process_id: Some(4242),
            exit_code: Some(0),
            started_at: Some("2026-03-23T11:58:00Z".to_string()),
            runtime_seconds: Some(240),
            event_count: 7,
            input_tokens: 120,
            output_tokens: 34,
            total_tokens: 154,
            last_activity_at: Some("2026-03-23T12:02:30Z".to_string()),
        },
    )
    .expect("extended control state persists");

    let state = load_session_control_state(&connection, "ses-control-extended")
        .expect("extended control state loads")
        .expect("extended control state exists");

    assert!(state.paused);
    assert_eq!(state.paused_at.as_deref(), Some("2026-03-23T12:02:00Z"));
    assert_eq!(state.process_state.as_deref(), Some("paused"));
    assert_eq!(state.process_id, Some(4242));
    assert_eq!(state.exit_code, Some(0));
    assert_eq!(state.started_at.as_deref(), Some("2026-03-23T11:58:00Z"));
    assert_eq!(state.runtime_seconds, Some(240));
    assert_eq!(state.event_count, 7);
    assert_eq!(state.input_tokens, 120);
    assert_eq!(state.output_tokens, 34);
    assert_eq!(state.total_tokens, 154);
    assert_eq!(
        state.last_activity_at.as_deref(),
        Some("2026-03-23T12:02:30Z")
    );
}
