//! `PiperTTSService` (H-06): runs the local Piper CLI to synthesize a sentence
//! to a WAV file, parsed into PCM16 + sample rate for the Web Audio player.
//! Gated by the binary and a voice being installed.
//!
//! Same shape as `WhisperLocalASR`: the engine is swappable behind the
//! `TTSService` trait. The WAV parser is pure + unit-tested; the spawn is
//! verified manually with a real Piper binary.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::domain::now_ms;
use crate::error::{AppError, AppResult};
use crate::services::tts::{SynthesizedAudio, TTSService, Voice};
use crate::services::DraffityHome;

use super::registry::{piper_voices, recommended_voice, voice_by_id};

pub struct PiperTTSService {
    home: DraffityHome,
}

impl PiperTTSService {
    pub fn new(home: &DraffityHome) -> Self {
        Self {
            home: DraffityHome::with_root(home.root().to_path_buf()),
        }
    }

    fn voice_installed(&self, onnx_filename: &str) -> bool {
        let onnx = self.home.voice_file_path(onnx_filename);
        let cfg = self.home.voice_file_path(&format!("{onnx_filename}.json"));
        onnx.exists() && cfg.exists()
    }

    /// Resolve the ONNX path to use: the requested voice if installed, else the
    /// recommended one, else the first installed voice.
    fn select_voice(&self, voice_id: &str) -> Option<std::path::PathBuf> {
        if !voice_id.is_empty() {
            if let Some(v) = voice_by_id(voice_id) {
                if self.voice_installed(v.onnx_filename) {
                    return Some(self.home.voice_file_path(v.onnx_filename));
                }
            }
        }
        if let Some(rec) = recommended_voice() {
            if self.voice_installed(rec.onnx_filename) {
                return Some(self.home.voice_file_path(rec.onnx_filename));
            }
        }
        piper_voices()
            .iter()
            .find(|v| self.voice_installed(v.onnx_filename))
            .map(|v| self.home.voice_file_path(v.onnx_filename))
    }

    fn any_voice_installed(&self) -> bool {
        piper_voices()
            .iter()
            .any(|v| self.voice_installed(v.onnx_filename))
    }
}

impl TTSService for PiperTTSService {
    fn available(&self) -> bool {
        self.home.piper_bin_path().exists() && self.any_voice_installed()
    }

    fn voices(&self) -> Vec<Voice> {
        piper_voices()
            .iter()
            .filter(|v| self.voice_installed(v.onnx_filename))
            .map(|v| Voice {
                id: v.id.to_string(),
                name: v.name.to_string(),
                lang: v.lang.to_string(),
            })
            .collect()
    }

    fn synthesize(&self, text: &str, voice_id: &str) -> AppResult<SynthesizedAudio> {
        let bin = self.home.piper_bin_path();
        if !bin.exists() {
            return Err(AppError::Unsupported(
                "el binario de Piper no está instalado".into(),
            ));
        }
        let onnx = self
            .select_voice(voice_id)
            .ok_or_else(|| AppError::Unsupported("no hay voz TTS instalada".into()))?;

        let tmp_dir = self.home.tmp_dir();
        std::fs::create_dir_all(&tmp_dir)?;
        let tmp = tmp_dir.join(format!("tts{}.wav", now_ms()));

        let mut cmd = Command::new(&bin);
        cmd.arg("--model").arg(&onnx).arg("--output_file").arg(&tmp);

        // Piper needs the espeak-ng phoneme data. Its bundled espeak-ng defaults
        // to a baked-in Unix path (`/usr/share/espeak-ng-data`) that never exists
        // on Windows, so we must point it at the `espeak-ng-data` directory that
        // ships next to piper.exe. Without this, piper prints an espeak error,
        // exits 0, and produces no WAV.
        if let Some(espeak_data) = bin.parent().map(|p| p.join("espeak-ng-data")) {
            if espeak_data.is_dir() {
                cmd.arg("--espeak_data").arg(&espeak_data);
            }
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Unexpected(format!("no se pudo ejecutar piper: {e}")))?;

        // Feed the text on stdin, then close it so Piper sees EOF.
        {
            let mut stdin = child
                .stdin
                .take()
                .ok_or_else(|| AppError::Unexpected("piper sin stdin".into()))?;
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| AppError::Unexpected(format!("escritura a piper: {e}")))?;
        }
        let out = child
            .wait_with_output()
            .map_err(|e| AppError::Unexpected(format!("piper falló: {e}")))?;
        let stderr = String::from_utf8_lossy(&out.stderr);
        if !out.status.success() {
            return Err(AppError::Unexpected(format!(
                "piper falló: {}",
                stderr.chars().take(300).collect::<String>()
            )));
        }

