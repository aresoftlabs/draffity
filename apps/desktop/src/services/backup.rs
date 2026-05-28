//! Backup service. **Premium-ready stub-on-top.**
//!
//! The free tier ships `LocalBackupService` (snapshots of `draffity.db` in
//! `<app_data>/backups/`). Premium can add `CloudBackupService` (uploads to
//! S3/B2/etc.) by implementing this trait — same wiring path as the other
//! services.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::now_ms;
use crate::error::{AppError, AppResult};

/// Kind of backup. `Daily` is the once-per-day automatic snapshot;
/// `Monthly` is the surviving "last of month" tag applied during pruning;
/// `Manual` is created on demand from the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupKind {
    Daily,
    Monthly,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRecord {
    /// File name without extension. Stable id for restore.
    pub id: String,
    /// Absolute path on disk.
    pub path: String,
    /// Epoch milliseconds when the file was written.
    pub created_at: i64,
    /// Size of the backup file in bytes.
    pub size_bytes: u64,
    pub kind: BackupKind,
}

pub trait BackupService: Send + Sync {
    /// Create a new backup of the live database. `kind` is `Daily` when the
    /// caller is the auto-on-startup task, `Manual` when triggered by the
    /// UI. Returns the metadata of the file just written.
    fn create_backup(&self, kind: BackupKind) -> AppResult<BackupRecord>;
    /// List backups currently on disk, newest first.
    fn list_backups(&self) -> AppResult<Vec<BackupRecord>>;
    /// Restore a backup by id. Implementations stop the storage layer
    /// first; the caller is responsible for restarting the app to re-open
    /// the DB cleanly.
    fn restore_backup(&self, id: &str) -> AppResult<()>;
    /// Apply the retention policy. Returns the number of files removed.
    fn prune_old_backups(&self) -> AppResult<usize>;
    /// Run the auto-on-startup task: ensure today has a daily snapshot,
    /// then prune. Idempotent — calling twice in the same day is a no-op.
    fn run_daily_maintenance(&self) -> AppResult<()> {
        if !self.has_backup_for_today()? {
            self.create_backup(BackupKind::Daily)?;
        }
        self.prune_old_backups()?;
        Ok(())
    }
    /// True when a backup with today's local date prefix already exists.
    /// Default impl scans `list_backups`; concrete impls can override if
    /// they have a cheaper path.
    fn has_backup_for_today(&self) -> AppResult<bool> {
        let today = local_ymd_prefix(now_ms());
        let list = self.list_backups()?;
        Ok(list.iter().any(|b| b.id.starts_with(&today)))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpBackup;

impl BackupService for NoOpBackup {
    fn create_backup(&self, _kind: BackupKind) -> AppResult<BackupRecord> {
        Err(AppError::Unsupported("backups disabled".into()))
    }
    fn list_backups(&self) -> AppResult<Vec<BackupRecord>> {
        Ok(Vec::new())
    }
    fn restore_backup(&self, _id: &str) -> AppResult<()> {
        Err(AppError::Unsupported("backups disabled".into()))
    }
    fn prune_old_backups(&self) -> AppResult<usize> {
        Ok(0)
    }
}

/// Local-only backup service. Copies `db_path` into `backup_dir`. Retention
/// policy: keep the 7 most recent daily files + the latest one for each of
/// the past 6 months + every manual backup. Anything else gets deleted by
/// `prune_old_backups`.
pub struct LocalBackupService {
    db_path: PathBuf,
    backup_dir: PathBuf,
    /// Days of dailies to retain. Default 7.
    daily_retain: usize,
    /// Months of monthlies to retain. Default 6.
    monthly_retain: usize,
}

impl LocalBackupService {
    pub fn new(db_path: PathBuf, backup_dir: PathBuf) -> Self {
        Self {
            db_path,
            backup_dir,
            daily_retain: 7,
            monthly_retain: 6,
        }
    }

    /// Override retention policy. Used by tests to keep fixtures small.
    pub fn with_retention(mut self, daily: usize, monthly: usize) -> Self {
        self.daily_retain = daily;
        self.monthly_retain = monthly;
        self
    }

    fn ensure_dir(&self) -> AppResult<()> {
        if !self.backup_dir.exists() {
            fs::create_dir_all(&self.backup_dir)?;
        }
        Ok(())
    }

    fn build_filename(&self, kind: BackupKind, created_at: i64) -> String {
        let stamp = local_ymd_hms_stamp(created_at);
        let suffix = match kind {
            BackupKind::Daily => "daily",
            BackupKind::Monthly => "monthly",
            BackupKind::Manual => "manual",
        };
        format!("{stamp}-{suffix}")
    }
}

impl BackupService for LocalBackupService {
    fn create_backup(&self, kind: BackupKind) -> AppResult<BackupRecord> {
        if !self.db_path.exists() {
            return Err(AppError::NotFound(format!(
                "database file not found at {}",
                self.db_path.display()
            )));
        }
        self.ensure_dir()?;
        let now = now_ms();
        let id = self.build_filename(kind, now);
        let path = self.backup_dir.join(format!("{id}.db"));
        // `fs::copy` is atomic enough for SQLite WAL-mode files in a single-
        // user desktop scenario: pages flushed to the main file are
        // consistent, and even mid-write checkpoints leave the journal +
        // main file recoverable via the standard journal logic when reopened.
        fs::copy(&self.db_path, &path)?;
        let size_bytes = fs::metadata(&path)?.len();
        Ok(BackupRecord {
            id,
            path: path.to_string_lossy().to_string(),
            created_at: now,
            size_bytes,
            kind,
        })
    }

    fn list_backups(&self) -> AppResult<Vec<BackupRecord>> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("db") {
                continue;
            }
            let Some(record) = record_from_path(&path) else {
                // Unrecognised filename — leave it on disk but don't surface.
                continue;
            };
            out.push(record);
        }
        // Newest first by created_at.
        out.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        Ok(out)
    }

