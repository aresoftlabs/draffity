//! Services layer. Each external concern is a trait so premium can swap in
//! richer implementations without touching the core.

pub mod ai;
pub mod asr;
pub mod exporter;
pub mod factory;
pub mod project_manager;
pub mod storage;
pub mod sync;
pub mod templates;
pub mod tier;

pub use ai::{AIService, NoOpAI};
pub use asr::{ASRService, NoOpASR};
pub use exporter::{
    export_config_settings_key, ExportConfig, ExportFormat, ExportService, LocalExporter, Margins,
    PageSize, SceneSeparator,
};
pub use factory::{ServiceBundle, ServiceFactory};
pub use project_manager::ProjectManager;
pub use storage::{LocalStorageService, StorageService};
pub use sync::{CloudSyncService, NoOpSync};
pub use templates::{BuiltInTemplates, TemplatesService};
pub use tier::{FreeTier, TierService};
