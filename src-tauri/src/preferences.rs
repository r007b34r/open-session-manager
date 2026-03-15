use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum PreferencesError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for PreferencesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
        }
    }
}

impl std::error::Error for PreferencesError {}

impl From<io::Error> for PreferencesError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for PreferencesError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportRootSource {
    Default,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePaths {
    pub audit_db_path: PathBuf,
    pub export_root: PathBuf,
    pub default_export_root: PathBuf,
    pub export_root_source: ExportRootSource,
    pub quarantine_root: PathBuf,
    pub preferences_path: PathBuf,
}

impl RuntimePaths {
    pub fn snapshot(&self) -> RuntimeSnapshot {
        RuntimeSnapshot {
            audit_db_path: self.audit_db_path.display().to_string(),
            export_root: self.export_root.display().to_string(),
            default_export_root: self.default_export_root.display().to_string(),
            export_root_source: match self.export_root_source {
                ExportRootSource::Default => "default".to_string(),
                ExportRootSource::Custom => "custom".to_string(),
            },
            quarantine_root: self.quarantine_root.display().to_string(),
            preferences_path: self.preferences_path.display().to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub audit_db_path: String,
    pub export_root: String,
    pub default_export_root: String,
    pub export_root_source: String,
    pub quarantine_root: String,
    pub preferences_path: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct StoredPreferences {
    #[serde(default)]
    export_root: Option<String>,
}

pub fn build_runtime_paths() -> Result<RuntimePaths, PreferencesError> {
    let data_root = resolve_app_data_root();
    let home_dir = resolve_home_dir();
    let preferences_path = data_root.join("preferences.json");
    let preferences = load_preferences(&preferences_path)?;

    Ok(resolve_runtime_paths(
        data_root,
        home_dir,
        preferences_path,
        &preferences,
    ))
}

pub fn save_export_root_preference(
    export_root: Option<String>,
) -> Result<RuntimePaths, PreferencesError> {
    let data_root = resolve_app_data_root();
    let home_dir = resolve_home_dir();
    let preferences_path = data_root.join("preferences.json");
    let mut preferences = load_preferences(&preferences_path)?;

    preferences.export_root = export_root.and_then(normalize_preference_value);
    write_preferences(&preferences_path, &preferences)?;

    Ok(resolve_runtime_paths(
        data_root,
        home_dir,
        preferences_path,
        &preferences,
    ))
}

fn resolve_runtime_paths(
    data_root: PathBuf,
    home_dir: PathBuf,
    preferences_path: PathBuf,
    preferences: &StoredPreferences,
) -> RuntimePaths {
    let default_export_root = resolve_default_export_root(&data_root, &home_dir);
    let export_root = match preferences.export_root.as_deref() {
        Some(value) => PathBuf::from(value),
        None => default_export_root.clone(),
    };

    RuntimePaths {
        audit_db_path: data_root.join("audit").join("audit.db"),
        export_root,
        default_export_root,
        export_root_source: if preferences.export_root.is_some() {
            ExportRootSource::Custom
        } else {
            ExportRootSource::Default
        },
        quarantine_root: data_root.join("quarantine"),
        preferences_path,
    }
}

fn load_preferences(path: &Path) -> Result<StoredPreferences, PreferencesError> {
    if !path.exists() {
        return Ok(StoredPreferences::default());
    }

    let content = fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(StoredPreferences::default());
    }

    serde_json::from_str(&content).map_err(Into::into)
}

fn write_preferences(
    path: &Path,
    preferences: &StoredPreferences,
) -> Result<(), PreferencesError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, serde_json::to_string_pretty(preferences)?)?;
    Ok(())
}

fn resolve_app_data_root() -> PathBuf {
    if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data).join("OpenSessionManager");
    }

    if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data_home).join("open-session-manager");
    }

    resolve_home_dir()
        .join(".local")
        .join("share")
        .join("open-session-manager")
}

fn resolve_default_export_root(data_root: &Path, home_dir: &Path) -> PathBuf {
    if home_dir.as_os_str().is_empty() {
        return data_root.join("exports");
    }

    home_dir.join("Documents").join("OpenSessionManager").join("exports")
}

fn resolve_home_dir() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("current dir resolves"))
}

fn normalize_preference_value(value: String) -> Option<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::{
        ExportRootSource, StoredPreferences, load_preferences, resolve_runtime_paths,
        write_preferences,
    };

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn runtime_paths_use_default_export_root_when_no_override_is_saved() {
        let sandbox = temp_root();
        let data_root = sandbox.join("data");
        let home_dir = sandbox.join("home");
        let preferences_path = data_root.join("preferences.json");

        let runtime = resolve_runtime_paths(
            data_root.clone(),
            home_dir.clone(),
            preferences_path.clone(),
            &StoredPreferences::default(),
        );

        assert_eq!(runtime.export_root, home_dir.join("Documents").join("OpenSessionManager").join("exports"));
        assert_eq!(runtime.export_root_source, ExportRootSource::Default);
        assert_eq!(runtime.preferences_path, preferences_path);
        assert_eq!(runtime.audit_db_path, data_root.join("audit").join("audit.db"));
    }

    #[test]
    fn runtime_paths_use_custom_export_root_when_override_exists() {
        let sandbox = temp_root();
        let data_root = sandbox.join("data");
        let home_dir = sandbox.join("home");
        let preferences_path = data_root.join("preferences.json");
        let custom_export_root = sandbox.join("custom-exports");

        let runtime = resolve_runtime_paths(
            data_root,
            home_dir,
            preferences_path,
            &StoredPreferences {
                export_root: Some(custom_export_root.display().to_string()),
            },
        );

        assert_eq!(runtime.export_root, custom_export_root);
        assert_eq!(runtime.export_root_source, ExportRootSource::Custom);
    }

    #[test]
    fn preferences_round_trip_export_root_override() {
        let sandbox = temp_root();
        let preferences_path = sandbox.join("preferences.json");
        let custom_export_root = sandbox.join("exports");

        write_preferences(
            &preferences_path,
            &StoredPreferences {
                export_root: Some(custom_export_root.display().to_string()),
            },
        )
        .expect("write preferences");

        let loaded = load_preferences(&preferences_path).expect("load preferences");
        assert_eq!(
            loaded.export_root,
            Some(custom_export_root.display().to_string())
        );
    }

    fn temp_root() -> PathBuf {
        let suffix = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "open-session-manager-preferences-tests-{}-{suffix}",
            std::process::id(),
        ));

        if root.exists() {
            fs::remove_dir_all(&root).expect("reset temp root");
        }

        fs::create_dir_all(&root).expect("create temp root");
        root
    }
}
