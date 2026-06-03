//! `WhisperLocalASR` (H-03): runs the local whisper.cpp CLI on a 16 kHz mono
//! WAV file and parses its JSON output into a `Transcript` with per-segment
//! timing. Gated by the binary and a model being installed; otherwise
//! `available()` is false and calls error cleanly.
//!
//! The pure parts — JSON parsing and autopunctuation — are unit-tested; the
//! actual spawn is exercised manually with a real binary present.

use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use crate::domain::now_ms;
use crate::error::{AppError, AppResult};
use crate::services::asr::{ASRService, Transcript, TranscriptSegment};

use super::registry::recommended_model;
use super::{bin_path, model_path, models_dir, voice_dir};

pub struct WhisperLocalASR {
    app_data: PathBuf,
}

impl WhisperLocalASR {
    pub fn new(app_data: PathBuf) -> Self {
        Self { app_data }
    }

    /// Pick a model: the recommended one if installed, else the first `.bin`
    /// found in the models dir.
    fn select_model(&self) -> Option<PathBuf> {
        if let Some(rec) = recommended_model() {
            let p = model_path(&self.app_data, rec.filename);
            if p.exists() {
                return Some(p);
            }
        }
        std::fs::read_dir(models_dir(&self.app_data))
            .ok()?
            .flatten()
            .map(|e| e.path())
            .find(|p| p.extension().map(|x| x == "bin").unwrap_or(false))
    }
}

impl ASRService for WhisperLocalASR {
    fn available(&self) -> bool {
        bin_path(&self.app_data).exists() && self.select_model().is_some()
    }

    fn transcribe_file(&self, path: &str) -> AppResult<Transcript> {
        let bin = bin_path(&self.app_data);
        if !bin.exists() {
            return Err(AppError::Unsupported(
                "el binario de Whisper no está instalado".into(),
            ));
        }
        let model = self
            .select_model()
            .ok_or_else(|| AppError::Unsupported("no hay modelo de voz instalado".into()))?;

        // Unique output base under voice/tmp; whisper writes `<base>.json`.
        let tmp_dir = voice_dir(&self.app_data).join("tmp");
        std::fs::create_dir_all(&tmp_dir)?;
        let base = tmp_dir.join(format!("t{}", now_ms()));

        let output = Command::new(&bin)
            .arg("-m")
            .arg(&model)
            .arg("-f")
            .arg(path)
            .arg("-l")
            .arg("auto")
            .arg("-oj")
            .arg("-of")
            .arg(&base)
            .output()
            .map_err(|e| AppError::Unexpected(format!("no se pudo ejecutar whisper: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Unexpected(format!(
                "whisper falló: {}",
                stderr.chars().take(300).collect::<String>()
            )));
        }

        let json_path = base.with_extension("json");
        let json = std::fs::read_to_string(&json_path).map_err(|e| {
            AppError::Unexpected(format!("no se pudo leer la salida de whisper: {e}"))
        })?;
        let _ = std::fs::remove_file(&json_path);
        Ok(parse_whisper_json(&json))
    }
}

#[derive(Deserialize, Default)]
struct WhisperJson {
    #[serde(default)]
    transcription: Vec<WhisperSegment>,
}

#[derive(Deserialize)]
struct WhisperSegment {
    #[serde(default)]
    text: String,
    #[serde(default)]
    offsets: WhisperOffsets,
}

#[derive(Deserialize, Default)]
struct WhisperOffsets {
    #[serde(default)]
    from: u64,
    #[serde(default)]
    to: u64,
}

/// Parse whisper.cpp `--output-json`: a `transcription` array of segments with
/// `offsets.{from,to}` in ms and `text`. Concatenates text + keeps timing for
/// the voice-notes player (H-11). Tolerant of malformed input → empty.
pub fn parse_whisper_json(json: &str) -> Transcript {
    let parsed: WhisperJson = serde_json::from_str(json).unwrap_or_default();
    let mut text = String::new();
    let mut segments = Vec::new();
    for seg in parsed.transcription {
        let t = seg.text.trim();
        if t.is_empty() {
            continue;
        }
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(t);
        segments.push(TranscriptSegment {
            text: t.to_string(),
            start_ms: seg.offsets.from,
            end_ms: seg.offsets.to,
        });
    }
    Transcript { text, segments }
}

/// Light cleanup of dictated text (H-04): trim, capitalise the first letter,
/// and ensure terminal punctuation. The model handles most punctuation; this
/// just tidies the seams when inserting at the cursor.
pub fn autopunctuate(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut chars = trimmed.chars();
    let mut out: String = match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    };
    if !out.ends_with(['.', '!', '?', '…', ':', ';']) {
        out.push('.');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_segments_with_offsets() {
        let json = r#"{"transcription":[
            {"offsets":{"from":0,"to":1500},"text":" Hola"},
            {"offsets":{"from":1500,"to":3000},"text":" mundo "}
        ]}"#;
        let t = parse_whisper_json(json);
        assert_eq!(t.text, "Hola mundo");
        assert_eq!(t.segments.len(), 2);
        assert_eq!(t.segments[0].start_ms, 0);
        assert_eq!(t.segments[1].end_ms, 3000);
    }

    #[test]
    fn parse_tolerates_garbage_and_empty_segments() {
        assert_eq!(parse_whisper_json("not json").text, "");
        let json = r#"{"transcription":[{"offsets":{"from":0,"to":0},"text":"   "}]}"#;
        assert!(parse_whisper_json(json).segments.is_empty());
    }

    #[test]
    fn autopunctuate_capitalises_and_terminates() {
        assert_eq!(autopunctuate("  hola mundo "), "Hola mundo.");
        assert_eq!(autopunctuate("ya termina."), "Ya termina.");
        assert_eq!(autopunctuate("¿qué tal?"), "¿qué tal?");
        assert_eq!(autopunctuate(""), "");
    }
}
