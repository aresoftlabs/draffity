//! Tauri IPC commands. Thin orchestration around services + event emission.
//! Business logic lives in `services/`. Domain rules live in `domain/`.

pub mod backup;
pub mod bibliography;
pub mod codex;
pub mod document;
pub mod export;
pub mod project;
pub mod search;
pub mod system;
pub mod templates;

pub use backup::*;
pub use bibliography::*;
pub use codex::*;
pub use document::*;
pub use export::*;
pub use project::*;
pub use search::*;
pub use system::*;
pub use templates::*;
