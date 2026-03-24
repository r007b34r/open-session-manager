use std::{
    env,
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    actions::config_writeback::ConfigWritebackUpdate,
    commands::{
        actions::{
            attach_existing_session as attach_existing_session_action,
            commit_git_project as commit_git_project_action,
            continue_existing_session as continue_existing_session_action, delete_session,
            detach_existing_session as detach_existing_session_action, export_session,
            pause_existing_session as pause_existing_session_action,
            push_git_project as push_git_project_action,
            resume_existing_session as resume_existing_session_action,
            switch_git_project_branch as switch_git_project_branch_action,
            write_config_artifact as apply_config_writeback,
        },
        dashboard::{
            DashboardSnapshot, IndexedSession, build_local_dashboard_snapshot_with_audit,
            build_local_indexed_sessions, find_local_config_target,
        },
        query::{
            expand_session, get_session, list_sessions_with_request, search_sessions_with_request,
            view_session,
        },
    },
    discovery::DiscoveryContext,
    preferences::{RuntimePaths, build_runtime_paths, save_export_root_preference},
    storage::sqlite::open_database,
};

pub use crate::commands::query::{ListSessionInventoryRequest, SearchSessionInventoryRequest};

const GIT_FILE_PREVIEW_MAX_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitProjectFilePreview {
    pub repo_root: String,
    pub relative_path: String,
    pub content: String,
    pub truncated: bool,
    pub byte_size: u64,
    pub line_count: usize,
}

pub fn run() -> Result<(), String> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_dashboard_snapshot,
            commit_git_project,
            attach_existing_session,
            export_session_markdown,
            push_git_project,
            preview_git_project_file,
            detach_existing_session,
            pause_existing_session,
            resume_existing_session,
            continue_existing_session,
            soft_delete_session,
            save_dashboard_preferences,
            switch_git_project_branch,
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
pub async fn list_session_inventory(
    request: Option<ListSessionInventoryRequest>,
) -> Result<Value, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        let snapshot = build_snapshot_with_runtime(&context, &paths)?;
        Ok(list_sessions_with_request(&snapshot, request.as_ref()))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn search_session_inventory(
    request: SearchSessionInventoryRequest,
) -> Result<Value, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        let snapshot = build_snapshot_with_runtime(&context, &paths)?;
        Ok(search_sessions_with_request(&snapshot, &request))
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
pub async fn attach_existing_session(session_id: String) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let indexed = resolve_indexed_session(&context, &session_id)?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

        attach_existing_session_action(&indexed.session, &actor, &connection)
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
pub async fn pause_existing_session(session_id: String) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let indexed = resolve_indexed_session(&context, &session_id)?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

        pause_existing_session_action(&indexed.session, &actor, &connection)
            .map_err(|error| error.to_string())?;

        build_snapshot_with_runtime(&context, &paths)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn detach_existing_session(session_id: String) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let indexed = resolve_indexed_session(&context, &session_id)?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

        detach_existing_session_action(&indexed.session, &actor, &connection)
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

