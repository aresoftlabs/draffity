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
            app.manage(AppState::from_bundle(bundle, log_guard));

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
            commands::reorder_documents,
            commands::set_document_status,
            commands::set_document_tags,
            commands::list_project_tags,
            commands::delete_document,
            commands::create_snapshot,
            commands::list_snapshots,
            commands::restore_snapshot,
            // templates
            commands::list_templates,
            commands::get_template,
            // search
            commands::search_documents,
            // export
            commands::export_project,
            commands::supported_export_formats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
