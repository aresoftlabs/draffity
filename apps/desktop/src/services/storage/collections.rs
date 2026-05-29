//! Collections CRUD + resolution (I-01..I-03). Smart collections store their
//! query as JSON; resolution loads the project's documents and filters them
//! with `CollectionQuery::matches` (pure domain logic). Manual collections
//! keep an explicit ordered membership in `collection_documents`.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{
    new_id, now_ms, Collection, CollectionInput, CollectionKind, CollectionQuery, DocNode,
};
use crate::error::{AppError, AppResult};

use super::documents;

fn row_to_collection(row: &rusqlite::Row<'_>) -> rusqlite::Result<Collection> {
    let kind_str: String = row.get(3)?;
    let query_json: Option<String> = row.get(4)?;
    let kind = CollectionKind::parse(&kind_str).unwrap_or(CollectionKind::Manual);
    let query = query_json
        .as_deref()
        .and_then(|j| serde_json::from_str::<CollectionQuery>(j).ok());
    Ok(Collection {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        kind,
        query,
        created_at: row.get(5)?,
    })
}

pub(super) fn create(conn: &Connection, input: CollectionInput) -> AppResult<Collection> {
    input.validate()?;
    let id = new_id();
    let now = now_ms();
    let name = input.name.trim().to_string();
    let query_json = match (input.kind, &input.query) {
        (CollectionKind::Smart, Some(q)) => Some(serde_json::to_string(q)?),
        _ => None,
    };
    conn.execute(
        "INSERT INTO collections(id, project_id, name, kind, query_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            input.project_id,
            name,
            input.kind.as_str(),
            query_json,
            now
        ],
    )?;
    Ok(Collection {
        id,
        project_id: input.project_id,
        name,
        kind: input.kind,
        query: input.query,
        created_at: now,
    })
}

pub(super) fn list(conn: &Connection, project_id: &str) -> AppResult<Vec<Collection>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, kind, query_json, created_at
         FROM collections WHERE project_id = ?1 ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt
        .query_map(params![project_id], row_to_collection)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub(super) fn get(conn: &Connection, id: &str) -> AppResult<Option<Collection>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, kind, query_json, created_at FROM collections WHERE id = ?1",
    )?;
    Ok(stmt.query_row(params![id], row_to_collection).optional()?)
}

pub(super) fn rename(conn: &Connection, id: &str, name: &str) -> AppResult<Collection> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invariant("collection name is empty".into()));
    }
    conn.execute(
        "UPDATE collections SET name = ?2 WHERE id = ?1",
        params![id, name],
    )?;
    get(conn, id)?.ok_or_else(|| AppError::NotFound(format!("collection {id}")))
}

/// Update a smart collection's query.
pub(super) fn set_query(
    conn: &Connection,
    id: &str,
    query: &CollectionQuery,
) -> AppResult<Collection> {
    if query.is_empty() {
        return Err(AppError::Invariant(
            "smart collection needs at least one filter".into(),
        ));
    }
    let json = serde_json::to_string(query)?;
    conn.execute(
        "UPDATE collections SET query_json = ?2 WHERE id = ?1 AND kind = 'smart'",
        params![id, json],
    )?;
    get(conn, id)?.ok_or_else(|| AppError::NotFound(format!("collection {id}")))
}

pub(super) fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM collections WHERE id = ?1", params![id])?;
    Ok(())
}

