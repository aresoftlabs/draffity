//! Local voice runtime (Épica H). The whisper.cpp binary + ggml models live
//! under `<app_data>/voice/`, downloaded opt-in (nothing ships in the
//! installer — see backlog-v4 decision #1). We run the binary directly via
//! `std::process` (not Tauri's `externalBin`, which would require bundling and
//! break `tauri build` when the files are absent).
//!
//! Everything here degrades gracefully: with no binary/model installed,
//! `WhisperLocalASR::available()` is `false` and the UI offers nothing.

use std::path::{Path, PathBuf};

pub mod download;
pub mod piper;
pub mod registry;
pub mod whisper;

pub use download::download_to_file;
pub use piper::{parse_wav_pcm16, PiperTTSService};
pub use registry::{
    binary_info, model_by_id, model_url, piper_voices, recommended_voice, voice_by_id,
    voice_config_filename, whisper_models, BinaryInfo, PiperVoiceInfo, WhisperModelInfo,
};
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

fn piper_binary_name() -> &'static str {
    if cfg!(windows) {
        "piper.exe"
    } else {
        "piper"
    }
}

pub fn piper_bin_path(app_data: &Path) -> PathBuf {
    voice_dir(app_data).join("bin").join(piper_binary_name())
}

pub fn voices_dir(app_data: &Path) -> PathBuf {
    voice_dir(app_data).join("voices")
}

pub fn voice_file_path(app_data: &Path, filename: &str) -> PathBuf {
    voices_dir(app_data).join(filename)
}

