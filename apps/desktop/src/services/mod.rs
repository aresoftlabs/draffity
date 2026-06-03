//! Services layer. Each external concern is a trait with a local implementation.

pub mod ai;
pub mod ai_openrouter;
pub mod ai_prompts;
pub mod ai_validators;
pub mod asr;
pub mod backup;
pub mod bibliography;
pub mod crash_reporter;
pub mod exporter;
pub mod factory;
pub mod importer;
pub mod media;
pub mod memory;
pub mod project_manager;
mod retention_policy;
pub mod secrets;
pub mod sidecar;
pub mod storage;
pub mod templates;
pub mod token_counter;
pub mod tts;
pub mod user_templates;
pub mod validation_context;
pub mod voice;

pub use ai::{
    AIService, ChatMessage, CompletionRequest, CompletionResponse, NoOpAI, Role, TokenUsage,
};
pub use ai_openrouter::{ByokAIService, OPENROUTER_KEY};
pub use ai_prompts::{build_messages, parse_action, ActionInput, AiAction, RewriteMode};
pub use ai_validators::{
    codex_coverage, summarize, AIValidatorService, CoverageReport, Finding, OpenRouterValidators,
    Severity, ValidationInput, ValidatorKind,
};
pub use asr::{ASRService, Transcript, TranscriptSegment};
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
pub use importer::{ImportFormat, ImportNode, ImportService, ImportTree, LocalImporter};
pub use media::{LocalMediaService, MediaService, NoOpMedia};
pub use memory::{LexicalProjectMemory, MemoryContext, MemoryRequest, ProjectMemoryService};
pub use project_manager::{LocalProjectManager, ProjectManagerService};
pub use secrets::{InMemorySecretStorage, KeyringSecretStorage, SecretStorage};
pub use storage::{CitationUpsert, LocalStorageService, StorageService};
pub use templates::{BuiltInTemplates, LayeredTemplatesService, TemplatesService};
pub use token_counter::{estimate_request_tokens, estimate_tokens};
pub use tts::{SynthesizedAudio, TTSService, Voice};
pub use user_templates::{template_from_project, UserTemplatesLoader};
pub use validation_context::ValidationContextBuilder;
pub use voice::{PiperTTSService, WhisperLocalASR};