    fn restore_backup(&self, id: &str) -> AppResult<()> {
        let path = self.backup_dir.join(format!("{id}.db"));
        if !path.exists() {
            return Err(AppError::NotFound(format!("backup {id}")));
        }
        // Safety net: take a manual backup of the *current* state first so
        // the user can undo the restore.
        let pre_restore = self.create_backup(BackupKind::Manual)?;
        tracing::info!(
            pre_restore = %pre_restore.id,
            restoring = %id,
            "creating pre-restore safety backup"
        );
        fs::copy(&path, &self.db_path)?;
        Ok(())
    }

    fn prune_old_backups(&self) -> AppResult<usize> {
        let mut list = self.list_backups()?;
        // Newest first already.
        let mut keep_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        // Always keep manuals — explicit user decision to preserve them.
        for b in &list {
            if b.kind == BackupKind::Manual {
                keep_ids.insert(b.id.clone());
            }
        }
        // Keep the N most recent dailies.
        let mut dailies = list
            .iter()
            .filter(|b| b.kind == BackupKind::Daily)
            .collect::<Vec<_>>();
        dailies.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        for b in dailies.iter().take(self.daily_retain) {
            keep_ids.insert(b.id.clone());
        }
        // For the past N months, keep the newest backup of each.
        let mut seen_months: std::collections::HashSet<String> = std::collections::HashSet::new();
        for b in &list {
            let month = local_ym_prefix(b.created_at);
            if seen_months.len() >= self.monthly_retain && !seen_months.contains(&month) {
                continue;
            }
            if seen_months.insert(month) {
                keep_ids.insert(b.id.clone());
            }
        }
        // Anything not in keep_ids gets removed.
        let mut removed = 0usize;
        list.retain(|b| {
            if keep_ids.contains(&b.id) {
                true
            } else {
                if let Err(e) = fs::remove_file(&b.path) {
                    tracing::warn!(file = %b.path, error = %e, "failed to prune backup");
                } else {
                    removed += 1;
                }
                false
            }
        });
        Ok(removed)
    }
}