#[tauri::command]
pub async fn commit_git_project(
    repo_root: String,
    message: String,
) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let resolved_repo_root = resolve_git_repo_root(&context, &paths, &repo_root)?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

        commit_git_project_action(&resolved_repo_root, &message, &actor, &connection)
            .map_err(|error| error.to_string())?;

        build_snapshot_with_runtime(&context, &paths)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn switch_git_project_branch(
    repo_root: String,
    branch: String,
) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let resolved_repo_root = resolve_git_repo_root(&context, &paths, &repo_root)?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

        switch_git_project_branch_action(&resolved_repo_root, &branch, &actor, &connection)
            .map_err(|error| error.to_string())?;

        build_snapshot_with_runtime(&context, &paths)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn push_git_project(
    repo_root: String,
    remote: Option<String>,
) -> Result<DashboardSnapshot, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;
    let actor = resolve_actor();

    tauri::async_runtime::spawn_blocking(move || {
        let resolved_repo_root = resolve_git_repo_root(&context, &paths, &repo_root)?;
        let connection = open_database(&paths.audit_db_path).map_err(|error| error.to_string())?;

        push_git_project_action(&resolved_repo_root, remote.as_deref(), &actor, &connection)
            .map_err(|error| error.to_string())?;

        build_snapshot_with_runtime(&context, &paths)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn preview_git_project_file(
    repo_root: String,
    relative_path: String,
) -> Result<GitProjectFilePreview, String> {
    let context = build_discovery_context();
    let paths = build_runtime_paths().map_err(|error| error.to_string())?;

    tauri::async_runtime::spawn_blocking(move || {
        let resolved_repo_root = resolve_git_repo_root(&context, &paths, &repo_root)?;
        build_git_project_file_preview(&resolved_repo_root, &relative_path)
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

fn resolve_git_repo_root(
    context: &DiscoveryContext,
    paths: &RuntimePaths,
    repo_root: &str,
) -> Result<PathBuf, String> {
    let snapshot = build_snapshot_with_runtime(context, paths)?;
    let requested = normalize_repo_root(repo_root);

    snapshot
        .git_projects
        .into_iter()
        .find(|project| normalize_repo_root(&project.repo_root) == requested)
        .map(|project| PathBuf::from(project.repo_root))
        .or_else(|| resolve_git_repo_root_fallback(repo_root))
        .ok_or_else(|| format!("git project not found: {repo_root}"))
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

fn normalize_repo_root(value: &str) -> String {
    PathBuf::from(value.trim()).display().to_string()
}

fn resolve_git_repo_root_fallback(repo_root: &str) -> Option<PathBuf> {
    let candidate = PathBuf::from(repo_root.trim());
    if !candidate.exists() {
        return None;
    }

    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&candidate)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if resolved.is_empty() {
        None
    } else {
        Some(PathBuf::from(resolved))
    }
}

fn build_git_project_file_preview(
    repo_root: &Path,
    relative_path: &str,
) -> Result<GitProjectFilePreview, String> {
    let canonical_repo_root = repo_root
        .canonicalize()
        .map_err(|error| format!("failed to resolve repo root {}: {error}", repo_root.display()))?;
    let normalized_relative_path = normalize_git_relative_path(relative_path)?;
    let resolved_path =
        resolve_git_project_preview_path(&canonical_repo_root, &normalized_relative_path)?;
    let mut file = File::open(&resolved_path)
        .map_err(|error| format!("failed to open {}: {error}", resolved_path.display()))?;
    let byte_size = file
        .metadata()
        .map_err(|error| format!("failed to read metadata for {}: {error}", resolved_path.display()))?
        .len();
    let mut preview_bytes = Vec::with_capacity(GIT_FILE_PREVIEW_MAX_BYTES.min(byte_size as usize));
    file.by_ref()
        .take(GIT_FILE_PREVIEW_MAX_BYTES as u64 + 1)
        .read_to_end(&mut preview_bytes)
        .map_err(|error| format!("failed to read preview for {}: {error}", resolved_path.display()))?;

    let truncated = preview_bytes.len() > GIT_FILE_PREVIEW_MAX_BYTES;
    if truncated {
        preview_bytes.truncate(GIT_FILE_PREVIEW_MAX_BYTES);
    }

    if preview_bytes.contains(&0) {
        return Err(format!(
            "binary file preview is not supported: {}",
            resolved_path.display()
        ));
    }

    let content = String::from_utf8_lossy(&preview_bytes).into_owned();
    let line_count = if content.is_empty() {
        0
    } else {
        content.lines().count()
    };

    Ok(GitProjectFilePreview {
        repo_root: canonical_repo_root.display().to_string(),
        relative_path: normalized_relative_path,
        content,
        truncated: truncated || byte_size > GIT_FILE_PREVIEW_MAX_BYTES as u64,
        byte_size,
        line_count,
    })
}

fn normalize_git_relative_path(value: &str) -> Result<String, String> {
    let normalized = value.trim().replace('\\', "/");
    if normalized.is_empty() {
        return Err("relative path is required".to_string());
    }

    let path = Path::new(&normalized);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "requested path points outside the repository root: {normalized}"
        ));
    }

    Ok(normalized)
}

fn resolve_git_project_preview_path(
    repo_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, String> {
    let candidate = repo_root.join(relative_path);
    let resolved = candidate
        .canonicalize()
        .map_err(|error| format!("failed to resolve preview path {}: {error}", candidate.display()))?;

    if !resolved.starts_with(repo_root) {
        return Err(format!(
            "requested path points outside the repository root: {relative_path}"
        ));
    }

    let metadata = resolved
        .metadata()
        .map_err(|error| format!("failed to read metadata for {}: {error}", resolved.display()))?;
    if !metadata.is_file() {
        return Err(format!("preview target is not a file: {}", resolved.display()));
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        future::Future,
        path::{Path, PathBuf},
        process::Command,
        sync::atomic::{AtomicU64, Ordering},
    };

    use serde_json::Value;

    use crate::test_support::lock_env;

    use super::{
        ConfigWritebackPayload, ListSessionInventoryRequest, SearchSessionInventoryRequest,
        attach_existing_session, commit_git_project, continue_existing_session,
        detach_existing_session, expand_session_detail, export_session_markdown,
        get_session_detail, list_session_inventory, load_dashboard_snapshot,
        pause_existing_session, preview_git_project_file, push_git_project,
        resume_existing_session, save_dashboard_preferences, search_session_inventory,
        soft_delete_session, switch_git_project_branch, view_session_detail,
        write_config_artifact,
    };

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

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

        let attach = attach_existing_session("session-id".to_string());
        assert_future(&attach);

        let pause = pause_existing_session("session-id".to_string());
        assert_future(&pause);

        let continue_session = continue_existing_session(
            "session-id".to_string(),
            "Continue with verification".to_string(),
        );
        assert_future(&continue_session);

        let detach = detach_existing_session("session-id".to_string());
        assert_future(&detach);

        let save = save_dashboard_preferences(Some("D:/OSM/exports".to_string()));
        assert_future(&save);

        let list = list_session_inventory(None);
        assert_future(&list);

        let search = search_session_inventory(SearchSessionInventoryRequest {
            query: "Claude".to_string(),
            assistant: None,
            limit: None,
            offset: None,
            sort_by: None,
            descending: None,
        });
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

        let commit = commit_git_project(
            "C:/Projects/osm".to_string(),
            "feat: git dashboard".to_string(),
        );
        assert_future(&commit);

        let switch_branch = switch_git_project_branch(
            "C:/Projects/osm".to_string(),
            "feature/git-dashboard".to_string(),
        );
        assert_future(&switch_branch);

        let push = push_git_project("C:/Projects/osm".to_string(), None);
        assert_future(&push);

        let preview = preview_git_project_file(
            "C:/Projects/osm".to_string(),
            "README.md".to_string(),
        );
        assert_future(&preview);
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
                let list = list_session_inventory(None)
                    .await
                    .expect("list query command");
                let search = search_session_inventory(SearchSessionInventoryRequest {
                    query: "Claude".to_string(),
                    assistant: None,
                    limit: None,
                    offset: None,
                    sort_by: None,
                    descending: None,
                })
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
                    list.get("sessions").and_then(Value::as_array).map(Vec::len),
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
                    list.get("sessions").and_then(Value::as_array).map(Vec::len),
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

    #[test]
    fn git_project_commands_commit_switch_and_push() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");
        let repo_root = sandbox.join("repos").join("git-demo");
        let remote_root = sandbox.join("remote.git");

        init_bare_git_repo(&remote_root);
        init_git_repo(&repo_root);
        git(
            &repo_root,
            &[
                "remote",
                "add",
                "origin",
                remote_root.to_str().expect("remote path"),
            ],
        );
        git(&repo_root, &["push", "-u", "origin", "main"]);
        seed_git_project_session(&home_dir, &repo_root);

        with_home_dir(&home_dir, || {
            tauri::async_runtime::block_on(async {
                fs::write(repo_root.join("README.md"), "# commit from desktop\n")
                    .expect("write dirty readme");

                let after_commit = commit_git_project(
                    repo_root.display().to_string(),
                    "feat: desktop git commit".to_string(),
                )
                .await
                .expect("commit git project");

                assert!(after_commit.audit_events.iter().any(|event| {
                    event.r#type == "git_commit"
                        && event.detail.contains("Committed feat: desktop git commit")
                }));
                assert_eq!(
                    git(&repo_root, &["log", "-1", "--pretty=%s"]),
                    "feat: desktop git commit"
                );

                let after_push = push_git_project(repo_root.display().to_string(), None)
                    .await
                    .expect("push git project");
                assert!(after_push.audit_events.iter().any(|event| {
                    event.r#type == "git_push" && event.detail.contains("Pushed main to origin")
                }));

                let after_switch = switch_git_project_branch(
                    repo_root.display().to_string(),
                    "feature/git-dashboard".to_string(),
                )
                .await
                .expect("switch git branch");

                assert!(after_switch.audit_events.iter().any(|event| {
                    event.r#type == "git_branch_switch"
                        && event.detail.contains("Switched to feature/git-dashboard")
                }));
                assert_eq!(
                    git(&repo_root, &["branch", "--show-current"]),
                    "feature/git-dashboard"
                );
            })
        });
    }

    #[test]
    fn git_project_file_preview_reads_files_and_rejects_escape() {
        let sandbox = temp_root();
        let home_dir = sandbox.join("home");
        let repo_root = sandbox.join("repos").join("git-preview");

        init_git_repo(&repo_root);
        fs::create_dir_all(repo_root.join("src")).expect("create src");
        fs::write(repo_root.join("src").join("main.rs"), "fn main() {}\n")
            .expect("write preview file");
        seed_git_project_session(&home_dir, &repo_root);

        with_home_dir(&home_dir, || {
            tauri::async_runtime::block_on(async {
                let preview = preview_git_project_file(
                    repo_root.display().to_string(),
                    "src/main.rs".to_string(),
                )
                .await
                .expect("preview repo file");

                assert_eq!(preview.relative_path, "src/main.rs");
                assert!(preview.content.contains("fn main()"));
                assert!(preview.byte_size > 0);
                assert!(preview.line_count > 0);

                let escaped = preview_git_project_file(
                    repo_root.display().to_string(),
                    "..\\secret.txt".to_string(),
                )
                .await;

                assert!(escaped.is_err());
                assert!(
                    escaped
                        .err()
                        .is_some_and(|error| error.contains("outside the repository root"))
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

    fn seed_git_project_session(home_dir: &Path, repo_root: &Path) {
        let codex_root = home_dir
            .join(".codex")
            .join("sessions")
            .join("2026")
            .join("03");
        let escaped_repo_path = repo_root.display().to_string().replace('\\', "\\\\");

        fs::create_dir_all(&codex_root).expect("create codex session root");
        fs::write(
            codex_root.join("git-dashboard-2026-03-23.jsonl"),
            format!(
                concat!(
                    "{{\"timestamp\":\"2026-03-23T04:00:00.000Z\",\"type\":\"session_meta\",",
                    "\"payload\":{{\"id\":\"codex-git-001\",\"timestamp\":\"2026-03-23T04:00:00.000Z\",",
                    "\"cwd\":\"{}\",\"originator\":\"codex_cli_rs\",",
                    "\"cli_version\":\"0.97.0\",\"source\":\"cli\"}}}}\n",
                    "{{\"timestamp\":\"2026-03-23T04:00:03.000Z\",\"type\":\"response_item\",",
                    "\"payload\":{{\"type\":\"message\",\"role\":\"user\",\"content\":[{{\"type\":\"input_text\",",
                    "\"text\":\"Inspect git governance state\"}}]}}}}\n",
                    "{{\"timestamp\":\"2026-03-23T04:00:06.000Z\",\"type\":\"response_item\",",
                    "\"payload\":{{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{{\"type\":\"output_text\",",
                    "\"text\":\"Git snapshot is pending.\"}}]}}}}\n"
                ),
                escaped_repo_path
            ),
        )
        .expect("write git project session");
    }

    fn with_home_dir<T>(home_dir: &Path, action: impl FnOnce() -> T) -> T {
        let _guard = lock_env();

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

    fn init_git_repo(repo_root: &Path) {
        fs::create_dir_all(repo_root).expect("create repo root");
        git(repo_root, &["init"]);
        git(repo_root, &["branch", "-M", "main"]);
        git(repo_root, &["config", "user.name", "OSM Test"]);
        git(repo_root, &["config", "user.email", "osm-test@example.com"]);
        fs::write(repo_root.join("README.md"), "# seed\n").expect("write readme");
        git(repo_root, &["add", "."]);
        git(repo_root, &["commit", "-m", "chore: seed repo"]);
    }

    fn init_bare_git_repo(repo_root: &Path) {
        fs::create_dir_all(repo_root).expect("create bare repo root");
        git(repo_root, &["init", "--bare"]);
    }

    fn git(repo_root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_root)
            .output()
            .expect("git command should run");

        if !output.status.success() {
            panic!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}
