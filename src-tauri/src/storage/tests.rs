use rusqlite::Connection;

use super::sqlite::{bootstrap_database, table_names};

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
    ] {
        assert!(
            tables.iter().any(|table| table == expected),
            "expected table `{expected}` in {tables:?}"
        );
    }
}
