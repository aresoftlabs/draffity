//! AI commands (Épica F). BYOK key management + status (slice 1) and the
//! inline actions Continue/Expand/Rewrite/Describe with event streaming,
//! cancellation, and accepted-generation history (slice 2).
//!
//! The OpenRouter key lives in the OS keyring via `SecretStorage` — never in
//! the plain `settings` table. Streaming runs off the main thread via
//! `spawn_blocking` (the blocking HTTP client must not be driven from inside
//! the Tokio runtime); each delta is relayed to the UI as an
//! `ai:suggestion:received` event tagged with the request id.

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

// --- curated AI model list ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub context_length: u32,
    pub cost_per_1k_tokens: f64,
}

/// Curated OpenRouter model list — hardcoded defaults, not fetched dynamically.
/// Ids match OpenRouter slugs so the user can paste them from their dashboard.
fn curated_ai_models() -> Vec<AiModelInfo> {
    vec![
        AiModelInfo {
            id: "openai/gpt-4o".into(),
            name: "GPT-4o".into(),
            provider: "OpenAI".into(),
            context_length: 128_000,
            cost_per_1k_tokens: 0.0025,
        },
        AiModelInfo {
            id: "openai/gpt-4o-mini".into(),
            name: "GPT-4o Mini".into(),
            provider: "OpenAI".into(),
            context_length: 128_000,
            cost_per_1k_tokens: 0.00015,
        },
        AiModelInfo {
            id: "openai/o3-mini".into(),
            name: "o3 Mini".into(),
            provider: "OpenAI".into(),
            context_length: 200_000,
            cost_per_1k_tokens: 0.0011,
        },
        AiModelInfo {
            id: "anthropic/claude-3.5-haiku".into(),
            name: "Claude 3.5 Haiku".into(),
            provider: "Anthropic".into(),
            context_length: 200_000,
            cost_per_1k_tokens: 0.0008,
        },
        AiModelInfo {
            id: "anthropic/claude-3.5-sonnet".into(),
            name: "Claude 3.5 Sonnet".into(),
            provider: "Anthropic".into(),
            context_length: 200_000,
            cost_per_1k_tokens: 0.003,
        },
        AiModelInfo {
            id: "anthropic/claude-opus-4".into(),
            name: "Claude Opus 4".into(),
            provider: "Anthropic".into(),
            context_length: 200_000,
            cost_per_1k_tokens: 0.015,
        },
        AiModelInfo {
            id: "google/gemini-2.0-flash-001".into(),
            name: "Gemini 2.0 Flash".into(),
            provider: "Google".into(),
            context_length: 1_048_576,
            cost_per_1k_tokens: 0.0001,
        },
        AiModelInfo {
            id: "google/gemini-2.0-pro-001".into(),
            name: "Gemini 2.0 Pro".into(),
            provider: "Google".into(),
            context_length: 2_000_000,
            cost_per_1k_tokens: 0.002,
        },
        AiModelInfo {
            id: "mistral/mistral-large-2411".into(),
            name: "Mistral Large".into(),
            provider: "Mistral".into(),
            context_length: 128_000,
            cost_per_1k_tokens: 0.002,
        },
        AiModelInfo {
            id: "mistral/mistral-small-2501".into(),
            name: "Mistral Small".into(),
            provider: "Mistral".into(),
            context_length: 32_000,
            cost_per_1k_tokens: 0.0004,
        },
        AiModelInfo {
            id: "deepseek/deepseek-chat".into(),
            name: "DeepSeek V3".into(),
            provider: "DeepSeek".into(),
            context_length: 128_000,
            cost_per_1k_tokens: 0.0005,
        },
        AiModelInfo {
            id: "qwen/qwq-32b".into(),
            name: "QwQ 32B".into(),
            provider: "Qwen".into(),
            context_length: 32_000,
            cost_per_1k_tokens: 0.0002,
        },
    ]
}

#[tauri::command]
pub fn list_ai_models() -> Vec<AiModelInfo> {
    curated_ai_models()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_ai_models_returns_non_empty_curated_list() {
        let models = list_ai_models();
        assert!(!models.is_empty(), "should return at least one model");
        // Verify it contains well-known providers
        let providers: Vec<&str> = models.iter().map(|m| m.provider.as_str()).collect();
        assert!(providers.contains(&"OpenAI"));
        assert!(providers.contains(&"Anthropic"));
        assert!(providers.contains(&"Google"));
    }

    #[test]
    fn list_ai_models_every_model_has_valid_fields() {
        for m in list_ai_models() {
            assert!(!m.id.is_empty(), "model id must not be empty");
            assert!(!m.name.is_empty(), "model name must not be empty");
            assert!(!m.provider.is_empty(), "model provider must not be empty");
            assert!(m.context_length > 0, "context length must be > 0");
            assert!(m.cost_per_1k_tokens >= 0.0, "cost must be non-negative");
        }
    }

    #[test]
    fn list_ai_models_serializes_camel_case() {
        let models = list_ai_models();
        let json = serde_json::to_value(&models[0]).unwrap();
        assert!(json.get("id").is_some());
        assert!(json.get("contextLength").is_some());
        assert!(json.get("costPer1kTokens").is_some());
    }
}
