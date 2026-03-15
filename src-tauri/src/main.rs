use std::{env, path::PathBuf, process::ExitCode};

use agent_session_governance_core::{
    commands::dashboard::{
        build_fixture_dashboard_snapshot_with_audit, build_local_dashboard_snapshot_with_audit,
    },
    discovery::DiscoveryContext,
    health_check,
};

fn main() -> ExitCode {
    match run() {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        None => Ok(health_check().to_string()),
        Some("snapshot") => run_snapshot_command(&args[1..]),
        Some(command) => Err(format!("unsupported command: {command}")),
    }
}

fn run_snapshot_command(args: &[String]) -> Result<String, String> {
    let audit_db_path = parse_flag_value(args, "--audit-db").map(PathBuf::from);

    if let Some(fixtures_path) = parse_flag_value(args, "--fixtures") {
        let snapshot =
            build_fixture_dashboard_snapshot_with_audit(
                &PathBuf::from(fixtures_path),
                audit_db_path.as_deref(),
            )
            .map_err(|error| error.to_string())?;
        return serde_json::to_string_pretty(&snapshot).map_err(|error| error.to_string());
    }

    let snapshot = build_local_dashboard_snapshot_with_audit(
        &build_discovery_context(),
        audit_db_path.as_deref(),
    )
    .map_err(|error| error.to_string())?;
    serde_json::to_string_pretty(&snapshot).map_err(|error| error.to_string())
}

fn build_discovery_context() -> DiscoveryContext {
    DiscoveryContext {
        home_dir: resolve_home_dir(),
        xdg_config_home: env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        xdg_data_home: env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        wsl_home_dir: env::var_os("AGENT_SESSION_GOVERNANCE_WSL_HOME").map(PathBuf::from),
    }
}

fn resolve_home_dir() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("current dir resolves"))
}

fn parse_flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find_map(|window| (window[0] == flag).then_some(window[1].as_str()))
}
