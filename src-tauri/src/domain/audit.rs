#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: String,
    pub target_type: String,
    pub target_id: String,
    pub actor: String,
    pub created_at: String,
    pub before_state: Option<String>,
    pub after_state: Option<String>,
    pub result: String,
    pub error_message: Option<String>,
}
