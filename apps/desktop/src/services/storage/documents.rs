//! Document CRUD (create, list, get, update, delete + scalar setters for
//! status / synopsis / goal / move). Tag set lives in `document_tags`,
//! tree reordering in `document_positions` — both call `select_one` here
//! when they need to return the post-mutation `DocNode`.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{new_id, now_ms, DocNode, DocumentInput, DocumentStatus};
use crate::error::{AppError, AppResult};

use super::row_mappers::row_to_document;

/// Column list for `SELECT` against `documents`. Kept in one place so adding
/// columns is a single-line change. The trailing correlated subqueries return
/// the document's tag set and label-id set as JSON arrays; an empty array
/// (not NULL) when the document has none.
const COLS: &str = "id, project_id, parent_id, title, doc_type, content, content_json, synopsis, \
     position, status, goal_words, is_research, is_front_matter, is_back_matter, created_at, updated_at, \
     (SELECT COALESCE(json_group_array(tag), '[]') FROM document_tags WHERE document_id = documents.id) AS tags_json, \
     (SELECT COALESCE(json_group_array(label_id), '[]') FROM document_labels WHERE document_id = documents.id) AS labels_json, \
     COALESCE((SELECT json_group_object(field_id, value) FROM document_custom_values WHERE document_id = documents.id), '{}') AS metadata_json";

/// Single-row fetcher shared with `document_tags` and `document_positions`
/// so those modules can return the updated `DocNode` without duplicating
/// the `COLS` list.
pub(super) fn select_one(conn: &Connection, id: &str) -> AppResult<DocNode> {
    let sql = format!("SELECT {COLS} FROM documents WHERE id=?1");
    Ok(conn.query_row(&sql, params![id], row_to_document)?)
}

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

    // Compute next position within (project, parent). `.optional()?` so a
    // genuine SQL error surfaces instead of being masked by a default 0
    // (which would also clash with an existing position-0 doc).
    let next_pos: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM documents
             WHERE project_id=?1 AND IFNULL(parent_id, '')=IFNULL(?2, '')",
            params![input.project_id, input.parent_id],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);

    let now = now_ms();
    let doc = DocNode {
        id: new_id(),
        project_id: input.project_id,
        parent_id: input.parent_id,
        title: input.title.trim().to_string(),
        doc_type: input.doc_type,
        content: input.content,
        content_json: None,
        synopsis: None,
        position: next_pos,
        status: DocumentStatus::Draft,
        tags: Vec::new(),
        label_ids: Vec::new(),
        metadata: std::collections::HashMap::new(),
        is_research: false,
        is_front_matter: false,
        is_back_matter: false,
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
    content_json: Option<&str>,
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
             content_json = COALESCE(?4, content_json),
             updated_at = ?5
         WHERE id=?1",
        params![id, title, content, content_json, now],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("document {id}")));
    }
    select_one(conn, id)
}

pub(super) fn set_status(
    conn: &Connection,
    id: &str,
    status: DocumentStatus,
) -> AppResult<DocNode> {
    let updated = conn.execute(
        "UPDATE documents SET status=?2, updated_at=?3 WHERE id=?1",
        params![id, status.as_str(), now_ms()],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("document {id}")));
    }
    select_one(conn, id)
}

/// Set or clear a document's synopsis. Trimming and empty-as-None is the
/// caller's responsibility — we store exactly what arrives.
pub(super) fn set_synopsis(
    conn: &Connection,
    id: &str,
    synopsis: Option<&str>,
) -> AppResult<DocNode> {
    let updated = conn.execute(
        "UPDATE documents SET synopsis=?2, updated_at=?3 WHERE id=?1",
        params![id, synopsis, now_ms()],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("document {id}")));
    }
    select_one(conn, id)
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
    select_one(conn, id)
}

/// Flag (or unflag) a document as research material (I-10).
pub(super) fn set_research(conn: &Connection, id: &str, is_research: bool) -> AppResult<DocNode> {
    let updated = conn.execute(
        "UPDATE documents SET is_research=?2, updated_at=?3 WHERE id=?1",
        params![id, is_research as i64, now_ms()],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("document {id}")));
    }
    select_one(conn, id)
}

