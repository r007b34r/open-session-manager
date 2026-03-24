use std::path::PathBuf;

use super::{
    linux::opencode_user_config,
    windows::{claude_user_settings, codex_user_config},
    DiscoveryContext,
    wsl::parse_wsl_distributions,
};
use crate::commands::discovery::discover_known_session_roots;

#[test]
fn resolves_known_windows_and_linux_roots() {
    let windows_home = PathBuf::from(r"C:\Users\Max");
    let linux_home = PathBuf::from("/home/max");

    assert_eq!(
        codex_user_config(&windows_home),
        PathBuf::from(r"C:\Users\Max\.codex\config.toml")
    );
    assert_eq!(
        claude_user_settings(&windows_home),
        PathBuf::from(r"C:\Users\Max\.claude\settings.json")
    );
    assert_eq!(
        opencode_user_config(&linux_home, None),
        PathBuf::from("/home/max/.config/opencode/opencode.json")
    );
}

#[test]
fn parses_wsl_distribution_output() {
    let output = "Ubuntu\r\nDebian\r\n\r\n";

    assert_eq!(
        parse_wsl_distributions(output),
        vec!["Ubuntu".to_string(), "Debian".to_string()]
    );
}

#[test]
fn discovers_qwen_and_roo_code_session_roots() {
    let home_dir = PathBuf::from(r"C:\Users\Max");
    let context = DiscoveryContext {
        home_dir: home_dir.clone(),
        xdg_config_home: None,
        xdg_data_home: None,
        wsl_home_dir: Some(PathBuf::from("/home/max")),
    };

    let roots = discover_known_session_roots(&context);

    assert!(roots.iter().any(|root| {
        root.assistant == "qwen-cli"
            && root.path == home_dir.join(".qwen").join("projects")
            && root.environment == "windows"
    }));
    assert!(roots.iter().any(|root| {
        root.assistant == "roo-code"
            && root.path
                == home_dir
                    .join(".config")
                    .join("Code")
                    .join("User")
                    .join("globalStorage")
                    .join("rooveterinaryinc.roo-cline")
                    .join("tasks")
            && root.environment == "windows"
    }));
    assert!(roots.iter().any(|root| {
        root.assistant == "qwen-cli"
            && root.path == PathBuf::from("/home/max").join(".qwen").join("projects")
            && root.environment == "wsl"
    }));
    assert!(roots.iter().any(|root| {
        root.assistant == "roo-code"
            && root.path
                == PathBuf::from("/home/max")
                    .join(".vscode-server")
                    .join("data")
                    .join("User")
                    .join("globalStorage")
                    .join("rooveterinaryinc.roo-cline")
                    .join("tasks")
            && root.environment == "wsl"
    }));
}
