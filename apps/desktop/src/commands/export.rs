use std::path::PathBuf;

use tauri::State;

use crate::error::{AppError, AppResult};
use crate::services::{export_config_settings_key, ExportConfig, ExportFormat};
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

/// Export a project to disk in the requested format. Returns the absolute
/// path that was written. The UI is responsible for picking `output_path`
/// (typically through the Tauri save dialog plugin).
///
/// `config` is optional — when absent we fall back to the per-project
/// persisted config, and finally to `ExportConfig::default()`.
#[tauri::command]
pub fn export_project(
    state: State<'_, AppState>,
    project_id: String,
    format: ExportFormat,
    output_path: String,
    config: Option<ExportConfig>,
) -> CmdResult<String> {
    let project = state
        .project_manager
        .get(&project_id)?
        .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;
    let documents = state.storage.list_documents(&project_id)?;
    let codex = state.storage.list_codex_entries(&project_id)?;

    let effective = match config {
        Some(c) => c,
        None => load_config(&state, &project_id)?,
    };

    let bytes = state
        .exporter
        .export(&project, &documents, &codex, format, &effective)?;

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

/// Read the persisted `ExportConfig` for a project. Returns defaults when
/// nothing was ever saved or the stored payload is unreadable.
#[tauri::command]
pub fn get_export_config(
    state: State<'_, AppState>,
    project_id: String,
) -> CmdResult<ExportConfig> {
    load_config(&state, &project_id)
}

/// Persist the `ExportConfig` for a project. Subsequent calls to
/// `export_project` without `config` will use this payload.
#[tauri::command]
pub fn set_export_config(
    state: State<'_, AppState>,
    project_id: String,
    config: ExportConfig,
) -> CmdResult<()> {
    let key = export_config_settings_key(&project_id);
    let json = serde_json::to_string(&config)?;
    state.storage.set_setting(&key, &json)?;
    Ok(())
}

fn load_config(state: &State<'_, AppState>, project_id: &str) -> AppResult<ExportConfig> {
    let key = export_config_settings_key(project_id);
    let Some(raw) = state.storage.get_setting(&key)? else {
        return Ok(ExportConfig::default());
    };
    // Tolerate a malformed payload (e.g. older schema) by falling back to
    // defaults rather than failing the entire export.
    Ok(serde_json::from_str(&raw).unwrap_or_default())
}
