//! Collection commands (I-01..I-03). Thin orchestration over the storage
//! layer; the smart-query matching lives in the domain.

use tauri::State;

use crate::domain::{Collection, CollectionInput, CollectionQuery, DocNode};
use crate::error::AppError;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn create_collection(
    state: State<'_, AppState>,
    input: CollectionInput,
) -> CmdResult<Collection> {
    state.storage.create_collection(input)
}

#[tauri::command]
pub fn list_collections(
    state: State<'_, AppState>,
    project_id: String,
) -> CmdResult<Vec<Collection>> {
    state.storage.list_collections(&project_id)
}

#[tauri::command]
pub fn rename_collection(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> CmdResult<Collection> {
    state.storage.rename_collection(&id, &name)
}

#[tauri::command]
pub fn set_collection_query(
    state: State<'_, AppState>,
    id: String,
    query: CollectionQuery,
) -> CmdResult<Collection> {
    state.storage.set_collection_query(&id, &query)
}

#[tauri::command]
pub fn delete_collection(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.storage.delete_collection(&id)
}

#[tauri::command]
pub fn set_collection_members(
    state: State<'_, AppState>,
    collection_id: String,
    ordered_ids: Vec<String>,
) -> CmdResult<()> {
    state
        .storage
        .set_collection_members(&collection_id, &ordered_ids)
}

/// The documents in a collection (manual order or live smart filter).
#[tauri::command]
pub fn resolve_collection(state: State<'_, AppState>, id: String) -> CmdResult<Vec<DocNode>> {
    state.storage.resolve_collection(&id)
}
