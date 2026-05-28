use tauri::State;

use crate::domain::Template;
use crate::error::{AppError, AppResult};
use crate::services::template_from_project;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> CmdResult<Vec<Template>> {
    Ok(state.templates.list())
}

#[tauri::command]
pub fn get_template(state: State<'_, AppState>, id: String) -> CmdResult<Option<Template>> {
    Ok(state.templates.get(&id))
}

/// Snapshot the current project's structure (binder + synopses) as a
/// reusable user template. Content is dropped — templates seed empty
/// bodies. Returns the saved template so the UI can confirm with a name.
#[tauri::command]
pub fn save_project_as_template(
    state: State<'_, AppState>,
    project_id: String,
    name: String,
    description: Option<String>,
    locale: Option<String>,
) -> AppResult<Template> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invariant("template name is empty".into()));
    }
    let project = state
        .project_manager
        .get(&project_id)?
        .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;
    let documents = state.storage.list_documents(&project_id)?;
    let locale = locale.unwrap_or_else(|| "en".into());
    let template =
        template_from_project(&project, &documents, name, description.as_deref(), &locale);
    state.user_templates.save(&template)?;
    Ok(template)
}

#[tauri::command]
pub fn delete_user_template(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    // Defence in depth: never let a built-in id reach the disk path. The
    // user loader only writes/deletes under its own dir, but blocking the
    // built-in prefix early gives a clearer error.
    if !id.starts_with("user-") {
        return Err(AppError::Invariant(
            "only user-authored templates can be deleted".into(),
        ));
    }
    let path = state.user_templates.dir().join(format!("{id}.json"));
    if !path.exists() {
        return Err(AppError::NotFound(format!("user template {id}")));
    }
    std::fs::remove_file(&path)?;
    Ok(())
}
