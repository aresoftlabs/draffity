//! Voice runtime commands (Épica H, slice 1): status, model catalogue,
//! opt-in model download (with progress events), delete, and importing a
//! user-provided whisper binary.
//!
//! The actual transcription command lands with the dictation UI (slice 2);
//! here we manage the assets the ASR depends on.
//!
//! All path resolution goes through `state.resources` (DraffityHome).

use std::path::PathBuf;

use tauri::{AppHandle, Emitter, State};

use crate::domain::{now_ms, MediaAsset};
use crate::error::AppError;
use crate::services::tts::SynthesizedAudio;
use crate::services::voice::catalog::{
    build_catalog, load_cached_or_seed, refresh_manifest_cache, CatalogLang,
};
use crate::services::voice::stream::StreamEvent;
use crate::services::voice::{
    download_and_extract_binary, download_and_extract_whisper, download_to_file, model_by_id,
    model_url, piper_voices, voice_by_id, voice_config_filename, whisper_models,
};
use crate::services::DraffityHome;
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

#[derive(serde::Serialize, Clone)]
struct TranscribeProgress {
    progress: u8,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccelStatus {
    /// "metal" | "vulkan" | "cpu"
    pub backend: String,
    /// Id del modelo activo (o null si no hay instalado).
    pub model: Option<String>,
    /// ¿Hay binario whisper-server instalado (motor rápido disponible)?
    pub server_available: bool,
}

#[tauri::command]
pub fn get_accel_status(state: State<'_, AppState>) -> AccelStatus {
    use crate::services::voice::accel::detect_backend;
    AccelStatus {
        backend: detect_backend().as_str().to_string(),
        model: active_model_id(&state),
        server_available: state.whisper_server.available(),
    }
}

#[tauri::command]
pub fn get_voice_status(state: State<'_, AppState>) -> VoiceStatus {
    VoiceStatus {
        dictation_available: state.asr.available(),
        binary_installed: state.resources.bin_dir().exists(),
        installed_models: installed_models_on_disk(&state.resources),
        tts_available: state.tts.available(),
        piper_installed: state.resources.piper_bin_path().exists(),
    }
}

/// List filenames of installed (present on disk) whisper models, in registry order.
fn installed_models_on_disk(home: &DraffityHome) -> Vec<String> {
    whisper_models()
        .iter()
        .filter(|m| home.model_path(m.filename).exists())
        .map(|m| m.filename.to_string())
        .collect()
}

#[tauri::command]
pub fn list_voice_voices(state: State<'_, AppState>) -> Vec<VoiceVoiceDto> {
    piper_voices()
        .iter()
        .map(|v| {
            let installed = state.resources.voice_file_path(v.onnx_filename).exists()
                && state
                    .resources
                    .voice_file_path(&voice_config_filename(v))
                    .exists();
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
    whisper_models()
        .iter()
        .map(|m| VoiceModelDto {
            id: m.id.to_string(),
            filename: m.filename.to_string(),
            size_mb: m.size_mb,
            recommended: m.recommended,
            installed: state.resources.model_path(m.filename).exists(),
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
    let dest = state.resources.model_path(model.filename);
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

/// Download a binary (whisper.cpp or Piper) from GitHub releases, extract
/// the executable, and place it in the voice bin directory.
/// Emits `voice.download.progress` events using the binary id as the model_id.
#[tauri::command]
pub async fn download_voice_binary(
    app: AppHandle,
    state: State<'_, AppState>,
    binary_id: String,
) -> CmdResult<()> {
    if binary_id != "whisper" && binary_id != "piper" {
        return Err(AppError::NotFound(format!("binary {binary_id}")));
    }

    if binary_id == "whisper" {
        let backend = crate::services::voice::accel::detect_backend();
        let app2 = app.clone();
        let id = binary_id.clone();
        let home = DraffityHome::with_root(state.resources.root().to_path_buf());
        return tauri::async_runtime::spawn_blocking(move || {
            download_and_extract_whisper(backend, &home, |downloaded, total| {
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
        .map_err(|e| AppError::Unexpected(format!("descarga: {e}")))?;
    }

    let app2 = app.clone();
    let id = binary_id.clone();
    let home = DraffityHome::with_root(state.resources.root().to_path_buf());

    tauri::async_runtime::spawn_blocking(move || {
        download_and_extract_binary(&id, &home, |downloaded, total| {
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
    let path = state.resources.model_path(model.filename);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// Borra los archivos de una voz instalada (`<id>.onnx` + `<id>.onnx.json`).
/// Idempotente: ausencia no es error. Helper puro para testear sin `State`.
pub fn remove_voice_files(home: &DraffityHome, voice_id: &str) -> CmdResult<()> {
    // Defensa contra path traversal: el id es el stem del archivo, nunca una ruta.
    if voice_id.is_empty()
        || voice_id.contains('/')
        || voice_id.contains('\\')
        || voice_id.contains("..")
    {
        return Err(AppError::Invariant(format!(
            "voice id inválido: {voice_id}"
        )));
    }
    let onnx = home.voice_file_path(&format!("{voice_id}.onnx"));
    let cfg = home.voice_file_path(&format!("{voice_id}.onnx.json"));
    for p in [onnx, cfg] {
        if p.exists() {
            std::fs::remove_file(&p)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn delete_voice_voice(state: State<'_, AppState>, voice_id: String) -> CmdResult<()> {
    let home = DraffityHome::with_root(state.resources.root().to_path_buf());
    remove_voice_files(&home, &voice_id)
}

/// Modelos instalados (ids cuyo `.bin` existe en el dir de modelos).
fn installed_model_ids(state: &AppState) -> Vec<&'static str> {
    crate::services::voice::whisper_models()
        .iter()
        .filter(|m| state.resources.model_path(m.filename).exists())
        .map(|m| m.id)
        .collect()
}

/// Id del modelo activo según backend + override del usuario (setting).
fn active_model_id(state: &AppState) -> Option<String> {
    use crate::services::voice::accel::{detect_backend, pick_model};
    let installed = installed_model_ids(state);
    let override_id = state.storage.get_setting("voice.asr.model").ok().flatten();
    pick_model(detect_backend(), &installed, override_id.as_deref())
}

/// Transcribe a recorded clip (16 kHz mono WAV bytes from the UI) to text.
/// If `sample_rate` is provided, `wav` is raw PCM16 samples (i16 LE) at that
/// rate — a WAV header is written first. Without it, `wav` is expected to be a
/// complete WAV file (backward-compatible with existing callers).
/// The engine is whatever `ASRService` is wired — this command only depends on
/// the trait, so swapping the ASR backend never touches it. Runs off-thread.
#[tauri::command]
pub async fn transcribe_audio(
    app: AppHandle,
    state: State<'_, AppState>,
    wav: Vec<u8>,
    sample_rate: Option<u32>,
) -> CmdResult<Transcript> {
    if !state.asr.available() {
        return Err(AppError::Unsupported(
            "el dictado no está disponible (instalá el binario y un modelo de voz)".into(),
        ));
    }
    let tmp_dir = state.resources.tmp_dir();
    std::fs::create_dir_all(&tmp_dir)?;
    let path = tmp_dir.join(format!("rec{}.wav", now_ms()));

    if let Some(sr) = sample_rate {
        write_pcm16_wav(&path, &wav, sr)?;
    } else {
        std::fs::write(&path, &wav)?;
    }

    // Modelo activo: backend-aware con override del usuario (Fase 3).
    let model = active_model_id(&state)
        .and_then(|id| crate::services::voice::model_by_id(&id).map(|m| m.filename))
        .map(|fname| state.resources.model_path(fname));
    let model_path = match model {
        Some(p) if p.exists() => Some(p),
        _ => None,
    };

    // Camino rápido: server caliente (si hay binario + VAD + modelo instalados).
    if let Some(ref model) = model_path {
        if state.whisper_server.available() && model.exists() {
            let mgr = state.whisper_server.clone();
            let wav_owned = std::fs::read(&path).unwrap_or_default();
            let model2 = model.clone();
            let res =
                tauri::async_runtime::spawn_blocking(move || mgr.transcribe(&model2, &wav_owned))
                    .await
                    .map_err(|e| AppError::Unexpected(format!("tarea server: {e}")))?;
            if let Ok(t) = res {
                let _ = std::fs::remove_file(&path);
                return Ok(t);
            }
            // Si el server falló, caer al CLI abajo (no propagar todavía).
        }
    }

    let asr = state.asr.clone();
    let path_str = path.to_string_lossy().into_owned();
    let app2 = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        asr.transcribe_file_with_progress(&path_str, &mut |p| {
            let _ = app2.emit(
                "voice.transcribe.progress",
                TranscribeProgress { progress: p },
            );
        })
    })
    .await
    .map_err(|e| AppError::Unexpected(format!("tarea de transcripción: {e}")))?;
    let _ = std::fs::remove_file(&path);
    result
}

#[derive(Clone, serde::Serialize)]
struct StreamPartial {
    text: String,
}
#[derive(Clone, serde::Serialize)]
pub(crate) struct StreamFinal {
    text: String,
    seq: u64,
}

fn emit_stream_events(app: &AppHandle, events: Vec<StreamEvent>) {
    for ev in events {
        match ev {
            StreamEvent::Partial(text) => {
                let _ = app.emit("voice.stream.partial", StreamPartial { text });
            }
            StreamEvent::Final { text, seq } => {
                let _ = app.emit("voice.stream.final", StreamFinal { text, seq });
            }
        }
    }
}

/// Inicia una sesión de dictado en streaming. `sample_rate` del audio que se
/// enviará por `dictation_stream_feed` (típicamente 16000).
#[tauri::command]
pub async fn dictation_stream_start(state: State<'_, AppState>, sample_rate: u32) -> CmdResult<()> {
    if !state.asr.available() {
        return Err(AppError::Unsupported(
            "el dictado no está disponible".into(),
        ));
    }
    state.dictation_stream.start(sample_rate);
    Ok(())
}

/// Alimenta PCM16 mono a la sesión activa y emite los eventos resultantes.
#[tauri::command]
pub async fn dictation_stream_feed(
    app: AppHandle,
    state: State<'_, AppState>,
    pcm: Vec<i16>,
) -> CmdResult<()> {
    let mgr = state.dictation_stream.clone();
    let events = tauri::async_runtime::spawn_blocking(move || mgr.feed(&pcm))
        .await
        .map_err(|e| AppError::Unexpected(format!("stream feed: {e}")))?;
    emit_stream_events(&app, events);
    Ok(())
}

/// Cierra la sesión: vacía la última frase y la devuelve como valor de retorno.
/// Los finales se retornan directamente en lugar de emitirse como eventos,
/// garantizando que el frontend no los pierda si ya se desubscribió.
#[tauri::command]
pub async fn dictation_stream_stop(state: State<'_, AppState>) -> CmdResult<Vec<StreamFinal>> {
    let mgr = state.dictation_stream.clone();
    let events = tauri::async_runtime::spawn_blocking(move || mgr.stop())
        .await
        .map_err(|e| AppError::Unexpected(format!("stream stop: {e}")))?;
    Ok(events
        .into_iter()
        .filter_map(|e| match e {
            crate::services::voice::stream::StreamEvent::Final { text, seq } => {
                Some(StreamFinal { text, seq })
            }
            crate::services::voice::stream::StreamEvent::Partial(_) => None,
        })
        .collect())
}

/// Descarta la sesión activa sin emitir nada.
#[tauri::command]
pub async fn dictation_stream_cancel(state: State<'_, AppState>) -> CmdResult<()> {
    state.dictation_stream.cancel();
    Ok(())
}

/// Write raw PCM16 samples to a WAV file with proper RIFF header.
pub fn write_pcm16_wav(
    path: &std::path::Path,
    samples: &[u8],
    sample_rate: u32,
) -> Result<(), AppError> {
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let block_align = channels * bits_per_sample / 8;
    let byte_rate = sample_rate * block_align as u32;
    let data_size = samples.len() as u32;
    let mut header = Vec::new();
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&(36 + data_size).to_le_bytes());
    header.extend_from_slice(b"WAVE");
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&16u32.to_le_bytes());
    header.extend_from_slice(&1u16.to_le_bytes()); // PCM
    header.extend_from_slice(&channels.to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&byte_rate.to_le_bytes());
    header.extend_from_slice(&block_align.to_le_bytes());
    header.extend_from_slice(&bits_per_sample.to_le_bytes());
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_size.to_le_bytes());

    let mut f = std::fs::File::create(path)?;
    use std::io::Write;
    f.write_all(&header)?;
    f.write_all(samples)?;
    Ok(())
}

/// Copy a user-provided whisper.cpp binary into the app's voice/bin dir. The
/// reliable alternative to downloading a platform release.
#[tauri::command]
pub fn import_voice_binary(state: State<'_, AppState>, source_path: String) -> CmdResult<()> {
    let src = PathBuf::from(&source_path);
    if !src.is_file() {
        return Err(AppError::NotFound(format!("binary {source_path}")));
    }
    let dest = state.resources.bin_dir();
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
    let dest = state.resources.piper_bin_path();
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(&src, &dest)?;
    Ok(())
}

/// Download a Piper voice (ONNX model + its `.onnx.json` config), emitting
/// `voice.download.progress` for the model file. Looks up the voice from the
/// manifest (cached or seed), so any manifest voice can be downloaded, not
/// just the 2 hardcoded ones. Verifies md5 when present.
#[tauri::command]
pub async fn download_voice_voice(
    app: AppHandle,
    state: State<'_, AppState>,
    voice_id: String,
) -> CmdResult<()> {
    let home = DraffityHome::with_root(state.resources.root().to_path_buf());
    let manifest = crate::services::voice::catalog::load_cached_or_seed(&home);
    let voice = crate::services::voice::catalog::voice_in_manifest(&manifest, &voice_id)
        .ok_or_else(|| AppError::NotFound(format!("voice {voice_id}")))?
        .clone();

    let onnx_dest = state.resources.voice_file_path(&format!("{voice_id}.onnx"));
    let cfg_dest = state
        .resources
        .voice_file_path(&format!("{voice_id}.onnx.json"));
    let app2 = app.clone();
    let id = voice_id.clone();

    tauri::async_runtime::spawn_blocking(move || -> CmdResult<()> {
        download_to_file(&voice.onnx_url, &onnx_dest, None, |downloaded, total| {
            let _ = app2.emit(
                "voice.download.progress",
                DownloadProgress {
                    model_id: id.clone(),
                    downloaded,
                    total,
                },
            );
        })?;
        download_to_file(&voice.config_url, &cfg_dest, None, |_, _| {})?;

        if let Some(expected) = voice.onnx_md5.as_deref() {
            let got = crate::services::voice::catalog::md5_hex(&std::fs::read(&onnx_dest)?);
            if !got.eq_ignore_ascii_case(expected) {
                let _ = std::fs::remove_file(&onnx_dest);
                let _ = std::fs::remove_file(&cfg_dest);
                return Err(AppError::Unexpected(format!("md5 mismatch en {id}")));
            }
        }
        Ok(())
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
    let result = tauri::async_runtime::spawn_blocking(move || tts.synthesize(&text, &voice_id))
        .await
        .map_err(|e| AppError::Unexpected(format!("tarea de síntesis: {e}")))?;
    result
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
    let audio_result =
        tauri::async_runtime::spawn_blocking(move || tts.synthesize(&text, &voice_id))
            .await
            .map_err(|e| AppError::Unexpected(format!("tarea de síntesis: {e}")))?;
    let audio = audio_result?;

    // Write a WAV file to a temp dir under voice/.
    let tmp = state.resources.tmp_dir();
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
        // `asset.path_relative` is stored as `media/<project>/<hash>.<ext>`;
        // resolve it relative to the media root.
        let path = state
            .resources
            .media_dir()
            .join(
                asset
                    .path_relative
                    .strip_prefix("media/")
                    .unwrap_or(&asset.path_relative),
            )
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
pub struct DiskUsageEntry {
    pub id: String,
    pub bytes: u64,
}

/// Get disk usage for installed models and voices.
#[tauri::command]
pub fn get_disk_usage(state: State<'_, AppState>) -> Vec<DiskUsageEntry> {
    let mut entries = Vec::new();

    // Piper voices
    for v in piper_voices() {
        let p = state.resources.voice_file_path(v.onnx_filename);
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
        let p = state.resources.model_path(m.filename);
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

/// Voces TTS instaladas (id ⇒ `<id>.onnx` y `<id>.onnx.json` presentes).
fn installed_voice_ids(
    state: &AppState,
    m: &crate::services::voice::registry::VoiceManifest,
) -> std::collections::HashSet<String> {
    m.voices
        .iter()
        .filter(|v| {
            state
                .resources
                .voice_file_path(&format!("{}.onnx", v.id))
                .exists()
                && state
                    .resources
                    .voice_file_path(&format!("{}.onnx.json", v.id))
                    .exists()
        })
        .map(|v| v.id.clone())
        .collect()
}

#[tauri::command]
pub async fn get_voice_catalog(state: State<'_, AppState>) -> CmdResult<Vec<CatalogLang>> {
    let home = DraffityHome::with_root(state.resources.root().to_path_buf());
    let home2 = DraffityHome::with_root(home.root().to_path_buf());
    if let Err(e) =
        tauri::async_runtime::spawn_blocking(move || refresh_manifest_cache(&home2)).await
    {
        tracing::warn!(error = %e, "voice manifest refresh task failed");
    }
    let manifest = load_cached_or_seed(&home);
    let installed = installed_voice_ids(&state, &manifest);
    Ok(build_catalog(&manifest, &installed))
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

    #[test]
    fn write_pcm16_wav_produces_valid_wav() {
        use std::io::Read;
        let tmp = std::env::temp_dir().join(format!("test_write_pcm16_{}", now_ms()));
        let samples: Vec<u8> = vec![0u8, 0, 0x64, 0, 0x9C, 0xFF, 0xFF, 0x7F]; // 0, 100, -100, 32767 (i16 LE)
        write_pcm16_wav(&tmp, &samples, 44100).unwrap();

        let mut file_bytes = Vec::new();
        std::fs::File::open(&tmp)
            .unwrap()
            .read_to_end(&mut file_bytes)
            .unwrap();

        // RIFF header
        assert_eq!(&file_bytes[0..4], b"RIFF");
        assert_eq!(&file_bytes[8..12], b"WAVE");
        assert_eq!(&file_bytes[12..16], b"fmt ");
        // PCM, 1 channel, 44100 Hz, 16 bit
        assert_eq!(&file_bytes[20..22], &1u16.to_le_bytes());
        assert_eq!(&file_bytes[22..24], &1u16.to_le_bytes());
        assert_eq!(&file_bytes[24..28], &44100u32.to_le_bytes());
        assert_eq!(&file_bytes[34..36], &16u16.to_le_bytes());
        // data chunk
        assert_eq!(&file_bytes[36..40], b"data");
        assert_eq!(&file_bytes[40..44], &(samples.len() as u32).to_le_bytes());
        // Total size: 44 header + 8 samples
        assert_eq!(file_bytes.len(), 44 + 8);
        // Sample bytes match
        assert_eq!(&file_bytes[44..], &samples[..]);

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn write_pcm16_wav_empty_samples() {
        use std::io::Read;
        let tmp = std::env::temp_dir().join(format!("test_write_pcm16_empty_{}", now_ms()));
        write_pcm16_wav(&tmp, &[], 16000).unwrap();

        let mut file_bytes = Vec::new();
        std::fs::File::open(&tmp)
            .unwrap()
            .read_to_end(&mut file_bytes)
            .unwrap();

        assert_eq!(file_bytes.len(), 44); // header only
        assert_eq!(&file_bytes[40..44], &0u32.to_le_bytes());

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn delete_voice_voice_removes_onnx_and_config() {
        let dir = tempfile::tempdir().unwrap();
        let home = crate::services::DraffityHome::with_root(dir.path().to_path_buf());
        let onnx = home.voice_file_path("es_ES-davefx-medium.onnx");
        let cfg = home.voice_file_path("es_ES-davefx-medium.onnx.json");
        std::fs::create_dir_all(onnx.parent().unwrap()).unwrap();
        std::fs::write(&onnx, b"x").unwrap();
        std::fs::write(&cfg, b"{}").unwrap();

        super::remove_voice_files(&home, "es_ES-davefx-medium").unwrap();

        assert!(!onnx.exists());
        assert!(!cfg.exists());
    }

    #[test]
    fn delete_voice_voice_is_idempotent_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let home = crate::services::DraffityHome::with_root(dir.path().to_path_buf());
        super::remove_voice_files(&home, "en_US-amy-medium").unwrap();
    }

    #[test]
    fn remove_voice_files_rejects_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let home = crate::services::DraffityHome::with_root(dir.path().to_path_buf());
        assert!(super::remove_voice_files(&home, "../evil").is_err());
        assert!(super::remove_voice_files(&home, "a/b").is_err());
    }
}