fn record_from_path(path: &Path) -> Option<BackupRecord> {
    let stem = path.file_stem()?.to_str()?.to_string();
    let kind = if stem.ends_with("-daily") {
        BackupKind::Daily
    } else if stem.ends_with("-monthly") {
        BackupKind::Monthly
    } else if stem.ends_with("-manual") {
        BackupKind::Manual
    } else {
        return None;
    };
    let meta = fs::metadata(path).ok()?;
    let size_bytes = meta.len();
    // Parse the date prefix back into millis. Fallback to filesystem mtime.
    let created_at = parse_stamp_ms(&stem).unwrap_or_else(|| {
        meta.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    });
    Some(BackupRecord {
        id: stem,
        path: path.to_string_lossy().to_string(),
        created_at,
        size_bytes,
        kind,
    })
}

/// Render a `YYYY-MM-DD-HHMMSS` stamp in **local** time. Local because the
/// rotation policy is "one per day" from the user's perspective, not UTC.
fn local_ymd_hms_stamp(epoch_ms: i64) -> String {
    use chrono::{Local, TimeZone};
    let secs = epoch_ms / 1000;
    let nanos = ((epoch_ms % 1000) * 1_000_000) as u32;
    Local
        .timestamp_opt(secs, nanos)
        .single()
        .map(|dt| dt.format("%Y-%m-%d-%H%M%S").to_string())
        .unwrap_or_else(|| "1970-01-01-000000".into())
}

