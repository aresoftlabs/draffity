//! AI history persistence (F-12). Append accepted generations; list newest
//! first. The trait lives in `storage::mod` and delegates here.

use rusqlite::{params, Connection};

use crate::domain::{new_id, now_ms, AiHistoryEntry};
use crate::error::AppResult;

pub(super) fn record(
    conn: &Connection,
    project_id: &str,
    doc_id: Option<&str>,
    action: &str,
    model: Option<&str>,
    response: &str,
) -> AppResult<AiHistoryEntry> {
    let id = new_id();
    let now = now_ms();
    conn.execute(
        "INSERT INTO ai_history(id, project_id, doc_id, action, model, response, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, project_id, doc_id, action, model, response, now],
    )?;
    Ok(AiHistoryEntry {
        id,
        project_id: project_id.to_string(),
        doc_id: doc_id.map(str::to_string),
        action: action.to_string(),
        model: model.map(str::to_string),
        response: response.to_string(),
        created_at: now,
    })
}

pub(super) fn list(
    conn: &Connection,
    project_id: &str,
    limit: u32,
) -> AppResult<Vec<AiHistoryEntry>> {
    let mut stmt = conn.prepare(
        // rowid tiebreak so inserts within the same millisecond still order by
        // insertion (created_at alone ties at ms resolution).
        "SELECT id, project_id, doc_id, action, model, response, created_at
         FROM ai_history WHERE project_id = ?1
         ORDER BY created_at DESC, rowid DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![project_id, limit], |r| {
        Ok(AiHistoryEntry {
            id: r.get(0)?,
            project_id: r.get(1)?,
            doc_id: r.get(2)?,
            action: r.get(3)?,
            model: r.get(4)?,
            response: r.get(5)?,
            created_at: r.get(6)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::{fresh, seed_project};
    use super::super::StorageService;

    #[test]
    fn record_then_list_newest_first() {
        let s = fresh();
        let p = seed_project(&s, "Novela");
        s.record_ai_history(&p.id, None, "continue", Some("m"), "primero")
            .unwrap();
        s.record_ai_history(&p.id, Some("d1"), "rewrite:vivid", None, "segundo")
            .unwrap();
        let list = s.list_ai_history(&p.id, 10).unwrap();
        assert_eq!(list.len(), 2);
        // Newest first.
        assert_eq!(list[0].response, "segundo");
        assert_eq!(list[0].action, "rewrite:vivid");
        assert_eq!(list[0].doc_id.as_deref(), Some("d1"));
        assert_eq!(list[1].response, "primero");
    }

    #[test]
    fn list_is_project_scoped_and_limited() {
        let s = fresh();
        let p = seed_project(&s, "P");
        for i in 0..5 {
            s.record_ai_history(&p.id, None, "expand", None, &format!("r{i}"))
                .unwrap();
        }
        assert_eq!(s.list_ai_history(&p.id, 3).unwrap().len(), 3);
        assert!(s.list_ai_history("other-project", 10).unwrap().is_empty());
    }
}
