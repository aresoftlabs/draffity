//! Document CRUD + tree reordering.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{new_id, now_ms, DocNode, DocumentInput, DocumentStatus};
use crate::error::{AppError, AppResult};

use super::row_mappers::row_to_document;

/// Column list for `SELECT` against `documents`. Kept in one place so adding
/// columns is a single-line change. The trailing correlated subquery returns
/// the document's tag set as a JSON array; an empty array (not NULL) when
/// the document has no tags.
const COLS: &str = "id, project_id, parent_id, title, doc_type, content, position, status, \
     goal_words, created_at, updated_at, \
     (SELECT COALESCE(json_group_array(tag), '[]') FROM document_tags WHERE document_id = documents.id) AS tags_json";

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
        goal_words: None,
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

/// Atomically replace the tag set of a document. Empty / blank tags are
/// silently dropped; duplicates are deduped (the PK enforces this anyway).
/// Tags are stored as case-sensitive strings.
pub(super) fn set_tags(conn: &mut Connection, id: &str, tags: &[String]) -> AppResult<DocNode> {
    // Verify the document exists before mutating.
    let exists: Option<i64> = conn
        .query_row("SELECT 1 FROM documents WHERE id=?1", params![id], |r| {
            r.get(0)
        })
        .optional()?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("document {id}")));
    }

    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM document_tags WHERE document_id=?1",
        params![id],
    )?;

    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for raw in tags {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !seen.insert(trimmed) {
            continue;
        }
        tx.execute(
            "INSERT INTO document_tags(document_id, tag) VALUES (?1, ?2)",
            params![id, trimmed],
        )?;
    }
    let now = now_ms();
    tx.execute(
        "UPDATE documents SET updated_at=?2 WHERE id=?1",
        params![id, now],
    )?;
    tx.commit()?;

    let sql = format!("SELECT {COLS} FROM documents WHERE id=?1");
    let doc = conn.query_row(&sql, params![id], row_to_document)?;
    Ok(doc)
}

/// Set or clear a document's target word count. `None` removes the goal.
pub(super) fn set_goal(conn: &Connection, id: &str, goal: Option<i64>) -> AppResult<DocNode> {
    let updated = conn.execute(
        "UPDATE documents SET goal_words=?2, updated_at=?3 WHERE id=?1",
        params![id, goal, now_ms()],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("document {id}")));
    }
    let sql = format!("SELECT {COLS} FROM documents WHERE id=?1");
    Ok(conn.query_row(&sql, params![id], row_to_document)?)
}

/// Distinct tags in use across all documents of a project, in alphabetical
/// order. Empty when the project has no tagged documents.
pub(super) fn list_project_tags(conn: &Connection, project_id: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT t.tag
         FROM document_tags t
         JOIN documents d ON d.id = t.document_id
         WHERE d.project_id = ?1
         ORDER BY t.tag COLLATE NOCASE ASC",
    )?;
    let rows = stmt
        .query_map(params![project_id], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
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
    fn set_document_tags_replaces_existing_set() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d = make_chapter(&s, &p.id, "A");

        let after = s
            .set_document_tags(
                &d,
                &[
                    "fantasy".into(),
                    "draft".into(),
                    "  ".into(),      // dropped (blank)
                    "fantasy".into(), // dropped (duplicate)
                ],
            )
            .unwrap();
        let mut tags = after.tags.clone();
        tags.sort();
        assert_eq!(tags, vec!["draft".to_string(), "fantasy".to_string()]);

        // Replace with a different set — old tags are gone.
        let after2 = s.set_document_tags(&d, &["mystery".into()]).unwrap();
        assert_eq!(after2.tags, vec!["mystery".to_string()]);
    }

    #[test]
    fn set_document_tags_missing_id_is_not_found() {
        let s = fresh();
        let err = s.set_document_tags("ghost", &["x".into()]).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }

    #[test]
    fn list_project_tags_returns_sorted_distinct() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d1 = make_chapter(&s, &p.id, "A");
        let d2 = make_chapter(&s, &p.id, "B");
        s.set_document_tags(&d1, &["fantasy".into(), "epic".into()])
            .unwrap();
        s.set_document_tags(&d2, &["fantasy".into(), "draft".into()])
            .unwrap();

        let tags = s.list_project_tags(&p.id).unwrap();
        assert_eq!(tags, vec!["draft", "epic", "fantasy"]);
    }

    #[test]
    fn set_document_goal_persists_and_clears() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d = make_chapter(&s, &p.id, "A");
        // Default = None.
        assert!(s.get_document(&d).unwrap().unwrap().goal_words.is_none());
        // Set a goal.
        let after = s.set_document_goal(&d, Some(2500)).unwrap();
        assert_eq!(after.goal_words, Some(2500));
        // Clear it.
        let cleared = s.set_document_goal(&d, None).unwrap();
        assert!(cleared.goal_words.is_none());
    }

    #[test]
    fn set_project_goal_persists_and_clears() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        assert!(p.goal_words.is_none());
        let after = s.set_project_goal(&p.id, Some(80_000)).unwrap();
        assert_eq!(after.goal_words, Some(80_000));
        let cleared = s.set_project_goal(&p.id, None).unwrap();
        assert!(cleared.goal_words.is_none());
    }

    #[test]
    fn deleting_document_cascades_its_tags() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d = make_chapter(&s, &p.id, "A");
        s.set_document_tags(&d, &["x".into(), "y".into()]).unwrap();
        s.delete_document(&d).unwrap();
        assert!(s.list_project_tags(&p.id).unwrap().is_empty());
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
