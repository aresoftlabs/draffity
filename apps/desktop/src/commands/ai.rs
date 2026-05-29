//! AI commands (Épica F). Slice 1 covers BYOK key management + status; the
//! inline actions (Continue/Expand/Rewrite/Describe) land in slice 2.
//!
//! The OpenRouter key lives in the OS keyring via `SecretStorage` — never in
//! the plain `settings` table.

use tauri::State;

use crate::error::AppError;
use crate::services::OPENROUTER_KEY;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    /// AI is usable right now: premium tier active AND a key is stored.
    pub available: bool,
    /// A key is stored (independent of tier) — lets the UI show "key saved"
    /// even before premium is active.
    pub has_key: bool,
}

fn status(state: &AppState) -> AiStatus {
    let has_key = state
        .secrets
        .get_secret(OPENROUTER_KEY)
        .ok()
        .flatten()
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false);
    AiStatus {
        available: state.ai.available(),
        has_key,
    }
}

#[tauri::command]
pub fn get_ai_status(state: State<'_, AppState>) -> AiStatus {
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
    Ok(status(&state))
}

#[tauri::command]
pub fn clear_openrouter_key(state: State<'_, AppState>) -> CmdResult<AiStatus> {
    state.secrets.delete_secret(OPENROUTER_KEY)?;
    Ok(status(&state))
}
