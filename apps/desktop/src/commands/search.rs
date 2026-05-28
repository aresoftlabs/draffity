use tauri::State;

use crate::domain::SearchHit;
use crate::error::AppError;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn search_documents(
    state: State<'_, AppState>,
    project_id: String,
    query: String,
) -> CmdResult<Vec<SearchHit>> {
    state.storage.search_documents(&project_id, &query)
}
