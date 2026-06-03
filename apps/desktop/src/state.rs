//! Tauri-managed application state. Built in `lib.rs::run` from a
//! `ServiceBundle` (services) + `WorkerGuard` (log lifecycle), consumed by
//! IPC commands via `State<AppState>`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::logging::LogGuards;
use crate::services::{
    AIService, AIValidatorService, ASRService, BackupService, BibliographyService,
    CrashReporterService, ExportService, ImportService, MediaService, ProjectManagerService,
    ProjectMemoryService, SecretStorage, ServiceBundle, StorageService, TTSService,
    TemplatesService, UserTemplatesLoader,
};

/// Per-request cancellation flags for in-flight AI streams (F-06). The
/// streaming sink checks the flag and stops emitting deltas once set, so the
/// UI preview halts promptly on Esc; the underlying HTTP request finishes in
/// the background (bounded by timeout) and its result is discarded.
#[derive(Default)]
pub struct AiCancelRegistry {
    flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl AiCancelRegistry {
    /// Register a fresh flag for `request_id` and return it for the sink.
    pub fn register(&self, request_id: &str) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        if let Ok(mut g) = self.flags.lock() {
            g.insert(request_id.to_string(), flag.clone());
        }
        flag
    }

    pub fn cancel(&self, request_id: &str) {
        if let Ok(g) = self.flags.lock() {
            if let Some(f) = g.get(request_id) {
                f.store(true, Ordering::Relaxed);
            }
        }
    }

    pub fn finish(&self, request_id: &str) {
        if let Ok(mut g) = self.flags.lock() {
            g.remove(request_id);
        }
    }
}

pub struct AppState {
    pub storage: Arc<dyn StorageService>,
    pub project_manager: Arc<dyn ProjectManagerService>,
    pub templates: Arc<dyn TemplatesService>,
    pub user_templates: Arc<UserTemplatesLoader>,
    /// BYOK AI service (gated at call time by the stored key). Consumed by the
    /// AI commands (Épica F).
    pub ai: Arc<dyn AIService>,
    /// Engram-aligned project memory feeding AI context (Épica F/G).
    pub memory: Arc<dyn ProjectMemoryService>,
    /// AI validators (Épica G). Consumed by the validation commands.
    pub validators: Arc<dyn AIValidatorService>,
    /// Local Whisper ASR (H). Consumed by the voice commands.
    pub asr: Arc<dyn ASRService>,
    /// Local Piper TTS (H). Consumed by the read-aloud command.
    pub tts: Arc<dyn TTSService>,
    pub exporter: Arc<dyn ExportService>,
    pub importer: Arc<dyn ImportService>,
    pub bibliography: Arc<dyn BibliographyService>,
    pub backup: Arc<dyn BackupService>,
    pub media: Arc<dyn MediaService>,
    pub crash_reporter: Arc<dyn CrashReporterService>,
    /// OS-keyring storage for BYOK keys (E-01). Never the plain
    /// `settings` table.
    pub secrets: Arc<dyn SecretStorage>,
    /// In-flight AI stream cancellation flags (F-06).
    pub ai_cancel: Arc<AiCancelRegistry>,
    /// App data dir — voice commands resolve binary/model paths (H).
    pub app_data_dir: PathBuf,
    /// Keeps the non-blocking log writers alive for the whole app lifecycle.
    /// Dropping these guards flushes any pending log lines.
    #[allow(dead_code)]
    pub _log_guards: LogGuards,
}

impl AppState {
    pub fn from_bundle(bundle: ServiceBundle, log_guards: LogGuards) -> Self {
        Self {
            storage: bundle.storage,
            project_manager: bundle.project_manager,
            templates: bundle.templates,
            user_templates: bundle.user_templates,
            ai: bundle.ai,
            memory: bundle.memory,
            validators: bundle.validators,
            asr: bundle.asr,
            tts: bundle.tts,
            exporter: bundle.exporter,
            importer: bundle.importer,
            bibliography: bundle.bibliography,
            backup: bundle.backup,
            media: bundle.media,
            crash_reporter: bundle.crash_reporter,
            secrets: bundle.secrets,
            ai_cancel: Arc::new(AiCancelRegistry::default()),
            app_data_dir: bundle.app_data_dir,
            _log_guards: log_guards,
        }
    }
}
