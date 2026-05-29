//! Custom metadata field commands (I-08/I-09). Thin orchestration over storage.

use tauri::State;

use crate::domain::{CustomField, CustomFieldInput, DocNode};
use crate::error::AppError;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn create_custom_field(
    state: State<'_, AppState>,
    input: CustomFieldInput,
) -> CmdResult<CustomField> {
    state.storage.create_custom_field(input)
}

#[tauri::command]
pub fn list_custom_fields(
    state: State<'_, AppState>,
    project_id: String,
) -> CmdResult<Vec<CustomField>> {
    state.storage.list_custom_fields(&project_id)
}

#[tauri::command]
pub fn update_custom_field(
    state: State<'_, AppState>,
    id: String,
    name: String,
    options: Vec<String>,
) -> CmdResult<CustomField> {
    state.storage.update_custom_field(&id, &name, &options)
}

#[tauri::command]
pub fn delete_custom_field(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.storage.delete_custom_field(&id)
}

/// Set or clear (`value=None`) a document's value for one custom field.
#[tauri::command]
pub fn set_document_metadata(
    state: State<'_, AppState>,
    document_id: String,
    field_id: String,
    value: Option<String>,
) -> CmdResult<DocNode> {
    state
        .storage
        .set_document_metadata(&document_id, &field_id, value.as_deref())
}