/// Replace a manual collection's membership with `ordered_ids` (index =
/// position). No-op-safe for smart collections (they have no manual members).
pub(super) fn set_members(
    conn: &mut Connection,
    collection_id: &str,
    ordered_ids: &[String],
) -> AppResult<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "DELETE FROM collection_documents WHERE collection_id = ?1",
        params![collection_id],
    )?;
    for (pos, doc_id) in ordered_ids.iter().enumerate() {
        tx.execute(
            "INSERT INTO collection_documents(collection_id, document_id, position)
             VALUES (?1, ?2, ?3)",
            params![collection_id, doc_id, pos as i64],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Resolve a collection to its documents: manual → membership in position
/// order; smart → project documents filtered by the query.
pub(super) fn resolve(conn: &Connection, id: &str) -> AppResult<Vec<DocNode>> {
    let collection =
        get(conn, id)?.ok_or_else(|| AppError::NotFound(format!("collection {id}")))?;
    match collection.kind {
        CollectionKind::Manual => {
            let mut stmt = conn.prepare(
                "SELECT cd.document_id FROM collection_documents cd
                 WHERE cd.collection_id = ?1 ORDER BY cd.position",
            )?;
            let ids: Vec<String> = stmt
                .query_map(params![id], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            let mut out = Vec::with_capacity(ids.len());
            for doc_id in ids {
                if let Some(doc) = documents::get(conn, &doc_id)? {
                    out.push(doc);
                }
            }
            Ok(out)
        }
        CollectionKind::Smart => {
            let query = collection.query.unwrap_or_default();
            let docs = documents::list(conn, &collection.project_id)?;
            Ok(docs.into_iter().filter(|d| query.matches(d)).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::{fresh, seed_project};
    use super::super::StorageService;
    use crate::domain::{
        CollectionInput, CollectionKind, CollectionQuery, DocumentInput, DocumentStatus,
        DocumentType,
    };

    fn add_doc(s: &impl StorageService, project_id: &str, title: &str) -> String {
        s.create_document(DocumentInput {
            project_id: project_id.to_string(),
            parent_id: None,
            title: title.into(),
            doc_type: DocumentType::Scene,
            content: Some("<p>x</p>".into()),
        })
        .unwrap()
        .id
    }

    #[test]
    fn manual_collection_round_trip_and_order() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d1 = add_doc(&s, &p.id, "uno");
        let d2 = add_doc(&s, &p.id, "dos");
        let c = s
            .create_collection(CollectionInput {
                project_id: p.id.clone(),
                name: "Mi colección".into(),
                kind: CollectionKind::Manual,
                query: None,
            })
            .unwrap();
        // Members in d2, d1 order.
        s.set_collection_members(&c.id, &[d2.clone(), d1.clone()])
            .unwrap();
        let resolved = s.resolve_collection(&c.id).unwrap();
        assert_eq!(
            resolved.iter().map(|d| d.id.clone()).collect::<Vec<_>>(),
            vec![d2, d1]
        );
    }

    #[test]
    fn smart_collection_filters_by_query() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let flash = add_doc(&s, &p.id, "Recuerdo");
        let _other = add_doc(&s, &p.id, "Presente");
        s.set_document_tags(&flash, &["flashback".to_string()])
            .unwrap();

        let c = s
            .create_collection(CollectionInput {
                project_id: p.id.clone(),
                name: "Flashbacks".into(),
                kind: CollectionKind::Smart,
                query: Some(CollectionQuery {
                    tags_any: vec!["flashback".into()],
                    ..Default::default()
                }),
            })
            .unwrap();
        let resolved = s.resolve_collection(&c.id).unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].id, flash);
    }

    #[test]
    fn smart_query_reacts_to_status_changes() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d = add_doc(&s, &p.id, "Cap");
        let c = s
            .create_collection(CollectionInput {
                project_id: p.id.clone(),
                name: "Finales".into(),
                kind: CollectionKind::Smart,
                query: Some(CollectionQuery {
                    statuses: vec![DocumentStatus::Final],
                    ..Default::default()
                }),
            })
            .unwrap();
        assert!(s.resolve_collection(&c.id).unwrap().is_empty());
        s.set_document_status(&d, DocumentStatus::Final).unwrap();
        // Smart collections are live — same query, now matches.
        assert_eq!(s.resolve_collection(&c.id).unwrap().len(), 1);
    }

    #[test]
    fn list_and_delete() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let c = s
            .create_collection(CollectionInput {
                project_id: p.id.clone(),
                name: "C".into(),
                kind: CollectionKind::Manual,
                query: None,
            })
            .unwrap();
        assert_eq!(s.list_collections(&p.id).unwrap().len(), 1);
        s.delete_collection(&c.id).unwrap();
        assert!(s.list_collections(&p.id).unwrap().is_empty());
    }
}
