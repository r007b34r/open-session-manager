use super::InsightInput;

pub fn derive_title(input: &InsightInput<'_>) -> String {
    if let Some(goal) = input.first_user_goal.and_then(normalize_text) {
        return truncate_title(&goal);
    }

    if let Some(message) = input.last_assistant_message.and_then(normalize_text) {
        if !looks_like_error_message(&message) {
            return truncate_title(&message);
        }
    }

    "Untitled session".to_string()
}

fn normalize_text(value: &str) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn truncate_title(value: &str) -> String {
    const LIMIT: usize = 72;
    if value.chars().count() <= LIMIT {
        return value.to_string();
    }

    let shortened = value.chars().take(LIMIT - 1).collect::<String>();
    format!("{shortened}...")
}

fn looks_like_error_message(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.starts_with("fatal:")
        || lowered.contains(" error")
        || lowered.starts_with("error:")
        || lowered.contains("missing")
        || lowered.contains("failed")
        || lowered.contains("exception")
}
