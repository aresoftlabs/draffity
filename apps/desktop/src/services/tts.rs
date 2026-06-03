//! Abstracción de text-to-speech.
//!
//! Implementado por `PiperTTSService` (sidecar Piper ONNX).
//! Separado de `ASRService` porque síntesis y reconocimiento son
//! responsabilidades independientes con impls independientes.

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

    /// Voces instaladas localmente. Vacío si no hay motor ni modelo descargado.
    fn voices(&self) -> Vec<Voice> {
        Vec::new()
    }

    /// Sintetiza `text` con la voz `voice_id`.
    fn synthesize(&self, _text: &str, _voice_id: &str) -> AppResult<SynthesizedAudio> {
        Err(crate::error::AppError::Unsupported(
            "la lectura en voz alta no está disponible".into(),
        ))
    }
}
