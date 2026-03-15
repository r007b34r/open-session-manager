use std::path::PathBuf;

use crate::discovery::{
    DiscoveryContext, KnownPath,
    linux::opencode_user_config,
    windows::{claude_user_settings, codex_user_config},
};

pub fn discover_known_roots(context: &DiscoveryContext) -> Vec<KnownPath> {
    let mut roots = vec![
        KnownPath::new("codex", "config", "windows", codex_user_config(&context.home_dir)),
        KnownPath::new(
            "claude-code",
            "config",
            "windows",
            claude_user_settings(&context.home_dir),
        ),
        KnownPath::new(
            "opencode",
            "config",
            "linux",
            opencode_user_config(&context.home_dir, context.xdg_config_home.as_deref()),
        ),
    ];

    if let Some(wsl_home) = &context.wsl_home_dir {
        roots.push(KnownPath::new(
            "codex",
            "config",
            "wsl",
            PathBuf::from(wsl_home).join(".codex").join("config.toml"),
        ));
    }

    roots
}
