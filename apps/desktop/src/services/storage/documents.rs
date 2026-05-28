//! Document CRUD + tree reordering.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{new_id, now_ms, DocNode, DocumentInput, DocumentStatus};
use crate::error::{AppError, AppResult};

use super::row_mappers::row_to_document;

/// Column list for `SELECT` against `documents`. Kept in one place so adding
/// columns (e.g. `goal_words` in the goals sprint) is a single-line change.
const COLS: &str =
    "id, project_id, parent_id, title, doc_type, content, position, status, created_at, updated_at";

pub(super) fn create(conn: &Connection, input: DocumentInput) -> AppResult<DocNode> {
    input.validate()?;

    // Verify parent project exists.
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM projects WHERE id=?1",
            params![input.project_id],
            |r| r.get(0),
        )
        .optional()?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("project {}", input.project_id)));
    }

    // Compute next position within (project, parent).
    let next_pos: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM documents
             WHERE project_id=?1 AND IFNULL(parent_id, '')=IFNULL(?2, '')",
            params![input.project_id, input.parent_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let now = now_ms();
    let doc = DocNode {
        id: new_id(),
        project_id: input.project_id,
        parent_id: input.parent_id,
        title: input.title.trim().to_string(),
        doc_type: input.doc_type,
        content: input.content,
        position: next_pos,
        status: DocumentStatus::Draft,
        tags: Vec::new(),
        created_at: now,
        updated_at: now,
    };

    conn.execute(
        "INSERT INTO documents(id, project_id, parent_id, title, doc_type, content, position, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            doc.id,
            doc.project_id,
            doc.parent_id,
            doc.title,
            doc.doc_type.as_str(),
            doc.content,
            doc.position,
            doc.created_at,
            doc.updated_at,
        ],
    )?;
    Ok(doc)
}

pub(super) fn list(conn: &Connection, project_id: &str) -> AppResult<Vec<DocNode>> {
    let sql = format!(
        "SELECT {COLS} FROM documents
         WHERE project_id=?1
         ORDER BY IFNULL(parent_id,''), position ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params![project_id], row_to_document)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub(super) fn get(conn: &Connection, id: &str) -> AppResult<Option<DocNode>> {
    let sql = format!("SELECT {COLS} FROM documents WHERE id=?1");
    let d = conn
        .query_row(&sql, params![id], row_to_document)
        .optional()?;
    Ok(d)
}

pub(super) fn update(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    content: Option<&str>,
) -> AppResult<DocNode> {
    if let Some(t) = title {
        if t.trim().is_empty() {
            return Err(AppError::Invariant("title cannot be empty".into()));
        }
    }
    let now = now_ms();
    let updated = conn.execute(
        "UPDATE documents
         SET title = COALESCE(?2, title),
             content = COALESCE(?3, content),
             updated_at = ?4
         WHERE id=?1",
        params![id, title, content, now],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("document {id}")));
    }
    let sql = format!("SELECT {COLS} FROM documents WHERE id=?1");
    let doc = conn.query_row(&sql, params![id], row_to_document)?;
    Ok(doc)
}

pub(super) fn set_status(
    conn: &Connection,
    id: &str,
    status: DocumentStatus,
) -> AppResult<DocNode> {
    let now = now_ms();
    let updated = conn.execute(
        "UPDATE documents SET status=?2, updated_at=?3 WHERE id=?1",
        params![id, status.as_str(), now],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("document {id}")));
    }
    let sql = format!("SELECT {COLS} FROM documents WHERE id=?1");
    let doc = conn.query_row(&sql, params![id], row_to_document)?;
    Ok(doc)
}

pub(super) fn move_to(
    conn: &Connection,
    id: &str,
    parent_id: Option<&str>,
    position: i64,
) -> AppResult<()> {
    let updated = conn.execute(
        "UPDATE documents SET parent_id=?2, position=?3, updated_at=?4 WHERE id=?1",
        params![id, parent_id, position, now_ms()],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("document {id}")));
    }
    Ok(())
}

pub(super) fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let removed = conn.execute("DELETE FROM documents WHERE id=?1", params![id])?;
    if removed == 0 {
        return Err(AppError::NotFound(format!("document {id}")));
    }
    Ok(())
}

