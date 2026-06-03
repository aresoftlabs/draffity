//! Composes the full set of services for a given tier.
//!
//! Single point of wiring for `LocalStorageService`, `FreeTier`, `LocalExporter`
//! and the `NoOp*` stubs. Adding a premium implementation means adding a match
//! arm here — never touching `lib.rs::run` or any service module.
//!
//! Logging is **not** initialised here: the caller owns the log lifecycle
//! because the `WorkerGuard` must outlive the whole app and the factory is
//! also useful in tests where logging is irrelevant.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::capabilities::Tier;
use crate::error::AppResult;
use crate::services::{
    AIService, AIValidatorService, ASRService, BackupService, BibliographyService,
    BuiltInTemplates, ByokAIService, CloudSyncService, CrashReporterService,
    DisabledLicenseValidator, Ed25519Validator, ExportService, ImportService, KeyringSecretStorage,
    LayeredTemplatesService, LexicalProjectMemory, LicenseValidator, LocalBackupService,
    LocalBibliographyService, LocalExporter, LocalFileCrashReporter, LocalImporter,
    LocalMediaService, LocalProjectManager, LocalStorageService, MediaService, MutableTier,
    NoOpCrashReporter, NoOpSync, OpenRouterValidators, PiperTTSService, ProjectManagerService,
    ProjectMemoryService, SecretStorage, StorageService, TTSService, TemplatesService, TierService,
    UserTemplatesLoader, WhisperLocalASR,
};

/// All services needed by the app, fully wired. Caller composes `AppState`
/// by adding the log guard (whose lifetime it owns).
pub struct ServiceBundle {
    pub storage: Arc<dyn StorageService>,
    pub tier: Arc<dyn TierService>,
    pub project_manager: Arc<dyn ProjectManagerService>,
    pub templates: Arc<dyn TemplatesService>,
    pub user_templates: Arc<UserTemplatesLoader>,
    pub ai: Arc<dyn AIService>,
    pub memory: Arc<dyn ProjectMemoryService>,
    pub validators: Arc<dyn AIValidatorService>,
    pub sync: Arc<dyn CloudSyncService>,
    pub asr: Arc<dyn ASRService>,
    pub tts: Arc<dyn TTSService>,
    pub exporter: Arc<dyn ExportService>,
    pub importer: Arc<dyn ImportService>,
    pub bibliography: Arc<dyn BibliographyService>,
    pub backup: Arc<dyn BackupService>,
    pub media: Arc<dyn MediaService>,
    pub crash_reporter: Arc<dyn CrashReporterService>,
    pub secrets: Arc<dyn SecretStorage>,
    pub license_validator: Arc<dyn LicenseValidator>,
    /// App data dir — voice commands resolve binary/model paths from it.
    pub app_data_dir: PathBuf,
}

/// Builds `ServiceBundle` from a tier + storage location. Idempotent w.r.t.
/// migrations (running twice on the same DB is a no-op past v1).
pub struct ServiceFactory;

impl ServiceFactory {
    pub fn build(tier: Tier, app_data_dir: &Path) -> AppResult<ServiceBundle> {
        let storage = Self::build_storage(app_data_dir)?;
        let tier_service = Self::build_tier(tier)?;
        let user_templates = Arc::new(UserTemplatesLoader::new(
            app_data_dir.join("templates").join("user"),
        ));
        let templates: Arc<dyn TemplatesService> = Self::build_templates(user_templates.clone())?;
        let project_manager: Arc<dyn ProjectManagerService> = Arc::new(LocalProjectManager::new(
            storage.clone(),
            tier_service.clone(),
            templates.clone(),
        ));

        let media: Arc<dyn MediaService> =
            Arc::new(LocalMediaService::new(storage.clone(), app_data_dir));
        let crash_reporter: Arc<dyn CrashReporterService> =
            Self::build_crash_reporter(app_data_dir);

        // Secrets feed the BYOK AI service; both share the keyring. AI is gated
        // at call time by the stored key (see ByokAIService).
        let secrets: Arc<dyn SecretStorage> = Arc::new(KeyringSecretStorage::new());
        let ai: Arc<dyn AIService> = Arc::new(ByokAIService::new(secrets.clone()));
        let memory: Arc<dyn ProjectMemoryService> =
            Arc::new(LexicalProjectMemory::new(storage.clone()));
        let validators: Arc<dyn AIValidatorService> =
            Arc::new(OpenRouterValidators::new(ai.clone()));
        // Local Whisper ASR (H). Gated by tier + installed binary/model; with
        // nothing installed `available()` is false, like the old NoOpASR.
        let asr: Arc<dyn ASRService> = Arc::new(WhisperLocalASR::new(
            app_data_dir.to_path_buf(),
            tier_service.clone(),
        ));
        // Local Piper TTS (H). Same gating: tier + installed binary/voice.
        let tts: Arc<dyn TTSService> = Arc::new(PiperTTSService::new(
            app_data_dir.to_path_buf(),
            tier_service.clone(),
        ));

        Ok(ServiceBundle {
            storage,
            tier: tier_service,
            project_manager,
            templates,
            user_templates,
            ai,
            memory,
            validators,
            sync: Self::build_sync(tier),
            asr,
            tts,
            exporter: Arc::new(LocalExporter),
            importer: Arc::new(LocalImporter),
            bibliography: Arc::new(LocalBibliographyService),
            backup: Self::build_backup(app_data_dir),
            media,
            crash_reporter,
            secrets,
            license_validator: Self::build_license_validator(),
            app_data_dir: app_data_dir.to_path_buf(),
        })
    }

