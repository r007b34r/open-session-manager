#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, path::PathBuf, process::ExitCode};

use open_session_manager_core::{
    commands::dashboard::{
        build_fixture_dashboard_snapshot_with_audit, build_local_dashboard_snapshot_with_audit,
    },
    desktop,
    discovery::DiscoveryContext,
};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();

    match args.first().map(String::as_str) {
        None => match desktop::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Some("snapshot") => match run_snapshot_command(&args[1..]) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Some(command) => {
            eprintln!("unsupported command: {command}");
            ExitCode::FAILURE
        }
    }
}

fn run_snapshot_command(args: &[String]) -> Result<String, String> {
    let audit_db_path = parse_flag_value(args, "--audit-db").map(PathBuf::from);

    if let Some(fixtures_path) = parse_flag_value(args, "--fixtures") {
        let snapshot = build_fixture_dashboard_snapshot_with_audit(
            &PathBuf::from(fixtures_path),
            audit_db_path.as_deref(),
        )
        .map_err(|error| error.to_string())?;
        return serde_json::to_string_pretty(&snapshot).map_err(|error| error.to_string());
    }

    let snapshot =
        build_local_dashboard_snapshot_with_audit(&build_discovery_context(), audit_db_path.as_deref())
            .map_err(|error| error.to_string())?;
    serde_json::to_string_pretty(&snapshot).map_err(|error| error.to_string())
}

fn build_discovery_context() -> DiscoveryContext {
    DiscoveryContext {
        home_dir: resolve_home_dir(),
        xdg_config_home: env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        xdg_data_home: env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        wsl_home_dir: env::var_os("OPEN_SESSION_MANAGER_WSL_HOME").map(PathBuf::from),
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
