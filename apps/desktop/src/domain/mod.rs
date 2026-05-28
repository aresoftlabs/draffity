//! Pure domain layer. No SQLite, no Tauri, no I/O.
//! Entities + invariants + value objects, fully testable in isolation.

pub mod document;
pub mod project;
pub mod search;
pub mod snapshot;
pub mod stats;
pub mod template;

pub use document::{DocNode, DocumentInput, DocumentStatus, DocumentType};
pub use project::{Project, ProjectInput, ProjectStatus};
pub use search::SearchHit;
pub use snapshot::Snapshot;
pub use stats::WritingStats;
pub use template::{
    FieldType, MetadataField, Template, TemplateKind, TemplateNode, TemplateTier,
    TEMPLATE_SCHEMA_VERSION,
};

/// Current epoch in milliseconds (UTC). Domain-level helper.
pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Generate a fresh ULID as a String.
pub fn new_id() -> String {
    ulid::Ulid::new().to_string()
}
