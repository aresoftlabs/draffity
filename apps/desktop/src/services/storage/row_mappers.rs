//! Pure `rusqlite::Row → domain` mappers, shared by the storage submodules.

use rusqlite::Row;
use serde_json::Value as JsonValue;

use crate::domain::{DocNode, DocumentType, Project, ProjectStatus};

pub(super) fn row_to_project(r: &Row<'_>) -> rusqlite::Result<Project> {
    let metadata_json: Option<String> = r.get("metadata")?;
    let metadata = metadata_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<JsonValue>(s).ok());
    let status_str: String = r.get("status")?;
    let status = match status_str.as_str() {
        "active" => ProjectStatus::Active,
        _ => ProjectStatus::Archived,
    };
    Ok(Project {
        id: r.get("id")?,
        title: r.get("title")?,
        template_id: r.get("template_id")?,
        status,
        metadata,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
    })
}

pub(super) fn row_to_document(r: &Row<'_>) -> rusqlite::Result<DocNode> {
    let doc_type_str: String = r.get("doc_type")?;
    let doc_type = match doc_type_str.as_str() {
        "chapter" => DocumentType::Chapter,
        "scene" => DocumentType::Scene,
        "note" => DocumentType::Note,
        "folder" => DocumentType::Folder,
        "manga_page" => DocumentType::MangaPage,
        _ => DocumentType::Note,
    };
    Ok(DocNode {
        id: r.get("id")?,
        project_id: r.get("project_id")?,
        parent_id: r.get("parent_id")?,
        title: r.get("title")?,
        doc_type,
        content: r.get("content")?,
        position: r.get("position")?,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
    })
}
