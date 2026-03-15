use super::InsightInput;

pub fn derive_progress_state(input: &InsightInput<'_>) -> (&'static str, Option<u8>) {
    let assistant = input
        .last_assistant_message
        .map(|message| message.to_ascii_lowercase())
        .unwrap_or_default();

    if input.error_count > 0 && input.message_count <= 2 {
        return ("blocked", Some(15));
    }

    if assistant.contains("task complete")
        || assistant.contains("completed successfully")
        || assistant.contains("all done")
        || assistant.contains("已全部完成")
        || assistant.contains("任务完成")
    {
        return ("completed", Some(100));
    }

    if input.tool_count > 0 || input.message_count >= 4 {
        return ("in_progress", Some(65));
    }

    ("new", Some(10))
}
