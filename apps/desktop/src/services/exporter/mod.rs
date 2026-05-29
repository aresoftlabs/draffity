//! Export pipeline. The MVP ships Markdown, DOCX and EPUB. PDF is wired into
//! the trait but returns `Unsupported` until Phase 4.5 / a future iteration.

mod config;
mod docx;
mod docx_helpers;
mod epub;
mod footnotes;
mod markdown;
mod media_bundle;
mod pdf;
mod util;

pub use media_bundle::{extract_media_ids, MediaBundle};

pub use config::{
    settings_key as export_config_settings_key, ExportConfig, Margins, PageSize, SceneSeparator,
};

use crate::domain::{CodexEntry, DocNode, Project};
use crate::error::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Markdown,
    Docx,
    Epub,
    Pdf,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "md",
            ExportFormat::Docx => "docx",
            ExportFormat::Epub => "epub",
            // PDF emits a print-ready HTML page. The UI opens it in the
            // browser/webview where the user converts to PDF via the
            // system print dialog ("Save as PDF").
            ExportFormat::Pdf => "html",
        }
    }

    pub fn mime(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "text/markdown",
            ExportFormat::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            ExportFormat::Epub => "application/epub+zip",
            ExportFormat::Pdf => "text/html",
        }
    }
}

pub trait ExportService: Send + Sync {
    fn supported_formats(&self) -> Vec<ExportFormat>;

    /// Render a project + documents to the requested format. Returns the
    /// serialized bytes — caller writes them to disk (UI uses the Tauri save
    /// dialog). `config` carries user-tunable options; pass
    /// `ExportConfig::default()` for the legacy behavior. `codex` is the
    /// project's codex catalogue — appended as an appendix when
    /// `config.include_codex` is true.
    fn export(
        &self,
        project: &Project,
        documents: &[DocNode],
        codex: &[CodexEntry],
        media: &MediaBundle,
        format: ExportFormat,
        config: &ExportConfig,
    ) -> AppResult<Vec<u8>>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalExporter;

impl ExportService for LocalExporter {
    fn supported_formats(&self) -> Vec<ExportFormat> {
        vec![
            ExportFormat::Markdown,
            ExportFormat::Docx,
            ExportFormat::Epub,
            ExportFormat::Pdf,
        ]
    }

    fn export(
        &self,
        project: &Project,
        documents: &[DocNode],
        codex: &[CodexEntry],
        media: &MediaBundle,
        format: ExportFormat,
        config: &ExportConfig,
    ) -> AppResult<Vec<u8>> {
        match format {
            ExportFormat::Markdown => markdown::render(project, documents, codex, media, config),
            ExportFormat::Docx => docx::render(project, documents, codex, media, config),
            ExportFormat::Epub => epub::render(project, documents, codex, media, config),
            ExportFormat::Pdf => pdf::render(project, documents, codex, media, config),
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use crate::domain::{
        new_id, now_ms, DocNode, DocumentStatus, DocumentType, Project, ProjectStatus,
    };

    pub fn project(title: &str) -> Project {
        let now = now_ms();
        Project {
            id: new_id(),
            title: title.into(),
            template_id: "novela-tres-actos".into(),
            status: ProjectStatus::Active,
            metadata: None,
            goal_words: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn doc(
        id: &str,
        project_id: &str,
        parent_id: Option<&str>,
        title: &str,
        doc_type: DocumentType,
        content: Option<&str>,
        position: i64,
    ) -> DocNode {
        let now = now_ms();
        DocNode {
            id: id.into(),
            project_id: project_id.into(),
            parent_id: parent_id.map(|s| s.into()),
            title: title.into(),
            doc_type,
            content: content.map(|s| s.into()),
            content_json: None,
            synopsis: None,
            position,
            status: DocumentStatus::Draft,
            tags: Vec::new(),
            label_ids: Vec::new(),
            goal_words: None,
            created_at: now,
            updated_at: now,
        }
    }
}
