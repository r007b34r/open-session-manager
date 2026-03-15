use chrono::{DateTime, Utc};

use super::{InsightInput, value::derive_value_score};

pub fn derive_garbage_score(input: &InsightInput<'_>) -> u8 {
    let mut score = 0_i32;

    if input.message_count <= 1 {
        score += 45;
    }

    if input.first_user_goal.is_none() {
        score += 20;
    }

    if input.tool_count == 0 {
        score += 10;
    }

    if input.error_count > 0 {
        score += 20;
    }

    if is_stale(input.last_activity_at) {
        score += 20;
    }

    score -= (derive_value_score(input) as i32) / 4;

    score.clamp(0, 100) as u8
}

fn is_stale(last_activity_at: Option<&str>) -> bool {
    let Some(last_activity_at) = last_activity_at else {
        return true;
    };

    let Ok(parsed) = DateTime::parse_from_rfc3339(last_activity_at) else {
        return false;
    };

    Utc::now()
        .signed_duration_since(parsed.with_timezone(&Utc))
        .num_days()
        >= 30
}