/// Set a document's front/back matter flags (K-03). The two are mutually
/// exclusive in practice but stored independently; the caller enforces it.
pub(super) fn set_matter(
    conn: &Connection,
    id: &str,
    is_front: bool,
    is_back: bool,
) -> AppResult<DocNode> {
    let updated = conn.execute(
        "UPDATE documents SET is_front_matter=?2, is_back_matter=?3, updated_at=?4 WHERE id=?1",
        params![id, is_front as i64, is_back as i64, now_ms()],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("document {id}")));
    }
    select_one(conn, id)
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

#[cfg(test)]
mod tests {
    use super::super::test_helpers::fresh;
    use super::super::StorageService;
    use crate::domain::{DocumentInput, DocumentStatus, DocumentType, ProjectInput, TemplateNode};
    use crate::error::AppError;

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
            .update_document(&d.id, Some("Cap 1 — bis"), Some("nuevo"), None)
            .unwrap();
        assert_eq!(updated.title, "Cap 1 — bis");
        assert_eq!(updated.content.as_deref(), Some("nuevo"));

        s.delete_document(&d2.id).unwrap();
        let docs = s.list_documents(&p.id).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].id, d.id);
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
    fn set_document_synopsis_persists_and_clears() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d = make_chapter(&s, &p.id, "A");
        assert!(s.get_document(&d).unwrap().unwrap().synopsis.is_none());

        let after = s
            .set_document_synopsis(&d, Some("Cuando el héroe llega al desierto"))
            .unwrap();
        assert_eq!(
            after.synopsis.as_deref(),
            Some("Cuando el héroe llega al desierto")
        );
        // Round-trip
        assert_eq!(
            s.get_document(&d).unwrap().unwrap().synopsis.as_deref(),
            Some("Cuando el héroe llega al desierto")
        );

        let cleared = s.set_document_synopsis(&d, None).unwrap();
        assert!(cleared.synopsis.is_none());
    }

    #[test]
    fn template_seed_populates_synopsis_column_not_content() {
        let s = fresh();
        let structure = vec![TemplateNode {
            title: "Acto 1".into(),
            doc_type: DocumentType::Folder,
            synopsis: Some("Planteamiento del conflicto".into()),
            children: vec![],
        }];
        let p = s
            .create_project_atomic(
                ProjectInput {
                    title: "Novela".into(),
                    template_id: "novela-tres-actos".into(),
                    metadata: None,
                },
                &structure,
                true,
            )
            .unwrap();
        let docs = s.list_documents(&p.id).unwrap();
        let acto = &docs[0];
        assert_eq!(
            acto.synopsis.as_deref(),
            Some("Planteamiento del conflicto")
        );
        // Content should NOT carry the synopsis anymore.
        assert!(acto.content.is_none());
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
    fn set_document_research_persists_and_defaults_false() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d = make_chapter(&s, &p.id, "A");
        // New documents are not research by default.
        assert!(!s.get_document(&d).unwrap().unwrap().is_research);

        let flagged = s.set_document_research(&d, true).unwrap();
        assert!(flagged.is_research);
        assert!(s.get_document(&d).unwrap().unwrap().is_research);

        let cleared = s.set_document_research(&d, false).unwrap();
        assert!(!cleared.is_research);
    }

    #[test]
    fn set_document_matter_persists() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d = make_chapter(&s, &p.id, "A");
        let front = s.set_document_matter(&d, true, false).unwrap();
        assert!(front.is_front_matter && !front.is_back_matter);
        let back = s.set_document_matter(&d, false, true).unwrap();
        assert!(!back.is_front_matter && back.is_back_matter);
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
    fn set_project_deadline_persists_and_clears() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        assert!(p.deadline.is_none());
        let after = s
            .set_project_deadline(&p.id, Some(1_900_000_000_000))
            .unwrap();
        assert_eq!(after.deadline, Some(1_900_000_000_000));
        let cleared = s.set_project_deadline(&p.id, None).unwrap();
        assert!(cleared.deadline.is_none());
    }
}
