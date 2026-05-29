use tauri::State;

use crate::capabilities::{is_enabled, Tier};
use crate::domain::{DailyWriting, WritingStats};
use crate::error::AppError;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn ping() -> String {
    format!("pong from draffity v{}", env!("CARGO_PKG_VERSION"))
}

#[tauri::command]
pub fn capability_enabled(state: State<'_, AppState>, name: String) -> bool {
    state.tier.is_enabled(&name)
}

/// Pure form (no state) used by smoke tests and as a fallback. The MVP
/// always runs Free tier so this matches `capability_enabled` exactly.
#[tauri::command]
pub fn capability_enabled_pure(name: String) -> bool {
    is_enabled(Tier::Free, &name)
}

#[tauri::command]
pub fn get_setting(state: State<'_, AppState>, key: String) -> CmdResult<Option<String>> {
    state.storage.get_setting(&key)
}

#[tauri::command]
pub fn set_setting(state: State<'_, AppState>, key: String, value: String) -> CmdResult<()> {
    state.storage.set_setting(&key, &value)
}

#[tauri::command]
pub fn get_writing_stats(state: State<'_, AppState>) -> CmdResult<WritingStats> {
    state.storage.get_writing_stats()
}

#[tauri::command]
pub fn get_recent_daily_writing(
    state: State<'_, AppState>,
    days: u32,
) -> CmdResult<Vec<DailyWriting>> {
    state.storage.list_recent_daily_writing(days)
}

/// Snapshot of the crash-reporting service state. `active` tells the UI
/// whether to show the opt-in toggle at all (no destination → no toggle).
/// `enabled` is the user's current consent value.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashReportingStatus {
    pub active: bool,
    pub enabled: bool,
}

#[tauri::command]
pub fn get_crash_reporting_status(state: State<'_, AppState>) -> CrashReportingStatus {
    CrashReportingStatus {
        active: state.crash_reporter.is_active(),
        enabled: state.crash_reporter.is_enabled(),
    }
}

#[tauri::command]
pub fn set_crash_reporting_enabled(state: State<'_, AppState>, enabled: bool) -> CmdResult<()> {
    state.crash_reporter.set_enabled(enabled);
    state
        .storage
        .set_setting("crash_reporting.enabled", if enabled { "1" } else { "0" })?;
    Ok(())
}
