//! Catalogue of downloadable Whisper models (H-02). The ggml models live on
//! Hugging Face under a stable path, so the URL is derived from the filename.
//!
//! `sha256` is `None` for now: real checksums must be filled by the owner
//! before shipping (we can't compute them here without fetching). The
//! downloader verifies when a checksum is present and logs a warning when it
//! isn't — downloads still work, integrity-checked once the values are set.

pub const HF_BASE: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/";

#[derive(Debug, Clone, Copy)]
pub struct WhisperModelInfo {
    /// Stable id used by the UI / commands.
    pub id: &'static str,
    pub filename: &'static str,
    pub size_mb: u32,
    /// Best performance/accuracy default (backlog decision #1).
    pub recommended: bool,
    /// Filled by the owner before shipping; verified when present.
    pub sha256: Option<&'static str>,
}

/// The offered models, smallest → most capable. `large-v3-turbo-q5_0` is the
/// recommended default: near `large-v3` accuracy at a fraction of the cost.
pub fn whisper_models() -> &'static [WhisperModelInfo] {
    &[
        WhisperModelInfo {
            id: "tiny",
            filename: "ggml-tiny.bin",
            size_mb: 75,
            recommended: false,
            sha256: None,
        },
        WhisperModelInfo {
            id: "base",
            filename: "ggml-base.bin",
            size_mb: 142,
            recommended: false,
            sha256: None,
        },
        WhisperModelInfo {
            id: "small",
            filename: "ggml-small.bin",
            size_mb: 466,
            recommended: false,
            sha256: None,
        },
        WhisperModelInfo {
            id: "large-v3-turbo-q5_0",
            filename: "ggml-large-v3-turbo-q5_0.bin",
            size_mb: 547,
            recommended: true,
            sha256: None,
        },
    ]
}

pub fn model_by_id(id: &str) -> Option<&'static WhisperModelInfo> {
    whisper_models().iter().find(|m| m.id == id)
}

pub fn model_url(m: &WhisperModelInfo) -> String {
    format!("{HF_BASE}{}", m.filename)
}

/// The recommended default model, if the catalogue marks one.
pub fn recommended_model() -> Option<&'static WhisperModelInfo> {
    whisper_models().iter().find(|m| m.recommended)
}

// ---------------------------------------------------------------------------
// Piper TTS voices (H-05). Each voice is an ONNX model + its `.onnx.json`
// config, hosted on the rhasspy/piper-voices repo. The config filename is the
// model filename + ".json"; both are downloaded together.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct PiperVoiceInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub lang: &'static str,
    pub onnx_filename: &'static str,
    /// Full URL of the ONNX model (config URL is this + ".json").
    pub onnx_url: &'static str,
    pub size_mb: u32,
    pub recommended: bool,
}

pub fn piper_voices() -> &'static [PiperVoiceInfo] {
    &[
        PiperVoiceInfo {
            id: "es_ES-davefx-medium",
            name: "Dave (es)",
            lang: "es",
            onnx_filename: "es_ES-davefx-medium.onnx",
            onnx_url: "https://huggingface.co/rhasspy/piper-voices/resolve/main/es/es_ES/davefx/medium/es_ES-davefx-medium.onnx",
            size_mb: 63,
            recommended: true,
        },
        PiperVoiceInfo {
            id: "en_US-amy-medium",
            name: "Amy (en)",
            lang: "en",
            onnx_filename: "en_US-amy-medium.onnx",
            onnx_url: "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/amy/medium/en_US-amy-medium.onnx",
            size_mb: 63,
            recommended: false,
        },
    ]
}

pub fn voice_by_id(id: &str) -> Option<&'static PiperVoiceInfo> {
    piper_voices().iter().find(|v| v.id == id)
}

/// Config (`.onnx.json`) filename for a voice.
pub fn voice_config_filename(v: &PiperVoiceInfo) -> String {
    format!("{}.json", v.onnx_filename)
}

pub fn recommended_voice() -> Option<&'static PiperVoiceInfo> {
    piper_voices().iter().find(|v| v.recommended)
}

// ---------------------------------------------------------------------------
// Binary info for whisper.cpp and Piper (auto-download). The URLs below point
// to the latest stable release archives per platform.
// ---------------------------------------------------------------------------

pub struct BinaryInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub win_url: &'static str,
    pub linux_url: &'static str,
    pub size_mb: u32,
}

pub fn binary_info(name: &str) -> Option<&'static BinaryInfo> {
    BINARY_INFOS.iter().find(|b| b.id == name)
}

const BINARY_INFOS: &[BinaryInfo] = &[
    BinaryInfo {
        id: "whisper",
        name: "whisper.cpp",
        win_url: "https://github.com/ggml-org/whisper.cpp/releases/download/v1.8.6/whisper-bin-x64.zip",
        linux_url: "https://github.com/ggml-org/whisper.cpp/releases/download/v1.8.6/whisper-bin-x64.zip",
        size_mb: 30,
    },
    BinaryInfo {
        id: "piper",
        name: "Piper",
        win_url: "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_windows_amd64.zip",
        linux_url: "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz",
        size_mb: 8,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_one_recommended_model() {
        assert_eq!(whisper_models().iter().filter(|m| m.recommended).count(), 1);
        assert_eq!(recommended_model().unwrap().id, "large-v3-turbo-q5_0");
    }

    #[test]
    fn lookup_and_url() {
        let m = model_by_id("base").unwrap();
        assert_eq!(m.filename, "ggml-base.bin");
        assert_eq!(
            model_url(m),
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
        );
        assert!(model_by_id("nope").is_none());
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<_> = whisper_models().iter().map(|m| m.id).collect();
        ids.sort_unstable();
        let len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), len, "duplicate model ids");
    }

    #[test]
    fn binary_info_lookup_whisper() {
        let info = binary_info("whisper").unwrap();
        assert_eq!(info.id, "whisper");
        assert_eq!(info.name, "whisper.cpp");
        assert!(info.win_url.contains("whisper.cpp"));
        assert!(info.linux_url.contains("whisper.cpp"));
        assert!(info.size_mb > 0);
    }

    #[test]
    fn binary_info_lookup_piper() {
        let info = binary_info("piper").unwrap();
        assert_eq!(info.id, "piper");
        assert_eq!(info.name, "Piper");
        assert!(info.win_url.contains("piper"));
        assert!(info.linux_url.contains("piper"));
        assert!(info.size_mb > 0);
    }

    #[test]
    fn binary_info_nonexistent() {
        assert!(binary_info("nope").is_none());
    }
}
