#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installation {
    pub installation_id: String,
    pub assistant: String,
    pub version: Option<String>,
    pub executable_path: String,
    pub environment: String,
    pub source: String,
    pub discovered_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRecord {
    pub session_id: String,
    pub installation_id: Option<String>,
    pub assistant: String,
    pub environment: String,
    pub project_path: Option<String>,
    pub source_path: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub message_count: u32,
    pub tool_count: u32,
    pub status: String,
    pub raw_format: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionInsight {
    pub session_id: String,
    pub title: String,
    pub topic_labels_json: String,
    pub summary: String,
    pub progress_state: String,
    pub progress_percent: Option<u8>,
    pub value_score: u8,
    pub stale_score: u8,
    pub garbage_score: u8,
    pub risk_flags_json: String,
    pub confidence: f32,
}