        // Piper can exit 0 yet write nothing (e.g. missing espeak-ng-data is only
        // reported on stderr). Treat a missing/empty output as a failure and
        // surface piper's stderr so the cause is visible instead of a generic
        // "playback failed".
        let bytes = std::fs::read(&tmp).map_err(|e| {
            AppError::Unexpected(format!(
                "piper no generó audio: {e}{}",
                if stderr.trim().is_empty() {
                    String::new()
                } else {
                    format!(" — {}", stderr.trim().chars().take(300).collect::<String>())
                }
            ))
        })?;
        let _ = std::fs::remove_file(&tmp);
        let (samples_pcm16, sample_rate) = parse_wav_pcm16(&bytes)
            .ok_or_else(|| AppError::Unexpected("WAV inválido de piper".into()))?;
        Ok(SynthesizedAudio {
            samples_pcm16,
            sample_rate,
        })
    }
}

/// Parse a 16-bit PCM WAV into `(samples, sample_rate)`. Mono is taken as-is;
/// stereo is downmixed by averaging L/R. Returns `None` for non-PCM16 or
/// malformed input. Pure → unit-tested.
pub fn parse_wav_pcm16(bytes: &[u8]) -> Option<(Vec<i16>, u32)> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }
    let le16 = |b: &[u8]| u16::from_le_bytes([b[0], b[1]]);
    let le32 = |b: &[u8]| u32::from_le_bytes([b[0], b[1], b[2], b[3]]);

    let mut pos = 12;
    let mut channels = 1u16;
    let mut sample_rate = 0u32;
    let mut bits = 16u16;
    let mut data: Option<&[u8]> = None;

    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = le32(&bytes[pos + 4..pos + 8]) as usize;
        let body_start = pos + 8;
        let body_end = body_start.saturating_add(size).min(bytes.len());
        if id == b"fmt " && body_end >= body_start + 16 {
            let f = &bytes[body_start..body_end];
            channels = le16(&f[2..4]);
            sample_rate = le32(&f[4..8]);
            bits = le16(&f[14..16]);
        } else if id == b"data" {
            data = Some(&bytes[body_start..body_end]);
        }
        // Chunks are word-aligned (pad byte if odd size).
        pos = body_start + size + (size & 1);
    }

    let data = data?;
    if bits != 16 || sample_rate == 0 {
        return None;
    }
    let ch = channels.max(1) as usize;
    let mut samples = Vec::with_capacity(data.len() / 2);
    let mut i = 0;
    while i + 2 * ch <= data.len() {
        if ch >= 2 {
            let l = i16::from_le_bytes([data[i], data[i + 1]]) as i32;
            let r = i16::from_le_bytes([data[i + 2], data[i + 3]]) as i32;
            samples.push(((l + r) / 2) as i16);
        } else {
            samples.push(i16::from_le_bytes([data[i], data[i + 1]]));
        }
        i += 2 * ch;
    }
    Some((samples, sample_rate))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal 16-bit WAV for tests.
    fn build_wav(samples: &[i16], channels: u16, sample_rate: u32) -> Vec<u8> {
        let block_align = channels * 2;
        let byte_rate = sample_rate * block_align as u32;
        let data_size = samples.len() * 2;
        let mut b = Vec::new();
        b.extend_from_slice(b"RIFF");
        b.extend_from_slice(&((36 + data_size) as u32).to_le_bytes());
        b.extend_from_slice(b"WAVE");
        b.extend_from_slice(b"fmt ");
        b.extend_from_slice(&16u32.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&channels.to_le_bytes());
        b.extend_from_slice(&sample_rate.to_le_bytes());
        b.extend_from_slice(&byte_rate.to_le_bytes());
        b.extend_from_slice(&block_align.to_le_bytes());
        b.extend_from_slice(&16u16.to_le_bytes());
        b.extend_from_slice(b"data");
        b.extend_from_slice(&(data_size as u32).to_le_bytes());
        for s in samples {
            b.extend_from_slice(&s.to_le_bytes());
        }
        b
    }

    #[test]
    fn parses_mono_pcm16() {
        let wav = build_wav(&[0, 100, -100, 32767], 1, 22050);
        let (samples, rate) = parse_wav_pcm16(&wav).unwrap();
        assert_eq!(rate, 22050);
        assert_eq!(samples, vec![0, 100, -100, 32767]);
    }

    #[test]
    fn downmixes_stereo() {
        // Interleaved L,R: (0,100)->50, (200,-200)->0
        let wav = build_wav(&[0, 100, 200, -200], 2, 16000);
        let (samples, rate) = parse_wav_pcm16(&wav).unwrap();
        assert_eq!(rate, 16000);
        assert_eq!(samples, vec![50, 0]);
    }

    #[test]
    fn rejects_non_wav_and_non_pcm16() {
        assert!(parse_wav_pcm16(b"not a wav file at all").is_none());
        let mut wav = build_wav(&[1, 2], 1, 16000);
        // Corrupt bits-per-sample to 8.
        // bits field is at offset 34 (12 + 8 + 14).
        wav[34] = 8;
        assert!(parse_wav_pcm16(&wav).is_none());
    }
}
