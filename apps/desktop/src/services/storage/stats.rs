//! Writing streak counters backed by the `settings` table:
//!   - `writing.last_date`        — YYYY-MM-DD of the last day the user wrote
//!   - `writing.current_streak`   — consecutive days up to and including that date
//!   - `writing.longest_streak`   — historical max
//!
//! `record_activity` is called from `update_document`. `get` is a pure read
//! that reports `current_streak = 0` if the last activity is older than
//! yesterday, without mutating storage.

use chrono::{Datelike, Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::WritingStats;
use crate::error::AppResult;

pub(super) fn record_activity(conn: &Connection) -> AppResult<WritingStats> {
    let today: NaiveDate = Local::now().date_naive();
    let today_str = format!(
        "{:04}-{:02}-{:02}",
        today.year(),
        today.month(),
        today.day()
    );

    let last: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key='writing.last_date'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let stored_streak: u32 = conn
        .query_row(
            "SELECT value FROM settings WHERE key='writing.current_streak'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut longest: u32 = conn
        .query_row(
            "SELECT value FROM settings WHERE key='writing.longest_streak'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let new_streak = match last
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    {
        Some(d) if d == today => stored_streak.max(1),
        Some(d) if d.succ_opt() == Some(today) => stored_streak + 1,
        _ => 1,
    };
    if new_streak > longest {
        longest = new_streak;
    }

    let upsert = |k: &str, v: &str| -> AppResult<()> {
        conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![k, v],
        )?;
        Ok(())
    };
    upsert("writing.last_date", &today_str)?;
    upsert("writing.current_streak", &new_streak.to_string())?;
    upsert("writing.longest_streak", &longest.to_string())?;

    Ok(WritingStats {
        current_streak: new_streak,
        longest_streak: longest,
        last_writing_date: Some(today_str),
    })
}

pub(super) fn get(conn: &Connection) -> AppResult<WritingStats> {
    let last: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key='writing.last_date'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let stored_streak: u32 = conn
        .query_row(
            "SELECT value FROM settings WHERE key='writing.current_streak'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let longest: u32 = conn
        .query_row(
            "SELECT value FROM settings WHERE key='writing.longest_streak'",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // The streak persists if the most recent activity was today or yesterday.
    let current = match last
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    {
        Some(d) => {
            let today = Local::now().date_naive();
            let diff = today.signed_duration_since(d).num_days();
            if diff <= 1 {
                stored_streak
            } else {
                0
            }
        }
        None => 0,
    };

    Ok(WritingStats {
        current_streak: current,
        longest_streak: longest,
        last_writing_date: last,
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::fresh;
    use super::super::StorageService;

    #[test]
    fn record_writing_activity_initializes_streak_and_sets_today() {
        let s = fresh();
        let stats = s.record_writing_activity().unwrap();
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.longest_streak, 1);
        assert!(stats.last_writing_date.is_some());
    }

    #[test]
    fn record_writing_activity_is_idempotent_within_same_day() {
        let s = fresh();
        let a = s.record_writing_activity().unwrap();
        let b = s.record_writing_activity().unwrap();
        assert_eq!(b.current_streak, a.current_streak);
    }

    #[test]
    fn get_writing_stats_reports_zero_streak_when_last_activity_is_old() {
        let s = fresh();
        s.set_setting("writing.last_date", "2020-01-01").unwrap();
        s.set_setting("writing.current_streak", "42").unwrap();
        s.set_setting("writing.longest_streak", "42").unwrap();
        let stats = s.get_writing_stats().unwrap();
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.longest_streak, 42);
    }
}
