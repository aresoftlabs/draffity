use tauri::{AppHandle, Emitter, State};

use crate::domain::{Project, ProjectInput};
use crate::error::AppError;
use crate::events;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    app: AppHandle,
    input: ProjectInput,
) -> CmdResult<Project> {
    let project = state.project_manager.create(input)?;
    let _ = app.emit(events::PROJECT_CREATED, &project);
    let _ = app.emit(events::PROJECT_OPENED, &project);
    Ok(project)
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> CmdResult<Vec<Project>> {
    state.project_manager.list()
}

#[tauri::command]
pub fn get_project(state: State<'_, AppState>, id: String) -> CmdResult<Option<Project>> {
    state.project_manager.get(&id)
}

#[tauri::command]
pub fn get_active_project(state: State<'_, AppState>) -> CmdResult<Option<Project>> {
    state.project_manager.active()
}

#[tauri::command]
pub fn open_project(state: State<'_, AppState>, app: AppHandle, id: String) -> CmdResult<Project> {
    let project = state.project_manager.activate(&id)?;
    let _ = app.emit(events::PROJECT_OPENED, &project);
    Ok(project)
}

#[tauri::command]
pub fn archive_project(state: State<'_, AppState>, app: AppHandle, id: String) -> CmdResult<()> {
    state.project_manager.archive(&id)?;
    let _ = app.emit(events::PROJECT_ARCHIVED, &id);
    Ok(())
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, app: AppHandle, id: String) -> CmdResult<()> {
    state.project_manager.delete(&id)?;
    let _ = app.emit(events::PROJECT_DELETED, &id);
    Ok(())
}

#[tauri::command]
pub fn set_project_goal(
    state: State<'_, AppState>,
    id: String,
    goal: Option<i64>,
) -> CmdResult<Project> {
    state.storage.set_project_goal(&id, goal)
}

#[tauri::command]
pub fn set_project_deadline(
    state: State<'_, AppState>,
    id: String,
    deadline: Option<i64>,
) -> CmdResult<Project> {
    state.storage.set_project_deadline(&id, deadline)
}
