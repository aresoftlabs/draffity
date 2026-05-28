//! Codex CRUD + search. Mirrors the `citations` submodule layout: the trait
//! lives in `storage::mod` and delegates here for the SQL bodies.
//!
//! Search is a `LIKE` scan rather than FTS — codex catalogues are small
//! (hundreds of entries, not chapters of prose) and a `LIKE` lets us match
//! across `name`, `body` and tags without a second virtual table to maintain.

use rusqlite::{params, Connection};

use crate::domain::{new_id, now_ms, CodexEntry, CodexInput, CodexKind, CodexUpdate};
use crate::error::{AppError, AppResult};

pub(super) fn create(conn: &Connection, input: CodexInput) -> AppResult<CodexEntry> {
    input.validate()?;
    let normalised = input.normalised();
    let id = new_id();
    let now = now_ms();
    let tags_json = serde_json::to_string(&normalised.tags)?;
    conn.execute(
        "INSERT INTO codex_entries(id, project_id, kind, name, body, tags_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            id,
            normalised.project_id,
            normalised.kind.as_str(),
            normalised.name,
            normalised.body,
            tags_json,
            now,
        ],
    )?;
    Ok(CodexEntry {
        id,
        project_id: normalised.project_id,
        kind: normalised.kind,
        name: normalised.name,
        body: normalised.body,
        tags: normalised.tags,
        created_at: now,
        updated_at: now,
    })
}

pub(super) fn list(conn: &Connection, project_id: &str) -> AppResult<Vec<CodexEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, kind, name, body, tags_json, created_at, updated_at
         FROM codex_entries WHERE project_id = ?1 ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![project_id], row_to_entry)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

pub(super) fn get(conn: &Connection, id: &str) -> AppResult<Option<CodexEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, kind, name, body, tags_json, created_at, updated_at
         FROM codex_entries WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], row_to_entry)?;
    Ok(rows.next().transpose()?)
}

pub(super) fn update(conn: &Connection, id: &str, patch: CodexUpdate) -> AppResult<CodexEntry> {
    let existing = get(conn, id)?.ok_or_else(|| AppError::NotFound(format!("codex entry {id}")))?;

    let name = patch
        .name
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(existing.name);
    let kind = patch.kind.unwrap_or(existing.kind);
    let body = match patch.body {
        Some(b) if b.trim().is_empty() => None,
        Some(b) => Some(b),
        None => existing.body,
    };
    let tags = patch.tags.unwrap_or(existing.tags);
    let tags_json = serde_json::to_string(&tags)?;
    let now = now_ms();
    conn.execute(
        "UPDATE codex_entries
         SET kind = ?2, name = ?3, body = ?4, tags_json = ?5, updated_at = ?6
         WHERE id = ?1",
        params![id, kind.as_str(), name, body, tags_json, now],
    )?;
    Ok(CodexEntry {
        id: existing.id,
        project_id: existing.project_id,
        kind,
        name,
        body,
        tags,
        created_at: existing.created_at,
        updated_at: now,
    })
}

pub(super) fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let removed = conn.execute("DELETE FROM codex_entries WHERE id = ?1", params![id])?;
    if removed == 0 {
        return Err(AppError::NotFound(format!("codex entry {id}")));
    }
    Ok(())
}

