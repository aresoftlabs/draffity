use tauri::State;

use crate::domain::Citation;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

/// Outcome of a bibliography import. The UI surfaces `imported` (final count
/// in the DB after upsert) and `skipped` (entries the BibTeX parser dropped).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub imported: Vec<Citation>,
    pub skipped: usize,
}

/// Parse a BibTeX/BibLaTeX string and upsert every entry into the project's
/// bibliography. Existing keys are overwritten.
#[tauri::command]
pub fn import_bibliography(
    state: State<'_, AppState>,
    project_id: String,
    bib_text: String,
) -> CmdResult<ImportSummary> {
    // Ensure the project exists first — fail fast with a useful error.
    if state.project_manager.get(&project_id)?.is_none() {
        return Err(AppError::NotFound(format!("project {project_id}")));
    }
    let summary = state.bibliography.parse(&bib_text)?;
    let imported = state
        .storage
        .upsert_citations(&project_id, &summary.entries)?;
    Ok(ImportSummary {
        imported,
        skipped: summary.skipped_entries,
    })
}

#[tauri::command]
pub fn list_citations(state: State<'_, AppState>, project_id: String) -> AppResult<Vec<Citation>> {
    state.storage.list_citations(&project_id)
}

#[tauri::command]
pub fn list_citation_keys(
    state: State<'_, AppState>,
    project_id: String,
) -> AppResult<Vec<String>> {
    state.storage.list_citation_keys(&project_id)
}

#[tauri::command]
pub fn delete_citation(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.storage.delete_citation(&id)?;
    Ok(())
}
