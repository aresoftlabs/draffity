//! Services layer. Each external concern is a trait so premium can swap in
//! richer implementations without touching the core.

pub mod ai;
pub mod asr;
pub mod backup;
pub mod bibliography;
pub mod exporter;
pub mod factory;
pub mod project_manager;
mod retention_policy;
pub mod storage;
pub mod sync;
pub mod templates;
pub mod tier;
pub mod user_templates;

pub use ai::{AIService, NoOpAI};
pub use asr::{ASRService, NoOpASR};
pub use backup::{BackupKind, BackupRecord, BackupService, LocalBackupService, NoOpBackup};
pub use bibliography::{BibliographyService, LocalBibliographyService, ParseSummary};
pub use exporter::{
    export_config_settings_key, ExportConfig, ExportFormat, ExportService, LocalExporter, Margins,
    PageSize, SceneSeparator,
};
pub use factory::{ServiceBundle, ServiceFactory};
pub use project_manager::{LocalProjectManager, ProjectManagerService};
pub use storage::{CitationUpsert, LocalStorageService, StorageService};
pub use sync::{CloudSyncService, NoOpSync};
pub use templates::{BuiltInTemplates, LayeredTemplatesService, TemplatesService};
pub use tier::{FreeTier, TierService};
pub use user_templates::{template_from_project, UserTemplatesLoader};
