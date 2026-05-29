//! Writing streak counters backed by the `settings` table:
//!   - `writing.last_date`        — YYYY-MM-DD of the last day the user wrote
//!   - `writing.current_streak`   — consecutive days up to and including that date
//!   - `writing.longest_streak`   — historical max
//!
//! `record_activity` is called from `update_document`. `get` is a pure read
//! that reports `current_streak = 0` if the last activity is older than
//! yesterday, without mutating storage.

use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{now_ms, DailyWriting, WritingStats};
use crate::error::AppResult;

fn today_iso() -> String {
    let t = Local::now().date_naive();
    format!("{:04}-{:02}-{:02}", t.year(), t.month(), t.day())
}

pub(super) fn record_activity(conn: &Connection) -> AppResult<WritingStats> {
    let today: NaiveDate = Local::now().date_naive();
    let today_str = today_iso();

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
        goal_met_streak: goal_met_streak(conn)?,
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
        goal_met_streak: goal_met_streak(conn)?,
    })
}

/// The persisted daily word goal (J-04), or `None` when unset.
pub(super) fn get_daily_goal(conn: &Connection) -> AppResult<Option<i64>> {
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key='writing.daily_goal'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    Ok(v.and_then(|s| s.parse().ok()))
}

/// Set or clear the daily word goal and recompute today's `goal_met`.
pub(super) fn set_daily_goal(conn: &Connection, goal: Option<i64>) -> AppResult<()> {
    let goal = goal.filter(|g| *g > 0);
    match goal {
        Some(g) => {
            conn.execute(
                "INSERT INTO settings(key, value) VALUES('writing.daily_goal', ?1)
                 ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![g.to_string()],
            )?;
        }
        None => {
            conn.execute("DELETE FROM settings WHERE key='writing.daily_goal'", [])?;
        }
    }
    // Re-snapshot today's row so the change is reflected immediately.
    conn.execute(
        "UPDATE daily_writing
         SET goal_words = ?2,
             goal_met = CASE WHEN ?2 IS NOT NULL AND words >= ?2 THEN 1 ELSE 0 END
         WHERE date = ?1",
        params![today_iso(), goal],
    )?;
    Ok(())
}

/// Upsert today's row, adding `words_delta` to the running total and
/// bumping the session counter. Called once per `update_document` save.
/// `words_delta == 0` still counts as a session (the user pressed save
/// even if they only deleted text). Also snapshots the active daily goal
/// and recomputes today's `goal_met` (J-04).
pub(super) fn record_daily(conn: &Connection, words_delta: u32) -> AppResult<()> {
    let today = today_iso();
    conn.execute(
        "INSERT INTO daily_writing(date, words, sessions, updated_at)
         VALUES (?1, ?2, 1, ?3)
         ON CONFLICT(date) DO UPDATE SET
             words = words + excluded.words,
             sessions = sessions + 1,
             updated_at = excluded.updated_at",
        params![today, words_delta, now_ms()],
    )?;
    let goal = get_daily_goal(conn)?;
    conn.execute(
        "UPDATE daily_writing
         SET goal_words = ?2,
             goal_met = CASE WHEN ?2 IS NOT NULL AND words >= ?2 THEN 1 ELSE 0 END
         WHERE date = ?1",
        params![today, goal],
    )?;
    Ok(())
}

/// Consecutive days with `goal_met=1` ending today or yesterday. 0 when the
/// most recent goal-met day is older than yesterday (the run is broken).
fn goal_met_streak(conn: &Connection) -> AppResult<u32> {
    let mut stmt =
        conn.prepare("SELECT date FROM daily_writing WHERE goal_met=1 ORDER BY date DESC")?;
    let dates: Vec<NaiveDate> = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .filter_map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
        .collect();
    let Some(&most_recent) = dates.first() else {
        return Ok(0);
    };
    let today = Local::now().date_naive();
    if today.signed_duration_since(most_recent).num_days() > 1 {
        return Ok(0);
    }
    let mut streak = 0u32;
    let mut expected = most_recent;
    for d in dates {
        if d == expected {
            streak += 1;
            match expected.pred_opt() {
                Some(p) => expected = p,
                None => break,
            }
        } else if d < expected {
            break;
        }
    }
    Ok(streak)
}

