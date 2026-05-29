//! Services layer. Each external concern is a trait so premium can swap in
//! richer implementations without touching the core.

pub mod ai;
pub mod ai_openrouter;
pub mod ai_prompts;
pub mod asr;
pub mod backup;
pub mod bibliography;
pub mod crash_reporter;
pub mod exporter;
pub mod factory;
pub mod importer;
pub mod license;
pub mod media;
pub mod memory;
pub mod project_manager;
mod retention_policy;
pub mod secrets;
pub mod sidecar;
pub mod storage;
pub mod sync;
pub mod templates;
pub mod tier;
pub mod token_counter;
pub mod tts;
pub mod user_templates;

pub use ai::{
    AIService, ChatMessage, CompletionRequest, CompletionResponse, NoOpAI, Role, TokenUsage,
};
pub use ai_openrouter::{ByokAIService, OPENROUTER_KEY};
pub use ai_prompts::{build_messages, parse_action, ActionInput, AiAction, RewriteMode};
pub use asr::{ASRService, NoOpASR, Transcript, TranscriptSegment};
pub use backup::{BackupKind, BackupRecord, BackupService, LocalBackupService, NoOpBackup};
pub use bibliography::{BibliographyService, LocalBibliographyService, ParseSummary};
pub use crash_reporter::{
    report_from_error, CrashReport, CrashReporterService, LocalFileCrashReporter, NoOpCrashReporter,
};
pub use exporter::{
    export_config_settings_key, ExportConfig, ExportFormat, ExportService, LocalExporter, Margins,
    PageSize, SceneSeparator,
};
pub use factory::{ServiceBundle, ServiceFactory};
pub use importer::{
    ImportFormat, ImportNode, ImportService, ImportTree, LocalImporter, LocalMarkdownImporter,
};
pub use license::{DisabledLicenseValidator, Ed25519Validator, LicenseClaims, LicenseValidator};
pub use media::{LocalMediaService, MediaService, NoOpMedia};
pub use memory::{LexicalProjectMemory, MemoryContext, MemoryRequest, ProjectMemoryService};
pub use project_manager::{LocalProjectManager, ProjectManagerService};
pub use secrets::{InMemorySecretStorage, KeyringSecretStorage, SecretStorage};
pub use storage::{CitationUpsert, LocalStorageService, StorageService};
pub use sync::{CloudSyncService, NoOpSync};
pub use templates::{BuiltInTemplates, LayeredTemplatesService, TemplatesService};
pub use tier::{FreeTier, MutableTier, PremiumTier, TierService};
pub use token_counter::{estimate_request_tokens, estimate_tokens};
pub use tts::{NoOpTTS, SynthesizedAudio, TTSService, Voice};
pub use user_templates::{template_from_project, UserTemplatesLoader};
