pub mod actions;
pub mod adapters;
pub mod api_server;
pub mod audit;
pub mod commands;
pub mod desktop;
pub mod discovery;
pub mod domain;
pub mod insights;
pub mod preferences;
pub mod session_text;
pub mod storage;
pub mod transcript;
pub mod usage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub app_name: &'static str,
    pub version: &'static str,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            app_name: "open-session-manager",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

pub fn health_check() -> &'static str {
    "ok"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_exposes_default_metadata() {
        let state = AppState::default();

        assert_eq!(state.app_name, "open-session-manager");
        assert_eq!(state.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(health_check(), "ok");
    }
}
