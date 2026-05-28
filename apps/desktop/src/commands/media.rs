use tauri::State;

use crate::domain::MediaAsset;
use crate::error::AppError;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

/// Persist a clipboard / drop blob for the given project. `bytes` is the
/// raw byte buffer (Tauri encodes `Vec<u8>` as JSON-friendly base64 on
/// the wire). Returns the registry row; if the same content already
/// exists for this project the existing asset is returned (dedupe by
/// sha256).
#[tauri::command]
pub fn upload_media(
    state: State<'_, AppState>,
    project_id: String,
    mime: String,
    bytes: Vec<u8>,
) -> CmdResult<MediaAsset> {
    // Validate project early so the user gets a clear "project not found"
    // instead of an FK error after writing the file.
    if state.project_manager.get(&project_id)?.is_none() {
        return Err(AppError::NotFound(format!("project {project_id}")));
    }
    state.media.store(&project_id, &mime, &bytes)
}

/// Read the bytes of a stored asset. The editor uses this to turn an
/// `<img data-media-id="…">` node into a Blob URL for display.
#[tauri::command]
pub fn read_media_bytes(state: State<'_, AppState>, id: String) -> CmdResult<Vec<u8>> {
    state.media.read(&id)
}

#[tauri::command]
pub fn get_media_asset(state: State<'_, AppState>, id: String) -> CmdResult<Option<MediaAsset>> {
    state.storage.get_media(&id)
}

#[tauri::command]
pub fn list_project_media(
    state: State<'_, AppState>,
    project_id: String,
) -> CmdResult<Vec<MediaAsset>> {
    state.storage.list_media(&project_id)
}

#[tauri::command]
pub fn delete_media(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.media.delete(&id)
}
