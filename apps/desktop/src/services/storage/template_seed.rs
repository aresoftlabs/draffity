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
/// per (project, parent) starting at 0. Synopsis becomes seed content
/// wrapped in a `<p>` so it round-trips through TipTap.
pub(super) fn insert_template_nodes(
    tx: &Transaction<'_>,
    project_id: &str,
    parent_id: Option<&str>,
    nodes: &[TemplateNode],
    now: i64,
) -> AppResult<()> {
    for (idx, node) in nodes.iter().enumerate() {
        let id = new_id();
        let content = node
            .synopsis
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| format!("<p>{}</p>", escape_html(s)));
        tx.execute(
            "INSERT INTO documents(id, project_id, parent_id, title, doc_type, content, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                project_id,
                parent_id,
                node.title.trim(),
                node.doc_type.as_str(),
                content,
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

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}
