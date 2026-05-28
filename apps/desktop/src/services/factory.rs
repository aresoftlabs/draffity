//! Composes the full set of services for a given tier.
//!
//! Single point of wiring for `LocalStorageService`, `FreeTier`, `LocalExporter`
//! and the `NoOp*` stubs. Adding a premium implementation means adding a match
//! arm here — never touching `lib.rs::run` or any service module.
//!
//! Logging is **not** initialised here: the caller owns the log lifecycle
//! because the `WorkerGuard` must outlive the whole app and the factory is
//! also useful in tests where logging is irrelevant.

use std::path::Path;
use std::sync::Arc;

use crate::capabilities::Tier;
use crate::error::{AppError, AppResult};
use crate::services::{
    AIService, ASRService, BackupService, BibliographyService, BuiltInTemplates, CloudSyncService,
    ExportService, FreeTier, LayeredTemplatesService, LocalBackupService, LocalBibliographyService,
    LocalExporter, LocalStorageService, NoOpAI, NoOpASR, NoOpSync, ProjectManager, StorageService,
    TemplatesService, TierService, UserTemplatesLoader,
};

/// All services needed by the app, fully wired. Caller composes `AppState`
/// by adding the log guard (whose lifetime it owns).
pub struct ServiceBundle {
    pub storage: Arc<dyn StorageService>,
    pub tier: Arc<dyn TierService>,
    pub project_manager: Arc<ProjectManager>,
    pub templates: Arc<dyn TemplatesService>,
    pub user_templates: Arc<UserTemplatesLoader>,
    pub ai: Arc<dyn AIService>,
    pub sync: Arc<dyn CloudSyncService>,
    pub asr: Arc<dyn ASRService>,
    pub exporter: Arc<dyn ExportService>,
    pub bibliography: Arc<dyn BibliographyService>,
    pub backup: Arc<dyn BackupService>,
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
        let project_manager = Arc::new(ProjectManager::new(
            storage.clone(),
            tier_service.clone(),
            templates.clone(),
        ));

        Ok(ServiceBundle {
            storage,
            tier: tier_service,
            project_manager,
            templates,
            user_templates,
            ai: Self::build_ai(tier),
            sync: Self::build_sync(tier),
            asr: Self::build_asr(tier),
            exporter: Arc::new(LocalExporter),
            bibliography: Arc::new(LocalBibliographyService),
            backup: Self::build_backup(app_data_dir),
        })
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

    fn build_tier(tier: Tier) -> AppResult<Arc<dyn TierService>> {
        match tier {
            Tier::Free => Ok(Arc::new(FreeTier)),
            Tier::Premium => Err(AppError::Unsupported(
                "premium tier service not yet implemented".into(),
            )),
        }
    }

    // Each builder below returns the free impl today and will gain a premium
    // match arm without touching callers. See docs/PREMIUM-INTEGRATION.md.

    fn build_ai(_tier: Tier) -> Arc<dyn AIService> {
        Arc::new(NoOpAI)
    }

    fn build_sync(_tier: Tier) -> Arc<dyn CloudSyncService> {
        Arc::new(NoOpSync)
    }

    fn build_asr(_tier: Tier) -> Arc<dyn ASRService> {
        Arc::new(NoOpASR)
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
    fn build_premium_tier_fails_until_wired() {
        let dir = tempdir();
        match ServiceFactory::build(Tier::Premium, &dir) {
            Err(AppError::Unsupported(_)) => {}
            Ok(_) => panic!("premium tier should not be wired yet"),
            Err(other) => panic!("unexpected error variant: {other}"),
        }
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
