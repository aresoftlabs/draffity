//! Label commands (I-05/I-06). Thin orchestration over the storage layer.

use tauri::State;

use crate::domain::{DocNode, Label, LabelInput};
use crate::error::AppError;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn create_label(state: State<'_, AppState>, input: LabelInput) -> CmdResult<Label> {
    state.storage.create_label(input)
}

#[tauri::command]
pub fn list_labels(state: State<'_, AppState>, project_id: String) -> CmdResult<Vec<Label>> {
    state.storage.list_labels(&project_id)
}

#[tauri::command]
pub fn update_label(
    state: State<'_, AppState>,
    id: String,
    name: String,
    color: String,
) -> CmdResult<Label> {
    state.storage.update_label(&id, &name, &color)
}

#[tauri::command]
pub fn delete_label(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.storage.delete_label(&id)
}

/// Replace the entire label set of a document; returns the refreshed node.
#[tauri::command]
pub fn set_document_labels(
    state: State<'_, AppState>,
    document_id: String,
    label_ids: Vec<String>,
) -> CmdResult<DocNode> {
    state.storage.set_document_labels(&document_id, &label_ids)
}