/// Filenames of installed (present on disk) whisper models, in registry order.
pub fn installed_models(app_data: &Path) -> Vec<String> {
    whisper_models()
        .iter()
        .filter(|m| model_path(app_data, m.filename).exists())
        .map(|m| m.filename.to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Binary extraction — download an archive and find the executable inside.
// ---------------------------------------------------------------------------

/// Download a binary archive and extract the executable to the target path.
/// The archive URL is looked up from the binary registry by id.
/// Emits progress via the provided callback.
pub fn download_and_extract_binary(
    binary_id: &str,
    app_data: &Path,
    on_progress: impl FnMut(u64, Option<u64>),
) -> crate::error::AppResult<()> {
    use crate::error::AppError;
    use registry::binary_info;

    let info =
        binary_info(binary_id).ok_or_else(|| AppError::NotFound(format!("binary {binary_id}")))?;

    let url = if cfg!(windows) {
        info.win_url
    } else {
        info.linux_url
    };

    // Determine target path for the binary.
    let target = if binary_id == "piper" {
        piper_bin_path(app_data)
    } else {
        bin_path(app_data)
    };

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Download archive to a temp file.
    let tmp_dir = voice_dir(app_data).join("tmp");
    std::fs::create_dir_all(&tmp_dir)?;
    let archive_path = tmp_dir.join(format!("{binary_id}_archive"));

    download_to_file(url, &archive_path, None, on_progress)?;

    // Determine format and extract.
    let format = if url.ends_with(".tar.gz") {
        ".tar.gz"
    } else if url.ends_with(".zip") {
        ".zip"
    } else {
        return Err(AppError::Unexpected(format!(
            "formato de archivo no soportado: {url}"
        )));
    };

    let archive_bytes = std::fs::read(&archive_path)?;
    std::fs::remove_file(&archive_path)?;

    extract_binary_impl(&archive_bytes, format, &target, binary_id)
}

/// Extract a binary from archive bytes (zip or tar.gz) to the target path.
fn extract_binary_impl(
    archive_bytes: &[u8],
    format: &str,
    target: &Path,
    binary_id: &str,
) -> crate::error::AppResult<()> {
    use crate::error::AppError;

    // Determine possible filenames the binary could have inside the archive.
    // whisper.cpp releases ship the CLI as `whisper-cli` not `whisper`.
    let binary_names: &[&str] = match binary_id {
        "piper" => {
            if cfg!(windows) {
                &["piper.exe"]
            } else {
                &["piper"]
            }
        }
        _ => {
            if cfg!(windows) {
                &["whisper.exe", "whisper-cli.exe"]
            } else {
                &["whisper", "whisper-cli"]
            }
        }
    };

    let target_base = target
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::Unexpected("ruta destino inválida".into()))?;

    match format {
        ".zip" => {
            let cursor = std::io::Cursor::new(archive_bytes);
            let mut archive = zip::ZipArchive::new(cursor)
                .map_err(|e| AppError::Unexpected(format!("zip: {e}")))?;

            // Log entries for debugging
            let all_names: Vec<String> = (0..archive.len())
                .filter_map(|i| archive.by_index(i).ok().map(|e| e.name().to_string()))
                .collect();
            tracing::debug!(?all_names, "zip entries for {binary_id}");

            let target_dir = target.parent().unwrap_or(target);
            std::fs::create_dir_all(target_dir)?;

            let mut main_extracted = false;
            let mut extra_files = Vec::new();

            for i in 0..archive.len() {
                let mut entry = archive
                    .by_index(i)
                    .map_err(|e| AppError::Unexpected(format!("entrada zip: {e}")))?;
                let name = entry.name().to_string();

                // whisper.cpp releases ship nested under Release/ — strip the directory
                let file_name = std::path::Path::new(&name)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                // Main binary matches (whisper.exe/whisper-cli.exe or piper.exe/piper)
                let is_main = binary_names.contains(&file_name.as_str());

                if is_main && !main_extracted {
                    // Copy to target path — may rename (e.g. whisper-cli.exe → whisper.exe)
                    let mut out = std::fs::File::create(target)?;
                    std::io::copy(&mut entry, &mut out)?;
                    main_extracted = true;
                    continue;
                }

                // Extract sibling DLLs / shared libs next to the binary
                let ext = std::path::Path::new(&file_name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                if matches!(ext, "dll" | "pdb" | "so" | "dylib") {
                    let dest = target_dir.join(&file_name);
                    let mut out = std::fs::File::create(&dest)?;
                    std::io::copy(&mut entry, &mut out)?;
                    extra_files.push(file_name);
                }
            }

            if main_extracted {
                tracing::debug!(extra = ?extra_files, "extracted binary to {}", target.display());
                return Ok(());
            }

            Err(AppError::NotFound(format!(
                "binario '{target_base}' no encontrado en el archivo zip"
            )))
        }
        ".tar.gz" => {
            let decoder = flate2::read::GzDecoder::new(archive_bytes);
            let mut archive = tar::Archive::new(decoder);

            for entry in archive
                .entries()
                .map_err(|e| AppError::Unexpected(format!("tar: {e}")))?
            {
                let mut entry =
                    entry.map_err(|e| AppError::Unexpected(format!("entrada tar: {e}")))?;
                let name = entry
                    .path()
                    .map_err(|e| AppError::Unexpected(format!("ruta tar: {e}")))?
                    .to_string_lossy()
                    .into_owned();
                let file_name = std::path::Path::new(&name)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                let matches = binary_names.contains(&file_name);
                if !matches {
                    continue;
                }

                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out = std::fs::File::create(target)?;
                std::io::copy(&mut entry, &mut out)?;
                return Ok(());
            }

            Err(AppError::NotFound(format!(
                "binario '{target_base}' no encontrado en el archivo tar.gz"
            )))
        }
        _ => Err(AppError::Unexpected(format!(
            "formato de archivo no soportado: {format}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: building a minimal zip in memory.
    fn build_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::{Cursor, Write};
        let buf = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buf);
        let opts = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, data) in files {
            zip.start_file(*name, opts).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn extract_binary_finds_exe_in_zip_root() {
        let zip_bytes = build_zip(&[
            ("whisper.exe", &[0x4D, 0x5A, 0x90]),
            ("readme.txt", b"info"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("whisper.exe");
        let result = (|| -> crate::error::AppResult<()> {
            extract_binary_impl(&zip_bytes, ".zip", &target, "whisper")?;
            Ok(())
        })();
        assert!(result.is_ok());
        assert!(target.exists());
        assert_eq!(std::fs::read(&target).unwrap(), vec![0x4D, 0x5A, 0x90]);
    }

    #[test]
    fn extract_binary_finds_nested_exe() {
        // Simulate nested dir structure like whisper.cpp-1.8.6/build/bin/Release/whisper.exe
        let zip_bytes = build_zip(&[
            (
                "whisper.cpp-1.8.6/build/bin/Release/whisper.exe",
                &[0x4D, 0x5A],
            ),
            ("whisper.cpp-1.8.6/README.md", b"docs"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("whisper.exe");
        let result = (|| -> crate::error::AppResult<()> {
            extract_binary_impl(&zip_bytes, ".zip", &target, "whisper")?;
            Ok(())
        })();
        assert!(result.is_ok());
        assert!(target.exists());
        assert_eq!(std::fs::read(&target).unwrap(), vec![0x4D, 0x5A]);
    }

    #[test]
    fn extract_binary_returns_error_when_not_found() {
        let zip_bytes = build_zip(&[("readme.txt", b"no binary here")]);
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("whisper.exe");
        let result = (|| -> crate::error::AppResult<()> {
            extract_binary_impl(&zip_bytes, ".zip", &target, "whisper")?;
            Ok(())
        })();
        assert!(result.is_err());
    }

    #[test]
    fn extract_binary_from_tar_gz_finds_binary() {
        // Build a tar.gz in memory. Use platform-appropriate binary name.
        let binary_entry = if cfg!(windows) { "piper.exe" } else { "piper" };
        use std::io::Write;
        let mut tar_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            let mut header = tar::Header::new_gnu();
            header.set_path(binary_entry).unwrap();
            header.set_size(4);
            header.set_cksum();
            builder.append(&header, &b"bin\n"[..]).unwrap();
            builder.finish().unwrap();
        }
        let mut gz_bytes = Vec::new();
        {
            let mut encoder =
                flate2::write::GzEncoder::new(&mut gz_bytes, flate2::Compression::fast());
            encoder.write_all(&tar_bytes).unwrap();
            encoder.finish().unwrap();
        }

        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("piper");
        let result = (|| -> crate::error::AppResult<()> {
            extract_binary_impl(&gz_bytes, ".tar.gz", &target, "piper")?;
            Ok(())
        })();
        assert!(result.is_ok(), "extract_binary_impl failed: {:?}", result);
        assert!(target.exists());
        assert_eq!(std::fs::read(&target).unwrap(), b"bin\n");
    }

    #[test]
    fn extract_binary_from_tar_gz_returns_error_when_not_found() {
        use std::io::Write;
        // Use a name that definitely won't match piper
        let mut tar_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            let mut header = tar::Header::new_gnu();
            header.set_path("some_random_file.txt").unwrap();
            header.set_size(5);
            header.set_cksum();
            builder.append(&header, &b"hello"[..]).unwrap();
            builder.finish().unwrap();
        }
        let mut gz_bytes = Vec::new();
        {
            let mut encoder =
                flate2::write::GzEncoder::new(&mut gz_bytes, flate2::Compression::fast());
            encoder.write_all(&tar_bytes).unwrap();
            encoder.finish().unwrap();
        }

        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("piper");
        let result = (|| -> crate::error::AppResult<()> {
            extract_binary_impl(&gz_bytes, ".tar.gz", &target, "piper")?;
            Ok(())
        })();
        assert!(result.is_err());
    }
}