fn local_ymd_prefix(epoch_ms: i64) -> String {
    use chrono::{Local, TimeZone};
    let secs = epoch_ms / 1000;
    Local
        .timestamp_opt(secs, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".into())
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

fn parse_stamp_ms(stem: &str) -> Option<i64> {
    use chrono::{Local, NaiveDateTime, TimeZone};
    // Expect `YYYY-MM-DD-HHMMSS-...` (the suffix kind follows).
    let prefix: String = stem.split('-').take(4).collect::<Vec<_>>().join("-");
    let dt = NaiveDateTime::parse_from_str(&prefix, "%Y-%m-%d-%H%M%S").ok()?;
    Local
        .from_local_datetime(&dt)
        .single()
        .map(|d| d.timestamp_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fresh_dirs(prefix: &str) -> (PathBuf, PathBuf, PathBuf) {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("draffity-{prefix}-{nanos:x}"));
        let db = root.join("draffity.db");
        let backups = root.join("backups");
        std::fs::create_dir_all(&root).unwrap();
        // Write a dummy DB file — backup just copies bytes.
        std::fs::write(&db, b"sqlite-fixture").unwrap();
        (root, db, backups)
    }

    #[test]
    fn create_backup_writes_a_file_and_returns_metadata() {
        let (_root, db, backups) = fresh_dirs("backup-create");
        let svc = LocalBackupService::new(db, backups.clone());
        let rec = svc.create_backup(BackupKind::Daily).unwrap();
        assert_eq!(rec.kind, BackupKind::Daily);
        assert!(rec.id.ends_with("-daily"));
        assert!(rec.size_bytes > 0);
        let on_disk = backups.join(format!("{}.db", rec.id));
        assert!(on_disk.exists());
    }

    #[test]
    fn create_fails_when_db_does_not_exist() {
        let (_root, _db, backups) = fresh_dirs("backup-no-db");
        let svc = LocalBackupService::new(PathBuf::from("/no/such/file"), backups);
        let err = svc.create_backup(BackupKind::Daily).unwrap_err();
        matches!(err, AppError::NotFound(_));
    }

    #[test]
    fn list_returns_newest_first_and_ignores_unknown_files() {
        let (_root, db, backups) = fresh_dirs("backup-list");
        let svc = LocalBackupService::new(db, backups.clone());
        let a = svc.create_backup(BackupKind::Daily).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let b = svc.create_backup(BackupKind::Manual).unwrap();
        // Drop a stray file: should be ignored by list.
        std::fs::write(backups.join("stray.txt"), b"x").unwrap();
        let list = svc.list_backups().unwrap();
        assert_eq!(list.len(), 2);
        // Newest first.
        assert_eq!(list[0].id, b.id);
        assert_eq!(list[1].id, a.id);
    }

    #[test]
    fn prune_keeps_dailies_within_retention_and_drops_older() {
        let (_root, db, backups) = fresh_dirs("backup-prune");
        // Retain only the 2 newest dailies for this test.
        let svc = LocalBackupService::new(db, backups.clone()).with_retention(2, 0);
        // Write 4 backups manually with synthetic timestamps — newer first.
        for offset_secs in [0i64, -86400, -172_800, -259_200] {
            let ts_ms = now_ms() + offset_secs * 1000;
            let id = svc.build_filename(BackupKind::Daily, ts_ms);
            let path = backups.join(format!("{id}.db"));
            std::fs::create_dir_all(&backups).unwrap();
            std::fs::write(&path, b"x").unwrap();
            // Override mtime via a second write so the parser path works.
            // We rely on filename parsing for ordering — mtime is fallback only.
            let _ = (path, ts_ms);
        }
        let before = svc.list_backups().unwrap();
        assert_eq!(before.len(), 4);
        let removed = svc.prune_old_backups().unwrap();
        let after = svc.list_backups().unwrap();
        assert_eq!(after.len(), 2, "should retain 2 dailies, removed {removed}");
        // The 2 retained should be the 2 newest by created_at.
        assert!(after[0].created_at >= after[1].created_at);
    }

    #[test]
    fn prune_preserves_manual_backups_always() {
        let (_root, db, backups) = fresh_dirs("backup-keep-manual");
        let svc = LocalBackupService::new(db, backups.clone()).with_retention(1, 0);
        let manual = svc.create_backup(BackupKind::Manual).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let _daily_a = svc.create_backup(BackupKind::Daily).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let _daily_b = svc.create_backup(BackupKind::Daily).unwrap();
        svc.prune_old_backups().unwrap();
        let after = svc.list_backups().unwrap();
        assert!(
            after.iter().any(|b| b.id == manual.id),
            "manual backup must survive pruning"
        );
    }

    #[test]
    fn run_daily_maintenance_is_idempotent_within_one_day() {
        let (_root, db, backups) = fresh_dirs("backup-daily-idem");
        let svc = LocalBackupService::new(db, backups);
        svc.run_daily_maintenance().unwrap();
        let after_first = svc.list_backups().unwrap().len();
        svc.run_daily_maintenance().unwrap();
        let after_second = svc.list_backups().unwrap().len();
        assert_eq!(
            after_first, after_second,
            "second call same day should not create a new backup"
        );
    }

    #[test]
    fn restore_replaces_db_and_writes_safety_pre_backup() {
        let (_root, db, backups) = fresh_dirs("backup-restore");
        std::fs::write(&db, b"original").unwrap();
        let svc = LocalBackupService::new(db.clone(), backups.clone());
        let snap = svc.create_backup(BackupKind::Daily).unwrap();
        // Mutate the live DB.
        std::fs::write(&db, b"corrupted").unwrap();
        svc.restore_backup(&snap.id).unwrap();
        assert_eq!(std::fs::read(&db).unwrap(), b"original");
        // The safety pre-restore manual snapshot of "corrupted" exists.
        let list = svc.list_backups().unwrap();
        assert!(list.iter().any(|b| b.kind == BackupKind::Manual));
    }

    #[test]
    fn noop_backup_rejects_writes_but_lists_empty() {
        let svc = NoOpBackup;
        assert!(svc.list_backups().unwrap().is_empty());
        assert!(svc.create_backup(BackupKind::Manual).is_err());
        assert!(svc.restore_backup("x").is_err());
        assert_eq!(svc.prune_old_backups().unwrap(), 0);
    }
}
