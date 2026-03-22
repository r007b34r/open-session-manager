use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use open_session_manager_core::{
    actions::config_writeback::{ConfigWritebackRequest, ConfigWritebackUpdate, write_config},
    audit::config_audit::ConfigAuditTarget,
    storage::sqlite::{bootstrap_database, load_audit_events},
};
use rusqlite::Connection;
use serde_json::Value;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn writes_back_config_audit_event_with_backup_manifest_path() {
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

    let event = load_audit_events(&connection, 10)
        .expect("load audit events")
        .into_iter()
        .find(|event| event.event_type == "config_writeback")
        .expect("config writeback event");
    let after_state = event.after_state.expect("config writeback after state");
    let parsed: Value = serde_json::from_str(&after_state).expect("parse after state json");

    assert_eq!(
        parsed.get("manifest_path").and_then(Value::as_str),
        Some(result.manifest_path.to_string_lossy().as_ref())
    );
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
        "open-session-manager-config-backup-tests-{}-{suffix}",
        std::process::id(),
    ));

    if root.exists() {
        fs::remove_dir_all(&root).expect("reset temp root");
    }

    fs::create_dir_all(&root).expect("create temp root");
    root
}
