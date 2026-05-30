//! Import pipeline. Mirrors the exporter shape: a thin trait + format-
//! specific renderers/parsers under it. The MVP ships Markdown; DOCX
//! lands in C-02. Both produce an `ImportTree` that the command layer
//! walks to create documents under a project — the trait stays pure (no
//! DB writes here) so importer tests can run in isolation.

mod docx;
mod markdown;

use crate::error::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportFormat {
    Markdown,
    Docx,
}

impl ImportFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Some(ImportFormat::Markdown),
            "docx" => Some(ImportFormat::Docx),
            _ => None,
        }
    }
}

/// Single node in the imported document tree. Mirrors the shape of
/// `DocumentInput` minus the project id (the caller binds the tree to a
/// project at insert time). `children` lets a Markdown H1+H2+H3 nesting
/// surface as a folder/chapter/scene tree in the binder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportNode {
    pub title: String,
    pub content_html: String,
    pub children: Vec<ImportNode>,
}

/// Result of running an importer over a payload. `project_title` is what
/// the importer thinks the document is named (first H1 / docx core props
/// / filename fallback); the UI can override before saving.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportTree {
    pub project_title: String,
    pub nodes: Vec<ImportNode>,
}

pub trait ImportService: Send + Sync {
    fn supported_formats(&self) -> Vec<ImportFormat>;

    /// Parse `bytes` into an `ImportTree`. The bytes are the raw file
    /// payload (UTF-8 for Markdown, ZIP for DOCX). `filename_hint` is the
    /// original file name without extension — used only as a fallback
    /// project title when the payload itself doesn't carry one.
    fn import(
        &self,
        format: ImportFormat,
        bytes: &[u8],
        filename_hint: &str,
    ) -> AppResult<ImportTree>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalImporter;

impl ImportService for LocalImporter {
    fn supported_formats(&self) -> Vec<ImportFormat> {
        vec![ImportFormat::Markdown, ImportFormat::Docx]
    }

    fn import(
        &self,
        format: ImportFormat,
        bytes: &[u8],
        filename_hint: &str,
    ) -> AppResult<ImportTree> {
        match format {
            ImportFormat::Markdown => markdown::import(bytes, filename_hint),
            ImportFormat::Docx => docx::import(bytes, filename_hint),
        }
    }
}
