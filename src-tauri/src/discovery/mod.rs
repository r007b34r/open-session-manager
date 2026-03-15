use std::path::PathBuf;

pub mod linux;
pub mod windows;
pub mod wsl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryContext {
    pub home_dir: PathBuf,
    pub xdg_config_home: Option<PathBuf>,
    pub wsl_home_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPath {
    pub assistant: String,
    pub kind: String,
    pub environment: String,
    pub path: PathBuf,
}

impl KnownPath {
    pub fn new(
        assistant: impl Into<String>,
        kind: impl Into<String>,
        environment: impl Into<String>,
        path: PathBuf,
    ) -> Self {
        Self {
            assistant: assistant.into(),
            kind: kind.into(),
            environment: environment.into(),
            path,
        }
    }
}

#[cfg(test)]
mod tests;
