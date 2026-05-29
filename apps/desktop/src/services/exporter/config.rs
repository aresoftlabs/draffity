//! User-tunable export options. Persisted per-project in the `settings`
//! table under `export_config:<project_id>` as JSON. The exporter consumes
//! `&ExportConfig` (never `Option`) — callers that don't care pass
//! `ExportConfig::default()`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PageSize {
    #[default]
    A4,
    Letter,
    Legal,
    #[serde(rename_all = "camelCase")]
    Custom {
        width_mm: u32,
        height_mm: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Margins {
    pub top_mm: u32,
    pub right_mm: u32,
    pub bottom_mm: u32,
    pub left_mm: u32,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top_mm: 25,
            right_mm: 25,
            bottom_mm: 25,
            left_mm: 25,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SceneSeparator {
    #[default]
    Stars,
    Dashes,
    Blank,
    Custom(String),
}

impl SceneSeparator {
    /// Plain-text form used by the Markdown/EPUB/DOCX exporters as a paragraph
    /// between top-level documents.
    pub fn as_text(&self) -> &str {
        match self {
            SceneSeparator::Stars => "* * *",
            SceneSeparator::Dashes => "---",
            SceneSeparator::Blank => "",
            SceneSeparator::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ExportConfig {
    /// Overrides `project.title` on the title page / cover. `None` falls back.
    pub title_override: Option<String>,
    pub author: Option<String>,
    /// Font family hint for formats that honor it (DOCX, EPUB CSS).
    /// `None` keeps the format's default.
    pub font_family: Option<String>,
    pub page_size: PageSize,
    pub margins: Margins,
    pub include_toc: bool,
    pub include_title_page: bool,
    pub scene_separator: SceneSeparator,
    /// Absolute path to a cover image (EPUB only for now).
    pub cover_image_path: Option<String>,
    /// When true, exporters append a "Codex" appendix at the end of the
    /// document listing every entry grouped by kind. Defaults to false to
    /// avoid leaking worldbuilding into reader-facing exports unsought.
    pub include_codex: bool,
    /// When true, research documents (I-10) and their subtree are included in
    /// the export. Defaults to false — research is reference material, not
    /// reader-facing manuscript.
    pub include_research: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            title_override: None,
            author: None,
            font_family: None,
            page_size: PageSize::default(),
            margins: Margins::default(),
            include_toc: true,
            include_title_page: true,
            scene_separator: SceneSeparator::default(),
            cover_image_path: None,
            include_codex: false,
            include_research: false,
        }
    }
}

/// Settings key used to persist a per-project `ExportConfig`.
pub fn settings_key(project_id: &str) -> String {
    format!("export_config:{project_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_round_trip_stable() {
        let cfg = ExportConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: ExportConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn custom_scene_separator_round_trip() {
        let cfg = ExportConfig {
            scene_separator: SceneSeparator::Custom("~ ~ ~".into()),
            ..ExportConfig::default()
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: ExportConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.scene_separator, SceneSeparator::Custom("~ ~ ~".into()));
    }

    #[test]
    fn partial_json_uses_defaults_for_missing_fields() {
        // Backwards-compatible read: an old stored payload that only had
        // `author` should deserialize cleanly with defaults for the rest.
        let partial = r#"{"author":"Borges"}"#;
        let cfg: ExportConfig = serde_json::from_str(partial).unwrap();
        assert_eq!(cfg.author.as_deref(), Some("Borges"));
        assert_eq!(cfg.page_size, PageSize::A4);
        assert!(cfg.include_toc);
    }

    #[test]
    fn settings_key_is_namespaced_by_project() {
        assert_eq!(settings_key("abc-123"), "export_config:abc-123");
    }
}