/// Project-scoped search. Empty/whitespace `query` lists everything. The
/// pattern checks `name`, `body` and tag JSON in a single `LIKE` — good
/// enough for a few hundred entries with sub-millisecond latency.
pub(super) fn search(
    conn: &Connection,
    project_id: &str,
    query: &str,
    kind: Option<CodexKind>,
) -> AppResult<Vec<CodexEntry>> {
    let q = query.trim();
    let kind_filter = kind.map(|k| k.as_str());

    if q.is_empty() && kind_filter.is_none() {
        return list(conn, project_id);
    }

    let pattern = if q.is_empty() {
        "%".to_string()
    } else {
        format!("%{}%", q.replace('%', r"\%").replace('_', r"\_"))
    };
    let kind_clause = if kind_filter.is_some() {
        " AND kind = ?3"
    } else {
        ""
    };
    let sql = format!(
        "SELECT id, project_id, kind, name, body, tags_json, created_at, updated_at
         FROM codex_entries
         WHERE project_id = ?1
           AND (
              name LIKE ?2 ESCAPE '\\'
              OR (body IS NOT NULL AND body LIKE ?2 ESCAPE '\\')
              OR tags_json LIKE ?2 ESCAPE '\\'
           )
           {kind_clause}
         ORDER BY name COLLATE NOCASE",
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = if let Some(k) = kind_filter {
        stmt.query_map(params![project_id, pattern, k], row_to_entry)?
            .collect::<Vec<_>>()
    } else {
        stmt.query_map(params![project_id, pattern], row_to_entry)?
            .collect::<Vec<_>>()
    };
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<CodexEntry> {
    let kind_str: String = row.get(2)?;
    let tags_json: String = row.get(5)?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    Ok(CodexEntry {
        id: row.get(0)?,
        project_id: row.get(1)?,
        kind: CodexKind::parse(&kind_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?,
        name: row.get(3)?,
        body: row.get(4)?,
        tags,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::{fresh, seed_project};
    use super::super::StorageService;
    use super::*;
    use crate::domain::ProjectStatus;

    fn aragorn(project_id: &str) -> CodexInput {
        CodexInput {
            project_id: project_id.into(),
            kind: CodexKind::Character,
            name: "Aragorn".into(),
            body: Some("Ranger of the North.".into()),
            tags: vec!["protagonista".into(), "ranger".into()],
        }
    }

    #[test]
    fn create_and_get_round_trip() {
        let s = fresh();
        let p = seed_project(&s, "LOTR");
        let e = s.create_codex_entry(aragorn(&p.id)).unwrap();
        let got = s.get_codex_entry(&e.id).unwrap().unwrap();
        assert_eq!(got.name, "Aragorn");
        assert_eq!(got.kind, CodexKind::Character);
        assert_eq!(got.tags, vec!["protagonista", "ranger"]);
        assert_eq!(got.body.as_deref(), Some("Ranger of the North."));
    }

    #[test]
    fn list_is_alphabetical_case_insensitive() {
        let s = fresh();
        let p = seed_project(&s, "X");
        for name in ["zelda", "Aragorn", "boromir"] {
            s.create_codex_entry(CodexInput {
                project_id: p.id.clone(),
                kind: CodexKind::Character,
                name: name.into(),
                body: None,
                tags: vec![],
            })
            .unwrap();
        }
        let list = s.list_codex_entries(&p.id).unwrap();
        assert_eq!(
            list.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            vec!["Aragorn", "boromir", "zelda"]
        );
    }

    #[test]
    fn list_is_project_scoped() {
        let s = fresh();
        let p1 = seed_project(&s, "P1");
        s.set_project_status(&p1.id, ProjectStatus::Archived)
            .unwrap();
        let p2 = seed_project(&s, "P2");
        s.create_codex_entry(aragorn(&p1.id)).unwrap();
        s.create_codex_entry(CodexInput {
            project_id: p2.id.clone(),
            kind: CodexKind::Place,
            name: "Mordor".into(),
            body: None,
            tags: vec![],
        })
        .unwrap();
        assert_eq!(s.list_codex_entries(&p1.id).unwrap().len(), 1);
        assert_eq!(s.list_codex_entries(&p2.id).unwrap()[0].name, "Mordor");
    }

    #[test]
    fn update_replaces_only_provided_fields() {
        let s = fresh();
        let p = seed_project(&s, "X");
        let e = s.create_codex_entry(aragorn(&p.id)).unwrap();
        let updated = s
            .update_codex_entry(
                &e.id,
                CodexUpdate {
                    name: Some("Strider".into()),
                    ..CodexUpdate::default()
                },
            )
            .unwrap();
        assert_eq!(updated.name, "Strider");
        // Body and tags preserved.
        assert_eq!(updated.body.as_deref(), Some("Ranger of the North."));
        assert_eq!(updated.tags, vec!["protagonista", "ranger"]);
    }

    #[test]
    fn update_with_empty_body_clears_it() {
        let s = fresh();
        let p = seed_project(&s, "X");
        let e = s.create_codex_entry(aragorn(&p.id)).unwrap();
        let updated = s
            .update_codex_entry(
                &e.id,
                CodexUpdate {
                    body: Some("   ".into()),
                    ..CodexUpdate::default()
                },
            )
            .unwrap();
        assert_eq!(updated.body, None);
    }

    #[test]
    fn update_missing_id_returns_not_found() {
        let s = fresh();
        let err = s
            .update_codex_entry("missing", CodexUpdate::default())
            .unwrap_err();
        matches!(err, AppError::NotFound(_));
    }

    #[test]
    fn delete_removes_entry() {
        let s = fresh();
        let p = seed_project(&s, "X");
        let e = s.create_codex_entry(aragorn(&p.id)).unwrap();
        s.delete_codex_entry(&e.id).unwrap();
        assert!(s.get_codex_entry(&e.id).unwrap().is_none());
    }

    #[test]
    fn delete_missing_id_returns_not_found() {
        let s = fresh();
        let err = s.delete_codex_entry("missing").unwrap_err();
        matches!(err, AppError::NotFound(_));
    }

    #[test]
    fn deleting_project_cascades_codex_entries() {
        let s = fresh();
        let p = seed_project(&s, "X");
        s.create_codex_entry(aragorn(&p.id)).unwrap();
        s.delete_project(&p.id).unwrap();
        // Re-creating the project gives a new id; list against the old one
        // should be empty.
        assert!(s.list_codex_entries(&p.id).unwrap().is_empty());
    }

    #[test]
    fn search_matches_name_body_and_tags() {
        let s = fresh();
        let p = seed_project(&s, "X");
        s.create_codex_entry(aragorn(&p.id)).unwrap();
        s.create_codex_entry(CodexInput {
            project_id: p.id.clone(),
            kind: CodexKind::Place,
            name: "Mordor".into(),
            body: Some("Land of shadow".into()),
            tags: vec!["evil".into()],
        })
        .unwrap();

        // Name match.
        let by_name = s.search_codex_entries(&p.id, "Aragorn", None).unwrap();
        assert_eq!(by_name.len(), 1);
        assert_eq!(by_name[0].name, "Aragorn");
        // Body match (case-sensitive LIKE — SQLite default).
        let by_body = s.search_codex_entries(&p.id, "shadow", None).unwrap();
        assert_eq!(by_body.len(), 1);
        assert_eq!(by_body[0].name, "Mordor");
        // Tag match through the JSON column.
        let by_tag = s.search_codex_entries(&p.id, "ranger", None).unwrap();
        assert_eq!(by_tag.len(), 1);
        assert_eq!(by_tag[0].name, "Aragorn");
    }

    #[test]
    fn search_with_kind_filter() {
        let s = fresh();
        let p = seed_project(&s, "X");
        s.create_codex_entry(aragorn(&p.id)).unwrap();
        s.create_codex_entry(CodexInput {
            project_id: p.id.clone(),
            kind: CodexKind::Place,
            name: "Mordor".into(),
            body: None,
            tags: vec![],
        })
        .unwrap();

        let only_places = s
            .search_codex_entries(&p.id, "", Some(CodexKind::Place))
            .unwrap();
        assert_eq!(only_places.len(), 1);
        assert_eq!(only_places[0].name, "Mordor");

        let only_chars = s
            .search_codex_entries(&p.id, "", Some(CodexKind::Character))
            .unwrap();
        assert_eq!(only_chars.len(), 1);
    }

    #[test]
    fn search_empty_query_returns_all_entries_of_project() {
        let s = fresh();
        let p = seed_project(&s, "X");
        s.create_codex_entry(aragorn(&p.id)).unwrap();
        s.create_codex_entry(CodexInput {
            project_id: p.id.clone(),
            kind: CodexKind::Note,
            name: "Worldbuilding TODO".into(),
            body: None,
            tags: vec![],
        })
        .unwrap();
        let all = s.search_codex_entries(&p.id, "  ", None).unwrap();
        assert_eq!(all.len(), 2);
    }
}
