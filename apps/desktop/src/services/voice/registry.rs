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
}
