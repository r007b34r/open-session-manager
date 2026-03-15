use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{
    commands::{
        actions::{delete_session, export_session},
        dashboard::{
            DashboardSnapshot, IndexedSession, build_local_dashboard_snapshot_with_audit,
            build_local_indexed_sessions,
        },
    },
    discovery::DiscoveryContext,
    storage::sqlite::open_database,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimePaths {
    audit_db_path: PathBuf,
    export_root: PathBuf,
    quarantine_root: PathBuf,
}

pub fn run() -> Result<(), String> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_dashboard_snapshot,
            export_session_markdown,
            soft_delete_session
        ])
        .run(tauri::generate_context!())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_dashboard_snapshot() -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths();

    build_local_dashboard_snapshot_with_audit(&context, Some(&paths.audit_db_path))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn export_session_markdown(session_id: String) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths();
    let indexed = resolve_indexed_session(&context, &session_id)?;
    let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

    export_session(
        &indexed.session,
        &indexed.insight,
        &paths.export_root,
        &resolve_actor(),
        &connection,
    )
    .map_err(|error| error.to_string())?;

    build_local_dashboard_snapshot_with_audit(&context, Some(&paths.audit_db_path))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn soft_delete_session(session_id: String) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths();
    let indexed = resolve_indexed_session(&context, &session_id)?;
    let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

    delete_session(
        &indexed.session,
        &paths.quarantine_root,
        &resolve_actor(),
        &connection,
    )
    .map_err(|error| error.to_string())?;

    build_local_dashboard_snapshot_with_audit(&context, Some(&paths.audit_db_path))
        .map_err(|error| error.to_string())
}

fn resolve_indexed_session(
    context: &DiscoveryContext,
    session_id: &str,
) -> Result<IndexedSession, String> {
    build_local_indexed_sessions(context)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|indexed| indexed.session.session_id == session_id)
        .ok_or_else(|| format!("session not found: {session_id}"))
}

fn build_discovery_context() -> DiscoveryContext {
    DiscoveryContext {
        home_dir: resolve_home_dir(),
        xdg_config_home: env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        xdg_data_home: env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        wsl_home_dir: env::var_os("AGENT_SESSION_GOVERNANCE_WSL_HOME").map(PathBuf::from),
    }
}

fn build_runtime_paths() -> RuntimePaths {
    let data_root = resolve_app_data_root();
    let export_root = resolve_export_root(&data_root);

    RuntimePaths {
        audit_db_path: data_root.join("audit").join("audit.db"),
        export_root,
        quarantine_root: data_root.join("quarantine"),
    }
}

fn resolve_app_data_root() -> PathBuf {
    if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data).join("AgentSessionGovernance");
    }

    if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data_home).join("agent-session-governance");
    }

    resolve_home_dir()
        .join(".local")
        .join("share")
        .join("agent-session-governance")
}

fn resolve_export_root(data_root: &Path) -> PathBuf {
    let home_dir = resolve_home_dir();
    if home_dir.as_os_str().is_empty() {
        return data_root.join("exports");
    }

    home_dir.join("Documents").join("AgentSessionGovernance").join("exports")
}

fn resolve_home_dir() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("current dir resolves"))
}

fn resolve_actor() -> String {
    env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .unwrap_or_else(|_| "local-user".to_string())
}
