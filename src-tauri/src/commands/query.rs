use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::commands::dashboard::{DashboardSnapshot, SessionDetailRecord};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionListEntry {
    session_id: String,
    title: String,
    assistant: String,
    progress_state: String,
    last_activity_at: String,
    project_path: String,
    risk_flags: Vec<String>,
    control_available: bool,
    value_score: u8,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchHit {
    session_id: String,
    title: String,
    assistant: String,
    score: f64,
    snippet: Option<String>,
    match_reasons: Vec<&'static str>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSessionInventoryRequest {
    pub assistant: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub descending: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchSessionInventoryRequest {
    pub query: String,
    pub assistant: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sort_by: Option<String>,
    pub descending: Option<bool>,
}

#[derive(Debug, Clone)]
struct OwnedSearchTerm {
    value: String,
    phrase: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SearchReason {
    Title,
    Assistant,
    Environment,
    Summary,
    Project,
    Source,
    Tag,
    Risk,
    Artifact,
    Transcript,
    Todo,
}

#[derive(Debug, Clone)]
struct SearchField {
    reason: SearchReason,
    text: String,
    original_text: String,
    weight: f64,
}

const SEARCH_REASON_ORDER: [SearchReason; 11] = [
    SearchReason::Title,
    SearchReason::Summary,
    SearchReason::Project,
    SearchReason::Artifact,
    SearchReason::Transcript,
    SearchReason::Todo,
    SearchReason::Assistant,
    SearchReason::Environment,
    SearchReason::Tag,
    SearchReason::Risk,
    SearchReason::Source,
];

pub fn list_sessions(snapshot: &DashboardSnapshot) -> Value {
    let sessions = build_session_entries(snapshot);

    json!({
        "sessions": sessions
    })
}

pub fn list_sessions_with_request(
    snapshot: &DashboardSnapshot,
    request: Option<&ListSessionInventoryRequest>,
) -> Value {
    let request = request.cloned().unwrap_or_default();
    let mut sessions = build_session_entries(snapshot);

    if let Some(assistant) = request.assistant.as_ref() {
        let normalized = assistant.trim().to_ascii_lowercase();
        sessions.retain(|session| session.assistant.eq_ignore_ascii_case(&normalized));
    }

    sort_session_entries(
        &mut sessions,
        request.sort_by.as_deref(),
        request.descending,
    );
    let total = sessions.len();
    let sessions = paginate(sessions, request.offset, request.limit);

    json!({
        "sessions": sessions,
        "total": total,
        "offset": request.offset.unwrap_or(0),
        "limit": request.limit.unwrap_or(total)
    })
}

fn build_session_entries(snapshot: &DashboardSnapshot) -> Vec<SessionListEntry> {
    snapshot
        .sessions
        .iter()
        .map(|session| SessionListEntry {
            session_id: session.session_id.clone(),
            title: session.title.clone(),
            assistant: session.assistant.clone(),
            progress_state: session.progress_state.clone(),
            last_activity_at: session.last_activity_at.clone(),
            project_path: session.project_path.clone(),
            risk_flags: session.risk_flags.clone(),
            control_available: session
                .session_control
                .as_ref()
                .is_some_and(|control| control.available),
            value_score: session.value_score,
        })
        .collect()
}

pub fn search_sessions(snapshot: &DashboardSnapshot, query: &str) -> Value {
    search_sessions_with_request(
        snapshot,
        &SearchSessionInventoryRequest {
            query: query.to_string(),
            assistant: None,
            limit: None,
            offset: None,
            sort_by: None,
            descending: None,
        },
    )
}

pub fn search_sessions_with_request(
    snapshot: &DashboardSnapshot,
    request: &SearchSessionInventoryRequest,
) -> Value {
    let terms = parse_search_terms(&request.query);
    let mut hits = snapshot
        .sessions
        .iter()
        .filter(|session| {
            request
                .assistant
                .as_ref()
                .is_none_or(|assistant| session.assistant.eq_ignore_ascii_case(assistant.trim()))
        })
        .filter_map(|session| score_session(session, &terms))
        .collect::<Vec<_>>();
    sort_search_hits(&mut hits, request.sort_by.as_deref(), request.descending);
    let total = hits.len();
    let hits = paginate(hits, request.offset, request.limit);

    json!({
        "query": request.query,
        "hits": hits,
        "total": total,
        "offset": request.offset.unwrap_or(0),
        "limit": request.limit.unwrap_or(total)
    })
}

pub fn get_session(snapshot: &DashboardSnapshot, session_id: &str) -> Option<Value> {
    let session = find_session(snapshot, session_id)?;
    serde_json::to_value(session).ok()
}

pub fn view_session(snapshot: &DashboardSnapshot, session_id: &str) -> Option<Value> {
    let session = find_session(snapshot, session_id)?;

    Some(json!({
        "sessionId": &session.session_id,
        "content": render_session_markdown(session)
    }))
}

pub fn expand_session(snapshot: &DashboardSnapshot, session_id: &str) -> Option<Value> {
    let session = find_session(snapshot, session_id)?;
    let related_configs = snapshot
        .configs
        .iter()
        .filter(|config| config.assistant == session.assistant)
        .collect::<Vec<_>>();
    let related_audit_events = snapshot
        .audit_events
        .iter()
        .filter(|event| event.target == session.session_id)
        .collect::<Vec<_>>();

    Some(json!({
        "session": session,
        "relatedConfigs": related_configs,
        "relatedAuditEvents": related_audit_events,
        "transcriptHighlights": &session.transcript_highlights,
        "todoItems": &session.todo_items,
        "keyArtifacts": &session.key_artifacts
    }))
}

fn find_session<'a>(
    snapshot: &'a DashboardSnapshot,
    session_id: &str,
) -> Option<&'a SessionDetailRecord> {
    snapshot
        .sessions
        .iter()
        .find(|session| session.session_id == session_id)
}

fn render_session_markdown(session: &SessionDetailRecord) -> String {
    let mut sections = vec![
        format!("# {}", session.title),
        String::new(),
        format!("- Session ID: {}", session.session_id),
        format!("- Assistant: {}", session.assistant),
        format!("- Environment: {}", session.environment),
        format!("- Project: {}", session.project_path),
        format!("- Last activity: {}", session.last_activity_at),
        String::new(),
        "## Summary".to_string(),
        session.summary.clone(),
        String::new(),
        "## Open Todos".to_string(),
    ];

    let open_todos = session
        .todo_items
        .iter()
        .filter(|todo| !todo.completed)
        .collect::<Vec<_>>();
    if open_todos.is_empty() {
        sections.push("- None.".to_string());
    } else {
        sections.extend(
            open_todos
                .into_iter()
                .map(|todo| format!("- [ ] {}", todo.content)),
        );
    }

    sections.push(String::new());
    sections.push("## Completed Todos".to_string());
    let completed_todos = session
        .todo_items
        .iter()
        .filter(|todo| todo.completed)
        .collect::<Vec<_>>();
    if completed_todos.is_empty() {
        sections.push("- None.".to_string());
    } else {
        sections.extend(
            completed_todos
                .into_iter()
                .map(|todo| format!("- [x] {}", todo.content)),
        );
    }

    sections.push(String::new());
    sections.push("## Transcript Highlights".to_string());
    if session.transcript_highlights.is_empty() {
        sections.push("- None.".to_string());
    } else {
        sections.extend(
            session
                .transcript_highlights
                .iter()
                .map(|highlight| format!("- {}: {}", highlight.role, highlight.content)),
        );
    }

    sections.join("\n")
}

fn score_session(session: &SessionDetailRecord, terms: &[OwnedSearchTerm]) -> Option<SearchHit> {
    if terms.is_empty() {
        return None;
    }

    let fields = build_fields(session);
    let mut matched_reasons = Vec::new();
    let mut matched_reason_set = std::collections::BTreeSet::new();
    let mut score = 0.0;
    let mut best_field: Option<&SearchField> = None;

    for term in terms {
        let mut matched_this_term = false;

        for field in &fields {
            if !matches_term(&field.text, term) {
                continue;
            }

            matched_this_term = true;
            score += field.weight + phrase_bonus(field, term);

            if matched_reason_set.insert(field.reason) {
                matched_reasons.push(field.reason);
            }

            if should_replace_best_field(best_field, field) {
                best_field = Some(field);
            }
        }

        if !matched_this_term {
            return None;
        }
    }

    matched_reasons.sort_by_key(|reason| {
        SEARCH_REASON_ORDER
            .iter()
            .position(|candidate| candidate == reason)
            .unwrap_or(usize::MAX)
    });

    Some(SearchHit {
        session_id: session.session_id.clone(),
        title: session.title.clone(),
        assistant: session.assistant.clone(),
        score: round_score(score),
        snippet: best_field.map(|field| extract_snippet(&field.original_text, terms)),
        match_reasons: matched_reasons
            .into_iter()
            .map(search_reason_label)
            .collect(),
    })
}

fn should_replace_best_field(current: Option<&SearchField>, candidate: &SearchField) -> bool {
    match current {
        None => true,
        Some(current) => {
            candidate.weight > current.weight
                || (candidate.weight == current.weight
                    && candidate.original_text.len() < current.original_text.len())
        }
    }
}

fn build_fields(session: &SessionDetailRecord) -> Vec<SearchField> {
    let mut fields = vec![
        build_field(SearchReason::Title, &session.title),
        build_field(SearchReason::Assistant, &session.assistant),
        build_field(SearchReason::Environment, &session.environment),
        build_field(SearchReason::Summary, &session.summary),
        build_field(SearchReason::Project, &session.project_path),
        build_field(SearchReason::Source, &session.source_path),
    ];

    for tag in &session.tags {
        fields.push(build_field(SearchReason::Tag, tag));
    }

    for risk in &session.risk_flags {
        fields.push(build_field(SearchReason::Risk, risk));
    }

    for artifact in &session.key_artifacts {
        fields.push(build_field(SearchReason::Artifact, artifact));
    }

    for highlight in &session.transcript_highlights {
        fields.push(build_field(SearchReason::Transcript, &highlight.content));
    }

    for todo in &session.todo_items {
        fields.push(build_field(SearchReason::Todo, &todo.content));
    }

    fields
}

fn build_field(reason: SearchReason, original_text: &str) -> SearchField {
    SearchField {
        reason,
        text: normalize_search_text(original_text),
        original_text: original_text.trim().to_string(),
        weight: field_weight(reason),
    }
}

fn parse_search_terms(query: &str) -> Vec<OwnedSearchTerm> {
    let mut terms = Vec::new();
    let mut characters = query.chars().peekable();

    while let Some(character) = characters.peek() {
        if character.is_whitespace() {
            characters.next();
            continue;
        }

        let (phrase, raw_term) = if *character == '"' {
            characters.next();
            let mut value = String::new();
            for next in characters.by_ref() {
                if next == '"' {
                    break;
                }
                value.push(next);
            }
            (true, value)
        } else {
            let mut value = String::new();
            while let Some(next) = characters.peek() {
                if next.is_whitespace() {
                    break;
                }
                value.push(*next);
                characters.next();
            }
            (false, value)
        };

        let normalized = normalize_search_text(&raw_term);
        if normalized.is_empty() {
            continue;
        }

        terms.push(OwnedSearchTerm {
            value: normalized,
            phrase,
        });
    }

    terms
}

fn normalize_search_text(value: &str) -> String {
    value
        .to_lowercase()
        .replace(['\r', '\n', '\t'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn matches_term(text: &str, term: &OwnedSearchTerm) -> bool {
    if text.is_empty() {
        return false;
    }

    if text.contains(&term.value) {
        return true;
    }

    if term.phrase {
        return false;
    }

    tokenize(text)
        .into_iter()
        .any(|token| token.starts_with(&term.value))
}

fn tokenize(text: &str) -> Vec<&str> {
    text.split_whitespace()
        .filter(|token| !token.is_empty())
        .collect()
}

fn phrase_bonus(field: &SearchField, term: &OwnedSearchTerm) -> f64 {
    if term.phrase {
        return field.weight * 0.25;
    }

    0.0
}

fn extract_snippet(text: &str, terms: &[OwnedSearchTerm]) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let lowered = trimmed.to_lowercase();
    let mut first_index = None;
    let mut match_length = 0usize;

    for term in terms {
        if let Some(index) = lowered.find(&term.value) {
            if first_index.is_none_or(|current| index < current) {
                first_index = Some(index);
                match_length = term.value.len();
            }
        }
    }

    let Some(first_index) = first_index else {
        return trimmed.to_string();
    };

    if trimmed.len() <= 140 {
        return trimmed.to_string();
    }

    let start = first_index.saturating_sub(36);
    let end = (first_index + match_length + 84).min(trimmed.len());
    let prefix = if start > 0 { "..." } else { "" };
    let suffix = if end < trimmed.len() { "..." } else { "" };
    format!("{}{}{}", prefix, trimmed[start..end].trim(), suffix)
}

fn round_score(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn field_weight(reason: SearchReason) -> f64 {
    match reason {
        SearchReason::Title => 120.0,
        SearchReason::Assistant => 30.0,
        SearchReason::Environment => 18.0,
        SearchReason::Summary => 70.0,
        SearchReason::Project => 42.0,
        SearchReason::Source => 12.0,
        SearchReason::Tag => 34.0,
        SearchReason::Risk => 22.0,
        SearchReason::Artifact => 48.0,
        SearchReason::Transcript => 52.0,
        SearchReason::Todo => 58.0,
    }
}

fn search_reason_label(reason: SearchReason) -> &'static str {
    match reason {
        SearchReason::Title => "title",
        SearchReason::Assistant => "assistant",
        SearchReason::Environment => "environment",
        SearchReason::Summary => "summary",
        SearchReason::Project => "project",
        SearchReason::Source => "source",
        SearchReason::Tag => "tag",
        SearchReason::Risk => "risk",
        SearchReason::Artifact => "artifact",
        SearchReason::Transcript => "transcript",
        SearchReason::Todo => "todo",
    }
}

fn sort_session_entries(
    sessions: &mut [SessionListEntry],
    sort_by: Option<&str>,
    descending: Option<bool>,
) {
    let sort_by = sort_by.unwrap_or("lastActivityAt");
    let descending = descending.unwrap_or(true);

    sessions.sort_by(|left, right| {
        let ordering = match sort_by {
            "title" => left.title.cmp(&right.title),
            "assistant" => left.assistant.cmp(&right.assistant),
            "valueScore" => left.value_score.cmp(&right.value_score),
            _ => compare_activity(&left.last_activity_at, &right.last_activity_at),
        };

        if descending {
            ordering.reverse()
        } else {
            ordering
        }
        .then_with(|| left.session_id.cmp(&right.session_id))
    });
}

fn sort_search_hits(hits: &mut [SearchHit], sort_by: Option<&str>, descending: Option<bool>) {
    let sort_by = sort_by.unwrap_or("score");
    let descending = descending.unwrap_or(true);

    hits.sort_by(|left, right| {
        let ordering = match sort_by {
            "title" => left.title.cmp(&right.title),
            "assistant" => left.assistant.cmp(&right.assistant),
            _ => left
                .score
                .partial_cmp(&right.score)
                .unwrap_or(std::cmp::Ordering::Equal),
        };

        if descending {
            ordering.reverse()
        } else {
            ordering
        }
        .then_with(|| left.session_id.cmp(&right.session_id))
    });
}

fn compare_activity(left: &str, right: &str) -> std::cmp::Ordering {
    parse_activity_timestamp(left).cmp(&parse_activity_timestamp(right))
}

fn parse_activity_timestamp(value: &str) -> i64 {
    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(value) {
        return timestamp.timestamp_millis();
    }

    value.parse::<i64>().unwrap_or(0)
}

fn paginate<T>(items: Vec<T>, offset: Option<usize>, limit: Option<usize>) -> Vec<T> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(items.len().saturating_sub(offset));

    items.into_iter().skip(offset).take(limit).collect()
}
