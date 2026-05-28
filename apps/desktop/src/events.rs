//! Event names emitted on the Tauri event bus.
//!
//! Premium features (cloud sync, AI background jobs, codex updates) subscribe
//! to these without modifying the core. Names are stable wire identifiers —
//! never rename, only add.

pub const PROJECT_CREATED: &str = "project.created";
pub const PROJECT_OPENED: &str = "project.opened";
pub const PROJECT_ARCHIVED: &str = "project.archived";
pub const PROJECT_DELETED: &str = "project.deleted";

pub const DOCUMENT_CREATED: &str = "document.created";
pub const DOCUMENT_SAVED: &str = "document.saved";
pub const DOCUMENT_MOVED: &str = "document.moved";
pub const DOCUMENT_DELETED: &str = "document.deleted";

pub const SNAPSHOT_CREATED: &str = "snapshot.created";
