//! Mantiene la sesión de streaming activa (una a la vez) tras un Mutex, para que
//! los comandos Tauri la inicien, alimenten, cierren y cancelen. La construcción
//! de la sesión usa una factory de `Transcriber` inyectable (testeable).

use std::sync::{Arc, Mutex};

use crate::services::voice::stream::{StreamEvent, Transcriber, WhisperStreamSession};

/// Crea el `Transcriber` para una sesión nueva. En producción devuelve un
/// `WhisperTranscriber`; en tests, un mock.
pub type TranscriberFactory = Arc<dyn Fn() -> Arc<dyn Transcriber> + Send + Sync>;

pub struct DictationStreamManager {
    make_transcriber: TranscriberFactory,
    session: Mutex<Option<WhisperStreamSession>>,
}

impl DictationStreamManager {
    pub fn new(make_transcriber: TranscriberFactory) -> Self {
        Self {
            make_transcriber,
            session: Mutex::new(None),
        }
    }

    /// Inicia una sesión nueva (reemplaza cualquiera previa).
    pub fn start(&self, sample_rate: u32) {
        let t = (self.make_transcriber)();
        *self.session.lock().unwrap() = Some(WhisperStreamSession::new(t, sample_rate));
    }

    /// Alimenta PCM; devuelve los eventos (vacío si no hay sesión).
    pub fn feed(&self, pcm: &[i16]) -> Vec<StreamEvent> {
        let mut guard = self.session.lock().unwrap();
        match guard.as_mut() {
            Some(s) => s.feed(pcm),
            None => Vec::new(),
        }
    }

    /// Cierra la sesión: vacía la última frase y la descarta.
    pub fn stop(&self) -> Vec<StreamEvent> {
        let mut guard = self.session.lock().unwrap();
        let ev = guard.as_mut().map(|s| s.finish()).unwrap_or_default();
        *guard = None;
        ev
    }

    /// Descarta la sesión sin emitir nada.
    pub fn cancel(&self) {
        *self.session.lock().unwrap() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppResult;

    struct Mock;
    impl Transcriber for Mock {
        fn transcribe(&self, _pcm: &[i16], _sr: u32) -> AppResult<String> {
            Ok("texto".into())
        }
    }

    fn manager() -> DictationStreamManager {
        DictationStreamManager::new(Arc::new(|| Arc::new(Mock) as Arc<dyn Transcriber>))
    }

    #[test]
    fn feed_without_start_is_empty() {
        let m = manager();
        assert!(m.feed(&[8000; 16000]).is_empty());
    }

    #[test]
    fn start_then_feed_emits_and_stop_flushes() {
        let m = manager();
        m.start(16000);
        // Suficiente voz para un parcial (partial_every = 11200 @ 16 kHz).
        let ev = m.feed(&vec![8000i16; 11200]);
        assert!(ev.iter().any(|e| matches!(e, StreamEvent::Partial(_))));
        // stop vacía la frase restante como final.
        let fin = m.stop();
        assert!(fin.iter().any(|e| matches!(e, StreamEvent::Final { .. })));
        // Tras stop no hay sesión.
        assert!(m.feed(&[8000; 16000]).is_empty());
    }

    #[test]
    fn cancel_drops_session_without_events() {
        let m = manager();
        m.start(16000);
        m.feed(&vec![8000i16; 4000]);
        m.cancel();
        assert!(m.feed(&[8000; 16000]).is_empty());
    }
}
