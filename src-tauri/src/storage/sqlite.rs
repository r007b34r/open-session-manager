use std::path::Path;

use rusqlite::{Connection, Result};

use crate::domain::audit::AuditEvent;

const SCHEMA: &str = include_str!("schema.sql");

pub fn bootstrap_database(connection: &Connection) -> Result<()> {
    connection.execute_batch(SCHEMA)
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
