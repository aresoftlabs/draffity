//! Local voice runtime (Épica H). The whisper.cpp binary + ggml models live
//! under `<draffity-home>/voice/`, downloaded opt-in (nothing ships in the
//! installer — see backlog-v4 decision #1). We run the binary directly via
//! `std::process` (not Tauri's `externalBin`, which would require bundling and
//! break `tauri build` when the files are absent).
//!
//! Everything here degrades gracefully: with no binary/model installed,
//! `WhisperLocalASR::available()` is `false` and the UI offers nothing.

pub mod accel;
pub mod catalog;
pub mod download;
pub mod piper;
pub mod proc;
pub mod registry;
pub mod server;
pub mod stream;
pub mod stream_manager;
pub mod stream_planner;
pub mod whisper;

pub use accel::{detect_backend, Backend};
pub use catalog::{CatalogLang, CatalogVoice};
pub use download::download_to_file;
pub use piper::{parse_wav_pcm16, PiperTTSService};
pub use registry::{
    binary_info, model_by_id, model_url, piper_voices, recommended_voice, voice_by_id,
    voice_config_filename, whisper_models, BinaryInfo, PiperVoiceInfo, WhisperModelInfo,
};
pub use server::{WhisperServer, WhisperServerManager};
pub use whisper::{autopunctuate, parse_whisper_json, WhisperLocalASR};

// ---------------------------------------------------------------------------
// Binary extraction — download an archive and find the executable inside.
// ---------------------------------------------------------------------------

use crate::services::DraffityHome;

/// Download a binary archive and extract the executable to the target path.
/// The archive URL is looked up from the binary registry by id.
/// Emits progress via the provided callback.
pub fn download_and_extract_binary(
    binary_id: &str,
    home: &DraffityHome,
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
        home.piper_bin_path()
    } else {
        home.bin_dir()
    };

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Download archive to a temp file.
    let tmp_dir = home.tmp_dir();
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

/// Descarga + extrae el binario whisper acelerado para el backend dado.
/// Reusa la extracción zip/tar de `extract_binary_impl`. En macOS quita el
/// atributo de cuarentena para que Gatekeeper no bloquee el binario descargado.
pub fn download_and_extract_whisper(
    backend: crate::services::voice::accel::Backend,
    home: &DraffityHome,
    on_progress: impl FnMut(u64, Option<u64>),
) -> crate::error::AppResult<()> {
    use crate::error::AppError;
    use registry::whisper_binary;

    let variant = whisper_binary(std::env::consts::OS, std::env::consts::ARCH, backend)
        .ok_or_else(|| AppError::Unsupported("plataforma sin binario whisper".into()))?;

    let target = home.bin_dir();
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_dir = home.tmp_dir();
    std::fs::create_dir_all(&tmp_dir)?;
    let archive_path = tmp_dir.join(&variant.archive);

    download_to_file(&variant.url, &archive_path, variant.sha256, on_progress)?;

    let format = if variant.archive.ends_with(".tar.gz") {
        ".tar.gz"
    } else {
        ".zip"
    };
    let bytes = std::fs::read(&archive_path)?;
    let _ = std::fs::remove_file(&archive_path);
    extract_binary_impl(&bytes, format, &target, "whisper")?;

    #[cfg(target_os = "macos")]
    if let Some(dir) = target.parent() {
        let _ = std::process::Command::new("xattr")
            .arg("-dr")
            .arg("com.apple.quarantine")
            .arg(dir)
            .status();
    }
    Ok(())
}

/// If a zip entry path is inside an `espeak-ng-data` directory, return the path
/// re-rooted at `espeak-ng-data/…` (e.g. `piper/espeak-ng-data/lang/es` →
/// `espeak-ng-data/lang/es`). Zip paths use forward slashes. Returns `None`
/// when the entry is unrelated to espeak data.
fn espeak_data_subpath(name: &str) -> Option<std::path::PathBuf> {
    let parts: Vec<&str> = name.split('/').filter(|s| !s.is_empty()).collect();
    let idx = parts.iter().position(|&p| p == "espeak-ng-data")?;
    Some(parts[idx..].iter().collect())
}

