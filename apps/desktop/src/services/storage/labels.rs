//! Labels (I-05/I-06): per-project label definitions + per-document
//! assignment. Label rows hold name + color; `document_labels` is the
//! many-to-many junction. `set_document` rewrites a document's whole label
//! set inside a transaction (same shape as `document_tags::set`).

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{new_id, now_ms, DocNode, Label, LabelInput};
use crate::error::{AppError, AppResult};

use super::documents::select_one;

const COLS: &str = "id, project_id, name, color, created_at";

fn row_to_label(r: &rusqlite::Row<'_>) -> rusqlite::Result<Label> {
    Ok(Label {
        id: r.get("id")?,
        project_id: r.get("project_id")?,
        name: r.get("name")?,
        color: r.get("color")?,
        created_at: r.get("created_at")?,
    })
}

pub(super) fn create(conn: &Connection, input: LabelInput) -> AppResult<Label> {
    input.validate()?;

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

    let label = Label {
        id: new_id(),
        project_id: input.project_id,
        name: input.name.trim().to_string(),
        color: input.color.trim().to_string(),
        created_at: now_ms(),
    };
    conn.execute(
        "INSERT INTO labels(id, project_id, name, color, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            label.id,
            label.project_id,
            label.name,
            label.color,
            label.created_at,
        ],
    )
    .map_err(map_unique)?;
    Ok(label)
}

/// A project's labels, alphabetical case-insensitive.
pub(super) fn list(conn: &Connection, project_id: &str) -> AppResult<Vec<Label>> {
    let sql =
        format!("SELECT {COLS} FROM labels WHERE project_id=?1 ORDER BY name COLLATE NOCASE ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params![project_id], row_to_label)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub(super) fn update(conn: &Connection, id: &str, name: &str, color: &str) -> AppResult<Label> {
    // Reuse the input validator (project_id is irrelevant to validation).
    LabelInput {
        project_id: "x".into(),
        name: name.into(),
        color: color.into(),
    }
    .validate()?;

    let updated = conn
        .execute(
            "UPDATE labels SET name=?2, color=?3 WHERE id=?1",
            params![id, name.trim(), color.trim()],
        )
        .map_err(map_unique)?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("label {id}")));
    }
    let sql = format!("SELECT {COLS} FROM labels WHERE id=?1");
    Ok(conn.query_row(&sql, params![id], row_to_label)?)
}

pub(super) fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    // `document_labels` rows cascade via the FK.
    let removed = conn.execute("DELETE FROM labels WHERE id=?1", params![id])?;
    if removed == 0 {
        return Err(AppError::NotFound(format!("label {id}")));
    }
    Ok(())
}

