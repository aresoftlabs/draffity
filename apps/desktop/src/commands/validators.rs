//! AI validation commands (Épica G, slice 2). Assemble the context for a
//! document (text + codex + prior-chapter anchor), run the requested
//! validators off-thread, persist each report, and return them. Also the
//! codex coverage pre-check (G-03).

use tauri::State;

use crate::domain::{AiValidation, CodexKind};
use crate::error::{AppError, AppResult};
use crate::services::validation_context::strip_html;
use crate::services::{
    codex_coverage, summarize, CoverageReport, Finding, Severity, ValidationContextBuilder,
    ValidationInput, ValidatorKind,
};
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

/// Fetch the rows a validation run needs and hand them to the (pure, tested)
/// `ValidationContextBuilder`. The command stays orchestration-only (AUD-27).
fn assemble_input(
    state: &AppState,
    project_id: &str,
    document_id: &str,
) -> AppResult<ValidationInput> {
    let doc = state
        .storage
        .get_document(document_id)?
        .ok_or_else(|| AppError::NotFound(format!("document {document_id}")))?;
    let codex = state.storage.list_codex_entries(project_id)?;
    let documents = state.storage.list_documents(project_id)?;
    Ok(ValidationContextBuilder::default().build(&doc, &codex, &documents))
}

/// G-03: how well the codex describes the document's apparent cast. The UI
/// warns before running if this comes back sparse.
#[tauri::command]
pub fn check_codex_coverage(
    state: State<'_, AppState>,
    project_id: String,
    document_id: String,
) -> CmdResult<CoverageReport> {
    let doc = state
        .storage
        .get_document(&document_id)?
        .ok_or_else(|| AppError::NotFound(format!("document {document_id}")))?;
    let text = strip_html(doc.content.as_deref().unwrap_or(""));
    let names: Vec<String> = state
        .storage
        .list_codex_entries(&project_id)?
        .into_iter()
        .filter(|e| matches!(e.kind, CodexKind::Character))
        .map(|e| e.name)
        .collect();
    Ok(codex_coverage(&text, &names))
}

/// G-10: run the requested validators on a document, persist each report, and
/// return them. AI validators run only when AI is available; a per-validator
/// failure becomes a warning finding rather than aborting the batch.
#[tauri::command]
pub async fn run_validators(
    state: State<'_, AppState>,
    project_id: String,
    document_id: String,
    validators: Vec<String>,
) -> CmdResult<Vec<AiValidation>> {
    let kinds: Vec<ValidatorKind> = validators
        .iter()
        .map(|s| ValidatorKind::parse(s))
        .collect::<AppResult<_>>()?;

    let needs_ai = kinds
        .iter()
        .any(|k| !matches!(k, ValidatorKind::Repetition | ValidatorKind::Style));
    if needs_ai && !state.validators.available() {
        return Err(AppError::Unsupported(
            "las funciones de IA no están activas".into(),
        ));
    }

    let input = assemble_input(&state, &project_id, &document_id)?;

    // Run off the main thread — AI validators do blocking HTTP.
    let svc = state.validators.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        kinds
            .into_iter()
            .map(|k| (k, svc.run(k, &input)))
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|e| AppError::Unexpected(format!("tarea de validación: {e}")))?;

    let mut reports = Vec::with_capacity(outcome.len());
    for (kind, result) in outcome {
        let findings = result.unwrap_or_else(|e| {
            vec![Finding {
                validator: kind.as_str().to_string(),
                severity: Severity::Warning,
                title: "El validador no se pudo ejecutar".to_string(),
                detail: e.to_string(),
                excerpt: None,
                suggestion: None,
                code: Some("validatorFailed".to_string()),
                params: Some(std::collections::BTreeMap::from([(
                    "error".to_string(),
                    e.to_string(),
                )])),
            }]
        });
        let json = serde_json::to_string(&findings)?;
        let summary = summarize(&findings);
        reports.push(state.storage.record_ai_validation(
            &document_id,
            kind.as_str(),
            &json,
            &summary,
        )?);
    }
    Ok(reports)
}

/// All persisted reports for a document, newest first (powers the report view).
#[tauri::command]
pub fn list_validations(
    state: State<'_, AppState>,
    document_id: String,
) -> CmdResult<Vec<AiValidation>> {
    state.storage.list_ai_validations(&document_id)
}