/// Atomically reassign `position` (and optionally `parent_id`) for every id
/// in `ordered_ids` to its index in the slice. Used by binder drag&drop to
/// compact positions and avoid duplicates / gaps after a rearrange.
///
/// All ids must belong to `project_id`. Fails atomically if any row is
/// missing — no partial update.
pub(super) fn reorder(
    conn: &mut Connection,
    project_id: &str,
    parent_id: Option<&str>,
    ordered_ids: &[String],
) -> AppResult<()> {
    let tx = conn.transaction()?;
    let now = now_ms();
    for (idx, id) in ordered_ids.iter().enumerate() {
        let updated = tx.execute(
            "UPDATE documents
             SET parent_id=?2, position=?3, updated_at=?4
             WHERE id=?1 AND project_id=?5",
            params![id, parent_id, idx as i64, now, project_id],
        )?;
        if updated == 0 {
            return Err(AppError::NotFound(format!(
                "document {id} in project {project_id}"
            )));
        }
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::fresh;
    use super::super::StorageService;
    use crate::domain::{DocumentInput, DocumentStatus, DocumentType, ProjectInput};
    use crate::error::AppError;

    #[test]
    fn document_crud_round_trip() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d = s
            .create_document(DocumentInput {
                project_id: p.id.clone(),
                parent_id: None,
                title: "Cap 1".into(),
                doc_type: DocumentType::Chapter,
                content: Some("hola".into()),
            })
            .unwrap();
        assert_eq!(d.position, 0);

        let d2 = s
            .create_document(DocumentInput {
                project_id: p.id.clone(),
                parent_id: None,
                title: "Cap 2".into(),
                doc_type: DocumentType::Chapter,
                content: None,
            })
            .unwrap();
        assert_eq!(d2.position, 1);

        let updated = s
            .update_document(&d.id, Some("Cap 1 — bis"), Some("nuevo"))
            .unwrap();
        assert_eq!(updated.title, "Cap 1 — bis");
        assert_eq!(updated.content.as_deref(), Some("nuevo"));

        s.delete_document(&d2.id).unwrap();
        let docs = s.list_documents(&p.id).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].id, d.id);
    }

    fn make_chapter(s: &impl StorageService, project_id: &str, title: &str) -> String {
        s.create_document(DocumentInput {
            project_id: project_id.into(),
            parent_id: None,
            title: title.into(),
            doc_type: DocumentType::Chapter,
            content: None,
        })
        .unwrap()
        .id
    }

    #[test]
    fn reorder_compacts_positions_within_same_parent() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let a = make_chapter(&s, &p.id, "A");
        let b = make_chapter(&s, &p.id, "B");
        let c = make_chapter(&s, &p.id, "C");

        // Reverse: C, B, A
        s.reorder_documents(&p.id, None, &[c.clone(), b.clone(), a.clone()])
            .unwrap();

        let docs = s.list_documents(&p.id).unwrap();
        let pos_of = |id: &str| docs.iter().find(|d| d.id == id).unwrap().position;
        assert_eq!(pos_of(&c), 0);
        assert_eq!(pos_of(&b), 1);
        assert_eq!(pos_of(&a), 2);
    }

    #[test]
    fn reorder_moves_node_to_new_parent() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        // Folder + 2 chapters at root
        let folder = s
            .create_document(DocumentInput {
                project_id: p.id.clone(),
                parent_id: None,
                title: "Folder".into(),
                doc_type: DocumentType::Folder,
                content: None,
            })
            .unwrap()
            .id;
        let ch = make_chapter(&s, &p.id, "Ch");

        // Move ch under folder
        s.reorder_documents(&p.id, Some(&folder), &[ch.clone()])
            .unwrap();

        let docs = s.list_documents(&p.id).unwrap();
        let moved = docs.iter().find(|d| d.id == ch).unwrap();
        assert_eq!(moved.parent_id.as_deref(), Some(folder.as_str()));
        assert_eq!(moved.position, 0);
    }

    #[test]
    fn new_document_defaults_to_draft_status() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d = s
            .create_document(DocumentInput {
                project_id: p.id.clone(),
                parent_id: None,
                title: "Cap".into(),
                doc_type: DocumentType::Chapter,
                content: None,
            })
            .unwrap();
        assert_eq!(d.status, DocumentStatus::Draft);
        // And persists across a re-read.
        let reread = s.get_document(&d.id).unwrap().unwrap();
        assert_eq!(reread.status, DocumentStatus::Draft);
    }

    #[test]
    fn set_document_status_persists() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d = make_chapter(&s, &p.id, "A");

        let after = s.set_document_status(&d, DocumentStatus::Revised).unwrap();
        assert_eq!(after.status, DocumentStatus::Revised);
        assert_eq!(
            s.get_document(&d).unwrap().unwrap().status,
            DocumentStatus::Revised
        );
    }

    #[test]
    fn set_document_status_missing_id_is_not_found() {
        let s = fresh();
        let err = s
            .set_document_status("ghost", DocumentStatus::Final)
            .unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn reorder_rolls_back_when_id_missing() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let a = make_chapter(&s, &p.id, "A");

        let result = s.reorder_documents(&p.id, None, &[a.clone(), "ghost".into()]);
        assert!(matches!(result, Err(AppError::NotFound(_))));

        // The valid id must not have been left with a partial update.
        let docs = s.list_documents(&p.id).unwrap();
        assert_eq!(docs.iter().find(|d| d.id == a).unwrap().position, 0);
    }
}
