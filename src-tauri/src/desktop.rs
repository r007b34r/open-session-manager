use std::{env, path::PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::{
    actions::config_writeback::ConfigWritebackUpdate,
    commands::{
        actions::{
            continue_existing_session as continue_existing_session_action, delete_session,
            export_session, resume_existing_session as resume_existing_session_action,
            write_config_artifact as apply_config_writeback,
        },
        dashboard::{
            DashboardSnapshot, IndexedSession, build_local_dashboard_snapshot_with_audit,
            build_local_indexed_sessions, find_local_config_target,
        },
        query::{
            expand_session, get_session, list_sessions, search_sessions, view_session,
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
            resume_existing_session,
            continue_existing_session,
            soft_delete_session,
            save_dashboard_preferences,
            write_config_artifact,
            list_session_inventory,
            search_session_inventory,
            get_session_detail,
            view_session_detail,
            expand_session_detail
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
pub async fn list_session_inventory() -> Result<Value, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        let snapshot = build_snapshot_with_runtime(&context, &paths)?;
        Ok(list_sessions(&snapshot))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn search_session_inventory(query: String) -> Result<Value, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        let snapshot = build_snapshot_with_runtime(&context, &paths)?;
        Ok(search_sessions(&snapshot, &query))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_session_detail(session_id: String) -> Result<Value, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        let snapshot = build_snapshot_with_runtime(&context, &paths)?;
        get_session(&snapshot, &session_id)
            .ok_or_else(|| format!("session not found: {session_id}"))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn view_session_detail(session_id: String) -> Result<Value, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        let snapshot = build_snapshot_with_runtime(&context, &paths)?;
        view_session(&snapshot, &session_id)
            .ok_or_else(|| format!("session not found: {session_id}"))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn expand_session_detail(session_id: String) -> Result<Value, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        let snapshot = build_snapshot_with_runtime(&context, &paths)?;
        expand_session(&snapshot, &session_id)
            .ok_or_else(|| format!("session not found: {session_id}"))
    })
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
pub async fn resume_existing_session(session_id: String) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let indexed = resolve_indexed_session(&context, &session_id)?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

        resume_existing_session_action(&indexed.session, &actor, &connection)
            .map_err(|error| error.to_string())?;

        build_snapshot_with_runtime(&context, &paths)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn continue_existing_session(
    session_id: String,
    prompt: String,
) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let indexed = resolve_indexed_session(&context, &session_id)?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

        continue_existing_session_action(&indexed.session, &prompt, &actor, &connection)
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
    use std::{
        env, fs,
        future::Future,
        path::{Path, PathBuf},
        sync::{
            Mutex, OnceLock,
            atomic::{AtomicU64, Ordering},
        },
    };

    use serde_json::Value;

    use super::{
        ConfigWritebackPayload, continue_existing_session, expand_session_detail,
        export_session_markdown, get_session_detail, list_session_inventory,
        ListSessionInventoryRequest,
        load_dashboard_snapshot, resume_existing_session, save_dashboard_preferences,
        SearchSessionInventoryRequest,
        search_session_inventory, soft_delete_session, view_session_detail,
        write_config_artifact,
    };

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn desktop_commands_are_async_futures() {
        fn assert_future<T: Future>(_: &T) {}

        let load = load_dashboard_snapshot();
        assert_future(&load);

        let export = export_session_markdown("session-id".to_string());
        assert_future(&export);

        let delete = soft_delete_session("session-id".to_string());
        assert_future(&delete);

        let resume = resume_existing_session("session-id".to_string());
        assert_future(&resume);

        let continue_session = continue_existing_session(
            "session-id".to_string(),
            "Continue with verification".to_string(),
        );
        assert_future(&continue_session);

        let save = save_dashboard_preferences(Some("D:/OSM/exports".to_string()));
        assert_future(&save);

        let list = list_session_inventory();
        assert_future(&list);

        let search = search_session_inventory("Claude".to_string());
        assert_future(&search);

        let get = get_session_detail("claude-ses-1".to_string());
        assert_future(&get);

        let view = view_session_detail("claude-ses-1".to_string());
        assert_future(&view);

        let expand = expand_session_detail("claude-ses-1".to_string());
        assert_future(&expand);

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

    #[test]
    fn desktop_query_commands_surface_shared_session_payloads() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");

        seed_session_fixture(
            &home_dir
                .join(".codex")
                .join("sessions")
                .join("2026")
                .join("03")
                .join("15")
                .join("rollout-2026-03-15.jsonl"),
            "codex/2026/03/15/rollout-2026-03-15T12-00-00-codex-ses-1.jsonl",
        );
        seed_session_fixture(
            &home_dir
                .join(".claude")
                .join("projects")
                .join("C--Projects-Claude-Demo")
                .join("claude-ses-1.jsonl"),
            "claude/projects/C--Projects-Claude-Demo/claude-ses-1.jsonl",
        );
        seed_session_fixture(
            &home_dir.join(".claude").join("settings.json"),
            "configs/claude/settings.json",
        );

        with_home_dir(&home_dir, || {
            tauri::async_runtime::block_on(async {
                let list = list_session_inventory().await.expect("list query command");
                let search = search_session_inventory("Claude".to_string())
                    .await
                    .expect("search query command");
                let get = get_session_detail("claude-ses-1".to_string())
                    .await
                    .expect("get query command");
                let view = view_session_detail("claude-ses-1".to_string())
                    .await
                    .expect("view query command");
                let expand = expand_session_detail("claude-ses-1".to_string())
                    .await
                    .expect("expand query command");

                assert_eq!(
                    list.get("sessions")
                        .and_then(Value::as_array)
                        .map(Vec::len),
                    Some(2)
                );
                assert_eq!(
                    search
                        .get("hits")
                        .and_then(Value::as_array)
                        .and_then(|hits| hits.first())
                        .and_then(|hit| hit.get("sessionId"))
                        .and_then(Value::as_str),
                    Some("claude-ses-1")
                );
                assert_eq!(
                    get.get("assistant").and_then(Value::as_str),
                    Some("claude-code")
                );
                assert!(
                    view.get("content")
                        .and_then(Value::as_str)
                        .is_some_and(|content| content.contains("# 扫描 Claude transcripts"))
                );
                assert!(
                    expand
                        .get("relatedConfigs")
                        .and_then(Value::as_array)
                        .is_some_and(|configs| {
                            configs.iter().any(|config| {
                                config.get("assistant").and_then(Value::as_str)
                                    == Some("claude-code")
                            })
                        })
                );
            })
        });
    }

    #[test]
    fn desktop_query_api_supports_pagination_filtering_and_sorting() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");

        seed_session_fixture(
            &home_dir
                .join(".factory")
                .join("sessions")
                .join("project-a")
                .join("droid-session-1.jsonl"),
            "factory/sessions/project-a/droid-session-1.jsonl",
        );
        seed_session_fixture(
            &home_dir
                .join(".factory")
                .join("projects")
                .join("project-a")
                .join("stream-session-1.jsonl"),
            "factory/projects/project-a/stream-session-1.jsonl",
        );

        with_home_dir(&home_dir, || {
            tauri::async_runtime::block_on(async {
                let list = list_session_inventory(Some(ListSessionInventoryRequest {
                    assistant: Some("factory-droid".to_string()),
                    limit: Some(1),
                    offset: Some(1),
                    sort_by: Some("lastActivityAt".to_string()),
                    descending: Some(true),
                }))
                .await
                .expect("list query command");

                let search = search_session_inventory(SearchSessionInventoryRequest {
                    query: "branch".to_string(),
                    assistant: Some("factory-droid".to_string()),
                    limit: Some(1),
                    offset: Some(0),
                    sort_by: Some("title".to_string()),
                    descending: Some(false),
                })
                .await
                .expect("search query command");

                assert_eq!(
                    list.get("sessions")
                        .and_then(Value::as_array)
                        .map(Vec::len),
                    Some(1)
                );
                assert_eq!(
                    list.get("sessions")
                        .and_then(Value::as_array)
                        .and_then(|sessions| sessions.first())
                        .and_then(|session| session.get("sessionId"))
                        .and_then(Value::as_str),
                    Some("droid-session-1")
                );
                assert_eq!(
                    search
                        .get("hits")
                        .and_then(Value::as_array)
                        .and_then(|hits| hits.first())
                        .and_then(|hit| hit.get("sessionId"))
                        .and_then(Value::as_str),
                    Some("droid-session-1")
                );
            })
        });
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures")
            .canonicalize()
            .expect("fixtures root resolves")
    }

    fn temp_root() -> PathBuf {
        let suffix = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "open-session-manager-desktop-tests-{}-{suffix}",
            std::process::id(),
        ));

        if root.exists() {
            fs::remove_dir_all(&root).expect("reset temp root");
        }

        fs::create_dir_all(&root).expect("create temp root");
        root
    }

    fn seed_session_fixture(target: &Path, fixture_relative: &str) {
        fs::create_dir_all(target.parent().expect("target parent")).expect("create target dir");
        fs::copy(fixtures_root().join(fixture_relative), target).expect("copy fixture");
    }

    fn with_home_dir<T>(home_dir: &Path, action: impl FnOnce() -> T) -> T {
        let _guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("lock env guard");

        let original_home = env::var_os("HOME");
        let original_userprofile = env::var_os("USERPROFILE");

        unsafe {
            env::set_var("HOME", home_dir);
            env::set_var("USERPROFILE", home_dir);
        }

        let result = action();

        match original_home {
            Some(value) => unsafe {
                env::set_var("HOME", value);
            },
            None => unsafe {
                env::remove_var("HOME");
            },
        }
        match original_userprofile {
            Some(value) => unsafe {
                env::set_var("USERPROFILE", value);
            },
            None => unsafe {
                env::remove_var("USERPROFILE");
            },
        }

        result
    }
}
