//! Tauri IPC commands. Thin orchestration around services + event emission.
//! Business logic lives in `services/`. Domain rules live in `domain/`.

pub mod document;
pub mod export;
pub mod project;
pub mod system;
pub mod templates;

pub use document::*;
pub use export::*;
pub use project::*;
pub use system::*;
pub use templates::*;
