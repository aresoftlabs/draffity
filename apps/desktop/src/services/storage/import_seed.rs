//! Recursive seeding of an imported document tree under an existing or
//! freshly-created project. Sister module of `template_seed.rs`: same
//! shape, different input type (`ImportNode` instead of `TemplateNode`).
//! Lives here because it's a storage-layer concern — the importer
//! itself produces the tree but doesn't touch SQLite.

use rusqlite::{params, Transaction};

use crate::domain::{new_id, DocumentType};
use crate::error::AppResult;
use crate::services::importer::ImportNode;

/// Insert the imported tree as documents under `project_id`. A node with
/// children is treated as a folder (no body content); a leaf is a
/// chapter carrying the rendered HTML body. Positions are per
/// (project, parent) starting at 0.
pub(super) fn insert_import_nodes(
    tx: &Transaction<'_>,
    project_id: &str,
    parent_id: Option<&str>,
    nodes: &[ImportNode],
    now: i64,
) -> AppResult<()> {
    for (idx, node) in nodes.iter().enumerate() {
        let id = new_id();
        let doc_type = if node.children.is_empty() {
            DocumentType::Chapter
        } else {
            DocumentType::Folder
        };
        let content: Option<&str> = if node.children.is_empty() && !node.content_html.is_empty() {
            Some(&node.content_html)
        } else {
            None
        };
        tx.execute(
            "INSERT INTO documents(id, project_id, parent_id, title, doc_type, content, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                project_id,
                parent_id,
                node.title.trim(),
                doc_type.as_str(),
                content,
                idx as i64,
                now,
                now,
            ],
        )?;
        if !node.children.is_empty() {
            insert_import_nodes(tx, project_id, Some(&id), &node.children, now)?;
        }
    }
    Ok(())
}
