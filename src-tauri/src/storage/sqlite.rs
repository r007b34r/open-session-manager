use rusqlite::{Connection, Result};

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
