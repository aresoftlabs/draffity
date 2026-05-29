//! Media asset registry entries. The bytes live on disk; this domain only
//! describes the catalogue row + the helper for picking a stable extension
//! from the MIME type.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAsset {
    pub id: String,
    pub project_id: String,
    /// Path relative to `<app_data>` (e.g. `media/<project>/<hash>.jpg`).
    /// Storing the relative form keeps the row portable if the app data
    /// dir ever moves (backups, future export-as-bundle).
    pub path_relative: String,
    pub mime: String,
    pub sha256: String,
    pub bytes: i64,
    pub created_at: i64,
    /// Voice-note metadata (H). `is_voice_note` distinguishes audio memos from
    /// images/fonts; duration + transcription are set when one is recorded.
    #[serde(default)]
    pub duration_ms: Option<i64>,
    #[serde(default)]
    pub transcribed_text: Option<String>,
    #[serde(default)]
    pub is_voice_note: bool,
}

/// Pick the right extension for a MIME type. Falls back to `bin` when
/// unknown — keeps the writer's clipboard paste from failing on edge
/// formats. The caller may reject unknown MIMEs at a higher layer.
pub fn extension_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "font/ttf" | "application/x-font-ttf" => "ttf",
        "font/otf" | "application/x-font-otf" => "otf",
        _ => "bin",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_image_mimes() {
        assert_eq!(extension_for_mime("image/jpeg"), "jpg");
        assert_eq!(extension_for_mime("image/png"), "png");
        assert_eq!(extension_for_mime("image/gif"), "gif");
        assert_eq!(extension_for_mime("image/webp"), "webp");
        assert_eq!(extension_for_mime("image/svg+xml"), "svg");
    }

    #[test]
    fn maps_font_mimes() {
        assert_eq!(extension_for_mime("font/ttf"), "ttf");
        assert_eq!(extension_for_mime("font/otf"), "otf");
    }

    #[test]
    fn unknown_falls_back_to_bin() {
        assert_eq!(extension_for_mime("application/octet-stream"), "bin");
        assert_eq!(extension_for_mime(""), "bin");
    }
}
