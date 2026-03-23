use std::path::PathBuf;

use open_session_manager_core::commands::dashboard::build_fixture_dashboard_snapshot;

#[test]
fn fixture_snapshot_exposes_mcp_servers_for_viewer() {
    let snapshot = build_fixture_dashboard_snapshot(&fixtures_root()).expect("snapshot builds");

    let copilot_filesystem = snapshot
        .configs
        .iter()
        .find(|config| config.assistant == "github-copilot-cli")
        .and_then(|config| {
            config
                .mcp_servers
                .iter()
                .find(|server| server.name == "filesystem")
        })
        .expect("copilot filesystem server should be exposed");
    assert_eq!(copilot_filesystem.status, "configured");
    assert_eq!(copilot_filesystem.transport, "stdio");
    assert_eq!(copilot_filesystem.command.as_deref(), Some("node"));
    assert_eq!(copilot_filesystem.args, vec!["mcp-filesystem.js"]);

    let factory_postgres = snapshot
        .configs
        .iter()
        .find(|config| config.assistant == "factory-droid")
        .and_then(|config| {
            config
                .mcp_servers
                .iter()
                .find(|server| server.name == "postgres")
        })
        .expect("factory postgres server should be exposed");
    assert_eq!(factory_postgres.status, "configured");
    assert_eq!(factory_postgres.transport, "stdio");
    assert_eq!(factory_postgres.command.as_deref(), Some("uvx"));
    assert!(
        factory_postgres
            .config_json
            .contains("\"command\": \"uvx\"")
    );

    let opencode_filesystem = snapshot
        .configs
        .iter()
        .find(|config| config.assistant == "opencode")
        .and_then(|config| {
            config
                .mcp_servers
                .iter()
                .find(|server| server.name == "filesystem")
        })
        .expect("opencode filesystem server should be exposed");
    assert_eq!(opencode_filesystem.status, "enabled");
    assert_eq!(opencode_filesystem.transport, "embedded");
    assert!(opencode_filesystem.command.is_none());
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures")
        .canonicalize()
        .expect("fixtures root resolves")
}
