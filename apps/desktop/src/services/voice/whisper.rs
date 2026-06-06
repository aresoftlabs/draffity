//! `WhisperLocalASR` (H-03): runs the local whisper.cpp CLI on a 16 kHz mono
//! WAV file and parses its JSON output into a `Transcript` with per-segment
//! timing. Gated by the binary and a model being installed; otherwise
//! `available()` is false and calls error cleanly.
//!
//! The pure parts — JSON parsing and autopunctuation — are unit-tested; the
//! actual spawn is exercised manually with a real binary present.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use serde::Deserialize;

use crate::domain::now_ms;
use crate::error::{AppError, AppResult};
use crate::services::asr::{ASRService, Transcript, TranscriptSegment};
use crate::services::DraffityHome;

use super::registry::recommended_model;

pub struct WhisperLocalASR {
    home: DraffityHome,
}

impl WhisperLocalASR {
    pub fn new(home: &DraffityHome) -> Self {
        Self {
            home: DraffityHome::with_root(home.root().to_path_buf()),
        }
    }

    /// Pick a model: the recommended one if installed, else the first `.bin`
    /// found in the models dir.
    fn select_model(&self) -> Option<std::path::PathBuf> {
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
            .find(|p| p.extension().map(|x| x == "bin").unwrap_or(false))
    }
}

impl ASRService for WhisperLocalASR {
    fn available(&self) -> bool {
        self.home.bin_dir().exists() && self.select_model().is_some()
    }

    fn transcribe_file(&self, path: &str, language: Option<&str>) -> AppResult<Transcript> {
        let bin = self.home.bin_dir();
        if !bin.exists() {
            return Err(AppError::Unsupported(
                "el binario de Whisper no está instalado".into(),
            ));
        }
        let model = self
            .select_model()
            .ok_or_else(|| AppError::Unsupported("no hay modelo de voz instalado".into()))?;

        // Unique output base under voice/tmp; whisper writes `<base>.json`.
        let tmp_dir = self.home.tmp_dir();
        std::fs::create_dir_all(&tmp_dir)?;
        let base = tmp_dir.join(format!("t{}", now_ms()));

        let mut cmd = Command::new(&bin);
        super::proc::no_window(&mut cmd);
        let output = cmd
            .arg("-m")
            .arg(&model)
            .arg("-f")
            .arg(path)
            .arg("-l")
            .arg(language.unwrap_or("auto"))
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

    fn transcribe_file_with_progress(
        &self,
        path: &str,
        language: Option<&str>,
        on_progress: &mut dyn FnMut(u8),
    ) -> AppResult<Transcript> {
        let bin = self.home.bin_dir();
        if !bin.exists() {
            return Err(AppError::Unsupported(
                "el binario de Whisper no está instalado".into(),
            ));
        }
        let model = self
            .select_model()
            .ok_or_else(|| AppError::Unsupported("no hay modelo de voz instalado".into()))?;

        let tmp_dir = self.home.tmp_dir();
        std::fs::create_dir_all(&tmp_dir)?;
        let base = tmp_dir.join(format!("t{}", now_ms()));

        let mut cmd = Command::new(&bin);
        super::proc::no_window(&mut cmd);
        let mut child = cmd
            .arg("-m")
            .arg(&model)
            .arg("-f")
            .arg(path)
            .arg("-l")
            .arg(language.unwrap_or("auto"))
            .arg("--print-progress")
            .arg("-oj")
            .arg("-of")
            .arg(&base)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Unexpected(format!("no se pudo ejecutar whisper: {e}")))?;

        // Leer stderr en streaming: parsear progreso y conservar el resto por si
        // hay un fallo (whisper escribe sus errores a stderr).
        let mut stderr_tail: Vec<String> = Vec::new();
        let mut last = 0u8;
        if let Some(stderr) = child.stderr.take() {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Some(p) = parse_progress_line(&line) {
                    if p != last {
                        last = p;
                        on_progress(p);
                    }
                } else {
                    stderr_tail.push(line);
                    if stderr_tail.len() > 20 {
                        stderr_tail.remove(0);
                    }
                }
            }
        }

        let status = child
            .wait()
            .map_err(|e| AppError::Unexpected(format!("whisper falló: {e}")))?;
        if !status.success() {
            return Err(AppError::Unexpected(format!(
                "whisper falló: {}",
                stderr_tail.join(" ").chars().take(300).collect::<String>()
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

/// Parsea una línea de stderr de whisper con `--print-progress` a un porcentaje.
/// Las líneas tienen la forma `whisper_print_progress_callback: progress = 42%`.
/// Devuelve `None` para líneas no relacionadas.
pub fn parse_progress_line(line: &str) -> Option<u8> {
    let idx = line.find("progress =")?;
    let rest = line[idx + "progress =".len()..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok().map(|n| n.min(100) as u8)
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

    #[test]
    fn parses_whisper_progress_lines() {
        assert_eq!(
            parse_progress_line("whisper_print_progress_callback: progress = 42%"),
            Some(42)
        );
        assert_eq!(
            parse_progress_line("whisper_print_progress_callback: progress = 100%"),
            Some(100)
        );
        // Tolera espacios y valores de un dígito.
        assert_eq!(parse_progress_line("progress =  5%"), Some(5));
        // Clampa a 100.
        assert_eq!(parse_progress_line("progress = 130%"), Some(100));
        // Líneas no-progreso → None.
        assert_eq!(parse_progress_line("output_json: saving output"), None);
        assert_eq!(parse_progress_line("whisper_init_from_file"), None);
    }
}
