use serde::Serialize;
use thiserror::Error;

/// Application-level error type. All IPC commands return this.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invariant violated: {0}")]
    Invariant(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),

    #[error("ai provider: {0}")]
    AiProvider(String),

    #[error("unexpected: {0}")]
    Unexpected(String),
}

pub type AppResult<T> = Result<T, AppError>;

// Tauri commands need to return values that serialize. We project AppError
// onto a stable wire shape so the UI can match on it.
#[derive(Debug, Serialize)]
pub struct WireError {
    pub code: &'static str,
    pub message: String,
}

impl From<AppError> for WireError {
    fn from(e: AppError) -> Self {
        let code = match &e {
            AppError::Io(_) => "io",
            AppError::Sqlite(_) => "sqlite",
            AppError::Json(_) => "json",
            AppError::Invariant(_) => "invariant",
            AppError::NotFound(_) => "not_found",
            AppError::Unsupported(_) => "unsupported",
            AppError::AiProvider(_) => "ai_provider",
            AppError::Unexpected(_) => "unexpected",
        };
        WireError {
            code,
            message: e.to_string(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        WireError::from(AppError::Unexpected(self.to_string())).serialize(ser)
    }
}
