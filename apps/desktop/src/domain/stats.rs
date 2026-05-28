use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WritingStats {
    /// Consecutive days with writing activity ending today (0 if today and
    /// yesterday both lack activity).
    pub current_streak: u32,
    pub longest_streak: u32,
    /// ISO date of the most recent activity (YYYY-MM-DD).
    pub last_writing_date: Option<String>,
}
