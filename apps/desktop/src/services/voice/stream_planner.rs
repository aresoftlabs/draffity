//! Decisión pura del pseudo-streaming: dado el audio que va llegando, decide
//! cuándo emitir un parcial (re-decodificar) y cuándo cortar la frase (endpoint
//! por silencio). No toca whisper — es 100% testeable.

/// Acción que el planner recomienda tras cada chunk de PCM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanAction {
    /// Seguir acumulando, no hacer nada.
    Wait,
    /// Re-decodificar el buffer actual y emitir el texto como parcial.
    EmitPartial,
    /// Fin de frase detectado: decodificar y emitir final, luego resetear.
    Endpoint,
}

pub struct StreamPlanner {
    partial_every: usize,  // muestras entre parciales
    silence_rms: f32,      // umbral RMS (0..1) bajo el cual es "silencio"
    silence_needed: usize, // muestras de silencio consecutivo para cortar
    // estado:
    since_last_partial: usize,
    silent_run: usize,
    had_speech: bool, // hubo voz desde el último endpoint (evita cortar en silencio puro)
}

impl StreamPlanner {
    /// `sample_rate` en Hz. Defaults: parcial cada 0.7 s, silencio 0.7 s, RMS 0.02.
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate as usize;
        Self {
            partial_every: sr * 7 / 10,
            silence_rms: 0.02,
            silence_needed: sr * 7 / 10,
            since_last_partial: 0,
            silent_run: 0,
            had_speech: false,
        }
    }

    /// Procesa el chunk recién agregado al buffer. `buffer_len` es el largo total
    /// del buffer de la frase (en muestras) tras agregar el chunk.
    pub fn on_chunk(&mut self, chunk: &[i16], buffer_len: usize) -> PlanAction {
        let rms = rms_i16(chunk);
        if rms >= self.silence_rms {
            self.had_speech = true;
            self.silent_run = 0;
        } else {
            self.silent_run += chunk.len();
        }
        self.since_last_partial += chunk.len();

        // Endpoint: hubo voz y ahora hay suficiente silencio sostenido.
        if self.had_speech && self.silent_run >= self.silence_needed {
            return PlanAction::Endpoint;
        }
        // Parcial: pasó la cadencia y hay algo de audio que valga la pena decodificar.
        if self.since_last_partial >= self.partial_every && buffer_len > 0 && self.had_speech {
            self.since_last_partial = 0;
            return PlanAction::EmitPartial;
        }
        PlanAction::Wait
    }

    /// Resetear el estado para empezar una frase nueva (tras un Endpoint o un final).
    pub fn reset_utterance(&mut self) {
        self.since_last_partial = 0;
        self.silent_run = 0;
        self.had_speech = false;
    }
}

/// RMS (0..1) de un chunk PCM16, normalizando por 32768.
pub fn rms_i16(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples
        .iter()
        .map(|&s| {
            let n = s as f64 / 32768.0;
            n * n
        })
        .sum();
    ((sum / samples.len() as f64).sqrt()) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loud(n: usize) -> Vec<i16> {
        vec![8000; n]
    }
    fn quiet(n: usize) -> Vec<i16> {
        vec![0; n]
    }

    #[test]
    fn rms_of_silence_is_zero_and_loud_is_high() {
        assert_eq!(rms_i16(&quiet(100)), 0.0);
        assert!(rms_i16(&loud(100)) > 0.2);
        assert_eq!(rms_i16(&[]), 0.0);
    }

    #[test]
    fn emits_partial_after_cadence_once_speech_started() {
        let mut p = StreamPlanner::new(16000); // partial_every = 11200
        let action = p.on_chunk(&loud(11200), 11200);
        assert_eq!(action, PlanAction::EmitPartial);
    }

    #[test]
    fn no_partial_before_speech() {
        let mut p = StreamPlanner::new(16000);
        let action = p.on_chunk(&quiet(11200), 11200);
        assert_eq!(action, PlanAction::Wait);
    }

    #[test]
    fn endpoint_after_speech_then_sustained_silence() {
        let mut p = StreamPlanner::new(16000); // silence_needed = 11200
        assert_eq!(p.on_chunk(&loud(2000), 2000), PlanAction::Wait);
        assert_eq!(p.on_chunk(&quiet(5000), 7000), PlanAction::Wait);
        assert_eq!(p.on_chunk(&quiet(7000), 14000), PlanAction::Endpoint);
    }

    #[test]
    fn reset_clears_speech_and_silence_state() {
        let mut p = StreamPlanner::new(16000);
        p.on_chunk(&loud(2000), 2000);
        p.reset_utterance();
        assert_eq!(p.on_chunk(&quiet(12000), 12000), PlanAction::Wait);
    }
}
