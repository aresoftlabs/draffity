//! Retention policy for backup files. Decides which `BackupRecord`s to keep
//! and which to delete. Lives outside `backup.rs` so it can be unit-tested
//! as a pure function on a list, with no I/O.
//!
//! Policy:
//! - Manuals are **always** kept (explicit user decision).
//! - Daily backups: keep the `daily_retain` most recent.
//! - Monthly slot: for each of the most recent `monthly_retain` calendar
//!   months observed, keep the newest backup of that month. The slot is
//!   filled regardless of kind so a manual or daily already kept also
//!   acts as the "monthly anchor" — we don't multiply backups for the
//!   same month.

use std::collections::HashSet;

use super::backup::{BackupKind, BackupRecord};

/// Decide which backup ids to keep given the policy.
///
/// `records` is assumed sorted newest-first (matches `list_backups()`).
pub(super) fn compute_keep_ids(
    records: &[BackupRecord],
    daily_retain: usize,
    monthly_retain: usize,
) -> HashSet<String> {
    let mut keep = HashSet::new();

    // 1) Manuals are non-negotiable.
    for b in records {
        if b.kind == BackupKind::Manual {
            keep.insert(b.id.clone());
        }
    }

    // 2) N most recent dailies.
    let mut dailies: Vec<&BackupRecord> = records
        .iter()
        .filter(|b| b.kind == BackupKind::Daily)
        .collect();
    dailies.sort_by_key(|b| std::cmp::Reverse(b.created_at));
    for b in dailies.iter().take(daily_retain) {
        keep.insert(b.id.clone());
    }

    // 3) Monthly anchors. Walk newest-first; for each new month (until
    //    monthly_retain slots are filled) keep the first hit — which is
    //    the newest of that month.
    let mut seen_months: HashSet<String> = HashSet::new();
    for b in records {
        let month = local_ym_prefix(b.created_at);
        if seen_months.len() >= monthly_retain && !seen_months.contains(&month) {
            continue;
        }
        if seen_months.insert(month) {
            keep.insert(b.id.clone());
        }
    }

    keep
}

fn local_ym_prefix(epoch_ms: i64) -> String {
    use chrono::{Local, TimeZone};
    let secs = epoch_ms / 1000;
    Local
        .timestamp_opt(secs, 0)
        .single()
        .map(|dt| dt.format("%Y-%m").to_string())
        .unwrap_or_else(|| "1970-01".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(id: &str, kind: BackupKind, created_at: i64) -> BackupRecord {
        BackupRecord {
            id: id.into(),
            path: format!("/tmp/{id}.db"),
            created_at,
            size_bytes: 1,
            kind,
        }
    }

    // 2026-05-28 12:00:00 UTC → `1748433600000` ms. Helper for readable
    // tests: each row is offset N days from this base.
    const BASE_MS: i64 = 1_748_433_600_000;
    const DAY_MS: i64 = 86_400_000;

    #[test]
    fn manuals_are_always_kept() {
        let records = vec![
            rec("m1", BackupKind::Manual, BASE_MS),
            rec("m2", BackupKind::Manual, BASE_MS - 365 * DAY_MS),
        ];
        let keep = compute_keep_ids(&records, 0, 0);
        assert!(keep.contains("m1"));
        assert!(keep.contains("m2"));
    }

    #[test]
    fn keeps_top_n_dailies_only() {
        let records: Vec<BackupRecord> = (0..10)
            .map(|i| rec(&format!("d{i}"), BackupKind::Daily, BASE_MS - i * DAY_MS))
            .collect();
        let keep = compute_keep_ids(&records, 3, 0);
        // Top 3 most recent are d0, d1, d2.
        assert!(keep.contains("d0"));
        assert!(keep.contains("d1"));
        assert!(keep.contains("d2"));
        assert!(!keep.contains("d3"));
    }

    #[test]
    fn monthly_slots_pick_newest_per_month() {
        // Build records across 4 distinct months, newest first.
        let records = vec![
            rec("may-late", BackupKind::Daily, BASE_MS), // 2026-05
            rec("may-early", BackupKind::Daily, BASE_MS - 5 * DAY_MS), // 2026-05
            rec("apr", BackupKind::Daily, BASE_MS - 35 * DAY_MS), // 2026-04
            rec("mar", BackupKind::Daily, BASE_MS - 70 * DAY_MS), // 2026-03
            rec("feb", BackupKind::Daily, BASE_MS - 100 * DAY_MS), // 2026-02
        ];
        // 0 dailies + 3 monthly slots → newest of may, apr, mar.
        let keep = compute_keep_ids(&records, 0, 3);
        assert!(keep.contains("may-late"));
        assert!(!keep.contains("may-early"));
        assert!(keep.contains("apr"));
        assert!(keep.contains("mar"));
        assert!(!keep.contains("feb"));
    }

    #[test]
    fn dailies_and_monthly_can_overlap() {
        // Top 1 daily + 1 monthly slot — `d0` qualifies for both, no
        // double-counting (HashSet handles it).
        let records = vec![
            rec("d0", BackupKind::Daily, BASE_MS),
            rec("d1", BackupKind::Daily, BASE_MS - DAY_MS),
            rec("d-old", BackupKind::Daily, BASE_MS - 90 * DAY_MS),
        ];
        let keep = compute_keep_ids(&records, 1, 1);
        assert!(keep.contains("d0"));
        // 1 monthly slot filled by d0's month → d-old's month doesn't get one.
        assert!(!keep.contains("d-old"));
        // d1 not retained as daily (only 1 slot) and shares month with d0.
        assert!(!keep.contains("d1"));
    }

    #[test]
    fn empty_input_returns_empty_set() {
        assert!(compute_keep_ids(&[], 7, 6).is_empty());
    }
}
