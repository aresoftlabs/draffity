//! Text-to-speech abstraction. **Premium-ready.**
//!
//! Free MVP ships `NoOpTTS`. Premium adds `PiperTTSService` (Piper ONNX
//! sidecar) implementing this trait — no core change. See `backlog-v4.md`
//! E-04 / H-06. Separate from `ASRService` because synthesis and recognition
//! are independent concerns with independent premium impls.

use serde::Serialize;

use crate::error::AppResult;

/// A voice the engine can speak with. `id` is the engine-specific model id;
/// `lang` is a BCP-47-ish tag (e.g. `es`, `en`) for the picker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Voice {
    pub id: String,
    pub name: String,
    pub lang: String,
}

/// Mono PCM16 audio ready to feed the Web Audio API in the UI.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SynthesizedAudio {
    pub samples_pcm16: Vec<i16>,
    pub sample_rate: u32,
}

pub trait TTSService: Send + Sync {
    fn available(&self) -> bool;

    /// Voices installed locally. Empty until a premium engine + model land.
    fn voices(&self) -> Vec<Voice> {
        Vec::new()
    }

    /// Synthesize `text` with the given `voice_id`. Default errors (free tier).
    fn synthesize(&self, _text: &str, _voice_id: &str) -> AppResult<SynthesizedAudio> {
        Err(crate::error::AppError::Unsupported(
            "text-to-speech not available in free tier".into(),
        ))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpTTS;

impl TTSService for NoOpTTS {
    fn available(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_is_unavailable_and_has_no_voices() {
        let t = NoOpTTS;
        assert!(!t.available());
        assert!(t.voices().is_empty());
        assert!(t.synthesize("hola", "es-default").is_err());
    }
}
