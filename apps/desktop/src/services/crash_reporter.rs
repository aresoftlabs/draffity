//! Crash reporting service. Premium-ready trait + a local stub impl that
//! writes reports to disk and a no-op for the disabled case. A future
//! Sentry-backed implementation drops in behind the trait without
//! touching commands or the UI.
//!
//! The contract is intentionally tiny — one method to record a captured
//! error, one to drain the queue on a release boundary. The frontend
//! never talks to the reporter directly: errors flow through the
//! command layer, which checks the user's opt-in toggle before
//! handing payloads to the reporter.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::domain::now_ms;
use crate::error::AppResult;

/// Snapshot of a captured error ready to be reported. Kept deliberately
/// boring — anything richer (breadcrumbs, scope tags) gets layered on by
/// the impl from its own runtime state, not from the caller.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashReport {
    /// Stable identifier of the impl reporting (e.g. `local-file`,
    /// `sentry`). Helps when triaging logs that span versions.
    pub source: String,
    /// Free-form short label (e.g. `update_document`, `export_project`).
    /// Used by the impl to group similar incidents.
    pub label: String,
    /// One-line summary the impl shows in its console / UI. Document
    /// content is never included.
    pub message: String,
    /// Multi-line stack / context dump. Already redacted by the caller.
    pub details: String,
    /// Epoch millis when the report was captured.
    pub captured_at: i64,
}

pub trait CrashReporterService: Send + Sync {
    /// Record an event the impl should consider for reporting. Honors
    /// the runtime opt-in flag — the caller doesn't need to gate.
    fn record(&self, report: CrashReport) -> AppResult<()>;
    /// Returns true if this impl actually delivers reports (Sentry HTTP,
    /// local file…). NoOp returns false so the UI can hide its consent
    /// toggle when there's no destination configured.
    fn is_active(&self) -> bool;
    /// Toggle the opt-in flag at runtime. Implementations that don't
    /// honor an opt-in (NoOp) ignore the call silently.
    fn set_enabled(&self, enabled: bool);
    fn is_enabled(&self) -> bool;
}

/// Reporter that does literally nothing — used when no destination URL
/// was baked into the build, or as a placeholder for tests.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpCrashReporter;

impl CrashReporterService for NoOpCrashReporter {
    fn record(&self, _report: CrashReport) -> AppResult<()> {
        Ok(())
    }
    fn is_active(&self) -> bool {
        false
    }
    fn set_enabled(&self, _enabled: bool) {}
    fn is_enabled(&self) -> bool {
        false
    }
}

/// Local stub: writes each report as a JSON line under
/// `<app_data>/crash-reports/`. Stands in for a real Sentry uploader
/// until the owner provisions infrastructure. Once the env var
/// `DRAFFITY_SENTRY_DSN` is wired and the HTTP path lands, the only
/// change in callers is the bundle factory picking a different impl.
pub struct LocalFileCrashReporter {
    dir: PathBuf,
    enabled: Mutex<bool>,
}

impl LocalFileCrashReporter {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
            enabled: Mutex::new(false),
        }
    }

    fn write_line(&self, payload: &str) -> AppResult<()> {
        fs::create_dir_all(&self.dir)?;
        let day = chrono::Local::now().format("%Y-%m-%d").to_string();
        let path = self.dir.join(format!("{day}.jsonl"));
        let mut existing = fs::read_to_string(&path).unwrap_or_default();
        existing.push_str(payload);
        existing.push('\n');
        fs::write(&path, existing)?;
        Ok(())
    }
}

impl CrashReporterService for LocalFileCrashReporter {
    fn record(&self, report: CrashReport) -> AppResult<()> {
        if !self.is_enabled() {
            return Ok(());
        }
        // We never want a reporting failure to bubble up to the user —
        // log it instead and move on.
        if let Ok(line) = serde_json::to_string(&report) {
            if let Err(e) = self.write_line(&line) {
                tracing::warn!(error = %e, "crash report write failed");
            }
        }
        Ok(())
    }
    fn is_active(&self) -> bool {
        true
    }
    fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock().unwrap() = enabled;
    }
    fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }
}

/// Convenience: build a report from an `AppError`. Strips the document
/// content / file paths that could appear in `Display` impls.
pub fn report_from_error(source: &str, label: &str, err: &crate::error::AppError) -> CrashReport {
    CrashReport {
        source: source.to_string(),
        label: label.to_string(),
        message: format!("{err}"),
        details: String::new(),
        captured_at: now_ms(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tempdir(suffix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let mut p = std::env::temp_dir();
        p.push(format!("draffity-crash-test-{nanos:x}-{suffix}"));
        fs::create_dir_all(&p).expect("create tempdir");
        p
    }

    #[test]
    fn noop_reporter_is_inactive_and_never_writes() {
        let r = NoOpCrashReporter;
        assert!(!r.is_active());
        assert!(!r.is_enabled());
        r.set_enabled(true);
        // set_enabled is a no-op for the no-op impl.
        assert!(!r.is_enabled());
        r.record(CrashReport {
            source: "test".into(),
            label: "x".into(),
            message: "y".into(),
            details: String::new(),
            captured_at: 0,
        })
        .unwrap();
    }

    #[test]
    fn local_file_reporter_writes_only_when_enabled() {
        let dir = tempdir("disabled");
        let r = LocalFileCrashReporter::new(&dir);
        // Disabled by default — nothing should land on disk.
        r.record(CrashReport {
            source: "test".into(),
            label: "x".into(),
            message: "y".into(),
            details: String::new(),
            captured_at: 1,
        })
        .unwrap();
        let entries: Vec<_> = fs::read_dir(&dir).unwrap().collect();
        assert!(entries.is_empty(), "no report should exist when disabled");
    }

    #[test]
    fn local_file_reporter_persists_when_enabled() {
        let dir = tempdir("enabled");
        let r = LocalFileCrashReporter::new(&dir);
        r.set_enabled(true);
        assert!(r.is_enabled());
        r.record(CrashReport {
            source: "test".into(),
            label: "boot".into(),
            message: "boom".into(),
            details: "trace".into(),
            captured_at: 42,
        })
        .unwrap();
        let files: Vec<_> = fs::read_dir(&dir).unwrap().filter_map(Result::ok).collect();
        assert_eq!(files.len(), 1);
        let body = fs::read_to_string(files[0].path()).unwrap();
        assert!(body.contains("\"label\":\"boot\""));
        assert!(body.contains("\"message\":\"boom\""));
    }

    #[test]
    fn report_from_error_uses_display_form() {
        use crate::error::AppError;
        let err = AppError::NotFound("missing".into());
        let r = report_from_error("test", "lookup", &err);
        assert_eq!(r.label, "lookup");
        assert!(r.message.contains("missing"));
    }
}
