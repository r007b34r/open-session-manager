use std::path::{Path, PathBuf};

pub fn codex_user_config(home_dir: &Path) -> PathBuf {
    home_dir.join(".codex").join("config.toml")
}

pub fn claude_user_settings(home_dir: &Path) -> PathBuf {
    home_dir.join(".claude").join("settings.json")
}
