//! AI validation report persistence (G-02). Append a report; list newest
//! first. The trait lives in `storage::mod` and delegates here.

use rusqlite::{params, Connection};

use crate::domain::{new_id, now_ms, AiValidation};
use crate::error::AppResult;

pub(super) fn record(
    conn: &Connection,
    document_id: &str,
    validator_name: &str,
    results_json: &str,
    severity_summary: &str,
) -> AppResult<AiValidation> {
    let id = new_id();
    let now = now_ms();
    conn.execute(
        "INSERT INTO ai_validations(id, document_id, validator_name, results_json, severity_summary, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, document_id, validator_name, results_json, severity_summary, now],
    )?;
    Ok(AiValidation {
        id,
        document_id: document_id.to_string(),
        validator_name: validator_name.to_string(),
        results_json: results_json.to_string(),
        severity_summary: severity_summary.to_string(),
        created_at: now,
    })
}

pub(super) fn list_for_document(
    conn: &Connection,
    document_id: &str,
) -> AppResult<Vec<AiValidation>> {
    let mut stmt = conn.prepare(
        "SELECT id, document_id, validator_name, results_json, severity_summary, created_at
         FROM ai_validations WHERE document_id = ?1
         ORDER BY created_at DESC, rowid DESC",
    )?;
    let rows = stmt.query_map(params![document_id], |r| {
        Ok(AiValidation {
            id: r.get(0)?,
            document_id: r.get(1)?,
            validator_name: r.get(2)?,
            results_json: r.get(3)?,
            severity_summary: r.get(4)?,
            created_at: r.get(5)?,
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
    use crate::domain::{DocumentInput, DocumentType};

    fn doc(s: &impl StorageService, project_id: &str) -> String {
        s.create_document(DocumentInput {
            project_id: project_id.to_string(),
            parent_id: None,
            title: "Cap".into(),
            doc_type: DocumentType::Chapter,
            content: Some("<p>x</p>".into()),
        })
        .unwrap()
        .id
    }

    #[test]
    fn record_and_list_newest_first() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d = doc(&s, &p.id);
        s.record_ai_validation(&d, "character", "[]", "sin hallazgos")
            .unwrap();
        s.record_ai_validation(&d, "voice", "[{}]", "1 advertencia")
            .unwrap();
        let list = s.list_ai_validations(&d).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].validator_name, "voice");
        assert_eq!(list[1].validator_name, "character");
    }

    #[test]
    fn cascade_deletes_with_document() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d = doc(&s, &p.id);
        s.record_ai_validation(&d, "style", "[]", "sin hallazgos")
            .unwrap();
        s.delete_document(&d).unwrap();
        assert!(s.list_ai_validations(&d).unwrap().is_empty());
    }
}
