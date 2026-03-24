use std::path::Path;

use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::domain::audit::AuditEvent;

const SCHEMA: &str = include_str!("schema.sql");

pub fn bootstrap_database(connection: &Connection) -> Result<()> {
    connection.execute_batch(SCHEMA)?;
    ensure_session_control_state_columns(connection)
}

pub fn table_names(connection: &Connection) -> Result<Vec<String>> {
    let mut statement = connection.prepare(
        "SELECT name
         FROM sqlite_master
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
         ORDER BY name",
    )?;

    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;

    rows.collect()
}

pub fn open_database(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
    }

    let connection = Connection::open(path)?;
    bootstrap_database(&connection)?;
    Ok(connection)
}

pub fn load_audit_events(connection: &Connection, limit: usize) -> Result<Vec<AuditEvent>> {
    let mut statement = connection.prepare(
        "SELECT
            event_id,
            event_type,
            target_type,
            target_id,
            actor,
            created_at,
            before_state,
            after_state,
            result,
            error_message
         FROM audit_events
         ORDER BY created_at DESC
         LIMIT ?1",
    )?;

    let rows = statement.query_map([limit as i64], |row| {
        Ok(AuditEvent {
            event_id: row.get(0)?,
            event_type: row.get(1)?,
            target_type: row.get(2)?,
            target_id: row.get(3)?,
            actor: row.get(4)?,
            created_at: row.get(5)?,
            before_state: row.get(6)?,
            after_state: row.get(7)?,
            result: row.get(8)?,
            error_message: row.get(9)?,
        })
    })?;

    rows.collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionIndexCacheRow {
    pub source_path: String,
    pub assistant: String,
    pub environment: String,
    pub source_size: i64,
    pub source_modified_at: i64,
    pub session_id: String,
    pub session_json: String,
    pub insight_json: String,
    pub detail_json: String,
    pub indexed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionIndexRunRecord {
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub discovered_files: i64,
    pub cache_hits: i64,
    pub cache_misses: i64,
    pub reindexed_files: i64,
    pub stale_deleted: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionControlStateRow {
    pub session_id: String,
    pub assistant: String,
    pub controller: String,
    pub available: bool,
    pub attached: bool,
    pub paused: bool,
    pub last_command: Option<String>,
    pub last_prompt: Option<String>,
    pub last_response: Option<String>,
    pub last_error: Option<String>,
    pub last_resumed_at: Option<String>,
    pub last_continued_at: Option<String>,
    pub paused_at: Option<String>,
    pub process_state: Option<String>,
    pub process_id: Option<i64>,
    pub exit_code: Option<i64>,
    pub started_at: Option<String>,
    pub runtime_seconds: Option<i64>,
    pub event_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub last_activity_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionControlEventRow {
    pub event_id: String,
    pub session_id: String,
    pub operation: String,
    pub created_at: String,
    pub prompt: Option<String>,
    pub response: Option<String>,
    pub result: String,
    pub error_message: Option<String>,
    pub command: Option<String>,
}

pub fn load_session_index_cache_row(
    connection: &Connection,
    source_path: &str,
) -> Result<Option<SessionIndexCacheRow>> {
    connection
        .query_row(
            "SELECT
                source_path,
                assistant,
                environment,
                source_size,
                source_modified_at,
                session_id,
                session_json,
                insight_json,
                detail_json,
                indexed_at
             FROM session_index_cache
             WHERE source_path = ?1",
            [source_path],
            |row| {
                Ok(SessionIndexCacheRow {
                    source_path: row.get(0)?,
                    assistant: row.get(1)?,
                    environment: row.get(2)?,
                    source_size: row.get(3)?,
                    source_modified_at: row.get(4)?,
                    session_id: row.get(5)?,
                    session_json: row.get(6)?,
                    insight_json: row.get(7)?,
                    detail_json: row.get(8)?,
                    indexed_at: row.get(9)?,
                })
            },
        )
        .optional()
}

pub fn upsert_session_index_cache_row(
    connection: &Connection,
    row: &SessionIndexCacheRow,
) -> Result<()> {
    connection.execute(
        "INSERT INTO session_index_cache (
            source_path,
            assistant,
            environment,
            source_size,
            source_modified_at,
            session_id,
            session_json,
            insight_json,
            detail_json,
            indexed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(source_path) DO UPDATE SET
            assistant = excluded.assistant,
            environment = excluded.environment,
            source_size = excluded.source_size,
            source_modified_at = excluded.source_modified_at,
            session_id = excluded.session_id,
            session_json = excluded.session_json,
            insight_json = excluded.insight_json,
            detail_json = excluded.detail_json,
            indexed_at = excluded.indexed_at",
        params![
            row.source_path,
            row.assistant,
            row.environment,
            row.source_size,
            row.source_modified_at,
            row.session_id,
            row.session_json,
            row.insight_json,
            row.detail_json,
            row.indexed_at,
        ],
    )?;

    Ok(())
}

pub fn list_session_index_cache_paths(connection: &Connection) -> Result<Vec<String>> {
    let mut statement = connection
        .prepare("SELECT source_path FROM session_index_cache ORDER BY source_path ASC")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;

    rows.collect()
}

pub fn delete_session_index_cache_rows(
    connection: &Connection,
    source_paths: &[String],
) -> Result<usize> {
    let mut deleted = 0;

    for source_path in source_paths {
        deleted += connection.execute(
            "DELETE FROM session_index_cache WHERE source_path = ?1",
            [source_path],
        )?;
    }

    Ok(deleted)
}

pub fn insert_session_index_run(
    connection: &Connection,
    run: &SessionIndexRunRecord,
) -> Result<()> {
    connection.execute(
        "INSERT INTO session_index_runs (
            run_id,
            started_at,
            finished_at,
            discovered_files,
            cache_hits,
            cache_misses,
            reindexed_files,
            stale_deleted
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            run.run_id,
            run.started_at,
            run.finished_at,
            run.discovered_files,
            run.cache_hits,
            run.cache_misses,
            run.reindexed_files,
            run.stale_deleted,
        ],
    )?;

    Ok(())
}

pub fn load_session_control_state(
    connection: &Connection,
    session_id: &str,
) -> Result<Option<SessionControlStateRow>> {
    connection
        .query_row(
            "SELECT
                session_id,
                assistant,
                controller,
                available,
                attached,
                paused,
                last_command,
                last_prompt,
                last_response,
                last_error,
                last_resumed_at,
                last_continued_at,
                paused_at,
                process_state,
                process_id,
                exit_code,
                started_at,
                runtime_seconds,
                event_count,
                input_tokens,
                output_tokens,
                total_tokens,
                last_activity_at
             FROM session_control_state
             WHERE session_id = ?1",
            [session_id],
            |row| {
                Ok(SessionControlStateRow {
                    session_id: row.get(0)?,
                    assistant: row.get(1)?,
                    controller: row.get(2)?,
                    available: row.get::<_, i64>(3)? != 0,
                    attached: row.get::<_, i64>(4)? != 0,
                    paused: row.get::<_, i64>(5)? != 0,
                    last_command: row.get(6)?,
                    last_prompt: row.get(7)?,
                    last_response: row.get(8)?,
                    last_error: row.get(9)?,
                    last_resumed_at: row.get(10)?,
                    last_continued_at: row.get(11)?,
                    paused_at: row.get(12)?,
                    process_state: row.get(13)?,
                    process_id: row.get(14)?,
                    exit_code: row.get(15)?,
                    started_at: row.get(16)?,
                    runtime_seconds: row.get(17)?,
                    event_count: row.get(18)?,
                    input_tokens: row.get(19)?,
                    output_tokens: row.get(20)?,
                    total_tokens: row.get(21)?,
                    last_activity_at: row.get(22)?,
                })
            },
        )
        .optional()
}

pub fn upsert_session_control_state(
    connection: &Connection,
    state: &SessionControlStateRow,
) -> Result<()> {
    connection.execute(
        "INSERT INTO session_control_state (
            session_id,
            assistant,
            controller,
            available,
            attached,
            paused,
            last_command,
            last_prompt,
            last_response,
            last_error,
            last_resumed_at,
            last_continued_at,
            paused_at,
            process_state,
            process_id,
            exit_code,
            started_at,
            runtime_seconds,
            event_count,
            input_tokens,
            output_tokens,
            total_tokens,
            last_activity_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)
        ON CONFLICT(session_id) DO UPDATE SET
            assistant = excluded.assistant,
            controller = excluded.controller,
            available = excluded.available,
            attached = excluded.attached,
            paused = excluded.paused,
            last_command = excluded.last_command,
            last_prompt = excluded.last_prompt,
            last_response = excluded.last_response,
            last_error = excluded.last_error,
            last_resumed_at = excluded.last_resumed_at,
            last_continued_at = excluded.last_continued_at,
            paused_at = excluded.paused_at,
            process_state = excluded.process_state,
            process_id = excluded.process_id,
            exit_code = excluded.exit_code,
            started_at = excluded.started_at,
            runtime_seconds = excluded.runtime_seconds,
            event_count = excluded.event_count,
            input_tokens = excluded.input_tokens,
            output_tokens = excluded.output_tokens,
            total_tokens = excluded.total_tokens,
            last_activity_at = excluded.last_activity_at",
        params![
            state.session_id,
            state.assistant,
            state.controller,
            if state.available { 1 } else { 0 },
            if state.attached { 1 } else { 0 },
            if state.paused { 1 } else { 0 },
            state.last_command,
            state.last_prompt,
            state.last_response,
            state.last_error,
            state.last_resumed_at,
            state.last_continued_at,
            state.paused_at,
            state.process_state,
            state.process_id,
            state.exit_code,
            state.started_at,
            state.runtime_seconds,
            state.event_count,
            state.input_tokens,
            state.output_tokens,
            state.total_tokens,
            state.last_activity_at,
        ],
    )?;

    Ok(())
}

fn ensure_session_control_state_columns(connection: &Connection) -> Result<()> {
    let existing = table_column_names(connection, "session_control_state")?;

    for (column, definition) in [
        ("paused", "INTEGER NOT NULL DEFAULT 0"),
        ("paused_at", "TEXT"),
        ("process_state", "TEXT"),
        ("process_id", "INTEGER"),
        ("exit_code", "INTEGER"),
        ("started_at", "TEXT"),
        ("runtime_seconds", "INTEGER"),
        ("event_count", "INTEGER NOT NULL DEFAULT 0"),
        ("input_tokens", "INTEGER NOT NULL DEFAULT 0"),
        ("output_tokens", "INTEGER NOT NULL DEFAULT 0"),
        ("total_tokens", "INTEGER NOT NULL DEFAULT 0"),
        ("last_activity_at", "TEXT"),
    ] {
        if existing.iter().any(|name| name == column) {
            continue;
        }

        connection.execute(
            &format!("ALTER TABLE session_control_state ADD COLUMN {column} {definition}"),
            [],
        )?;
    }

    Ok(())
}

fn table_column_names(connection: &Connection, table_name: &str) -> Result<Vec<String>> {
    let mut statement = connection.prepare(&format!(
        "PRAGMA table_info({})",
        quote_identifier(table_name)
    ))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    rows.collect()
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('\"', "\"\""))
}

pub fn insert_session_control_event(
    connection: &Connection,
    event: &SessionControlEventRow,
) -> Result<()> {
    connection.execute(
        "INSERT INTO session_control_events (
            event_id,
            session_id,
            operation,
            created_at,
            prompt,
            response,
            result,
            error_message,
            command
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            event.event_id,
            event.session_id,
            event.operation,
            event.created_at,
            event.prompt,
            event.response,
            event.result,
            event.error_message,
            event.command,
        ],
    )?;

    Ok(())
}
