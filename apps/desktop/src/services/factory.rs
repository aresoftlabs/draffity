//! Composes the full set of services.
//!
//! Single point of wiring for `LocalStorageService`, `LocalExporter`
//! and the `NoOp*` stubs — never touching `lib.rs::run` or any service module.
//!
//! Logging is **not** initialised here: the caller owns the log lifecycle
//! because the `WorkerGuard` must outlive the whole app and the factory is
//! also useful in tests where logging is irrelevant.

use std::path::PathBuf;
use std::sync::Arc;

use crate::error::AppResult;
use crate::services::{
    AIService, AIValidatorService, ASRService, BackupService, BibliographyService,
    BuiltInTemplates, ByokAIService, CrashReporterService, DraffityHome, ExportService,
    ImportService, KeyringSecretStorage, LayeredTemplatesService, LexicalProjectMemory,
    LocalBackupService, LocalBibliographyService, LocalExporter, LocalFileCrashReporter,
    LocalImporter, LocalMediaService, LocalProjectManager, LocalStorageService, MediaService,
    NoOpCrashReporter, OpenRouterValidators, PiperTTSService, ProjectManagerService,
    ProjectMemoryService, SecretStorage, StorageService, TTSService, TemplatesService,
    UserTemplatesLoader, WhisperLocalASR,
};

/// All services needed by the app, fully wired. Caller composes `AppState`
/// by adding the log guard (whose lifetime it owns).
pub struct ServiceBundle {
    pub storage: Arc<dyn StorageService>,
    pub project_manager: Arc<dyn ProjectManagerService>,
    pub templates: Arc<dyn TemplatesService>,
    pub user_templates: Arc<UserTemplatesLoader>,
    pub ai: Arc<dyn AIService>,
    pub memory: Arc<dyn ProjectMemoryService>,
    pub validators: Arc<dyn AIValidatorService>,
    pub asr: Arc<dyn ASRService>,
    pub tts: Arc<dyn TTSService>,
    pub exporter: Arc<dyn ExportService>,
    pub importer: Arc<dyn ImportService>,
    pub bibliography: Arc<dyn BibliographyService>,
    pub backup: Arc<dyn BackupService>,
    pub media: Arc<dyn MediaService>,
    pub crash_reporter: Arc<dyn CrashReporterService>,
    pub secrets: Arc<dyn SecretStorage>,
}

/// Builds `ServiceBundle` from a storage location. Idempotent w.r.t.
/// migrations (running twice on the same DB is a no-op past v1).
pub struct ServiceFactory;

impl ServiceFactory {
    pub fn build(resources: &DraffityHome) -> AppResult<ServiceBundle> {
        let storage = Self::build_storage(resources.db_path())?;
        let user_templates = Arc::new(UserTemplatesLoader::new(resources.templates_dir()));
        let templates: Arc<dyn TemplatesService> = Self::build_templates(user_templates.clone())?;
        let project_manager: Arc<dyn ProjectManagerService> =
            Arc::new(LocalProjectManager::new(storage.clone(), templates.clone()));

        let media: Arc<dyn MediaService> =
            Arc::new(LocalMediaService::new(storage.clone(), resources));
        let crash_reporter: Arc<dyn CrashReporterService> = Self::build_crash_reporter(resources);

        // Secrets feed the BYOK AI service; both share the keyring. AI is gated
        // at call time by the stored key (see ByokAIService).
        let secrets: Arc<dyn SecretStorage> = Arc::new(KeyringSecretStorage::new());
        let ai: Arc<dyn AIService> = Arc::new(ByokAIService::new(secrets.clone()));
        let memory: Arc<dyn ProjectMemoryService> =
            Arc::new(LexicalProjectMemory::new(storage.clone()));
        let validators: Arc<dyn AIValidatorService> =
            Arc::new(OpenRouterValidators::new(ai.clone()));
        // Local Whisper ASR. Available when the binary + a model are installed.
        let asr: Arc<dyn ASRService> = Arc::new(WhisperLocalASR::new(resources));
        // Local Piper TTS. Available when the binary + a voice are installed.
        let tts: Arc<dyn TTSService> = Arc::new(PiperTTSService::new(resources));

        Ok(ServiceBundle {
            storage,
            project_manager,
            templates,
            user_templates,
            ai,
            memory,
            validators,
            asr,
            tts,
            exporter: Arc::new(LocalExporter),
            importer: Arc::new(LocalImporter),
            bibliography: Arc::new(LocalBibliographyService),
            backup: Self::build_backup(resources),
            media,
            crash_reporter,
            secrets,
        })
    }

    /// Pick a crash reporter impl. When `DRAFFITY_SENTRY_DSN` is empty
    /// or absent (the default for OSS builds) we wire a local-file
    /// stub: it queues reports under `<root>/crash-reports/` so the
    /// pipeline is exercised end-to-end without a remote dependency.
    /// A real Sentry-backed impl plugs in here.
    fn build_crash_reporter(resources: &DraffityHome) -> Arc<dyn CrashReporterService> {
        match std::env::var("DRAFFITY_SENTRY_DSN") {
            Ok(dsn) if !dsn.trim().is_empty() => {
                // TODO: swap in a real Sentry uploader when infra lands.
                Arc::new(LocalFileCrashReporter::new(resources.crash_reports_dir()))
            }
            _ => Arc::new(NoOpCrashReporter),
        }
    }

    fn build_backup(resources: &DraffityHome) -> Arc<dyn BackupService> {
        Arc::new(LocalBackupService::new(
            resources.db_path(),
            resources.backups_dir(),
        ))
    }

    fn build_templates(user: Arc<UserTemplatesLoader>) -> AppResult<Arc<dyn TemplatesService>> {
        let built_in = BuiltInTemplates::load()?;
        // `LayeredTemplatesService` doesn't own the loader — it borrows a
        // clone of the same `Arc` we hand to the IPC layer. Both views stay
        // consistent because the loader re-reads disk on every call.
        Ok(Arc::new(LayeredTemplatesService::new(
            built_in,
            (*user).clone(),
        )))
    }

    fn build_storage(db_path: PathBuf) -> AppResult<Arc<dyn StorageService>> {
        tracing::info!(path = %db_path.display(), "opening canonical database");
        let storage = LocalStorageService::open(&db_path)?;
        storage.migrate()?;
        Ok(Arc::new(storage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_in_tempdir_wires_services() {
        let dir = tempdir();
        let home = DraffityHome::with_root(dir.join("draffity-home"));
        let bundle = ServiceFactory::build(&home).expect("build bundle");
        assert!(bundle.templates.get("novela-tres-actos").is_some());
        // Migrations applied: listing projects on a fresh DB returns empty.
        assert_eq!(bundle.storage.list_projects().expect("list").len(), 0);
        // Voice/AI are unavailable without installed resources (no tier gate).
        assert!(!bundle.asr.available());
        assert!(!bundle.tts.available());
        // ServiceBundle no longer carries app_data_dir
    }

    #[test]
    fn build_is_idempotent_on_same_dir() {
        let dir = tempdir();
        let home = DraffityHome::with_root(dir.join("draffity-home"));
        ServiceFactory::build(&home).expect("first build");
        ServiceFactory::build(&home).expect("second build");
    }

    fn tempdir() -> std::path::PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let mut p = std::env::temp_dir();
        p.push(format!("draffity-factory-test-{nanos:x}"));
        std::fs::create_dir_all(&p).expect("create tempdir");
        p
    }
}
