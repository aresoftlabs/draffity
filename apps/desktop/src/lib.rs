//! Draffity desktop entry point.

mod capabilities;
mod commands;
pub mod domain;
mod error;
mod events;
mod logging;
pub mod services;
mod state;

pub use error::{AppError, AppResult};

use tauri::Manager;

use crate::capabilities::Tier;
use crate::services::ServiceFactory;
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

            // Log guard must outlive the app — hand it to AppState.
            let log_guard = logging::init(&app_data_dir.join("logs"));
            let bundle = ServiceFactory::build(Tier::Free, &app_data_dir)?;
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
            // Restore a previously-activated premium license from the OS
            // keyring and hot-swap the tier (E-07). A missing/invalid key
            // simply leaves the app on Free — never blocks startup.
            if let Ok(Some(key)) = bundle
                .secrets
                .get_secret(commands::license::LICENSE_SECRET_KEY)
            {
                match bundle.license_validator.validate(&key) {
                    Ok(claims) => bundle.tier.set_tier(claims.tier),
                    Err(e) => tracing::warn!(error = %e, "stored license invalid; staying on free"),
                }
            }
            app.manage(AppState::from_bundle(bundle, log_guard));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // system
            commands::ping,
            commands::capability_enabled,
            commands::get_setting,
            commands::set_setting,
            commands::get_writing_stats,
            commands::get_recent_daily_writing,
            commands::get_daily_goal,
            commands::set_daily_goal,
            commands::get_crash_reporting_status,
            commands::set_crash_reporting_enabled,
            // premium / license
            commands::get_premium_status,
            commands::activate_premium,
            commands::deactivate_premium,
            // ai (byok)
            commands::get_ai_status,
            commands::set_openrouter_key,
            commands::clear_openrouter_key,
            commands::ai_run_action,
            commands::ai_cancel,
            commands::ai_record_accepted,
            commands::list_ai_history,
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
            commands::list_voice_models,
            commands::download_voice_model,
            commands::delete_voice_model,
            commands::import_voice_binary,
            commands::transcribe_audio,
            commands::list_voice_voices,
            commands::import_piper_binary,
            commands::download_voice_voice,
            commands::synthesize_speech,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
