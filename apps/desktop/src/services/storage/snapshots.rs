//! Document snapshots (manual versioning). `restore` is the only operation
//! requiring a transaction — it creates an auto-snapshot of the current
//! state before overwriting, so the user can undo.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{new_id, now_ms, DocNode, Snapshot};
use crate::error::{AppError, AppResult};

use super::row_mappers::row_to_document;

pub(super) fn create(
    conn: &Connection,
    document_id: &str,
    label: Option<&str>,
) -> AppResult<Snapshot> {
    let content: Option<String> = conn
        .query_row(
            "SELECT content FROM documents WHERE id=?1",
            params![document_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("document {document_id}")))?;
    let snap = Snapshot {
        id: new_id(),
        document_id: document_id.to_string(),
        content: content.unwrap_or_default(),
        label: label.map(|s| s.to_string()),
        created_at: now_ms(),
    };
    conn.execute(
        "INSERT INTO snapshots(id, document_id, content, label, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            snap.id,
            snap.document_id,
            snap.content,
            snap.label,
            snap.created_at
        ],
    )?;
    Ok(snap)
}

pub(super) fn list(conn: &Connection, document_id: &str) -> AppResult<Vec<Snapshot>> {
    let mut stmt = conn.prepare(
        "SELECT id, document_id, content, label, created_at
         FROM snapshots WHERE document_id=?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map(params![document_id], |r| {
            Ok(Snapshot {
                id: r.get(0)?,
                document_id: r.get(1)?,
                content: r.get(2)?,
                label: r.get(3)?,
                created_at: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub(super) fn restore(conn: &mut Connection, snapshot_id: &str) -> AppResult<DocNode> {
    let tx = conn.transaction()?;

    let (document_id, snapshot_content): (String, String) = tx
        .query_row(
            "SELECT document_id, content FROM snapshots WHERE id=?1",
            params![snapshot_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("snapshot {snapshot_id}")))?;

    let current: Option<String> = tx
        .query_row(
            "SELECT content FROM documents WHERE id=?1",
            params![document_id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("document {document_id}")))?;

    let now = now_ms();
    // Auto-snapshot of the pre-restore state so the user can undo.
    tx.execute(
        "INSERT INTO snapshots(id, document_id, content, label, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            new_id(),
            document_id,
            current.unwrap_or_default(),
            "auto-restore",
            now,
        ],
    )?;

    tx.execute(
        "UPDATE documents SET content=?2, updated_at=?3 WHERE id=?1",
        params![document_id, snapshot_content, now],
    )?;

    let doc = tx.query_row(
        "SELECT id, project_id, parent_id, title, doc_type, content, content_json, synopsis, position, status, created_at, updated_at
         FROM documents WHERE id=?1",
        params![document_id],
        row_to_document,
    )?;

    tx.commit()?;
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::fresh;
    use super::super::StorageService;
    use crate::domain::{DocumentInput, DocumentType, ProjectInput};
    use crate::error::AppError;

    #[test]
    fn snapshot_round_trip() {
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
                project_id: p.id,
                parent_id: None,
                title: "X".into(),
                doc_type: DocumentType::Note,
                content: Some("v1".into()),
            })
            .unwrap();
        let snap = s.create_snapshot(&d.id, Some("draft 1")).unwrap();
        assert_eq!(snap.content, "v1");
        let all = s.list_snapshots(&d.id).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn restore_snapshot_replaces_content_and_creates_auto_backup() {
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
                project_id: p.id,
                parent_id: None,
                title: "D".into(),
                doc_type: DocumentType::Note,
                content: Some("v1".into()),
            })
            .unwrap();
        let snap = s.create_snapshot(&d.id, Some("draft 1")).unwrap();
        s.update_document(&d.id, None, Some("v2"), None).unwrap();

        let restored = s.restore_snapshot(&snap.id).unwrap();
        assert_eq!(restored.content.as_deref(), Some("v1"));

        let all = s.list_snapshots(&d.id).unwrap();
        assert!(all
            .iter()
            .any(|x| x.label.as_deref() == Some("auto-restore") && x.content == "v2"));
    }

    #[test]
    fn restore_unknown_snapshot_returns_not_found() {
        let s = fresh();
        let err = s.restore_snapshot("does-not-exist").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
