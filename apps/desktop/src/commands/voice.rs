//! Voice runtime commands (Épica H, slice 1): status, model catalogue,
//! opt-in model download (with progress events), delete, and importing a
//! user-provided whisper binary.
//!
//! The actual transcription command lands with the dictation UI (slice 2);
//! here we manage the assets the ASR depends on.

use std::path::PathBuf;

use tauri::{AppHandle, Emitter, State};

use crate::domain::{now_ms, MediaAsset};
use crate::error::AppError;
use crate::services::tts::SynthesizedAudio;
use crate::services::voice::{
    bin_path, download_to_file, installed_models, model_by_id, model_path, model_url,
    piper_bin_path, piper_voices, voice_by_id, voice_config_filename, voice_dir, voice_file_path,
    whisper_models,
};
use crate::services::Transcript;
use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceStatus {
    /// Dictation usable now: Whisper binary + a model installed.
    pub dictation_available: bool,
    pub binary_installed: bool,
    pub installed_models: Vec<String>,
    /// Read-aloud usable now: Piper binary + a voice installed.
    pub tts_available: bool,
    pub piper_installed: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceVoiceDto {
    pub id: String,
    pub name: String,
    pub lang: String,
    pub size_mb: u32,
    pub recommended: bool,
    pub installed: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceModelDto {
    pub id: String,
    pub filename: String,
    pub size_mb: u32,
    pub recommended: bool,
    pub installed: bool,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress {
    model_id: String,
    downloaded: u64,
    total: Option<u64>,
}

#[tauri::command]
pub fn get_voice_status(state: State<'_, AppState>) -> VoiceStatus {
    let dir = &state.app_data_dir;
    VoiceStatus {
        dictation_available: state.asr.available(),
        binary_installed: bin_path(dir).exists(),
        installed_models: installed_models(dir),
        tts_available: state.tts.available(),
        piper_installed: piper_bin_path(dir).exists(),
    }
}

#[tauri::command]
pub fn list_voice_voices(state: State<'_, AppState>) -> Vec<VoiceVoiceDto> {
    let dir = &state.app_data_dir;
    piper_voices()
        .iter()
        .map(|v| {
            let installed = voice_file_path(dir, v.onnx_filename).exists()
                && voice_file_path(dir, &voice_config_filename(v)).exists();
            VoiceVoiceDto {
                id: v.id.to_string(),
                name: v.name.to_string(),
                lang: v.lang.to_string(),
                size_mb: v.size_mb,
                recommended: v.recommended,
                installed,
            }
        })
        .collect()
}

#[tauri::command]
pub fn list_voice_models(state: State<'_, AppState>) -> Vec<VoiceModelDto> {
    let dir = &state.app_data_dir;
    whisper_models()
        .iter()
        .map(|m| VoiceModelDto {
            id: m.id.to_string(),
            filename: m.filename.to_string(),
            size_mb: m.size_mb,
            recommended: m.recommended,
            installed: model_path(dir, m.filename).exists(),
        })
        .collect()
}

/// Download a Whisper model opt-in, emitting `voice.download.progress` events.
#[tauri::command]
pub async fn download_voice_model(
    app: AppHandle,
    state: State<'_, AppState>,
    model_id: String,
) -> CmdResult<()> {
    let model = model_by_id(&model_id)
        .ok_or_else(|| AppError::NotFound(format!("voice model {model_id}")))?;
    let url = model_url(model);
    let dest = model_path(&state.app_data_dir, model.filename);
    let sha = model.sha256;
    let app2 = app.clone();
    let id = model_id.clone();

    tauri::async_runtime::spawn_blocking(move || {
        download_to_file(&url, &dest, sha, |downloaded, total| {
            let _ = app2.emit(
                "voice.download.progress",
                DownloadProgress {
                    model_id: id.clone(),
                    downloaded,
                    total,
                },
            );
        })
    })
    .await
    .map_err(|e| AppError::Unexpected(format!("descarga: {e}")))?
}

#[tauri::command]
pub fn delete_voice_model(state: State<'_, AppState>, model_id: String) -> CmdResult<()> {
    let model = model_by_id(&model_id)
        .ok_or_else(|| AppError::NotFound(format!("voice model {model_id}")))?;
    let path = model_path(&state.app_data_dir, model.filename);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// Transcribe a recorded clip (16 kHz mono WAV bytes from the UI) to text.
/// The engine is whatever `ASRService` is wired — this command only depends on
/// the trait, so swapping the ASR backend never touches it. Runs off-thread.
#[tauri::command]
pub async fn transcribe_audio(state: State<'_, AppState>, wav: Vec<u8>) -> CmdResult<Transcript> {
    if !state.asr.available() {
        return Err(AppError::Unsupported(
            "el dictado no está disponible (instalá el binario y un modelo de voz)".into(),
        ));
    }
    let tmp_dir = voice_dir(&state.app_data_dir).join("tmp");
    std::fs::create_dir_all(&tmp_dir)?;
    let path = tmp_dir.join(format!("rec{}.wav", now_ms()));
    std::fs::write(&path, &wav)?;

    let asr = state.asr.clone();
    let path_str = path.to_string_lossy().into_owned();
    let result = tauri::async_runtime::spawn_blocking(move || asr.transcribe_file(&path_str))
        .await
        .map_err(|e| AppError::Unexpected(format!("tarea de transcripción: {e}")))?;
    let _ = std::fs::remove_file(&path);
    result
}

/// Copy a user-provided whisper.cpp binary into the app's voice/bin dir. The
/// reliable alternative to downloading a platform release.
#[tauri::command]
pub fn import_voice_binary(state: State<'_, AppState>, source_path: String) -> CmdResult<()> {
    let src = PathBuf::from(&source_path);
    if !src.is_file() {
        return Err(AppError::NotFound(format!("binary {source_path}")));
    }
    let dest = bin_path(&state.app_data_dir);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(&src, &dest)?;
    Ok(())
}

/// Copy a user-provided Piper binary into the app's voice/bin dir.
#[tauri::command]
pub fn import_piper_binary(state: State<'_, AppState>, source_path: String) -> CmdResult<()> {
    let src = PathBuf::from(&source_path);
    if !src.is_file() {
        return Err(AppError::NotFound(format!("binary {source_path}")));
    }
    let dest = piper_bin_path(&state.app_data_dir);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(&src, &dest)?;
    Ok(())
}

/// Download a Piper voice (ONNX model + its `.onnx.json` config), emitting
/// `voice.download.progress` for the model file.
#[tauri::command]
pub async fn download_voice_voice(
    app: AppHandle,
    state: State<'_, AppState>,
    voice_id: String,
) -> CmdResult<()> {
    let voice =
        voice_by_id(&voice_id).ok_or_else(|| AppError::NotFound(format!("voice {voice_id}")))?;
    let onnx_url = voice.onnx_url.to_string();
    let config_url = format!("{}.json", voice.onnx_url);
    let onnx_dest = voice_file_path(&state.app_data_dir, voice.onnx_filename);
    let config_dest = voice_file_path(&state.app_data_dir, &voice_config_filename(voice));
    let app2 = app.clone();
    let id = voice_id.clone();

    tauri::async_runtime::spawn_blocking(move || -> CmdResult<()> {
        // Config first (tiny), then the model with progress.
        download_to_file(&config_url, &config_dest, None, |_, _| {})?;
        download_to_file(&onnx_url, &onnx_dest, None, |downloaded, total| {
            let _ = app2.emit(
                "voice.download.progress",
                DownloadProgress {
                    model_id: id.clone(),
                    downloaded,
                    total,
                },
            );
        })
    })
    .await
    .map_err(|e| AppError::Unexpected(format!("descarga: {e}")))?
}

/// Synthesize one sentence to PCM16 audio (read-aloud, sentence by sentence so
/// the UI can highlight). Engine-agnostic: depends only on the `TTSService`
/// trait. Runs off-thread.
#[tauri::command]
pub async fn synthesize_speech(
    state: State<'_, AppState>,
    text: String,
    voice_id: String,
) -> CmdResult<SynthesizedAudio> {
    if !state.tts.available() {
        return Err(AppError::Unsupported(
            "la lectura en voz alta no está disponible (instalá Piper y una voz)".into(),
        ));
    }
    let tts = state.tts.clone();
    tauri::async_runtime::spawn_blocking(move || tts.synthesize(&text, &voice_id))
        .await
        .map_err(|e| AppError::Unexpected(format!("tarea de síntesis: {e}")))?
}

/// Test-synthesize: render `text` with `voice_id` and write the PCM16 output
/// as a WAV file in a temp location. Returns the absolute file path for the UI
/// to play via an `<audio>` element.
#[tauri::command]
pub async fn test_synthesize(
    state: State<'_, AppState>,
    voice_id: String,
    text: String,
) -> CmdResult<String> {
    // Validate voice exists in registry.
    if voice_by_id(&voice_id).is_none() {
        return Err(AppError::NotFound(format!("voice {voice_id}")));
    }
    if !state.tts.available() {
        return Err(AppError::Unsupported(
            "la lectura en voz alta no está disponible (instalá Piper y una voz)".into(),
        ));
    }
    let tts = state.tts.clone();
    let audio = tauri::async_runtime::spawn_blocking(move || tts.synthesize(&text, &voice_id))
        .await
        .map_err(|e| AppError::Unexpected(format!("tarea de síntesis: {e}")))??;

    // Write a WAV file to a temp dir under voice/.
    let tmp = voice_dir(&state.app_data_dir).join("tmp");
    std::fs::create_dir_all(&tmp)?;
    let path = tmp.join(format!("test_synth_{}.wav", now_ms()));
    let bytes = encode_wav_pcm16(&audio.samples_pcm16, audio.sample_rate);
    std::fs::write(&path, &bytes)?;
    Ok(path.to_string_lossy().into_owned())
}

/// Pure: encode PCM16 samples + sample rate into a proper WAV file.
pub fn encode_wav_pcm16(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let block_align = channels * bits_per_sample / 8;
    let byte_rate = sample_rate * block_align as u32;
    let data_size = (samples.len() as u32) * 2;
    let mut b = Vec::new();
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_size).to_le_bytes());
    b.extend_from_slice(b"WAVE");
    b.extend_from_slice(b"fmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&channels.to_le_bytes());
    b.extend_from_slice(&sample_rate.to_le_bytes());
    b.extend_from_slice(&byte_rate.to_le_bytes());
    b.extend_from_slice(&block_align.to_le_bytes());
    b.extend_from_slice(&bits_per_sample.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_size.to_le_bytes());
    for s in samples {
        b.extend_from_slice(&s.to_le_bytes());
    }
    b
}

/// Save a recorded voice note (16 kHz mono WAV bytes) as project media, and —
/// when requested and the ASR is available — transcribe it in the background.
/// Transcription failure never fails the save.
#[tauri::command]
pub async fn save_voice_note(
    state: State<'_, AppState>,
    project_id: String,
    wav: Vec<u8>,
    duration_ms: i64,
    transcribe: bool,
) -> CmdResult<MediaAsset> {
    if state.project_manager.get(&project_id)?.is_none() {
        return Err(AppError::NotFound(format!("project {project_id}")));
    }
    let asset = state.media.store(&project_id, "audio/wav", &wav)?;

    let mut text: Option<String> = None;
    if transcribe && state.asr.available() {
        let path = state
            .app_data_dir
            .join(&asset.path_relative)
            .to_string_lossy()
            .into_owned();
        let asr = state.asr.clone();
        let transcript = tauri::async_runtime::spawn_blocking(move || asr.transcribe_file(&path))
            .await
            .map_err(|e| AppError::Unexpected(format!("tarea de transcripción: {e}")))?;
        if let Ok(t) = transcript {
            let trimmed = t.text.trim();
            if !trimmed.is_empty() {
                text = Some(trimmed.to_string());
            }
        }
    }

    state
        .storage
        .set_media_voice_note(&asset.id, Some(duration_ms), text.as_deref())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableModelEntry {
    pub id: String,
    pub name: String,
    pub lang: String,
    pub size_mb: u32,
    pub recommended: bool,
    pub installed: bool,
    pub disk_bytes: u64,
    pub kind: &'static str, // "voice" or "model"
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageGroup {
    pub lang: String,
    pub items: Vec<AvailableModelEntry>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskUsageEntry {
    pub id: String,
    pub bytes: u64,
}

/// Get disk usage for installed models and voices.
#[tauri::command]
pub fn get_disk_usage(state: State<'_, AppState>) -> Vec<DiskUsageEntry> {
    let dir = &state.app_data_dir;
    let mut entries = Vec::new();

    // Piper voices
    for v in piper_voices() {
        let p = voice_file_path(dir, v.onnx_filename);
        if p.exists() {
            if let Ok(meta) = std::fs::metadata(&p) {
                entries.push(DiskUsageEntry {
                    id: v.id.to_string(),
                    bytes: meta.len(),
                });
            }
        }
    }

    // Whisper models
    for m in whisper_models() {
        let p = model_path(dir, m.filename);
        if p.exists() {
            if let Ok(meta) = std::fs::metadata(&p) {
                entries.push(DiskUsageEntry {
                    id: m.id.to_string(),
                    bytes: meta.len(),
                });
            }
        }
    }

    entries
}

/// Pure: group voices + models by language. Takes pre-computed installed status
/// so the function is testable without filesystem access.
pub fn group_available_models<'v>(
    voices: impl Iterator<Item = (&'v crate::services::voice::PiperVoiceInfo, bool, u64)>,
    models: impl Iterator<Item = (&'v crate::services::voice::WhisperModelInfo, bool, u64)>,
) -> Vec<LanguageGroup> {
    let entries: Vec<AvailableModelEntry> = voices
        .map(|(v, installed, bytes)| AvailableModelEntry {
            id: v.id.to_string(),
            name: v.name.to_string(),
            lang: v.lang.to_string(),
            size_mb: v.size_mb,
            recommended: v.recommended,
            installed,
            disk_bytes: bytes,
            kind: "voice",
        })
        .chain(models.map(|(m, installed, bytes)| AvailableModelEntry {
            id: m.id.to_string(),
            name: m.filename.to_string(),
            lang: "other".to_string(),
            size_mb: m.size_mb,
            recommended: m.recommended,
            installed,
            disk_bytes: bytes,
            kind: "model",
        }))
        .collect();

    // Group by lang preserving order of first occurrence.
    let mut groups: Vec<LanguageGroup> = Vec::new();
    for entry in entries {
        if let Some(g) = groups
            .iter_mut()
            .find(|g: &&mut LanguageGroup| g.lang == entry.lang)
        {
            g.items.push(entry);
        } else {
            groups.push(LanguageGroup {
                lang: entry.lang.clone(),
                items: vec![entry],
            });
        }
    }
    groups
}

#[tauri::command]
pub fn list_available_models(state: State<'_, AppState>) -> Vec<LanguageGroup> {
    let dir = &state.app_data_dir;
    let voices = piper_voices().iter().map(|v| {
        let installed = voice_file_path(dir, v.onnx_filename).exists()
            && voice_file_path(dir, &voice_config_filename(v)).exists();
        let bytes = if installed {
            std::fs::metadata(voice_file_path(dir, v.onnx_filename))
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };
        (v, installed, bytes)
    });
    let models = whisper_models().iter().map(|m| {
        let p = model_path(dir, m.filename);
        let installed = p.exists();
        let bytes = if installed {
            std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };
        (m, installed, bytes)
    });
    group_available_models(voices, models)
}

#[tauri::command]
pub fn list_voice_notes(
    state: State<'_, AppState>,
    project_id: String,
) -> CmdResult<Vec<MediaAsset>> {
    state.storage.list_voice_notes(&project_id)
}

#[tauri::command]
pub fn delete_voice_note(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.media.delete(&id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_voices_and_models_by_language() {
        use crate::services::voice::{PiperVoiceInfo, WhisperModelInfo};

        let voice = PiperVoiceInfo {
            id: "es_ES-carlfm",
            name: "Carl (es)",
            lang: "es",
            onnx_filename: "es_ES-carlfm.onnx",
            onnx_url: "",
            size_mb: 42,
            recommended: false,
        };
        let model = WhisperModelInfo {
            id: "base",
            filename: "ggml-base.bin",
            size_mb: 142,
            recommended: false,
            sha256: None,
        };
        let groups = group_available_models(
            [(&voice, true, 44000)].into_iter(),
            [(&model, false, 0)].into_iter(),
        );

        assert_eq!(groups.len(), 2);
        let es = groups.iter().find(|g| g.lang == "es").unwrap();
        assert_eq!(es.items.len(), 1);
        assert_eq!(es.items[0].id, "es_ES-carlfm");
        assert_eq!(es.items[0].installed, true);
        assert_eq!(es.items[0].disk_bytes, 44000);
        assert_eq!(es.items[0].kind, "voice");

        let other = groups.iter().find(|g| g.lang == "other").unwrap();
        assert_eq!(other.items.len(), 1);
        assert_eq!(other.items[0].id, "base");
        assert_eq!(other.items[0].installed, false);
        assert_eq!(other.items[0].kind, "model");
    }

    #[test]
    fn empty_input_returns_empty_groups() {
        let groups = group_available_models(std::iter::empty(), std::iter::empty());
        assert!(groups.is_empty());
    }

    #[test]
    fn models_from_same_language_group_together() {
        use crate::services::voice::PiperVoiceInfo;

        let v1 = PiperVoiceInfo {
            id: "en_US-amy-medium",
            name: "Amy (en)",
            lang: "en",
            onnx_filename: "en_US-amy-medium.onnx",
            onnx_url: "",
            size_mb: 63,
            recommended: false,
        };
        let v2 = PiperVoiceInfo {
            id: "en_GB-alan-medium",
            name: "Alan (en)",
            lang: "en",
            onnx_filename: "en_GB-alan-medium.onnx",
            onnx_url: "",
            size_mb: 60,
            recommended: false,
        };
        let groups = group_available_models(
            [(&v1, false, 0), (&v2, false, 0)].into_iter(),
            std::iter::empty(),
        );
        let en = groups.iter().find(|g| g.lang == "en").unwrap();
        assert_eq!(en.items.len(), 2);
        assert_eq!(en.items[0].id, "en_US-amy-medium");
        assert_eq!(en.items[1].id, "en_GB-alan-medium");
    }

    #[test]
    fn encode_wav_pcm16_produces_valid_header() {
        let samples = vec![0i16, 100, -100, 32767];
        let wav = encode_wav_pcm16(&samples, 22050);
        // RIFF header
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        // PCM, 1 channel, 22050 Hz, 16 bit
        assert_eq!(&wav[20..22], &1u16.to_le_bytes());
        assert_eq!(&wav[22..24], &1u16.to_le_bytes());
        assert_eq!(&wav[24..28], &22050u32.to_le_bytes());
        assert_eq!(&wav[34..36], &16u16.to_le_bytes());
        // data chunk
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(&wav[40..44], &(samples.len() as u32 * 2).to_le_bytes());
        // data (8 bytes for 4 samples)
        assert_eq!(wav.len(), 44 + 8);
    }

    #[test]
    fn encode_wav_pcm16_empty_samples() {
        let wav = encode_wav_pcm16(&[], 16000);
        assert_eq!(wav.len(), 44); // header only
        assert_eq!(&wav[40..44], &0u32.to_le_bytes());
    }

    #[test]
    fn encode_wav_pcm16_sample_values_are_binary_exact() {
        let samples = vec![1i16, -1, 0, 32767];
        let wav = encode_wav_pcm16(&samples, 44100);
        // After the 44-byte header, each sample is 2 bytes LE
        assert_eq!(wav[44], 1);
        assert_eq!(wav[45], 0);
        assert_eq!(wav[46], 0xFF);
        assert_eq!(wav[47], 0xFF);
        assert_eq!(wav[48], 0);
        assert_eq!(wav[49], 0);
        assert_eq!(wav[50], 0xFF);
        assert_eq!(wav[51], 0x7F);
    }

    #[test]
    fn disk_usage_entry_serializes_camel_case() {
        let entry = DiskUsageEntry {
            id: "test".into(),
            bytes: 12345,
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["id"], "test");
        assert_eq!(json["bytes"], 12345);
    }
}
