#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, path::PathBuf, process::ExitCode};

use open_session_manager_core::{
    commands::dashboard::{
        DashboardSnapshot, build_fixture_dashboard_snapshot_with_audit,
        build_local_dashboard_snapshot_with_audit, build_local_doctor_report,
    },
    api_server,
    mcp_server,
    commands::query::{expand_session, get_session, list_sessions, search_sessions, view_session},
    desktop,
    discovery::DiscoveryContext,
    preferences::build_runtime_paths,
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
        Some("doctor") => match run_doctor_command(&args[1..]) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Some("list") => match run_list_command(&args[1..]) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Some("search") => match run_search_command(&args[1..]) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Some("get") => match run_get_command(&args[1..]) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Some("view") => match run_view_command(&args[1..]) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Some("expand") => match run_expand_command(&args[1..]) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Some("serve") => match api_server::run(&args[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        },
        Some("mcp") => match mcp_server::run(&args[1..]) {
            Ok(()) => ExitCode::SUCCESS,
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
    let snapshot = load_snapshot_data(args)?;
    render_json(&snapshot, has_flag(args, "--json"))
}

fn run_list_command(args: &[String]) -> Result<String, String> {
    let snapshot = load_snapshot_data(args)?;
    render_json(&list_sessions(&snapshot), has_flag(args, "--json"))
}

fn run_search_command(args: &[String]) -> Result<String, String> {
    let snapshot = load_snapshot_data(args)?;
    let query = require_flag_value(args, "--query")?;
    render_json(&search_sessions(&snapshot, query), has_flag(args, "--json"))
}

fn run_get_command(args: &[String]) -> Result<String, String> {
    let snapshot = load_snapshot_data(args)?;
    let session_id = require_flag_value(args, "--session")?;
    let payload =
        get_session(&snapshot, session_id).ok_or_else(|| format!("session not found: {session_id}"))?;
    render_json(&payload, has_flag(args, "--json"))
}

fn run_view_command(args: &[String]) -> Result<String, String> {
    let snapshot = load_snapshot_data(args)?;
    let session_id = require_flag_value(args, "--session")?;
    let payload =
        view_session(&snapshot, session_id).ok_or_else(|| format!("session not found: {session_id}"))?;
    render_json(&payload, has_flag(args, "--json"))
}

fn run_expand_command(args: &[String]) -> Result<String, String> {
    let snapshot = load_snapshot_data(args)?;
    let session_id = require_flag_value(args, "--session")?;
    let payload = expand_session(&snapshot, session_id)
        .ok_or_else(|| format!("session not found: {session_id}"))?;
    render_json(&payload, has_flag(args, "--json"))
}

fn load_snapshot_data(args: &[String]) -> Result<DashboardSnapshot, String> {
    let audit_db_path = parse_flag_value(args, "--audit-db").map(PathBuf::from);

    if let Some(fixtures_path) = parse_flag_value(args, "--fixtures") {
        let mut snapshot = build_fixture_dashboard_snapshot_with_audit(
            &PathBuf::from(fixtures_path),
            audit_db_path.as_deref(),
        )
        .map_err(|error| error.to_string())?;
        if let Ok(runtime) = build_runtime_paths() {
            snapshot.runtime = runtime.snapshot();
        }
        return Ok(snapshot);
    }

    let mut snapshot = build_local_dashboard_snapshot_with_audit(
        &build_discovery_context(),
        audit_db_path.as_deref(),
    )
    .map_err(|error| error.to_string())?;
    if let Ok(mut runtime) = build_runtime_paths() {
        if let Some(custom_audit_db_path) = audit_db_path {
            runtime.audit_db_path = custom_audit_db_path.to_path_buf();
        }
        snapshot.runtime = runtime.snapshot();
    }
    Ok(snapshot)
}

fn run_doctor_command(args: &[String]) -> Result<String, String> {
    let report =
        build_local_doctor_report(&build_discovery_context()).map_err(|error| error.to_string())?;
    render_json(&report, has_flag(args, "--json"))
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

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|argument| argument == flag)
}

fn require_flag_value<'a>(args: &'a [String], flag: &str) -> Result<&'a str, String> {
    parse_flag_value(args, flag).ok_or_else(|| format!("missing required flag: {flag}"))
}

fn render_json(value: &impl serde::Serialize, compact: bool) -> Result<String, String> {
    if compact {
        serde_json::to_string(value).map_err(|error| error.to_string())
    } else {
        serde_json::to_string_pretty(value).map_err(|error| error.to_string())
    }
}
