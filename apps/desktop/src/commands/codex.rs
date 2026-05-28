use tauri::State;

use crate::domain::{CodexEntry, CodexInput, CodexKind, CodexUpdate};
use crate::error::AppError;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn create_codex_entry(state: State<'_, AppState>, input: CodexInput) -> CmdResult<CodexEntry> {
    // Validate project existence early so the user gets a clear "project
    // not found" instead of an FK error at the SQL layer.
    if state.project_manager.get(&input.project_id)?.is_none() {
        return Err(AppError::NotFound(format!("project {}", input.project_id)));
    }
    state.storage.create_codex_entry(input)
}

#[tauri::command]
pub fn list_codex_entries(
    state: State<'_, AppState>,
    project_id: String,
) -> CmdResult<Vec<CodexEntry>> {
    state.storage.list_codex_entries(&project_id)
}

#[tauri::command]
pub fn get_codex_entry(state: State<'_, AppState>, id: String) -> CmdResult<Option<CodexEntry>> {
    state.storage.get_codex_entry(&id)
}

#[tauri::command]
pub fn update_codex_entry(
    state: State<'_, AppState>,
    id: String,
    patch: CodexUpdate,
) -> CmdResult<CodexEntry> {
    state.storage.update_codex_entry(&id, patch)
}

#[tauri::command]
pub fn delete_codex_entry(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.storage.delete_codex_entry(&id)
}

#[tauri::command]
pub fn search_codex_entries(
    state: State<'_, AppState>,
    project_id: String,
    query: String,
    kind: Option<CodexKind>,
) -> CmdResult<Vec<CodexEntry>> {
    state
        .storage
        .search_codex_entries(&project_id, &query, kind)
}
