// Draffity — aplicación de escritura desktop multi-formato.
// Copyright (C) 2026 Aresoft SpA
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version. This program is distributed WITHOUT ANY WARRANTY; without even the
// implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See
// the GNU General Public License for more details. You should have received a
// copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

//! Draffity desktop entry point.

mod commands;
pub mod domain;
mod error;
mod events;
mod logging;
pub mod services;
mod state;

pub use error::{AppError, AppResult};

use tauri::Manager;

use crate::services::{DraffityHome, ServiceFactory};
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_data_dir)?;

            // Build the canonical DraffityHome (defaults to ~/.draffity/).
            let home = DraffityHome::new();
            home.ensure_dirs()?;

            // One-time migration from old Tauri app_data_dir to ~/.draffity/.
            if let Err(e) = home.run_migration(&app_data_dir) {
                tracing::warn!(error = %e, "migration from old app_data_dir failed");
            }

            // Log guard must outlive the app — hand it to AppState.
            let log_guard = logging::init(&home.logs_dir());
            let bundle = ServiceFactory::build(&home)?;
            // Daily backup + prune. Idempotent — calling again today is a
            // no-op. Failures are logged, not fatal: a missed backup must
            // never prevent the app from launching.
            if let Err(e) = bundle.backup.run_daily_maintenance() {
                tracing::warn!(error = %e, "backup maintenance failed at startup");
            }
            // Restore the user's saved crash-reporting opt-in. Storage
            // failures are logged but don't block startup — the user can
            // always re-toggle from Settings.
            if let Ok(Some(value)) = bundle.storage.get_setting("crash_reporting.enabled") {
                let enabled = matches!(value.as_str(), "1" | "true" | "on");
                bundle.crash_reporter.set_enabled(enabled);
            }
            app.manage(AppState::from_bundle(bundle, home, log_guard));

            // Grant the WebView2 microphone permission so `getUserMedia` works
            // for dictation/ASR. wry only auto-grants clipboard-read, leaving
            // the microphone denied by default, so we register our own handler.
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.with_webview(grant_microphone_permission);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // system
            commands::ping,
            commands::get_setting,
            commands::set_setting,
            commands::get_writing_stats,
            commands::get_recent_daily_writing,
            commands::get_daily_goal,
            commands::set_daily_goal,
            commands::get_crash_reporting_status,
            commands::set_crash_reporting_enabled,
            // ai (byok)
            commands::get_ai_status,
            commands::set_openrouter_key,
            commands::clear_openrouter_key,
            commands::ai_run_action,
            commands::ai_cancel,
            commands::ai_record_accepted,
            commands::list_ai_history,
            commands::list_ai_models,
            // ai validators (épica g)
            commands::check_codex_coverage,
            commands::run_validators,
            commands::list_validations,
            // collections (épica i)
            commands::create_collection,
            commands::list_collections,
            commands::rename_collection,
            commands::set_collection_query,
            commands::delete_collection,
            commands::set_collection_members,
            commands::resolve_collection,
            // labels (épica i)
            commands::create_label,
            commands::list_labels,
            commands::update_label,
            commands::delete_label,
            commands::set_document_labels,
            // custom metadata fields (épica i)
            commands::create_custom_field,
            commands::list_custom_fields,
            commands::update_custom_field,
            commands::delete_custom_field,
            commands::set_document_metadata,
            // voice (épica h)
            commands::get_voice_status,
            commands::get_accel_status,
            commands::list_voice_models,
            commands::download_voice_model,
            commands::delete_voice_model,
            commands::import_voice_binary,
            commands::download_voice_binary,
            commands::transcribe_audio,
            commands::list_voice_voices,
            commands::import_piper_binary,
            commands::download_voice_voice,
            commands::delete_voice_voice,
            commands::synthesize_speech,
            commands::test_synthesize,
            commands::get_disk_usage,
            commands::list_available_models,
            commands::save_voice_note,
            commands::list_voice_notes,
            commands::delete_voice_note,
            // projects
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::get_active_project,
            commands::open_project,
            commands::archive_project,
            commands::delete_project,
            commands::set_project_goal,
            commands::set_project_deadline,
            // documents
            commands::create_document,
            commands::list_documents,
            commands::get_document,
            commands::update_document,
            commands::move_document,
            commands::reorder_documents,
            commands::set_document_status,
            commands::set_document_tags,
            commands::list_project_tags,
            commands::set_document_goal,
            commands::set_document_synopsis,
            commands::set_document_research,
            commands::set_document_matter,
            commands::delete_document,
            commands::create_snapshot,
            commands::list_snapshots,
            commands::restore_snapshot,
            // templates
            commands::list_templates,
            commands::get_template,
            commands::save_project_as_template,
            commands::delete_user_template,
            // search
            commands::search_documents,
            // export
            commands::export_project,
            commands::supported_export_formats,
            commands::get_export_config,
            commands::set_export_config,
            // import
            commands::import_project,
            commands::supported_import_formats,
            // bibliography
            commands::import_bibliography,
            commands::list_citations,
            commands::list_citation_keys,
            commands::delete_citation,
            // backup
            commands::list_backups,
            commands::create_manual_backup,
            commands::restore_backup,
            commands::prune_backups,
            // codex
            commands::create_codex_entry,
            commands::list_codex_entries,
            commands::get_codex_entry,
            commands::update_codex_entry,
            commands::delete_codex_entry,
            commands::search_codex_entries,
            // media
            commands::upload_media,
            commands::read_media_bytes,
            commands::get_media_asset,
            commands::list_project_media,
            commands::delete_media,
            // resources
            commands::get_resources_path,
            commands::set_resources_path,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                if let Some(state) = app_handle.try_state::<crate::state::AppState>() {
                    state.whisper_server.shutdown();
                }
            }
        });
}

/// Register a WebView2 `PermissionRequested` handler that allows the microphone
/// kind. Without this, embedded WebView2 denies `getUserMedia({ audio: true })`
/// and dictation/ASR recording never starts. Mirrors what wry does for the
/// clipboard, applied to the microphone instead.
#[cfg(target_os = "windows")]
fn grant_microphone_permission(webview: tauri::webview::PlatformWebview) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_PERMISSION_KIND, COREWEBVIEW2_PERMISSION_KIND_MICROPHONE,
        COREWEBVIEW2_PERMISSION_STATE_ALLOW,
    };
    use webview2_com::PermissionRequestedEventHandler;

    unsafe {
        let core = match webview.controller().CoreWebView2() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "could not access CoreWebView2 for mic permission");
                return;
            }
        };
        let mut token = 0_i64;
        let result = core.add_PermissionRequested(
            &PermissionRequestedEventHandler::create(Box::new(|_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
                args.PermissionKind(&mut kind)?;
                if kind == COREWEBVIEW2_PERMISSION_KIND_MICROPHONE {
                    args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?;
                }
                Ok(())
            })),
            &mut token,
        );
        if let Err(e) = result {
            tracing::warn!(error = %e, "failed to register WebView2 microphone permission handler");
        }
    }
}
