//! Per-project bibliographic entries. The migration (`007_citations.sql`)
//! enforces `UNIQUE(project_id, key)` so upserts are safe.

use std::collections::BTreeMap;

use rusqlite::{params, Connection};

use crate::domain::{new_id, now_ms, Citation};
use crate::error::AppResult;

/// One entry to upsert. `id`/`created_at`/`updated_at` are assigned by the
/// store, so callers don't have to.
#[derive(Debug, Clone)]
pub struct UpsertEntry {
    pub key: String,
    pub entry_type: String,
    pub fields: BTreeMap<String, String>,
}

pub(super) fn list(conn: &Connection, project_id: &str) -> AppResult<Vec<Citation>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, key, entry_type, fields_json, created_at, updated_at
         FROM citations WHERE project_id = ?1 ORDER BY key",
    )?;
    let rows = stmt.query_map(params![project_id], row_to_citation)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub(super) fn list_keys(conn: &Connection, project_id: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT key FROM citations WHERE project_id = ?1 ORDER BY key")?;
    let rows = stmt.query_map(params![project_id], |r| r.get::<_, String>(0))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Upsert a batch atomically. Existing keys are overwritten (their
/// `created_at` is preserved). Returns the resulting citations.
pub(super) fn upsert_batch(
    conn: &mut Connection,
    project_id: &str,
    entries: &[UpsertEntry],
) -> AppResult<Vec<Citation>> {
    let tx = conn.transaction()?;
    let now = now_ms();
    for entry in entries {
        let fields_json = serde_json::to_string(&entry.fields)?;
        // ON CONFLICT keeps the original `id` and `created_at`; only the
        // mutable parts (`entry_type`, `fields_json`, `updated_at`) move.
        tx.execute(
            "INSERT INTO citations(id, project_id, key, entry_type, fields_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
             ON CONFLICT(project_id, key) DO UPDATE SET
                 entry_type = excluded.entry_type,
                 fields_json = excluded.fields_json,
                 updated_at = excluded.updated_at",
            params![
                new_id(),
                project_id,
                entry.key,
                entry.entry_type,
                fields_json,
                now,
            ],
        )?;
    }
    tx.commit()?;
    list(conn, project_id)
}

pub(super) fn delete_one(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM citations WHERE id = ?1", params![id])?;
    Ok(())
}

fn row_to_citation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Citation> {
    let fields_json: String = row.get(4)?;
    let fields: BTreeMap<String, String> = serde_json::from_str(&fields_json).unwrap_or_default();
    Ok(Citation {
        id: row.get(0)?,
        project_id: row.get(1)?,
        key: row.get(2)?,
        entry_type: row.get(3)?,
        fields,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::{fresh, seed_project};
    use super::super::StorageService;
    use super::*;

    #[test]
    fn upsert_inserts_and_then_overwrites() {
        let s = fresh();
        let p = seed_project(&s, "Trabajo");

        let entry = UpsertEntry {
            key: "borges1944".into(),
            entry_type: "book".into(),
            fields: BTreeMap::from([
                ("author".into(), "Borges, J. L.".into()),
                ("title".into(), "Ficciones".into()),
                ("year".into(), "1944".into()),
            ]),
        };
        let list1 = s
            .upsert_citations(&p.id, std::slice::from_ref(&entry))
            .unwrap();
        assert_eq!(list1.len(), 1);
        assert_eq!(list1[0].fields.get("title").unwrap(), "Ficciones");
        assert_eq!(list1[0].fields.get("year").unwrap(), "1944");

        // Overwrite with a different title — same key collides and replaces.
        let updated = UpsertEntry {
            key: "borges1944".into(),
            entry_type: "book".into(),
            fields: BTreeMap::from([
                ("author".into(), "Borges, J. L.".into()),
                ("title".into(), "Ficciones (edición revisada)".into()),
                ("year".into(), "1944".into()),
            ]),
        };
        let list2 = s
            .upsert_citations(&p.id, std::slice::from_ref(&updated))
            .unwrap();
        assert_eq!(list2.len(), 1);
        assert_eq!(
            list2[0].fields.get("title").unwrap(),
            "Ficciones (edición revisada)"
        );
        // Same row id (upsert kept the original).
        assert_eq!(list1[0].id, list2[0].id);
    }

    #[test]
    fn list_is_project_scoped_and_sorted_by_key() {
        use crate::domain::ProjectStatus;
        let s = fresh();
        let p1 = seed_project(&s, "P1");
        // Only one project can be active at a time (unique partial index);
        // archive P1 before creating P2 so both can coexist in the DB.
        s.set_project_status(&p1.id, ProjectStatus::Archived)
            .unwrap();
        let p2 = seed_project(&s, "P2");
        s.upsert_citations(&p1.id, &[entry("b"), entry("a")])
            .unwrap();
        s.upsert_citations(&p2.id, &[entry("c")]).unwrap();

        let l1 = s.list_citations(&p1.id).unwrap();
        assert_eq!(
            l1.iter().map(|c| c.key.as_str()).collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        let l2 = s.list_citations(&p2.id).unwrap();
        assert_eq!(l2.len(), 1);
        assert_eq!(l2[0].key, "c");
    }

    #[test]
    fn delete_removes_single_entry() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let entries = s
            .upsert_citations(&p.id, &[entry("k1"), entry("k2")])
            .unwrap();
        s.delete_citation(&entries[0].id).unwrap();
        let remaining = s.list_citations(&p.id).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_ne!(remaining[0].id, entries[0].id);
    }

    fn entry(key: &str) -> UpsertEntry {
        UpsertEntry {
            key: key.into(),
            entry_type: "misc".into(),
            fields: BTreeMap::new(),
        }
    }
}
