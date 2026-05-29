//! ASR (voice-to-text) abstraction. **Premium-ready.**
//!
//! Free MVP ships `NoOpASR`. Premium adds `WhisperLocalASR` (whisper.cpp
//! sidecar) implementing this trait — no core change. See `backlog-v4.md`
//! E-04 / H-03.
//!
//! Two contracts: `transcribe_file` (batch, for voice notes H-09) and
//! `transcribe_stream` (chunked dictation H-03). Streaming uses a callback
//! sink — same object-safe, runtime-free pattern as `AIService`.

use serde::Serialize;

use crate::error::AppResult;

/// A recognized span of audio with its timing, used both for the final
/// dictation text and for word-level seek in the voice-notes player (H-11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub text: String,
    pub segments: Vec<TranscriptSegment>,
}

impl Transcript {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            segments: Vec::new(),
        }
    }
}

pub trait ASRService: Send + Sync {
    fn available(&self) -> bool;

    /// Batch transcription of a complete audio file (voice notes).
    fn transcribe_file(&self, _path: &str) -> AppResult<Transcript> {
        Err(crate::error::AppError::Unsupported(
            "voice-to-text not available in free tier".into(),
        ))
    }

    /// Streaming dictation. `samples` is mono PCM16 at `sample_rate`; the
    /// premium impl recognizes incrementally and calls `on_partial` with the
    /// best-so-far transcript, returning the final transcript at the end.
    /// The default errors (free tier).
    fn transcribe_stream(
        &self,
        _samples: &[i16],
        _sample_rate: u32,
        _on_partial: &mut dyn FnMut(&Transcript),
    ) -> AppResult<Transcript> {
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

    #[test]
    fn noop_stream_errors_without_partials() {
        let a = NoOpASR;
        let mut partials = 0usize;
        let res = a.transcribe_stream(&[0i16; 16], 16_000, &mut |_| partials += 1);
        assert!(res.is_err());
        assert_eq!(partials, 0);
    }
}
