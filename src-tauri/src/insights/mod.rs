pub mod garbage;
pub mod progress;
pub mod title;
pub mod value;

#[derive(Debug)]
pub struct InsightInput<'a> {
    pub first_user_goal: Option<&'a str>,
    pub last_assistant_message: Option<&'a str>,
    pub message_count: u32,
    pub tool_count: u32,
    pub error_count: u32,
    pub last_activity_at: Option<&'a str>,
}

#[cfg(test)]
mod tests;
