//! Streaming downloader for voice assets (models / binary). Reports progress,
//! verifies an optional SHA-256, and writes atomically via a `.part` temp file
//! so a cancelled/failed download never leaves a corrupt asset in place.
//!
//! Uses `reqwest::blocking` — callers MUST run this off the main thread
//! (`spawn_blocking`); driving the blocking client inside the Tokio runtime
//! panics.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

const BUF: usize = 64 * 1024;
/// HTTP timeout for an asset download — generous: large voice models stream
/// for a long time over slow links.
const DOWNLOAD_TIMEOUT_SECS: u64 = 7200;

/// Lowercase hex of a SHA-256 digest. Exposed for tests + reuse.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex(&h.finalize())
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Download `url` to `dest`, calling `on_progress(downloaded, total)` as bytes
/// arrive (`total` is `None` when the server omits Content-Length). Verifies
/// `expected_sha256` if given. Atomic: streams to `dest.part` then renames.
pub fn download_to_file(
    url: &str,
    dest: &Path,
    expected_sha256: Option<&str>,
    mut on_progress: impl FnMut(u64, Option<u64>),
) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .build()
        .map_err(|e| AppError::Unexpected(format!("cliente de descarga: {e}")))?;
    let mut resp = client
        .get(url)
        .send()
        .map_err(|e| AppError::Unexpected(format!("descarga fallida: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::Unexpected(format!(
            "descarga devolvió {}",
            resp.status()
        )));
    }

    let total = resp.content_length();
    let tmp = dest.with_extension("part");
    let mut file = File::create(&tmp)?;
    let mut hasher = Sha256::new();
    let mut downloaded = 0u64;
    let mut buf = vec![0u8; BUF];
    loop {
        let n = resp
            .read(&mut buf)
            .map_err(|e| AppError::Unexpected(format!("lectura de descarga: {e}")))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        hasher.update(&buf[..n]);
        downloaded += n as u64;
        on_progress(downloaded, total);
    }
    file.flush()?;
    drop(file);

    if let Some(expected) = expected_sha256 {
        let got = hex(&hasher.finalize());
        if !got.eq_ignore_ascii_case(expected) {
            let _ = std::fs::remove_file(&tmp);
            return Err(AppError::Unexpected(
                "el checksum del archivo descargado no coincide".into(),
            ));
        }
    } else {
        tracing::warn!(url, "downloaded voice asset without checksum verification");
    }

    std::fs::rename(&tmp, dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_known_vector() {
        // SHA-256("abc")
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_empty() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
