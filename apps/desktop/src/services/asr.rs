//! ASR (voice-to-text) abstraction.
//!
//! Implementado por `WhisperLocalASR` (sidecar whisper.cpp).
//! Dos contratos: `transcribe_file` (batch, para notas de voz) y
//! `transcribe_stream` (dictado por chunks). El streaming usa un callback
//! sink — mismo patrón object-safe sin runtime que `AIService`.

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

    /// Transcripción batch de un archivo de audio completo (notas de voz).
    fn transcribe_file(&self, _path: &str) -> AppResult<Transcript> {
        Err(crate::error::AppError::Unsupported(
            "la transcripción de voz no está disponible".into(),
        ))
    }

    /// Como `transcribe_file`, pero reporta progreso 0..100 vía `on_progress`.
    /// El default ignora el progreso y delega en `transcribe_file`.
    fn transcribe_file_with_progress(
        &self,
        path: &str,
        _on_progress: &mut dyn FnMut(u8),
    ) -> AppResult<Transcript> {
        self.transcribe_file(path)
    }

    /// Dictado en streaming. `samples` es PCM16 mono a `sample_rate`; la impl
    /// reconoce de forma incremental y llama a `on_partial` con el mejor
    /// transcript parcial, devolviendo el final al terminar.
    fn transcribe_stream(
        &self,
        _samples: &[i16],
        _sample_rate: u32,
        _on_partial: &mut dyn FnMut(&Transcript),
    ) -> AppResult<Transcript> {
        Err(crate::error::AppError::Unsupported(
            "la transcripción de voz no está disponible".into(),
        ))
    }
}
