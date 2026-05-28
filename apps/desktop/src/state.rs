//! Tauri-managed application state. Built in `lib.rs::run` and consumed by
//! IPC commands via `State<AppState>`.

use std::sync::Arc;

use tracing_appender::non_blocking::WorkerGuard;

use crate::services::{
    AIService, ASRService, CloudSyncService, ExportService, ProjectManager, StorageService,
    TemplatesService, TierService,
};

pub struct AppState {
    pub storage: Arc<dyn StorageService>,
    pub tier: Arc<dyn TierService>,
    pub project_manager: Arc<ProjectManager>,
    pub templates: Arc<dyn TemplatesService>,
    #[allow(dead_code)] // wired up in Phase 1+; consumed by premium impls
    pub ai: Arc<dyn AIService>,
    #[allow(dead_code)]
    pub sync: Arc<dyn CloudSyncService>,
    #[allow(dead_code)]
    pub asr: Arc<dyn ASRService>,
    pub exporter: Arc<dyn ExportService>,
    /// Keeps the non-blocking log writer alive for the whole app lifecycle.
    /// Dropping this guard flushes any pending log lines.
    #[allow(dead_code)]
    pub _log_guard: WorkerGuard,
}
