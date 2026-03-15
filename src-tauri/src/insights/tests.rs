use super::{
    InsightInput, garbage::derive_garbage_score, progress::derive_progress_state,
    title::derive_title, value::derive_value_score,
};

#[test]
fn derives_non_empty_title_progress_and_scores() {
    let input = InsightInput {
        first_user_goal: Some("扫描并清理本地 agent 会话"),
        last_assistant_message: Some("已经完成目录扫描，下一步可以生成清理建议。"),
        message_count: 8,
        tool_count: 3,
        error_count: 0,
        last_activity_at: Some("2026-03-15T05:10:00Z"),
    };

    let title = derive_title(&input);
    let (progress_state, progress_percent) = derive_progress_state(&input);
    let value_score = derive_value_score(&input);
    let garbage_score = derive_garbage_score(&input);

    assert!(!title.is_empty());
    assert_eq!(progress_state, "in_progress");
    assert_eq!(progress_percent, Some(65));
    assert!(value_score >= 70);
    assert!(garbage_score <= 20);
}

#[test]
fn marks_short_broken_sessions_as_garbage() {
    let input = InsightInput {
        first_user_goal: None,
        last_assistant_message: Some("fatal: missing configuration"),
        message_count: 1,
        tool_count: 0,
        error_count: 1,
        last_activity_at: Some("2025-01-10T05:10:00Z"),
    };

    let title = derive_title(&input);
    let (progress_state, progress_percent) = derive_progress_state(&input);
    let value_score = derive_value_score(&input);
    let garbage_score = derive_garbage_score(&input);

    assert_eq!(title, "Untitled session");
    assert_eq!(progress_state, "blocked");
    assert_eq!(progress_percent, Some(15));
    assert!(value_score <= 20);
    assert!(garbage_score >= 80);
}
