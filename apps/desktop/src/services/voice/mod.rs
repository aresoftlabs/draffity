//! Local voice runtime (Épica H). The whisper.cpp binary + ggml models live
//! under `<app_data>/voice/`, downloaded opt-in (nothing ships in the
//! installer — see backlog-v4 decision #1). We run the binary directly via
//! `std::process` (not Tauri's `externalBin`, which would require bundling and
//! break `tauri build` when the files are absent).
//!
//! Everything here degrades gracefully: with no binary/model installed,
//! `WhisperLocalASR::available()` is `false` and the UI offers nothing — same
//! surface as the old `NoOpASR`.

use std::path::{Path, PathBuf};

pub mod download;
pub mod registry;
pub mod whisper;

pub use download::download_to_file;
pub use registry::{model_by_id, model_url, whisper_models, WhisperModelInfo};
pub use whisper::{autopunctuate, parse_whisper_json, WhisperLocalASR};

/// Root of the voice runtime under the app data dir.
pub fn voice_dir(app_data: &Path) -> PathBuf {
    app_data.join("voice")
}

/// Platform binary name (the user-provided / downloaded whisper.cpp CLI).
fn binary_name() -> &'static str {
    if cfg!(windows) {
        "whisper.exe"
    } else {
        "whisper"
    }
}

pub fn bin_path(app_data: &Path) -> PathBuf {
    voice_dir(app_data).join("bin").join(binary_name())
}

pub fn models_dir(app_data: &Path) -> PathBuf {
    voice_dir(app_data).join("models")
}

pub fn model_path(app_data: &Path, filename: &str) -> PathBuf {
    models_dir(app_data).join(filename)
}

/// Filenames of installed (present on disk) whisper models, in registry order.
pub fn installed_models(app_data: &Path) -> Vec<String> {
    whisper_models()
        .iter()
        .filter(|m| model_path(app_data, m.filename).exists())
        .map(|m| m.filename.to_string())
        .collect()
}
