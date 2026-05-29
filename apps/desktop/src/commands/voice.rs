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
    /// Dictation usable now: premium voice + binary + a model installed.
    pub dictation_available: bool,
    pub binary_installed: bool,
    pub installed_models: Vec<String>,
    /// Read-aloud usable now: premium voice + Piper binary + a voice installed.
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
