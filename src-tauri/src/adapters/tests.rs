use std::path::PathBuf;

use super::{
    claude_code::ClaudeCodeAdapter, codex::CodexAdapter, opencode::OpenCodeAdapter,
    traits::SessionAdapter,
};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures")
        .canonicalize()
        .expect("fixtures root resolves")
}

#[test]
fn codex_adapter_discovers_and_parses_fixture() {
    let adapter = CodexAdapter;
    let root = fixtures_root().join("codex");

    let discovered = adapter
        .discover_session_files(&root)
        .expect("codex fixtures discover");

    assert_eq!(discovered.len(), 1);

    let session = adapter
        .parse_session(&discovered[0])
        .expect("codex fixture parses");

    assert_eq!(session.assistant, "codex");
    assert_eq!(session.session_id, "codex-ses-1");
    assert_eq!(session.project_path.as_deref(), Some(r"C:\Projects\demo"));
    assert_eq!(session.message_count, 2);
    assert_eq!(session.raw_format, "codex-jsonl");
    assert!(!session.content_hash.is_empty());
}

#[test]
fn claude_adapter_discovers_and_parses_fixture() {
    let adapter = ClaudeCodeAdapter;
    let root = fixtures_root().join("claude");

    let discovered = adapter
        .discover_session_files(&root)
        .expect("claude fixtures discover");

    assert_eq!(discovered.len(), 1);

    let session = adapter
        .parse_session(&discovered[0])
        .expect("claude fixture parses");

    assert_eq!(session.assistant, "claude-code");
    assert_eq!(session.session_id, "claude-ses-1");
    assert_eq!(
        session.project_path.as_deref(),
        Some(r"C:\Projects\claude-demo")
    );
    assert_eq!(session.message_count, 2);
    assert_eq!(session.raw_format, "claude-code-jsonl");
    assert!(!session.content_hash.is_empty());
}

#[test]
fn opencode_adapter_discovers_and_parses_fixture() {
    let adapter = OpenCodeAdapter;
    let root = fixtures_root().join("opencode");

    let discovered = adapter
        .discover_session_files(&root)
        .expect("opencode fixtures discover");

    assert_eq!(discovered.len(), 1);

    let session = adapter
        .parse_session(&discovered[0])
        .expect("opencode fixture parses");

    assert_eq!(session.assistant, "opencode");
    assert_eq!(session.session_id, "ses_demo");
    assert_eq!(session.project_path.as_deref(), Some("/home/max/project"));
    assert_eq!(session.message_count, 2);
    assert_eq!(session.tool_count, 1);
    assert_eq!(session.raw_format, "opencode-storage");
    assert!(!session.content_hash.is_empty());
}
