//! Sesión de dictado en streaming (pseudo-streaming con whisper). Mantiene el
//! buffer PCM de la frase en curso; cuando el `StreamPlanner` lo indica,
//! re-decodifica el buffer y emite un parcial; al detectar fin de frase emite el
//! final y resetea. El motor concreto se inyecta como `Transcriber` (mockeable).

use std::path::PathBuf;
use std::sync::Arc;

use crate::error::AppResult;
use crate::services::asr::ASRService;
use crate::services::voice::registry::recommended_model;
use crate::services::voice::server::WhisperServerManager;
use crate::services::voice::stream_planner::{PlanAction, StreamPlanner};
use crate::services::DraffityHome;

/// Transcribe un buffer PCM16 mono completo a texto. Abstrae whisper para tests.
pub trait Transcriber: Send + Sync {
    fn transcribe(
        &self,
        pcm: &[i16],
        sample_rate: u32,
        language: Option<&str>,
    ) -> AppResult<String>;
}

/// Evento emitido por la sesión.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamEvent {
    /// Mejor hipótesis actual (texto en vivo / fantasma). Puede revisarse.
    Partial(String),
    /// Texto consolidado de una frase, con su número de secuencia.
    Final { text: String, seq: u64 },
}

pub struct WhisperStreamSession {
    transcriber: Arc<dyn Transcriber>,
    planner: StreamPlanner,
    buf: Vec<i16>,
    sample_rate: u32,
    seq: u64,
    language: Option<String>,
}

impl WhisperStreamSession {
    pub fn new(
        transcriber: Arc<dyn Transcriber>,
        sample_rate: u32,
        language: Option<String>,
    ) -> Self {
        Self {
            transcriber,
            planner: StreamPlanner::new(sample_rate),
            buf: Vec::new(),
            sample_rate,
            seq: 0,
            language,
        }
    }

    /// Agrega PCM y devuelve los eventos producidos (0..1). El transcribe es
    /// síncrono: el caller debe correr `feed` en un hilo bloqueante.
    pub fn feed(&mut self, pcm: &[i16]) -> Vec<StreamEvent> {
        self.buf.extend_from_slice(pcm);
        match self.planner.on_chunk(pcm, self.buf.len()) {
            PlanAction::Wait => Vec::new(),
            PlanAction::EmitPartial => {
                let text = self.decode();
                if text.is_empty() {
                    Vec::new()
                } else {
                    vec![StreamEvent::Partial(text)]
                }
            }
            PlanAction::Endpoint => self.finalize_utterance(),
        }
    }

    /// Cierra la frase en curso (al parar la sesión): emite su final si hay audio.
    pub fn finish(&mut self) -> Vec<StreamEvent> {
        if self.buf.is_empty() {
            return Vec::new();
        }
        self.finalize_utterance()
    }

    fn finalize_utterance(&mut self) -> Vec<StreamEvent> {
        let text = self.decode();
        self.buf.clear();
        self.planner.reset_utterance();
        if text.is_empty() {
            return Vec::new();
        }
        let seq = self.seq;
        self.seq += 1;
        vec![StreamEvent::Final { text, seq }]
    }

    fn decode(&self) -> String {
        self.transcriber
            .transcribe(&self.buf, self.sample_rate, self.language.as_deref())
            .unwrap_or_default()
            .trim()
            .to_string()
    }
}

/// `Transcriber` real: escribe el buffer PCM16 a un WAV temporal y transcribe con
/// whisper-server (modelo residente → decodes repetidos rápidos) o, si no está o
/// falla, con el CLI batch (`ASRService::transcribe_file`).
pub struct WhisperTranscriber {
    home: DraffityHome,
    server: Arc<WhisperServerManager>,
    asr: Arc<dyn ASRService>,
}

impl WhisperTranscriber {
    pub fn new(
        home: DraffityHome,
        server: Arc<WhisperServerManager>,
        asr: Arc<dyn ASRService>,
    ) -> Self {
        Self { home, server, asr }
    }

    /// Modelo recomendado si está instalado; si no, el primer `.bin` que haya.
    fn select_model(&self) -> Option<PathBuf> {
        if let Some(rec) = recommended_model() {
            let p = self.home.model_path(rec.filename);
            if p.exists() {
                return Some(p);
            }
        }
        std::fs::read_dir(self.home.models_dir())
            .ok()?
            .flatten()
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|x| x == "bin"))
    }
}

