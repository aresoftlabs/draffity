use tauri::{AppHandle, Emitter, State};

use crate::domain::{DocNode, DocumentInput, Snapshot};
use crate::error::AppError;
use crate::events;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn create_document(
    state: State<'_, AppState>,
    app: AppHandle,
    input: DocumentInput,
) -> CmdResult<DocNode> {
    let doc = state.storage.create_document(input)?;
    let _ = app.emit(events::DOCUMENT_CREATED, &doc);
    Ok(doc)
}

#[tauri::command]
pub fn list_documents(state: State<'_, AppState>, project_id: String) -> CmdResult<Vec<DocNode>> {
    state.storage.list_documents(&project_id)
}

#[tauri::command]
pub fn get_document(state: State<'_, AppState>, id: String) -> CmdResult<Option<DocNode>> {
    state.storage.get_document(&id)
}

#[tauri::command]
pub fn update_document(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
    title: Option<String>,
    content: Option<String>,
) -> CmdResult<DocNode> {
    let doc = state
        .storage
        .update_document(&id, title.as_deref(), content.as_deref())?;
    // Best-effort: a writing-stats failure must not block the save.
    let _ = state.storage.record_writing_activity();
    let _ = app.emit(events::DOCUMENT_SAVED, &doc);
    Ok(doc)
}

#[tauri::command]
pub fn move_document(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
    parent_id: Option<String>,
    position: i64,
) -> CmdResult<()> {
    state
        .storage
        .move_document(&id, parent_id.as_deref(), position)?;
    let _ = app.emit(events::DOCUMENT_MOVED, &id);
    Ok(())
}

#[tauri::command]
pub fn reorder_documents(
    state: State<'_, AppState>,
    app: AppHandle,
    project_id: String,
    parent_id: Option<String>,
    ordered_ids: Vec<String>,
) -> CmdResult<()> {
    state
        .storage
        .reorder_documents(&project_id, parent_id.as_deref(), &ordered_ids)?;
    let _ = app.emit(events::DOCUMENT_MOVED, &project_id);
    Ok(())
}

#[tauri::command]
pub fn delete_document(state: State<'_, AppState>, app: AppHandle, id: String) -> CmdResult<()> {
    state.storage.delete_document(&id)?;
    let _ = app.emit(events::DOCUMENT_DELETED, &id);
    Ok(())
}

#[tauri::command]
pub fn create_snapshot(
    state: State<'_, AppState>,
    app: AppHandle,
    document_id: String,
    label: Option<String>,
) -> CmdResult<Snapshot> {
    let snap = state
        .storage
        .create_snapshot(&document_id, label.as_deref())?;
    let _ = app.emit(events::SNAPSHOT_CREATED, &snap);
    Ok(snap)
}

#[tauri::command]
pub fn list_snapshots(state: State<'_, AppState>, document_id: String) -> CmdResult<Vec<Snapshot>> {
    state.storage.list_snapshots(&document_id)
}

#[tauri::command]
pub fn restore_snapshot(
    state: State<'_, AppState>,
    app: AppHandle,
    snapshot_id: String,
) -> CmdResult<DocNode> {
    let doc = state.storage.restore_snapshot(&snapshot_id)?;
    let _ = app.emit(events::DOCUMENT_SAVED, &doc);
    Ok(doc)
}
