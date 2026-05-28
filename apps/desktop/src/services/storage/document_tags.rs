//! Document tags: per-document tag set + project-wide tag listing.
//!
//! Tags live in a separate junction table (`document_tags`) so the document
//! row stays narrow. `set` rewrites the whole set inside a transaction —
//! we don't expose `add`/`remove` because the editor always sends the full
//! list anyway.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{now_ms, DocNode};
use crate::error::{AppError, AppResult};

use super::documents::select_one;

/// Atomically replace the tag set of a document. Empty / blank tags are
/// silently dropped; duplicates are deduped (the PK enforces this anyway).
/// Tags are stored as case-sensitive strings.
pub(super) fn set(conn: &mut Connection, id: &str, tags: &[String]) -> AppResult<DocNode> {
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

    select_one(conn, id)
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

#[cfg(test)]
mod tests {
    use super::super::test_helpers::fresh;
    use super::super::StorageService;
    use crate::domain::{DocumentInput, DocumentType, ProjectInput};
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
}