/// Extract a binary from archive bytes (zip or tar.gz) to the target path.
fn extract_binary_impl(
    archive_bytes: &[u8],
    format: &str,
    target: &std::path::Path,
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

                // Piper needs its `espeak-ng-data/` directory for phonemization.
                // The zip nests it under `piper/espeak-ng-data/…`; re-root the
                // whole tree next to the binary, preserving subdirectories
                // (flattening to file_name like the DLLs below would lose it).
                if binary_id == "piper" {
                    if let Some(rel) = espeak_data_subpath(&name) {
                        if entry.is_dir() {
                            std::fs::create_dir_all(target_dir.join(&rel))?;
                        } else {
                            let dest = target_dir.join(&rel);
                            if let Some(parent) = dest.parent() {
                                std::fs::create_dir_all(parent)?;
                            }
                            let mut out = std::fs::File::create(&dest)?;
                            std::io::copy(&mut entry, &mut out)?;
                        }
                        continue;
                    }
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
        // Binary name is platform-specific (whisper.exe on Windows, whisper elsewhere).
        let bin_name = if cfg!(windows) {
            "whisper.exe"
        } else {
            "whisper"
        };
        let zip_bytes = build_zip(&[(bin_name, &[0x4D, 0x5A, 0x90]), ("readme.txt", b"info")]);
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join(bin_name);
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
        // Simulate nested dir like whisper.cpp-1.8.6/build/bin/Release/whisper(.exe).
        let bin_name = if cfg!(windows) {
            "whisper.exe"
        } else {
            "whisper"
        };
        let nested = format!("whisper.cpp-1.8.6/build/bin/Release/{bin_name}");
        let zip_bytes = build_zip(&[
            (nested.as_str(), &[0x4D, 0x5A]),
            ("whisper.cpp-1.8.6/README.md", b"docs"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join(bin_name);
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
    fn espeak_data_subpath_reroots_nested_paths() {
        assert_eq!(
            espeak_data_subpath("piper/espeak-ng-data/phontab"),
            Some(std::path::PathBuf::from("espeak-ng-data/phontab"))
        );
        assert_eq!(
            espeak_data_subpath("piper/espeak-ng-data/lang/gmw/en"),
            Some(std::path::PathBuf::from("espeak-ng-data/lang/gmw/en"))
        );
        assert_eq!(espeak_data_subpath("piper/piper.exe"), None);
        assert_eq!(espeak_data_subpath("piper/espeak-ng.dll"), None);
    }

    #[test]
    fn extract_piper_zip_preserves_espeak_ng_data_tree() {
        // Piper binary is platform-specific; the sibling shared lib is copied by
        // extension regardless, so espeak-ng.dll stays a valid fixture cross-platform.
        let bin_name = if cfg!(windows) { "piper.exe" } else { "piper" };
        let bin_entry = format!("piper/{bin_name}");
        let zip_bytes = build_zip(&[
            (bin_entry.as_str(), &[0x4D, 0x5A]),
            ("piper/espeak-ng.dll", &[0x4D, 0x5A]),
            ("piper/espeak-ng-data/phontab", b"phon"),
            ("piper/espeak-ng-data/lang/es", b"es-lang"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join(bin_name);
        extract_binary_impl(&zip_bytes, ".zip", &target, "piper").unwrap();

        let bin_dir = tmp.path();
        assert!(target.exists(), "piper.exe extracted");
        assert!(bin_dir.join("espeak-ng.dll").exists(), "dll extracted");
        // espeak-ng-data tree re-rooted next to the binary, structure preserved.
        assert_eq!(
            std::fs::read(bin_dir.join("espeak-ng-data/phontab")).unwrap(),
            b"phon"
        );
        assert_eq!(
            std::fs::read(bin_dir.join("espeak-ng-data/lang/es")).unwrap(),
            b"es-lang"
        );
    }

    #[test]
    fn whisper_download_url_follows_detected_backend() {
        use crate::services::voice::accel::Backend;
        use crate::services::voice::registry::whisper_binary;
        let v = whisper_binary("windows", "x86_64", Backend::Vulkan).unwrap();
        assert_eq!(v.archive, "whisper-windows-x86_64-vulkan.zip");
        let m = whisper_binary("macos", "aarch64", Backend::Metal).unwrap();
        assert_eq!(m.archive, "whisper-macos-aarch64-metal.tar.gz");
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
