pub fn normalize_session_text(value: &str) -> Option<String> {
    let normalized = normalize_whitespace(value);
    if normalized.is_empty()
        || is_placeholder_text(&normalized)
        || is_session_scaffolding(&normalized)
    {
        return None;
    }

    Some(normalized)
}

pub fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn shorten_session_id(value: &str) -> String {
    if value.chars().count() <= 14 {
        return value.to_string();
    }

    let prefix = value.chars().take(8).collect::<String>();
    let suffix = value
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("{prefix}...{suffix}")
}

fn is_placeholder_text(value: &str) -> bool {
    value
        .to_ascii_lowercase()
        .contains("[request interrupted by user]")
}

fn is_session_scaffolding(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();

    lowered.contains("# agents.md instructions")
        || lowered.contains("<environment_context>")
        || lowered.contains("<permissions instructions>")
        || lowered.contains("filesystem sandboxing defines which files can be read or written")
        || (lowered.contains("### available skills") && lowered.contains("how to use skills"))
}
