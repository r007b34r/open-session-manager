use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{
    claude_code::ClaudeCodeAdapter,
    codex::CodexAdapter,
    copilot_cli::CopilotCliAdapter,
    factory_droid::FactoryDroidAdapter,
    gemini_cli::GeminiCliAdapter,
    openclaw::OpenClawAdapter,
    opencode::OpenCodeAdapter,
    traits::SessionAdapter,
};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures")
        .canonicalize()
        .expect("fixtures root resolves")
}

fn temp_root() -> PathBuf {
    let suffix = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "open-session-manager-adapter-tests-{}-{suffix}",
        std::process::id(),
    ));

    if root.exists() {
        fs::remove_dir_all(&root).expect("reset temp root");
    }

    fs::create_dir_all(&root).expect("create temp root");
    root
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
fn claude_adapter_discovery_skips_file_history_only_jsonl() {
    let adapter = ClaudeCodeAdapter;
    let root = temp_root();

    fs::write(
        root.join("claude-ses-real.jsonl"),
        concat!(
            "{\"type\":\"progress\",\"sessionId\":\"claude-ses-real\",",
            "\"cwd\":\"C:/Projects/claude-real\",\"timestamp\":\"2026-03-16T12:00:00Z\"}\n",
            "{\"type\":\"user\",\"sessionId\":\"claude-ses-real\",",
            "\"cwd\":\"C:/Projects/claude-real\",",
            "\"timestamp\":\"2026-03-16T12:00:01Z\",",
            "\"message\":{\"content\":\"Investigate the real cleanup candidates.\"}}\n",
            "{\"type\":\"assistant\",\"sessionId\":\"claude-ses-real\",",
            "\"cwd\":\"C:/Projects/claude-real\",",
            "\"timestamp\":\"2026-03-16T12:00:03Z\",",
            "\"message\":{\"content\":[{\"type\":\"text\",\"text\":\"Mapped the stale sessions and exports.\"}]}}\n"
        ),
    )
    .expect("write valid claude session");

    fs::write(
        root.join("file-history-only.jsonl"),
        concat!(
            "{\"type\":\"file-history-snapshot\",\"messageId\":\"msg-1\",",
            "\"snapshot\":{\"messageId\":\"msg-1\",\"timestamp\":\"2026-03-16T12:00:00Z\"},",
            "\"isSnapshotUpdate\":false}\n",
            "{\"type\":\"file-history-snapshot\",\"messageId\":\"msg-2\",",
            "\"snapshot\":{\"messageId\":\"msg-2\",\"timestamp\":\"2026-03-16T12:05:00Z\"},",
            "\"isSnapshotUpdate\":false}\n"
        ),
    )
    .expect("write file history snapshot");

    let discovered = adapter
        .discover_session_files(&root)
        .expect("claude discovery should succeed");

    assert_eq!(discovered.len(), 1);
    assert!(discovered[0].ends_with("claude-ses-real.jsonl"));
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

#[test]
fn gemini_adapter_discovers_and_parses_fixture() {
    let adapter = GeminiCliAdapter;
    let root = fixtures_root().join("gemini").join("tmp");

    let discovered = adapter
        .discover_session_files(&root)
        .expect("gemini fixtures discover");

    assert_eq!(discovered.len(), 1);

    let session = adapter
        .parse_session(&discovered[0])
        .expect("gemini fixture parses");

    assert_eq!(session.assistant, "gemini-cli");
    assert_eq!(session.session_id, "gemini-ses-1");
    assert_eq!(session.project_path.as_deref(), Some(r"C:\Projects\gemini-demo"));
    assert_eq!(session.message_count, 2);
    assert_eq!(session.tool_count, 1);
    assert_eq!(session.raw_format, "gemini-cli-json");
    assert!(!session.content_hash.is_empty());
}

#[test]
fn copilot_adapter_discovers_and_parses_fixture() {
    let adapter = CopilotCliAdapter;
    let root = fixtures_root().join("copilot");

    let discovered = adapter
        .discover_session_files(&root)
        .expect("copilot fixtures discover");

    assert_eq!(discovered.len(), 1);

    let session = adapter
        .parse_session(&discovered[0])
        .expect("copilot fixture parses");

    assert_eq!(session.assistant, "github-copilot-cli");
    assert_eq!(session.session_id, "copilot-ses-1");
    assert_eq!(session.project_path.as_deref(), Some(r"C:\Projects\copilot-demo"));
    assert_eq!(session.message_count, 2);
    assert_eq!(session.tool_count, 1);
    assert_eq!(session.raw_format, "github-copilot-cli-jsonl");
    assert!(!session.content_hash.is_empty());
}

#[test]
fn droid_adapter_discovers_and_parses_fixture() {
    let adapter = FactoryDroidAdapter;
    let root = fixtures_root().join("factory");

    let discovered = adapter
        .discover_session_files(&root)
        .expect("factory fixtures discover");

    assert_eq!(discovered.len(), 2);

    let session_store = discovered
        .iter()
        .find(|path: &&std::path::PathBuf| path.to_string_lossy().contains("droid-session-1"))
        .expect("session-store fixture exists");
    let stream_json = discovered
        .iter()
        .find(|path: &&std::path::PathBuf| path.to_string_lossy().contains("stream-session-1"))
        .expect("stream-json fixture exists");

    let store_session = adapter
        .parse_session(session_store)
        .expect("session-store fixture parses");
    let stream_session = adapter
        .parse_session(stream_json)
        .expect("stream-json fixture parses");

    assert_eq!(store_session.assistant, "factory-droid");
    assert_eq!(store_session.session_id, "droid-session-1");
    assert_eq!(
        store_session.project_path.as_deref(),
        Some(r"C:\Projects\factory-demo")
    );
    assert_eq!(store_session.message_count, 2);
    assert_eq!(store_session.tool_count, 1);
    assert_eq!(store_session.raw_format, "factory-droid-session-store");

    assert_eq!(stream_session.assistant, "factory-droid");
    assert_eq!(stream_session.session_id, "droid-stream-1");
    assert_eq!(
        stream_session.project_path.as_deref(),
        Some(r"C:\Projects\factory-stream")
    );
    assert_eq!(stream_session.message_count, 2);
    assert_eq!(stream_session.tool_count, 1);
    assert_eq!(stream_session.raw_format, "factory-droid-stream-json");
}

#[test]
fn openclaw_adapter_discovers_and_parses_fixture() {
    let adapter = OpenClawAdapter;
    let root = fixtures_root().join("openclaw");

    let discovered = adapter
        .discover_session_files(&root)
        .expect("openclaw fixtures discover");

    assert_eq!(discovered.len(), 1);

    let session = adapter
        .parse_session(&discovered[0])
        .expect("openclaw fixture parses");

    assert_eq!(session.assistant, "openclaw");
    assert_eq!(session.session_id, "openclaw-ses-1");
    assert_eq!(
        session.project_path.as_deref(),
        Some(r"C:\Projects\openclaw-demo")
    );
    assert_eq!(session.message_count, 2);
    assert_eq!(session.tool_count, 1);
    assert_eq!(session.raw_format, "openclaw-jsonl");
    assert!(!session.content_hash.is_empty());
}