/// Last `days` worth of activity, oldest first. Missing days are filled
/// in with zeros so the chart can render a flat baseline instead of a
/// jagged line. `days` is clamped to [1, 365].
pub(super) fn list_recent_daily(conn: &Connection, days: u32) -> AppResult<Vec<DailyWriting>> {
    let days = days.clamp(1, 365);
    let today = Local::now().date_naive();
    let start = today - Duration::days((days - 1) as i64);
    let start_str = format!(
        "{:04}-{:02}-{:02}",
        start.year(),
        start.month(),
        start.day()
    );

    let mut stmt = conn.prepare(
        "SELECT date, words, sessions, goal_words, goal_met FROM daily_writing
         WHERE date >= ?1
         ORDER BY date ASC",
    )?;
    #[allow(clippy::type_complexity)]
    let rows: Vec<(String, u32, u32, Option<u32>, bool)> = stmt
        .query_map(params![start_str], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get::<_, Option<u32>>(3)?,
                r.get::<_, i64>(4)? != 0,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Fill in gaps with zero rows so the consumer always gets exactly
    // `days` entries spanning [start, today].
    let mut by_date: std::collections::HashMap<String, (u32, u32, Option<u32>, bool)> = rows
        .into_iter()
        .map(|(d, w, s, g, m)| (d, (w, s, g, m)))
        .collect();
    let mut out = Vec::with_capacity(days as usize);
    for i in 0..days as i64 {
        let d = start + Duration::days(i);
        let key = format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day());
        let (words, sessions, goal_words, goal_met) =
            by_date.remove(&key).unwrap_or((0, 0, None, false));
        out.push(DailyWriting {
            date: key,
            words,
            sessions,
            goal_words,
            goal_met,
        });
    }
    Ok(out)
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

    #[test]
    fn list_recent_daily_pads_missing_days_with_zero() {
        let s = fresh();
        let series = s.list_recent_daily_writing(7).unwrap();
        assert_eq!(series.len(), 7);
        assert!(series.iter().all(|d| d.words == 0 && d.sessions == 0));
    }

    #[test]
    fn record_daily_accumulates_words_and_sessions() {
        let s = fresh();
        s.record_daily_writing(120).unwrap();
        s.record_daily_writing(30).unwrap();
        s.record_daily_writing(0).unwrap();
        let series = s.list_recent_daily_writing(1).unwrap();
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].words, 150);
        assert_eq!(series[0].sessions, 3);
    }

    #[test]
    fn list_recent_daily_clamps_window_size() {
        let s = fresh();
        let series = s.list_recent_daily_writing(0).unwrap();
        // Clamped to a minimum of 1 day.
        assert_eq!(series.len(), 1);
    }

    #[test]
    fn daily_goal_drives_goal_met_and_streak() {
        let s = fresh();
        s.set_daily_goal(Some(100)).unwrap();
        assert_eq!(s.get_daily_goal().unwrap(), Some(100));

        // Below the goal → not met, no streak.
        s.record_daily_writing(40).unwrap();
        let series = s.list_recent_daily_writing(1).unwrap();
        assert_eq!(series[0].goal_words, Some(100));
        assert!(!series[0].goal_met);
        assert_eq!(s.get_writing_stats().unwrap().goal_met_streak, 0);

        // Reaching the goal flips goal_met and lights the streak.
        s.record_daily_writing(70).unwrap();
        let series = s.list_recent_daily_writing(1).unwrap();
        assert!(series[0].goal_met);
        assert_eq!(s.get_writing_stats().unwrap().goal_met_streak, 1);
    }

    #[test]
    fn clearing_daily_goal_clears_goal_met() {
        let s = fresh();
        s.set_daily_goal(Some(10)).unwrap();
        s.record_daily_writing(50).unwrap();
        assert!(s.list_recent_daily_writing(1).unwrap()[0].goal_met);

        s.set_daily_goal(None).unwrap();
        let series = s.list_recent_daily_writing(1).unwrap();
        assert!(!series[0].goal_met);
        assert_eq!(series[0].goal_words, None);
        assert_eq!(s.get_daily_goal().unwrap(), None);
    }
}
