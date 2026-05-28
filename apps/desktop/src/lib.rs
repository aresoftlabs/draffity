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

use std::sync::Arc;

use tauri::Manager;

use crate::services::{
    BuiltInTemplates, FreeTier, LocalExporter, LocalStorageService, NoOpAI, NoOpASR, NoOpSync,
    ProjectManager, StorageService,
};
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

            // Initialise tracing once we know where to put logs. The guard
            // must outlive the app process — we hand it to AppState.
            let log_dir = app_data_dir.join("logs");
            let log_guard = logging::init(&log_dir);

            let db_path = app_data_dir.join("draffity.db");
            tracing::info!(path = %db_path.display(), "opening canonical database");

            let storage = Arc::new(LocalStorageService::open(&db_path)?);
            storage.migrate()?;

            let tier = Arc::new(FreeTier);
            let templates = Arc::new(BuiltInTemplates::load()?);
            let project_manager = Arc::new(ProjectManager::new(
                storage.clone(),
                tier.clone(),
                templates.clone(),
            ));

            app.manage(AppState {
                storage,
                tier,
                project_manager,
                templates,
                ai: Arc::new(NoOpAI),
                sync: Arc::new(NoOpSync),
                asr: Arc::new(NoOpASR),
                exporter: Arc::new(LocalExporter),
                _log_guard: log_guard,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // system
            commands::ping,
            commands::capability_enabled,
            commands::capability_enabled_pure,
            commands::get_setting,
            commands::set_setting,
            commands::get_writing_stats,
            // projects
            commands::create_project,
            commands::list_projects,
            commands::get_project,
            commands::get_active_project,
            commands::open_project,
            commands::archive_project,
            commands::delete_project,
            // documents
            commands::create_document,
            commands::list_documents,
            commands::get_document,
            commands::update_document,
            commands::move_document,
            commands::delete_document,
            commands::create_snapshot,
            commands::list_snapshots,
            commands::restore_snapshot,
            // templates
            commands::list_templates,
            commands::get_template,
            // export
            commands::export_project,
            commands::supported_export_formats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
