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
