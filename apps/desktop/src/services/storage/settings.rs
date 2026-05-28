//! Key-value settings store (used both for user preferences and
//! app-internal counters like the writing streak — see `stats.rs`).

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppResult;

pub(super) fn get(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key=?1",
            params![key],
            |r| r.get(0),
        )
        .optional()?;
    Ok(v)
}

pub(super) fn set(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::fresh;
    use super::super::StorageService;

    #[test]
    fn settings_round_trip_and_upsert() {
        let s = fresh();
        assert!(s.get_setting("k").unwrap().is_none());
        s.set_setting("k", "v1").unwrap();
        assert_eq!(s.get_setting("k").unwrap().as_deref(), Some("v1"));
        s.set_setting("k", "v2").unwrap();
        assert_eq!(s.get_setting("k").unwrap().as_deref(), Some("v2"));
    }
}
