//! Recursive seeding of a project's initial document tree from a
//! `Template.structure`. Only used by `projects::create_atomic`.
//!
//! Lives apart from `projects.rs` so the SQL for documents seeding stays
//! co-located with its purpose (template instantiation) rather than with
//! generic document CRUD.

use rusqlite::{params, Transaction};

use crate::domain::{new_id, TemplateNode};
use crate::error::AppResult;

/// Recursively insert a template's structure as documents. Position is
/// per (project, parent) starting at 0. The template's `synopsis` is
/// stored in the dedicated `synopsis` column (it's metadata about the
/// document, not its editable content).
pub(super) fn insert_template_nodes(
    tx: &Transaction<'_>,
    project_id: &str,
    parent_id: Option<&str>,
    nodes: &[TemplateNode],
    now: i64,
) -> AppResult<()> {
    for (idx, node) in nodes.iter().enumerate() {
        let id = new_id();
        let synopsis = node
            .synopsis
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        tx.execute(
            "INSERT INTO documents(id, project_id, parent_id, title, doc_type, content, synopsis, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                project_id,
                parent_id,
                node.title.trim(),
                node.doc_type.as_str(),
                None::<String>,
                synopsis,
                idx as i64,
                now,
                now,
            ],
        )?;
        if !node.children.is_empty() {
            insert_template_nodes(tx, project_id, Some(&id), &node.children, now)?;
        }
    }
    Ok(())
}