    /// Wire the license validator from the build-time `DRAFFITY_LICENSE_PUBKEY`
    /// (base64url of the 32-byte Ed25519 public key). When absent — the default
    /// for OSS builds — premium cannot be activated. Mirrors the crash-reporter
    /// env gating above. A malformed key fails closed (disabled), logging why.
    fn build_license_validator() -> Arc<dyn LicenseValidator> {
        match std::env::var("DRAFFITY_LICENSE_PUBKEY") {
            Ok(b64) if !b64.trim().is_empty() => match Ed25519Validator::from_base64(&b64) {
                Ok(v) => Arc::new(v),
                Err(e) => {
                    tracing::warn!(error = %e, "invalid DRAFFITY_LICENSE_PUBKEY; licensing disabled");
                    Arc::new(DisabledLicenseValidator)
                }
            },
            _ => Arc::new(DisabledLicenseValidator),
        }
    }

    /// Pick a crash reporter impl. When `DRAFFITY_SENTRY_DSN` is empty
    /// or absent (the default for OSS builds) we wire a local-file
    /// stub: it queues reports under `<app_data>/crash-reports/` so the
    /// pipeline is exercised end-to-end without a remote dependency.
    /// A real Sentry-backed impl plugs in here.
    fn build_crash_reporter(app_data_dir: &Path) -> Arc<dyn CrashReporterService> {
        match std::env::var("DRAFFITY_SENTRY_DSN") {
            Ok(dsn) if !dsn.trim().is_empty() => {
                // TODO: swap in a real Sentry uploader when infra lands.
                Arc::new(LocalFileCrashReporter::new(
                    app_data_dir.join("crash-reports"),
                ))
            }
            _ => Arc::new(NoOpCrashReporter),
        }
    }

    fn build_backup(app_data_dir: &Path) -> Arc<dyn BackupService> {
        let db_path = app_data_dir.join("draffity.db");
        let backup_dir = app_data_dir.join("backups");
        Arc::new(LocalBackupService::new(db_path, backup_dir))
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

    fn build_storage(app_data_dir: &Path) -> AppResult<Arc<dyn StorageService>> {
        let db_path = app_data_dir.join("draffity.db");
        tracing::info!(path = %db_path.display(), "opening canonical database");
        let storage = LocalStorageService::open(&db_path)?;
        storage.migrate()?;
        Ok(Arc::new(storage))
    }

    /// Always wraps the tier in a `MutableTier` so premium activation (E-07)
    /// can flip capabilities at runtime without a restart — the same
    /// `Arc<dyn TierService>` is shared by the IPC layer and the project
    /// manager, so one `set_tier` is seen everywhere live.
    fn build_tier(tier: Tier) -> AppResult<Arc<dyn TierService>> {
        Ok(Arc::new(MutableTier::new(tier)))
    }

    // `ai` is the BYOK `ByokAIService` (F-01), gated at call time. The voice
    // builders below still return NoOp stubs until `WhisperLocalASR` /
    // `PiperTTSService` land in Épica H; activating premium flips their
    // capability *gates* but the services stay stubs until then. See
    // docs/PREMIUM-INTEGRATION.md.

    fn build_sync(_tier: Tier) -> Arc<dyn CloudSyncService> {
        Arc::new(NoOpSync)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_free_tier_in_tempdir() {
        let dir = tempdir();
        let bundle = ServiceFactory::build(Tier::Free, &dir).expect("build free tier bundle");
        // The wired services answer correctly to a smoke probe.
        assert!(!bundle.tier.is_enabled("ai_features"));
        assert!(bundle.templates.get("novela-tres-actos").is_some());
        // Migrations applied: listing projects on a fresh DB returns empty.
        assert_eq!(bundle.storage.list_projects().expect("list").len(), 0);
    }

    #[test]
    fn build_premium_tier_grants_capabilities() {
        let dir = tempdir();
        let bundle = ServiceFactory::build(Tier::Premium, &dir).expect("build premium bundle");
        // Premium flips capability gates on…
        assert!(bundle.tier.is_enabled("ai_inline"));
        assert!(bundle.tier.is_enabled("voice_dictation"));
        // …voice impls don't exist yet (land in Épica H), so they stay NoOp.
        // (We don't probe `ai.available()` here — it reads the OS keyring, so
        // it isn't hermetic in tests; ByokAIService gating is unit-tested in
        // its own module.)
        assert!(!bundle.asr.available());
        assert!(!bundle.tts.available());
    }

    #[test]
    fn tier_hot_swaps_free_to_premium_at_runtime() {
        let dir = tempdir();
        let bundle = ServiceFactory::build(Tier::Free, &dir).expect("build free bundle");
        assert!(!bundle.tier.is_enabled("ai_inline"));
        bundle.tier.set_tier(Tier::Premium);
        assert!(bundle.tier.is_enabled("ai_inline"), "hot-swap to premium");
    }

    #[test]
    fn build_is_idempotent_on_same_dir() {
        let dir = tempdir();
        ServiceFactory::build(Tier::Free, &dir).expect("first build");
        // Re-running on the same dir reuses the DB and migrations no-op.
        ServiceFactory::build(Tier::Free, &dir).expect("second build");
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
