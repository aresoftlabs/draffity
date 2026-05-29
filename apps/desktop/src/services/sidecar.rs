//! Sidecar process helper (E-02). Thin wrapper over `tauri-plugin-shell`
//! (already wired in `lib.rs`) for running bundled native binaries — the
//! whisper.cpp and Piper executables that Épica H downloads and ships.
//!
//! **What Épica H still must add** before this resolves a real binary at
//! runtime:
//!   1. Register each binary in `tauri.conf.json` → `bundle.externalBin`
//!      (omitted today: pointing at non-existent files breaks `tauri build`).
//!   2. Grant `shell:allow-execute` (scoped to the sidecar names) in a
//!      capability file under `apps/desktop/capabilities/`.
//!
//! Until then this errors cleanly at runtime; it compiles now so the AI/voice
//! command layer can be built against a stable signature.

use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

use crate::error::{AppError, AppResult};

/// Run a bundled sidecar to completion and return its stdout as a string.
/// Non-zero exit is mapped to an error carrying stderr. `args` are passed
/// verbatim.
#[allow(dead_code)] // consumed by WhisperLocalASR / PiperTTSService in Épica H
pub async fn run_sidecar(app: &AppHandle, name: &str, args: &[&str]) -> AppResult<String> {
    let command = app
        .shell()
        .sidecar(name)
        .map_err(|e| AppError::Unexpected(format!("sidecar {name} not available: {e}")))?;

    let output = command
        .args(args)
        .output()
        .await
        .map_err(|e| AppError::Unexpected(format!("sidecar {name} failed to run: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Unexpected(format!(
            "sidecar {name} exited with {:?}: {stderr}",
            output.status.code()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
