use tauri::State;

use crate::domain::Template;
use crate::error::AppError;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> CmdResult<Vec<Template>> {
    Ok(state.templates.list())
}

#[tauri::command]
pub fn get_template(state: State<'_, AppState>, id: String) -> CmdResult<Option<Template>> {
    Ok(state.templates.get(&id))
}
