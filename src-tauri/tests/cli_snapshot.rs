use std::{path::PathBuf, process::Command};

use serde_json::Value;

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures")
        .canonicalize()
        .expect("fixtures root resolves")
}

#[test]
fn snapshot_command_emits_real_dashboard_json_from_fixtures() {
    let output = Command::new(env!("CARGO_BIN_EXE_agent-session-governance-core"))
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
