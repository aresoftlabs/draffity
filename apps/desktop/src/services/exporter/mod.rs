//! Export pipeline. The MVP ships Markdown, DOCX and EPUB. PDF is wired into
//! the trait but returns `Unsupported` until Phase 4.5 / a future iteration.

mod docx;
mod epub;
mod markdown;
mod util;

use crate::domain::{DocNode, Project};
use crate::error::{AppError, AppResult};

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
            ExportFormat::Pdf => "pdf",
        }
    }

    pub fn mime(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "text/markdown",
            ExportFormat::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            ExportFormat::Epub => "application/epub+zip",
            ExportFormat::Pdf => "application/pdf",
        }
    }
}

pub trait ExportService: Send + Sync {
    fn supported_formats(&self) -> Vec<ExportFormat>;

    /// Render a project + documents to the requested format. Returns the
    /// serialized bytes — caller writes them to disk (UI uses the Tauri save
    /// dialog).
    fn export(
        &self,
        project: &Project,
        documents: &[DocNode],
        format: ExportFormat,
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
        ]
    }

    fn export(
        &self,
        project: &Project,
        documents: &[DocNode],
        format: ExportFormat,
    ) -> AppResult<Vec<u8>> {
        match format {
            ExportFormat::Markdown => markdown::render(project, documents),
            ExportFormat::Docx => docx::render(project, documents),
            ExportFormat::Epub => epub::render(project, documents),
            ExportFormat::Pdf => Err(AppError::Unsupported(
                "PDF export not implemented yet".into(),
            )),
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
            position,
            status: DocumentStatus::Draft,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}
