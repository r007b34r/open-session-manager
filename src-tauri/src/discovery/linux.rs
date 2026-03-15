use std::path::{Path, PathBuf};

pub fn opencode_user_config(home_dir: &Path, xdg_config_home: Option<&Path>) -> PathBuf {
    match xdg_config_home {
        Some(config_home) => config_home.join("opencode").join("opencode.json"),
        None => home_dir
            .join(".config")
            .join("opencode")
            .join("opencode.json"),
    }
}
