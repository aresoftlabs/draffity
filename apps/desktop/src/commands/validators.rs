//! AI validation commands (Épica G, slice 2). Assemble the context for a
//! document (text + codex + prior-chapter anchor), run the requested
//! validators off-thread, persist each report, and return them. Also the
//! codex coverage pre-check (G-03).

use tauri::State;

use crate::domain::{AiValidation, CodexKind, DocumentType};
use crate::error::{AppError, AppResult};
use crate::services::{
    codex_coverage, summarize, CoverageReport, Finding, Severity, ValidationInput, ValidatorKind,
};
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

/// Char caps so prompts stay bounded regardless of chapter size.
const TEXT_CAP: usize = 12_000;
const CODEX_CAP: usize = 4_000;
const ANCHOR_CAP: usize = 4_000;
const ANCHOR_DOCS: usize = 3;

/// Strip HTML tags and collapse whitespace to plain text for the model.
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cap(mut s: String, max: usize) -> String {
    if s.chars().count() > max {
        s = s.chars().take(max).collect();
        s.push('…');
    }
    s
}

/// Gather everything a validator needs for one document.
fn assemble_input(
    state: &AppState,
    project_id: &str,
    document_id: &str,
) -> AppResult<ValidationInput> {
    let doc = state
        .storage
        .get_document(document_id)?
        .ok_or_else(|| AppError::NotFound(format!("document {document_id}")))?;
    let text = cap(strip_html(doc.content.as_deref().unwrap_or("")), TEXT_CAP);

    // Codex block (all entries) for character + plot validators.
    let entries = state.storage.list_codex_entries(project_id)?;
    let codex_block = cap(
        entries
            .iter()
            .map(|e| {
                let body = e.body.as_deref().unwrap_or("—");
                let tags = if e.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [tags: {}]", e.tags.join(", "))
                };
                format!("- {} ({}): {}{}", e.name, e.kind.as_str(), body, tags)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        CODEX_CAP,
    );

    // Voice anchor: the last few prior chapters/scenes, in order.
    let docs = state.storage.list_documents(project_id)?;
    let current_pos = docs
        .iter()
        .find(|d| d.id == document_id)
        .map(|d| d.position)
        .unwrap_or(i64::MAX);
    let mut prior: Vec<_> = docs
        .iter()
        .filter(|d| {
            d.position < current_pos
                && matches!(d.doc_type, DocumentType::Chapter | DocumentType::Scene)
                && d.content.is_some()
        })
        .collect();
    prior.sort_by_key(|d| d.position);
    let start = prior.len().saturating_sub(ANCHOR_DOCS);
    let anchor_text = cap(
        prior[start..]
            .iter()
            .map(|d| strip_html(d.content.as_deref().unwrap_or("")))
            .collect::<Vec<_>>()
            .join("\n\n"),
        ANCHOR_CAP,
    );

    Ok(ValidationInput {
        text,
        codex_block,
        anchor_text,
    })
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
