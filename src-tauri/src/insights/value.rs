use super::InsightInput;

pub fn derive_value_score(input: &InsightInput<'_>) -> u8 {
    let mut score = 0_i32;

    score += (input.message_count as i32) * 5;
    score += (input.tool_count as i32) * 10;

    if input.first_user_goal.is_some() {
        score += 20;
    }

    if input.last_assistant_message.is_some() {
        score += 10;
    }

    score -= (input.error_count as i32) * 15;

    score.clamp(0, 100) as u8
}
