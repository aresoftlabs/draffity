//! Document tree reordering. Binder drag&drop calls `reorder` after every
//! drop to compact positions and avoid duplicates / gaps. The op is
//! transactional — a single missing id rolls back the whole batch.

use rusqlite::{params, Connection};

use crate::domain::now_ms;
use crate::error::{AppError, AppResult};

/// Atomically reassign `position` (and optionally `parent_id`) for every id
/// in `ordered_ids` to its index in the slice.
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
        s.reorder_documents(&p.id, Some(&folder), std::slice::from_ref(&ch))
            .unwrap();

        let docs = s.list_documents(&p.id).unwrap();
        let moved = docs.iter().find(|d| d.id == ch).unwrap();
        assert_eq!(moved.parent_id.as_deref(), Some(folder.as_str()));
        assert_eq!(moved.position, 0);
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
