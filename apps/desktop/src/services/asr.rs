//! ASR (voice-to-text) abstraction. **Premium-ready stub.**
//!
//! Free MVP ships `NoOpASR`. Premium adds `WhisperLocalASR` (whisper.cpp
//! sidecar) implementing this trait.

use crate::error::AppResult;

pub trait ASRService: Send + Sync {
    fn available(&self) -> bool;
    fn transcribe_file(&self, _path: &str) -> AppResult<String> {
        Err(crate::error::AppError::Unsupported(
            "voice-to-text not available in free tier".into(),
        ))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpASR;

impl ASRService for NoOpASR {
    fn available(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_is_unavailable() {
        let a = NoOpASR;
        assert!(!a.available());
        assert!(a.transcribe_file("x.wav").is_err());
    }
}
