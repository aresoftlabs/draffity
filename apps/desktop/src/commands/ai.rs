//! AI commands (Épica F). BYOK key management + status (slice 1) and the
//! inline actions Continue/Expand/Rewrite/Describe with event streaming,
//! cancellation, and accepted-generation history (slice 2).
//!
//! The OpenRouter key lives in the OS keyring via `SecretStorage` — never in
//! the plain `settings` table. Streaming runs off the main thread via
//! `spawn_blocking` (the blocking HTTP client must not be driven from inside
//! the Tokio runtime); each delta is relayed to the UI as an
//! `ai.suggestion.received` event tagged with the request id.

use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::domain::AiHistoryEntry;
use crate::error::AppError;
use crate::events::AI_SUGGESTION_RECEIVED;
use crate::services::{
    build_messages, parse_action, ActionInput, CompletionRequest, MemoryRequest, OPENROUTER_KEY,
};
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

// --- BYOK key + status ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    /// AI is usable right now: a key is stored.
    pub available: bool,
    /// A key is stored.
    pub has_key: bool,
}

fn status(state: &AppState) -> CmdResult<AiStatus> {
    // Propagate a keyring/OS failure instead of swallowing it as "no key":
    // a user with a stored key would otherwise see the app as un-configured
    // (AUD-18). `None` means genuinely no key; `Err` means we couldn't tell.
    let has_key = state
        .secrets
        .get_secret(OPENROUTER_KEY)?
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false);
    Ok(AiStatus {
        available: state.ai.available(),
        has_key,
    })
}

#[tauri::command]
pub fn get_ai_status(state: State<'_, AppState>) -> CmdResult<AiStatus> {
    status(&state)
}

/// Store (or, when blank, clear) the BYOK OpenRouter key.
#[tauri::command]
pub fn set_openrouter_key(state: State<'_, AppState>, key: String) -> CmdResult<AiStatus> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        state.secrets.delete_secret(OPENROUTER_KEY)?;
    } else {
        state.secrets.set_secret(OPENROUTER_KEY, trimmed)?;
    }
    status(&state)
}

#[tauri::command]
pub fn clear_openrouter_key(state: State<'_, AppState>) -> CmdResult<AiStatus> {
    state.secrets.delete_secret(OPENROUTER_KEY)?;
    status(&state)
}

// --- inline actions ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiActionRequest {
    /// UI-generated id correlating the stream events + cancellation.
    pub request_id: String,
    /// `continue` | `expand` | `rewrite` | `describe`.
    pub action: String,
    /// Rewrite sub-mode: rephrase | shorter | vivid | show_not_tell |
    /// inner_conflict | custom.
    #[serde(default)]
    pub sub_mode: Option<String>,
    pub project_id: String,
    #[serde(default)]
    pub doc_id: Option<String>,
    /// Selected text the action operates on (empty for Continue).
    #[serde(default)]
    pub selected_text: String,
    /// Text before the cursor/selection, for continuity + memory anchoring.
    #[serde(default)]
    pub preceding_text: String,
    /// Free-form instruction for rewrite `custom`.
    #[serde(default)]
    pub custom_prompt: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiActionResult {
    pub request_id: String,
    pub text: String,
    /// True when the user cancelled mid-stream; the text is partial and the
    /// UI should discard it.
    pub cancelled: bool,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AiDelta {
    request_id: String,
    delta: String,
}

/// Run an inline AI action, streaming deltas as events and returning the
/// assembled result. Gated by `ai.available()` (key required).
#[tauri::command]
pub async fn ai_run_action(
    app: AppHandle,
    state: State<'_, AppState>,
    req: AiActionRequest,
) -> CmdResult<AiActionResult> {
    if !state.ai.available() {
        return Err(AppError::Unsupported(
            "las funciones de IA no están activas".into(),
        ));
    }

    let action = parse_action(
        &req.action,
        req.sub_mode.as_deref(),
        req.custom_prompt.as_deref(),
    )?;

    // Memory context (sync, cheap) — anchored on the working window.
    let window = format!("{}\n{}", req.preceding_text, req.selected_text);
    let memory = state.memory.build_context(&MemoryRequest {
        project_id: req.project_id.clone(),
        doc_id: req.doc_id.clone(),
        window_text: window,
        token_budget: MemoryRequest::DEFAULT_BUDGET,
    })?;
    let messages = build_messages(
        &action,
        &ActionInput {
            selected_text: &req.selected_text,
            preceding_text: &req.preceding_text,
            memory_block: &memory.render(),
        },
    )?;
    let completion = CompletionRequest {
        messages,
        model: req.model.clone(),
        temperature: None,
        max_tokens: req.max_tokens,
    };

    // Extract everything the blocking task needs so we never touch `State`
    // across the await.
    let ai = state.ai.clone();
    let cancel = state.ai_cancel.clone();
    let flag = cancel.register(&req.request_id);
    let flag_for_sink = flag.clone();
    let app_for_sink = app.clone();
    let rid = req.request_id.clone();

    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let mut sink = |delta: &str| {
            if flag_for_sink.load(Ordering::Relaxed) {
                return; // cancelled: stop feeding the UI
            }
            let _ = app_for_sink.emit(
                AI_SUGGESTION_RECEIVED,
                AiDelta {
                    request_id: rid.clone(),
                    delta: delta.to_string(),
                },
            );
        };
        ai.stream_complete(completion, &mut sink)
    })
    .await
    .map_err(|e| AppError::Unexpected(format!("tarea de IA: {e}")))?;

    cancel.finish(&req.request_id);
    let cancelled = flag.load(Ordering::Relaxed);
    let resp = outcome?;
    Ok(AiActionResult {
        request_id: req.request_id,
        text: resp.text,
        cancelled,
        prompt_tokens: resp.usage.map(|u| u.prompt_tokens),
        completion_tokens: resp.usage.map(|u| u.completion_tokens),
    })
}

/// Cancel an in-flight stream by request id (Esc in the preview). Idempotent.
#[tauri::command]
pub fn ai_cancel(state: State<'_, AppState>, request_id: String) {
    state.ai_cancel.cancel(&request_id);
}

// --- accepted-generation history (F-12) ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiHistoryInput {
    pub project_id: String,
    #[serde(default)]
    pub doc_id: Option<String>,
    pub action: String,
    #[serde(default)]
    pub model: Option<String>,
    pub response: String,
}

/// Persist an accepted generation (called by the UI on Accept).
#[tauri::command]
pub fn ai_record_accepted(
    state: State<'_, AppState>,
    input: AiHistoryInput,
) -> CmdResult<AiHistoryEntry> {
    state.storage.record_ai_history(
        &input.project_id,
        input.doc_id.as_deref(),
        &input.action,
        input.model.as_deref(),
        &input.response,
    )
}

#[tauri::command]
pub fn list_ai_history(
    state: State<'_, AppState>,
    project_id: String,
    limit: u32,
) -> CmdResult<Vec<AiHistoryEntry>> {
    state.storage.list_ai_history(&project_id, limit)
}
