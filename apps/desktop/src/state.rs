//! Tauri-managed application state. Built in `lib.rs::run` from a
//! `ServiceBundle` (services) + `WorkerGuard` (log lifecycle), consumed by
//! IPC commands via `State<AppState>`.

use std::sync::Arc;

use crate::logging::LogGuards;
use crate::services::{
    AIService, ASRService, BackupService, BibliographyService, CloudSyncService,
    CrashReporterService, ExportService, ImportService, LicenseValidator, MediaService,
    ProjectManagerService, SecretStorage, ServiceBundle, StorageService, TTSService,
    TemplatesService, TierService, UserTemplatesLoader,
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
    #[allow(dead_code)] // consumed by PiperTTSService wiring in Épica H
    pub tts: Arc<dyn TTSService>,
    pub exporter: Arc<dyn ExportService>,
    pub importer: Arc<dyn ImportService>,
    pub bibliography: Arc<dyn BibliographyService>,
    pub backup: Arc<dyn BackupService>,
    pub media: Arc<dyn MediaService>,
    pub crash_reporter: Arc<dyn CrashReporterService>,
    /// OS-keyring storage for BYOK keys + license (E-01). Never the plain
    /// `settings` table.
    pub secrets: Arc<dyn SecretStorage>,
    /// Offline Ed25519 license validator (E-07).
    pub license_validator: Arc<dyn LicenseValidator>,
    /// Keeps the non-blocking log writers alive for the whole app lifecycle.
    /// Dropping these guards flushes any pending log lines.
    #[allow(dead_code)]
    pub _log_guards: LogGuards,
}

impl AppState {
    pub fn from_bundle(bundle: ServiceBundle, log_guards: LogGuards) -> Self {
        Self {
            storage: bundle.storage,
            tier: bundle.tier,
            project_manager: bundle.project_manager,
            templates: bundle.templates,
            user_templates: bundle.user_templates,
            ai: bundle.ai,
            sync: bundle.sync,
            asr: bundle.asr,
            tts: bundle.tts,
            exporter: bundle.exporter,
            importer: bundle.importer,
            bibliography: bundle.bibliography,
            backup: bundle.backup,
            media: bundle.media,
            crash_reporter: bundle.crash_reporter,
            secrets: bundle.secrets,
            license_validator: bundle.license_validator,
            _log_guards: log_guards,
        }
    }
}
