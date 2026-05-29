use tauri::{AppHandle, Emitter, State};

use crate::domain::{count_words_in_html, DocNode, DocumentInput, DocumentStatus, Snapshot};
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
    content_json: Option<String>,
) -> CmdResult<DocNode> {
    // Snapshot the pre-update word count so we can attribute the delta to
    // today's daily-writing row. Only matters when `content` is being
    // changed — pure title/metadata edits don't count as writing.
    let prev_words = if content.is_some() {
        state
            .storage
            .get_document(&id)?
            .and_then(|d| d.content)
            .as_deref()
            .map(count_words_in_html)
            .unwrap_or(0)
    } else {
        0
    };

    let doc = state.storage.update_document(
        &id,
        title.as_deref(),
        content.as_deref(),
        content_json.as_deref(),
    )?;
    // Best-effort: a writing-stats failure must not block the save.
    let _ = state.storage.record_writing_activity();
    if let Some(new_html) = content.as_deref() {
        let new_words = count_words_in_html(new_html);
        let delta = new_words.saturating_sub(prev_words);
        let _ = state.storage.record_daily_writing(delta);
    }
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
pub fn set_document_status(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
    status: DocumentStatus,
) -> CmdResult<DocNode> {
    let doc = state.storage.set_document_status(&id, status)?;
    let _ = app.emit(events::DOCUMENT_SAVED, &doc);
    Ok(doc)
}

#[tauri::command]
pub fn set_document_tags(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
    tags: Vec<String>,
) -> CmdResult<DocNode> {
    let doc = state.storage.set_document_tags(&id, &tags)?;
    let _ = app.emit(events::DOCUMENT_SAVED, &doc);
    Ok(doc)
}

#[tauri::command]
pub fn list_project_tags(state: State<'_, AppState>, project_id: String) -> CmdResult<Vec<String>> {
    state.storage.list_project_tags(&project_id)
}

#[tauri::command]
pub fn set_document_goal(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
    goal: Option<i64>,
) -> CmdResult<DocNode> {
    let doc = state.storage.set_document_goal(&id, goal)?;
    let _ = app.emit(events::DOCUMENT_SAVED, &doc);
    Ok(doc)
}

#[tauri::command]
pub fn set_document_synopsis(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
    synopsis: Option<String>,
) -> CmdResult<DocNode> {
    // Normalise: trim + treat empty/whitespace as None so the column is
    // either NULL or a meaningful string.
    let cleaned = synopsis.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let doc = state.storage.set_document_synopsis(&id, cleaned)?;
    let _ = app.emit(events::DOCUMENT_SAVED, &doc);
    Ok(doc)
}

#[tauri::command]
pub fn set_document_research(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
    is_research: bool,
) -> CmdResult<DocNode> {
    let doc = state.storage.set_document_research(&id, is_research)?;
    let _ = app.emit(events::DOCUMENT_SAVED, &doc);
    Ok(doc)
}

#[tauri::command]
pub fn set_document_matter(
    state: State<'_, AppState>,
    app: AppHandle,
    id: String,
    is_front: bool,
    is_back: bool,
) -> CmdResult<DocNode> {
    let doc = state.storage.set_document_matter(&id, is_front, is_back)?;
    let _ = app.emit(events::DOCUMENT_SAVED, &doc);
    Ok(doc)
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
