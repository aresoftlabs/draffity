use tauri::State;

use crate::domain::{DailyWriting, WritingStats};
use crate::error::AppError;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn ping() -> String {
    format!("pong from draffity v{}", env!("CARGO_PKG_VERSION"))
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

#[tauri::command]
pub fn get_daily_goal(state: State<'_, AppState>) -> CmdResult<Option<i64>> {
    state.storage.get_daily_goal()
}

#[tauri::command]
pub fn set_daily_goal(state: State<'_, AppState>, goal: Option<i64>) -> CmdResult<()> {
    state.storage.set_daily_goal(goal)
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

/// Returns the current resources root path.
#[tauri::command]
pub fn get_resources_path(state: State<'_, AppState>) -> String {
    state.resources.root().to_string_lossy().into_owned()
}

/// Set a custom resources path. Persists to config.json — applies on next restart.
#[tauri::command]
pub fn set_resources_path(state: State<'_, AppState>, path: String) -> CmdResult<()> {
    let p = std::path::PathBuf::from(&path);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Unexpected(format!("no se pudo crear el directorio: {e}")))?;
    }
    let marker = p.join(".draffity-write-test");
    std::fs::write(&marker, b"")
        .map_err(|e| AppError::Unexpected(format!("sin permiso de escritura en {path}: {e}")))?;
    let _ = std::fs::remove_file(&marker);

    // Persist to config.json at the default home location
    let config_path = state.resources.config_path();
    let config = serde_json::json!({ "root": path });
    std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)
        .map_err(|e| AppError::Unexpected(format!("no se pudo guardar la configuración: {e}")))?;
    tracing::info!(?path, config = %config_path.display(), "resources path changed — restart to apply");
    Ok(())
}
