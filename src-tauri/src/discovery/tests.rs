use std::path::PathBuf;

use super::{
    linux::opencode_user_config,
    windows::{claude_user_settings, codex_user_config},
    wsl::parse_wsl_distributions,
};

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
