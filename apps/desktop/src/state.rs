//! Tauri-managed application state. Built in `lib.rs::run` from a
//! `ServiceBundle` (services) + `WorkerGuard` (log lifecycle), consumed by
//! IPC commands via `State<AppState>`.

use std::sync::Arc;

use tracing_appender::non_blocking::WorkerGuard;

use crate::services::{
    AIService, ASRService, BackupService, BibliographyService, CloudSyncService,
    CrashReporterService, ExportService, ImportService, MediaService, ProjectManagerService,
    ServiceBundle, StorageService, TemplatesService, TierService, UserTemplatesLoader,
};

pub struct AppState {
    pub storage: Arc<dyn StorageService>,
    pub tier: Arc<dyn TierService>,
    pub project_manager: Arc<dyn ProjectManagerService>,
    pub templates: Arc<dyn TemplatesService>,
    pub user_templates: Arc<UserTemplatesLoader>,
    #[allow(dead_code)] // wired up in Phase 1+; consumed by premium impls
    pub ai: Arc<dyn AIService>,
    #[allow(dead_code)]
    pub sync: Arc<dyn CloudSyncService>,
    #[allow(dead_code)]
    pub asr: Arc<dyn ASRService>,
    pub exporter: Arc<dyn ExportService>,
    pub importer: Arc<dyn ImportService>,
    pub bibliography: Arc<dyn BibliographyService>,
    pub backup: Arc<dyn BackupService>,
    pub media: Arc<dyn MediaService>,
    pub crash_reporter: Arc<dyn CrashReporterService>,
    /// Keeps the non-blocking log writer alive for the whole app lifecycle.
    /// Dropping this guard flushes any pending log lines.
    #[allow(dead_code)]
    pub _log_guard: WorkerGuard,
}

impl AppState {
    pub fn from_bundle(bundle: ServiceBundle, log_guard: WorkerGuard) -> Self {
        Self {
            storage: bundle.storage,
            tier: bundle.tier,
            project_manager: bundle.project_manager,
            templates: bundle.templates,
            user_templates: bundle.user_templates,
            ai: bundle.ai,
            sync: bundle.sync,
            asr: bundle.asr,
            exporter: bundle.exporter,
            importer: bundle.importer,
            bibliography: bundle.bibliography,
            backup: bundle.backup,
            media: bundle.media,
            crash_reporter: bundle.crash_reporter,
            _log_guard: log_guard,
        }
    }
}
