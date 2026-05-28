use tauri::State;

use crate::error::AppError;
use crate::services::{BackupKind, BackupRecord};
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn list_backups(state: State<'_, AppState>) -> CmdResult<Vec<BackupRecord>> {
    state.backup.list_backups()
}

/// Create an on-demand backup. Goes into the rotation but is tagged
/// `manual` so the prune pass never deletes it.
#[tauri::command]
pub fn create_manual_backup(state: State<'_, AppState>) -> CmdResult<BackupRecord> {
    state.backup.create_backup(BackupKind::Manual)
}

/// Restore a backup by id. The caller is expected to restart the app to
/// re-open the DB cleanly — the command returns success once the file
/// copy lands.
#[tauri::command]
pub fn restore_backup(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.backup.restore_backup(&id)
}

#[tauri::command]
pub fn prune_backups(state: State<'_, AppState>) -> CmdResult<usize> {
    state.backup.prune_old_backups()
}
