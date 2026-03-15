PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS installations (
    installation_id TEXT PRIMARY KEY,
    assistant TEXT NOT NULL,
    version TEXT,
    executable_path TEXT NOT NULL,
    environment TEXT NOT NULL,
    source TEXT NOT NULL,
    discovered_at TEXT
);

CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT PRIMARY KEY,
    installation_id TEXT,
    assistant TEXT NOT NULL,
    environment TEXT NOT NULL,
    project_path TEXT,
    source_path TEXT NOT NULL,
    started_at TEXT,
    ended_at TEXT,
    last_activity_at TEXT,
    message_count INTEGER NOT NULL DEFAULT 0,
    tool_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    raw_format TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    FOREIGN KEY (installation_id) REFERENCES installations(installation_id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS session_insights (
    session_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    topic_labels_json TEXT NOT NULL DEFAULT '[]',
    summary TEXT NOT NULL,
    progress_state TEXT NOT NULL,
    progress_percent INTEGER,
    value_score INTEGER NOT NULL DEFAULT 0,
    stale_score INTEGER NOT NULL DEFAULT 0,
    garbage_score INTEGER NOT NULL DEFAULT 0,
    risk_flags_json TEXT NOT NULL DEFAULT '[]',
    confidence REAL NOT NULL DEFAULT 0,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS config_artifacts (
    artifact_id TEXT PRIMARY KEY,
    assistant TEXT NOT NULL,
    scope TEXT NOT NULL,
    path TEXT NOT NULL,
    source_layer TEXT NOT NULL,
    provider TEXT,
    model TEXT,
    base_url TEXT,
    permissions_json TEXT NOT NULL DEFAULT '[]',
    mcp_json TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE IF NOT EXISTS credential_artifacts (
    artifact_id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    kind TEXT NOT NULL,
    location TEXT NOT NULL,
    source_type TEXT NOT NULL,
    masked_value TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    official_or_proxy TEXT NOT NULL,
    last_modified_at TEXT
);

CREATE TABLE IF NOT EXISTS audit_events (
    event_id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    actor TEXT NOT NULL,
    created_at TEXT NOT NULL,
    before_state TEXT,
    after_state TEXT,
    result TEXT NOT NULL,
    error_message TEXT
);
