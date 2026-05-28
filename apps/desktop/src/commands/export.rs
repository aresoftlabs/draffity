use std::path::PathBuf;

use tauri::State;

use crate::error::{AppError, AppResult};
use crate::services::ExportFormat;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

/// Export a project to disk in the requested format. Returns the absolute
/// path that was written. The UI is responsible for picking `output_path`
/// (typically through the Tauri save dialog plugin).
#[tauri::command]
pub fn export_project(
    state: State<'_, AppState>,
    project_id: String,
    format: ExportFormat,
    output_path: String,
) -> CmdResult<String> {
    let project = state
        .project_manager
        .get(&project_id)?
        .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;
    let documents = state.storage.list_documents(&project_id)?;

    let bytes = state.exporter.export(&project, &documents, format)?;

    let path = PathBuf::from(&output_path);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(&path, &bytes)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn supported_export_formats(state: State<'_, AppState>) -> AppResult<Vec<ExportFormat>> {
    Ok(state.exporter.supported_formats())
}
