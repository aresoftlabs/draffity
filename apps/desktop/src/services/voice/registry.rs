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
// Binary info for Piper TTS (auto-download). The URLs below point to the
// latest stable release archives per platform.
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

const BINARY_INFOS: &[BinaryInfo] = &[BinaryInfo {
    id: "piper",
    name: "Piper",
    win_url:
        "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_windows_amd64.zip",
    linux_url:
        "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz",
    size_mb: 8,
}];

// ---------------------------------------------------------------------------
// Whisper binary matrix — per-backend pre-built archives from our own CI
// release. Task 4 will wire download_and_extract_binary to this matrix.
// ---------------------------------------------------------------------------

use crate::services::voice::accel::Backend;

/// Base de descarga: binarios compilados por nuestro CI, publicados en un
/// release propio. Pinear el tag al release vigente.
pub const WHISPER_BINS_BASE: &str =
    "https://github.com/arezouski/draffity/releases/download/whisper-bins-v1/";

#[derive(Debug, Clone)]
pub struct WhisperBinary {
    pub archive: String,
    pub url: String,
    pub sha256: Option<&'static str>,
}

/// sha256 de cada archivo publicado en el release `whisper-bins-v1` (tomados
/// de los sidecars `.sha256` que produce el CI). `None` para archivos no
/// publicados — el downloader tolera `None` (descarga sin verificar).
fn archive_sha256(archive: &str) -> Option<&'static str> {
    match archive {
        "whisper-linux-x86_64-cpu.tar.gz" => {
            Some("ba9ebe33f55c2ab3fdde8b0a159b9b34f360e34bf7e5af8a0154d69446de3ec8")
        }
        "whisper-linux-x86_64-vulkan.tar.gz" => {
            Some("6d4cdea4bfb78a4dcd8885bd8df84db376284822a61e7b4e5bf986a24d7ae9cb")
        }
        "whisper-macos-aarch64-metal.tar.gz" => {
            Some("8301e269507e6f37351edc379ee030d4f398898724044154f2ec95f24b0861d0")
        }
        "whisper-windows-x86_64-cpu.zip" => {
            Some("459285ca35326498fe8a3813fb562a956ef0f18ef9031fcfeaaa15f9a81053db")
        }
        "whisper-windows-x86_64-vulkan.zip" => {
            Some("5cb6a6ed60cb57b885e36e05e74c44cb2cdfcd742373b74da3c2effd615f080e")
        }
        _ => None,
    }
}

/// Binario whisper para `(os, arch, backend)`. `None` para combos no soportados.
/// Archivo: `whisper-<os>-<arch>-<backend>.<ext>` (zip en Windows, tar.gz resto).
pub fn whisper_binary(os: &str, arch: &str, backend: Backend) -> Option<WhisperBinary> {
    let supported = matches!(
        (os, backend),
        ("macos", Backend::Metal)
            | ("windows", Backend::Vulkan)
            | ("windows", Backend::Cpu)
            | ("linux", Backend::Vulkan)
            | ("linux", Backend::Cpu)
    );
    if !supported {
        return None;
    }
    let ext = if os == "windows" { "zip" } else { "tar.gz" };
    let archive = format!("whisper-{os}-{arch}-{}.{ext}", backend.as_str());
    let sha256 = archive_sha256(&archive);
    Some(WhisperBinary {
        url: format!("{WHISPER_BINS_BASE}{archive}"),
        archive,
        sha256,
    })
}

// ---------------------------------------------------------------------------
// VAD model (Voice Activity Detection — Silero v5).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct VadModelInfo {
    pub filename: &'static str,
    pub url: &'static str,
}

/// Modelo VAD Silero (pocos MB), hospedado en HF por ggml-org.
pub fn vad_model() -> VadModelInfo {
    VadModelInfo {
        filename: "silero-v5.1.2-ggml.bin",
        url: "https://huggingface.co/ggml-org/silero-v5.1.2/resolve/main/silero-v5.1.2-ggml.bin",
    }
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

    #[test]
    fn whisper_binary_matrix_resolves_per_backend() {
        use crate::services::voice::accel::Backend;
        let mac = whisper_binary("macos", "aarch64", Backend::Metal).unwrap();
        assert!(mac.archive.contains("macos") && mac.archive.contains("metal"));
        let winv = whisper_binary("windows", "x86_64", Backend::Vulkan).unwrap();
        assert!(winv.archive.contains("windows") && winv.archive.contains("vulkan"));
        let wincpu = whisper_binary("windows", "x86_64", Backend::Cpu).unwrap();
        assert!(wincpu.archive.contains("windows") && wincpu.archive.contains("cpu"));
        assert!(winv.url.starts_with(WHISPER_BINS_BASE));
        assert!(whisper_binary("freebsd", "x86_64", Backend::Cpu).is_none());
    }

    #[test]
    fn whisper_archive_naming_is_exact() {
        use crate::services::voice::accel::Backend;
        assert_eq!(
            whisper_binary("macos", "aarch64", Backend::Metal)
                .unwrap()
                .archive,
            "whisper-macos-aarch64-metal.tar.gz"
        );
        assert_eq!(
            whisper_binary("windows", "x86_64", Backend::Vulkan)
                .unwrap()
                .archive,
            "whisper-windows-x86_64-vulkan.zip"
        );
    }

    #[test]
    fn vad_model_is_silero() {
        let v = vad_model();
        assert_eq!(v.filename, "silero-v5.1.2-ggml.bin");
        assert!(v.url.ends_with(v.filename));
    }

    #[test]
    fn macos_without_metal_is_unsupported() {
        use crate::services::voice::accel::Backend;
        // Intel Mac (x86_64 → Cpu) no tiene binario: error limpio, no 404.
        assert!(whisper_binary("macos", "x86_64", Backend::Cpu).is_none());
        assert!(whisper_binary("macos", "aarch64", Backend::Metal).is_some());
    }

    #[test]
    fn published_variants_have_pinned_sha256() {
        use crate::services::voice::accel::Backend;
        // Las 5 variantes del release `whisper-bins-v1` tienen checksum fijado.
        for (os, arch, backend) in [
            ("macos", "aarch64", Backend::Metal),
            ("windows", "x86_64", Backend::Cpu),
            ("windows", "x86_64", Backend::Vulkan),
            ("linux", "x86_64", Backend::Cpu),
            ("linux", "x86_64", Backend::Vulkan),
        ] {
            let b = whisper_binary(os, arch, backend).unwrap();
            let sha = b.sha256.unwrap_or("");
            assert_eq!(sha.len(), 64, "{} debe tener sha256 de 64 hex", b.archive);
            assert!(sha.bytes().all(|c| c.is_ascii_hexdigit()));
        }
    }
}
