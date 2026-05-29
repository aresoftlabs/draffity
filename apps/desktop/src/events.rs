//! Event names emitted on the Tauri event bus.
//!
//! Premium features (cloud sync, AI background jobs, codex updates) subscribe
//! to these without modifying the core. Names are stable wire identifiers —
//! never rename, only add.
//!
//! The string consts remain the source of truth for the wire name (existing
//! emitters keep using them). `AppEvent` (E-09) adds a typed front door so new
//! AI/voice emitters get a checked enum instead of a stringly-typed name; its
//! `name()` returns exactly these consts.

pub const PROJECT_CREATED: &str = "project.created";
pub const PROJECT_OPENED: &str = "project.opened";
pub const PROJECT_ARCHIVED: &str = "project.archived";
pub const PROJECT_DELETED: &str = "project.deleted";

pub const DOCUMENT_CREATED: &str = "document.created";
pub const DOCUMENT_SAVED: &str = "document.saved";
pub const DOCUMENT_MOVED: &str = "document.moved";
pub const DOCUMENT_DELETED: &str = "document.deleted";

pub const SNAPSHOT_CREATED: &str = "snapshot.created";

// Premium events (Épicas F/G/H). Placeholders wired by their épica; declared
// here so the typed enum and subscribers share one stable name. Marked
// dead_code until their emitters land — same convention as AppState's
// not-yet-consumed service fields.
#[allow(dead_code)]
pub const AI_SUGGESTION_RECEIVED: &str = "ai.suggestion.received";
#[allow(dead_code)]
pub const AI_VALIDATION_COMPLETED: &str = "ai.validation.completed";
#[allow(dead_code)]
pub const VOICE_TRANSCRIPTION_PARTIAL: &str = "voice.transcription.partial";
#[allow(dead_code)]
pub const VOICE_TRANSCRIPTION_COMPLETE: &str = "voice.transcription.complete";
#[allow(dead_code)]
pub const VOICE_TTS_PROGRESS: &str = "voice.tts.progress";

/// Typed view over the event bus. `name()` yields the stable wire string, so
/// `app.emit(AppEvent::DocumentSaved.name(), payload)` is equivalent to the
/// legacy `app.emit(events::DOCUMENT_SAVED, payload)` — emitters can migrate
/// incrementally with zero wire change.
#[allow(dead_code)] // typed emitters land in Épicas F/G/H
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    ProjectCreated,
    ProjectOpened,
    ProjectArchived,
    ProjectDeleted,
    DocumentCreated,
    DocumentSaved,
    DocumentMoved,
    DocumentDeleted,
    SnapshotCreated,
    AiSuggestionReceived,
    AiValidationCompleted,
    VoiceTranscriptionPartial,
    VoiceTranscriptionComplete,
    VoiceTtsProgress,
}

impl AppEvent {
    #[allow(dead_code)] // consumed by typed emitters in Épicas F/G/H
    pub fn name(self) -> &'static str {
        match self {
            AppEvent::ProjectCreated => PROJECT_CREATED,
            AppEvent::ProjectOpened => PROJECT_OPENED,
            AppEvent::ProjectArchived => PROJECT_ARCHIVED,
            AppEvent::ProjectDeleted => PROJECT_DELETED,
            AppEvent::DocumentCreated => DOCUMENT_CREATED,
            AppEvent::DocumentSaved => DOCUMENT_SAVED,
            AppEvent::DocumentMoved => DOCUMENT_MOVED,
            AppEvent::DocumentDeleted => DOCUMENT_DELETED,
            AppEvent::SnapshotCreated => SNAPSHOT_CREATED,
            AppEvent::AiSuggestionReceived => AI_SUGGESTION_RECEIVED,
            AppEvent::AiValidationCompleted => AI_VALIDATION_COMPLETED,
            AppEvent::VoiceTranscriptionPartial => VOICE_TRANSCRIPTION_PARTIAL,
            AppEvent::VoiceTranscriptionComplete => VOICE_TRANSCRIPTION_COMPLETE,
            AppEvent::VoiceTtsProgress => VOICE_TTS_PROGRESS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_event_name_matches_legacy_const() {
        assert_eq!(AppEvent::DocumentSaved.name(), DOCUMENT_SAVED);
        assert_eq!(AppEvent::ProjectCreated.name(), PROJECT_CREATED);
    }

    #[test]
    fn premium_event_names_are_namespaced() {
        assert_eq!(
            AppEvent::AiSuggestionReceived.name(),
            "ai.suggestion.received"
        );
        assert_eq!(
            AppEvent::VoiceTranscriptionPartial.name(),
            "voice.transcription.partial"
        );
    }
}
