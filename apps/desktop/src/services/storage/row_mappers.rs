//! Pure `rusqlite::Row → domain` mappers, shared by the storage submodules.

use rusqlite::Row;
use serde_json::Value as JsonValue;

use crate::domain::{DocNode, DocumentStatus, DocumentType, Project, ProjectStatus};

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
    let goal_words: Option<i64> = r.get("goal_words").ok().flatten();
    Ok(Project {
        id: r.get("id")?,
        title: r.get("title")?,
        template_id: r.get("template_id")?,
        status,
        metadata,
        goal_words,
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
    // `status` may be NULL on rows produced by the bare INSERT in
    // template_seed (the column has DEFAULT 'draft' in SQL but rusqlite
    // returns the stored value). We tolerate the missing column gracefully.
    let status_str: Option<String> = r.get("status").ok();
    let status = status_str
        .as_deref()
        .map(|s| match s {
            "revised" => DocumentStatus::Revised,
            "final" => DocumentStatus::Final,
            "trashed" => DocumentStatus::Trashed,
            _ => DocumentStatus::Draft,
        })
        .unwrap_or(DocumentStatus::Draft);
    // `tags_json` is a JSON array string provided by a correlated subquery
    // in the SELECT. Queries that don't include it (e.g. legacy SELECTs)
    // get an empty tag list instead of failing.
    let tags = r
        .get::<_, Option<String>>("tags_json")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default();
    // `labels_json` mirrors `tags_json`: a JSON array of label ids from a
    // correlated subquery, empty (not NULL) when the document has no labels.
    let label_ids = r
        .get::<_, Option<String>>("labels_json")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default();
    let goal_words: Option<i64> = r.get("goal_words").ok().flatten();
    let synopsis: Option<String> = r.get("synopsis").ok().flatten();
    let content_json: Option<String> = r.get("content_json").ok().flatten();
    Ok(DocNode {
        id: r.get("id")?,
        project_id: r.get("project_id")?,
        parent_id: r.get("parent_id")?,
        title: r.get("title")?,
        doc_type,
        content: r.get("content")?,
        content_json,
        synopsis,
        position: r.get("position")?,
        status,
        tags,
        label_ids,
        goal_words,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
    })
}
