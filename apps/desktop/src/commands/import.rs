//! Import command. Parses a file payload (Markdown for now, DOCX in C-02)
//! into an `ImportTree` and seeds a fresh project with the resulting
//! document tree. The UI uses `tauri-plugin-fs` to read the file and
//! passes the bytes here.

use tauri::{AppHandle, Emitter, State};

use crate::domain::Project;
use crate::error::AppError;
use crate::events;
use crate::services::ImportFormat;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

/// Parse `bytes` as `format`, create a project named after the parsed
/// title (falling back to `filename_hint`) and insert the imported
/// document tree atomically. Returns the freshly-created project.
#[tauri::command]
pub fn import_project(
    state: State<'_, AppState>,
    app: AppHandle,
    format: ImportFormat,
    bytes: Vec<u8>,
    filename_hint: String,
) -> CmdResult<Project> {
    if bytes.is_empty() {
        return Err(AppError::Invariant("import payload is empty".into()));
    }
    let tree = state.importer.import(format, &bytes, &filename_hint)?;
    // Imported projects start without a template binding — the seeded
    // tree owns its structure, so we tag them as `generic` for the
    // purposes of the templates picker.
    let project = state.storage.create_project_from_import(&tree, "generic")?;
    let _ = app.emit(events::PROJECT_CREATED, &project);
    Ok(project)
}

/// Lists the formats the wired-up importer can handle. Lets the UI hide
/// disabled options without hard-coding the list on the frontend.
#[tauri::command]
pub fn supported_import_formats(state: State<'_, AppState>) -> Vec<ImportFormat> {
    state.importer.supported_formats()
}