/// Atomically replace the label set of a document. Unknown / blank ids are
/// dropped; duplicates are deduped (the PK enforces this anyway). Only labels
/// belonging to the document's project are accepted.
pub(super) fn set_document(
    conn: &mut Connection,
    doc_id: &str,
    label_ids: &[String],
) -> AppResult<DocNode> {
    let project_id: Option<String> = conn
        .query_row(
            "SELECT project_id FROM documents WHERE id=?1",
            params![doc_id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(project_id) = project_id else {
        return Err(AppError::NotFound(format!("document {doc_id}")));
    };

    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM document_labels WHERE document_id=?1",
        params![doc_id],
    )?;

    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for raw in label_ids {
        let id = raw.trim();
        if id.is_empty() || !seen.insert(id) {
            continue;
        }
        // Guard: the label must exist and belong to this document's project.
        let ok: Option<i64> = tx
            .query_row(
                "SELECT 1 FROM labels WHERE id=?1 AND project_id=?2",
                params![id, project_id],
                |r| r.get(0),
            )
            .optional()?;
        if ok.is_none() {
            return Err(AppError::Invariant(format!(
                "label {id} not in project {project_id}"
            )));
        }
        tx.execute(
            "INSERT INTO document_labels(document_id, label_id) VALUES (?1, ?2)",
            params![doc_id, id],
        )?;
    }
    tx.execute(
        "UPDATE documents SET updated_at=?2 WHERE id=?1",
        params![doc_id, now_ms()],
    )?;
    tx.commit()?;

    select_one(conn, doc_id)
}

/// Map a UNIQUE-constraint violation to a friendly invariant error.
fn map_unique(e: rusqlite::Error) -> AppError {
    let msg = e.to_string();
    if msg.contains("UNIQUE") {
        AppError::Invariant("a label with that name already exists".into())
    } else {
        AppError::from(e)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::{fresh, seed_project};
    use super::super::StorageService;
    use crate::domain::{DocumentInput, DocumentType, LabelInput};
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

    fn mk_label(s: &impl StorageService, project_id: &str, name: &str, color: &str) -> String {
        s.create_label(LabelInput {
            project_id: project_id.into(),
            name: name.into(),
            color: color.into(),
        })
        .unwrap()
        .id
    }

    #[test]
    fn create_and_list_labels_sorted() {
        let s = fresh();
        let p = seed_project(&s, "P");
        mk_label(&s, &p.id, "Zeta", "#111111");
        mk_label(&s, &p.id, "alfa", "#222222");
        let labels = s.list_labels(&p.id).unwrap();
        assert_eq!(
            labels.iter().map(|l| l.name.as_str()).collect::<Vec<_>>(),
            vec!["alfa", "Zeta"]
        );
    }

    #[test]
    fn duplicate_name_in_project_is_rejected() {
        let s = fresh();
        let p = seed_project(&s, "P");
        mk_label(&s, &p.id, "POV", "#ef4444");
        let err = s
            .create_label(LabelInput {
                project_id: p.id.clone(),
                name: "POV".into(),
                color: "#000000".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Invariant(_)));
    }

    #[test]
    fn update_label_changes_name_and_color() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let id = mk_label(&s, &p.id, "Old", "#ef4444");
        let updated = s.update_label(&id, "New", "#00ff00").unwrap();
        assert_eq!(updated.name, "New");
        assert_eq!(updated.color, "#00ff00");
    }

    #[test]
    fn set_document_labels_round_trip_and_dedupe() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d = make_chapter(&s, &p.id, "A");
        let l1 = mk_label(&s, &p.id, "Importante", "#ef4444");
        let l2 = mk_label(&s, &p.id, "Revisar", "#3b82f6");

        let after = s
            .set_document_labels(&d, &[l1.clone(), l2.clone(), l1.clone(), "  ".into()])
            .unwrap();
        let mut ids = after.label_ids.clone();
        ids.sort();
        let mut want = vec![l1.clone(), l2.clone()];
        want.sort();
        assert_eq!(ids, want);

        // Replace with a narrower set.
        let after2 = s
            .set_document_labels(&d, std::slice::from_ref(&l2))
            .unwrap();
        assert_eq!(after2.label_ids, vec![l2]);
    }

    #[test]
    fn cannot_assign_label_from_other_project() {
        let s = fresh();
        let p1 = seed_project(&s, "P1");
        // Only one project may be active at a time (idx_projects_one_active),
        // so archive p1 before creating the second project.
        s.set_project_status(&p1.id, crate::domain::ProjectStatus::Archived)
            .unwrap();
        let p2 = seed_project(&s, "P2");
        let d = make_chapter(&s, &p1.id, "A");
        let foreign = mk_label(&s, &p2.id, "X", "#ef4444");
        let err = s.set_document_labels(&d, &[foreign]).unwrap_err();
        assert!(matches!(err, AppError::Invariant(_)));
    }

    #[test]
    fn deleting_label_cascades_assignments() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d = make_chapter(&s, &p.id, "A");
        let l = mk_label(&s, &p.id, "Tmp", "#ef4444");
        s.set_document_labels(&d, std::slice::from_ref(&l)).unwrap();
        assert_eq!(
            s.get_document(&d).unwrap().unwrap().label_ids,
            vec![l.clone()]
        );

        s.delete_label(&l).unwrap();
        assert!(s.get_document(&d).unwrap().unwrap().label_ids.is_empty());
        assert!(s.list_labels(&p.id).unwrap().is_empty());
    }

    #[test]
    fn deleting_document_cascades_its_labels() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d = make_chapter(&s, &p.id, "A");
        let l = mk_label(&s, &p.id, "Tmp", "#ef4444");
        s.set_document_labels(&d, std::slice::from_ref(&l)).unwrap();
        s.delete_document(&d).unwrap();
        // The label itself survives; only the assignment is gone.
        assert_eq!(s.list_labels(&p.id).unwrap().len(), 1);
    }
}
