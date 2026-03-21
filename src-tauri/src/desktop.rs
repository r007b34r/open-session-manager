use std::{env, path::PathBuf};

use serde::Deserialize;

use crate::{
    actions::config_writeback::ConfigWritebackUpdate,
    commands::{
        actions::{
            delete_session, export_session, write_config_artifact as apply_config_writeback,
        },
        dashboard::{
            DashboardSnapshot, IndexedSession, build_local_dashboard_snapshot_with_audit,
            build_local_indexed_sessions, find_local_config_target,
        },
    },
    discovery::DiscoveryContext,
    preferences::{RuntimePaths, build_runtime_paths, save_export_root_preference},
    storage::sqlite::open_database,
};

pub fn run() -> Result<(), String> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_dashboard_snapshot,
            export_session_markdown,
            soft_delete_session,
            save_dashboard_preferences,
            write_config_artifact
        ])
        .run(tauri::generate_context!())
        .map_err(|error| error.to_string())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigWritebackPayload {
    pub artifact_id: String,
    pub assistant: String,
    pub scope: String,
    pub path: String,
    pub provider: String,
    pub model: Option<String>,
    pub base_url: String,
    pub secret: Option<String>,
}

#[tauri::command]
pub async fn load_dashboard_snapshot() -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || build_snapshot_with_runtime(&context, &paths))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn export_session_markdown(session_id: String) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let indexed = resolve_indexed_session(&context, &session_id)?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

        export_session(
            &indexed.session,
            &indexed.insight,
            &paths.export_root,
            &actor,
            &connection,
        )
        .map_err(|error| error.to_string())?;

        build_snapshot_with_runtime(&context, &paths)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn soft_delete_session(session_id: String) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let indexed = resolve_indexed_session(&context, &session_id)?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

        delete_session(
            &indexed.session,
            &paths.quarantine_root,
            &actor,
            &connection,
        )
        .map_err(|error| error.to_string())?;

        build_snapshot_with_runtime(&context, &paths)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn save_dashboard_preferences(
    export_root: Option<String>,
) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = save_export_root_preference(export_root).map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || build_snapshot_with_runtime(&context, &paths))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn write_config_artifact(
    payload: ConfigWritebackPayload,
) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let target = find_local_config_target(&context, &payload.artifact_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("config artifact not found: {}", payload.artifact_id))?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;
        let backup_root = build_config_backup_root(&paths);

        apply_config_writeback(
            &target,
            &ConfigWritebackUpdate {
                provider: Some(payload.provider),
                model: payload.model,
                base_url: Some(payload.base_url),
                secret: payload.secret,
            },
            &backup_root,
            &actor,
            &connection,
        )
        .map_err(|error| error.to_string())?;

        build_snapshot_with_runtime(&context, &paths)
    })
    .await
    .map_err(|error| error.to_string())?
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
        wsl_home_dir: env::var_os("OPEN_SESSION_MANAGER_WSL_HOME").map(PathBuf::from),
    }
}

fn build_snapshot_with_runtime(
    context: &DiscoveryContext,
    paths: &RuntimePaths,
) -> Result<DashboardSnapshot, String> {
    let mut snapshot =
        build_local_dashboard_snapshot_with_audit(context, Some(&paths.audit_db_path))
            .map_err(|error| error.to_string())?;
    snapshot.runtime = paths.snapshot();
    Ok(snapshot)
}

fn build_config_backup_root(paths: &RuntimePaths) -> PathBuf {
    paths
        .audit_db_path
        .parent()
        .and_then(|path| path.parent())
        .map(|path| path.join("config-backups"))
        .unwrap_or_else(|| paths.quarantine_root.join("..").join("config-backups"))
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

#[cfg(test)]
mod tests {
    use std::future::Future;

    use super::{
        ConfigWritebackPayload, export_session_markdown, load_dashboard_snapshot,
        save_dashboard_preferences, soft_delete_session, write_config_artifact,
    };

    #[test]
    fn desktop_commands_are_async_futures() {
        fn assert_future<T: Future>(_: &T) {}

        let load = load_dashboard_snapshot();
        assert_future(&load);

        let export = export_session_markdown("session-id".to_string());
        assert_future(&export);

        let delete = soft_delete_session("session-id".to_string());
        assert_future(&delete);

        let save = save_dashboard_preferences(Some("D:/OSM/exports".to_string()));
        assert_future(&save);

        let write = write_config_artifact(ConfigWritebackPayload {
            artifact_id: "cfg-004".to_string(),
            assistant: "github-copilot-cli".to_string(),
            scope: "global".to_string(),
            path: "C:/Users/Max/.copilot/config.json".to_string(),
            provider: "github".to_string(),
            model: Some("gpt-5-mini".to_string()),
            base_url: "https://github.com/api/copilot".to_string(),
            secret: Some("ghu_new_secret_123454321".to_string()),
        });
        assert_future(&write);
    }
}