impl Transcriber for WhisperTranscriber {
    fn transcribe(
        &self,
        pcm: &[i16],
        sample_rate: u32,
        language: Option<&str>,
    ) -> AppResult<String> {
        if pcm.is_empty() {
            return Ok(String::new());
        }
        // PCM16 little-endian a bytes.
        let mut bytes = Vec::with_capacity(pcm.len() * 2);
        for &s in pcm {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        let tmp = self.home.tmp_dir();
        std::fs::create_dir_all(&tmp)?;
        let path = tmp.join(format!("stream{}.wav", crate::domain::now_ms()));

        let model = self.select_model();
        let result = (|| -> AppResult<String> {
            crate::commands::voice::write_pcm16_wav(&path, &bytes, sample_rate)?;
            // Camino rápido: whisper-server con el modelo resuelto.
            if let Some(ref m) = model {
                if self.server.available() {
                    let wav = std::fs::read(&path)?;
                    if let Ok(t) = self.server.transcribe(m, &wav, language) {
                        return Ok(t.text);
                    }
                }
            }
            // Fallback: CLI batch (resuelve el modelo internamente).
            self.asr
                .transcribe_file(&path.to_string_lossy(), language)
                .map(|t| t.text)
        })();
        let _ = std::fs::remove_file(&path);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Transcriber falso: devuelve textos de una cola en orden, "" si se agota.
    struct MockTranscriber {
        replies: Mutex<std::collections::VecDeque<&'static str>>,
    }
    impl MockTranscriber {
        fn new(replies: Vec<&'static str>) -> Arc<Self> {
            Arc::new(Self {
                replies: Mutex::new(replies.into_iter().collect()),
            })
        }
    }
    impl Transcriber for MockTranscriber {
        fn transcribe(&self, _pcm: &[i16], _sr: u32, _language: Option<&str>) -> AppResult<String> {
            Ok(self
                .replies
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or("")
                .to_string())
        }
    }

    fn loud(n: usize) -> Vec<i16> {
        vec![8000; n]
    }
    fn quiet(n: usize) -> Vec<i16> {
        vec![0; n]
    }

    #[test]
    fn emits_partial_on_cadence() {
        let t = MockTranscriber::new(vec!["hola que tal"]);
        let mut s = WhisperStreamSession::new(t, 16000, None);
        let ev = s.feed(&loud(11200)); // supera partial_every
        assert_eq!(ev, vec![StreamEvent::Partial("hola que tal".into())]);
    }

    #[test]
    fn final_seq_increments_per_utterance_and_resets_buffer() {
        let t = MockTranscriber::new(vec!["hola mundo", "segunda"]);
        let mut s = WhisperStreamSession::new(t, 16000, None);
        s.feed(&loud(2000)); // voz, sin endpoint aún
        let ev1 = s.feed(&quiet(12000)); // silencio sostenido → endpoint
        assert_eq!(
            ev1,
            vec![StreamEvent::Final {
                text: "hola mundo".into(),
                seq: 0
            }]
        );
        // Segunda frase: el buffer se reseteó, arranca con seq=1.
        s.feed(&loud(2000));
        let ev2 = s.feed(&quiet(12000));
        assert_eq!(
            ev2,
            vec![StreamEvent::Final {
                text: "segunda".into(),
                seq: 1
            }]
        );
    }

    #[test]
    fn empty_transcript_emits_nothing() {
        let t = MockTranscriber::new(vec!["   "]);
        let mut s = WhisperStreamSession::new(t, 16000, None);
        let ev = s.feed(&loud(11200));
        assert!(ev.is_empty());
    }

    #[test]
    fn finish_flushes_remaining_as_final() {
        let t = MockTranscriber::new(vec!["resto de frase"]);
        let mut s = WhisperStreamSession::new(t, 16000, None);
        s.feed(&loud(2000)); // audio sin endpoint
        let ev = s.finish();
        assert_eq!(
            ev,
            vec![StreamEvent::Final {
                text: "resto de frase".into(),
                seq: 0
            }]
        );
    }

    #[test]
    fn finish_on_empty_buffer_emits_nothing() {
        let t = MockTranscriber::new(vec![]);
        let mut s = WhisperStreamSession::new(t, 16000, None);
        assert!(s.finish().is_empty());
    }
}
