pub mod actions;
pub mod adapters;
pub mod audit;
pub mod commands;
pub mod discovery;
pub mod domain;
pub mod insights;
pub mod storage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub app_name: &'static str,
    pub version: &'static str,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            app_name: "agent-session-governance",
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

        assert_eq!(state.app_name, "agent-session-governance");
        assert_eq!(state.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(health_check(), "ok");
    }
}
