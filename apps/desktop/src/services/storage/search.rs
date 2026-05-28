//! Full-text search against `documents_fts` (FTS5). Always scoped by
//! `project_id` to avoid cross-project leakage.
//!
//! The query string is wrapped in a single-token MATCH expression with
//! double-quoting + escaping so a user query like `foo " bar` doesn't
//! break the parser or inject FTS operators.

use rusqlite::{params, Connection};

use crate::domain::SearchHit;
use crate::error::AppResult;

/// Cap on results per query. Keeps payloads bounded and the UI snappy.
const LIMIT: i64 = 50;

pub(super) fn search(
    conn: &Connection,
    project_id: &str,
    query: &str,
) -> AppResult<Vec<SearchHit>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let match_expr = build_match_expr(trimmed);

    let mut stmt = conn.prepare(
        "SELECT d.id, d.project_id, d.title,
                snippet(documents_fts, 1, '<mark>', '</mark>', '…', 24) AS excerpt
         FROM documents_fts
         JOIN documents d ON d.rowid = documents_fts.rowid
         WHERE documents_fts MATCH ?1 AND d.project_id = ?2
         ORDER BY rank
         LIMIT ?3",
    )?;
    let rows = stmt
        .query_map(params![match_expr, project_id, LIMIT], |r| {
            Ok(SearchHit {
                document_id: r.get(0)?,
                project_id: r.get(1)?,
                title: r.get(2)?,
                excerpt: r.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Wrap the user query as a single phrase, doubling embedded quotes per
/// FTS5 syntax. Treating it as a phrase intentionally disables operator
/// parsing (AND/OR/NEAR/-) — safer default; advanced syntax can be opt-in
/// later.
fn build_match_expr(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 2);
    out.push('"');
    for ch in raw.chars() {
        if ch == '"' {
            out.push('"');
            out.push('"');
        } else {
            out.push(ch);
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::fresh;
    use super::super::StorageService;
    use super::build_match_expr;
    use crate::domain::{DocumentInput, DocumentType, ProjectInput};

    fn seed_project_with_two_docs() -> (impl StorageService, String, String, String) {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let d1 = s
            .create_document(DocumentInput {
                project_id: p.id.clone(),
                parent_id: None,
                title: "Capítulo 1: el viaje".into(),
                doc_type: DocumentType::Chapter,
                content: Some("<p>El protagonista emprende el viaje al norte.</p>".into()),
            })
            .unwrap();
        let d2 = s
            .create_document(DocumentInput {
                project_id: p.id.clone(),
                parent_id: None,
                title: "Capítulo 2".into(),
                doc_type: DocumentType::Chapter,
                content: Some("<p>Llega al desierto y conoce a un mercader.</p>".into()),
            })
            .unwrap();
        (s, p.id, d1.id, d2.id)
    }

    #[test]
    fn finds_match_in_content() {
        let (s, p, _d1, d2) = seed_project_with_two_docs();
        let hits = s.search_documents(&p, "desierto").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document_id, d2);
        assert!(hits[0].excerpt.contains("<mark>"));
    }

    #[test]
    fn diacritics_are_insensitive() {
        let (s, p, d1, _d2) = seed_project_with_two_docs();
        // "capitulo" without accent should still match "Capítulo".
        let hits = s.search_documents(&p, "capitulo").unwrap();
        assert!(hits.iter().any(|h| h.document_id == d1));
    }

    #[test]
    fn search_is_project_scoped() {
        use crate::domain::ProjectStatus;
        let (s, p1, _d1, _d2) = seed_project_with_two_docs();
        // Archive p1 first to satisfy the "1 active project" SQL invariant,
        // then a second active project lives alongside.
        s.set_project_status(&p1, ProjectStatus::Archived).unwrap();
        let p2 = s
            .create_project(ProjectInput {
                title: "Other".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        let hits = s.search_documents(&p2.id, "desierto").unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn empty_query_returns_empty() {
        let (s, p, _, _) = seed_project_with_two_docs();
        assert!(s.search_documents(&p, "").unwrap().is_empty());
        assert!(s.search_documents(&p, "   ").unwrap().is_empty());
    }

    #[test]
    fn updated_content_is_reindexed() {
        let (s, p, d1, _) = seed_project_with_two_docs();
        // Pre: "viaje" hits d1 (it's in the content).
        assert!(!s.search_documents(&p, "viaje").unwrap().is_empty());
        // Replace BOTH title and content so no surviving token references d1.
        s.update_document(
            &d1,
            Some("Capítulo renombrado"),
            Some("<p>Texto reemplazado: dragón.</p>"),
            None,
        )
        .unwrap();
        let viaje_hits = s.search_documents(&p, "viaje").unwrap();
        assert!(
            viaje_hits.iter().all(|h| h.document_id != d1),
            "stale FTS entry: d1 still matches 'viaje' after both title and content were replaced"
        );
        let dragon_hits = s.search_documents(&p, "dragón").unwrap();
        assert!(dragon_hits.iter().any(|h| h.document_id == d1));
    }

    #[test]
    fn deleted_document_is_removed_from_index() {
        let (s, p, d1, _) = seed_project_with_two_docs();
        s.delete_document(&d1).unwrap();
        let hits = s.search_documents(&p, "viaje").unwrap();
        assert!(hits.iter().all(|h| h.document_id != d1));
    }

    #[test]
    fn user_quotes_do_not_break_query() {
        let (s, p, _, _) = seed_project_with_two_docs();
        // Should not error or panic even with embedded quote / FTS operator.
        let r = s.search_documents(&p, "AND \" - foo");
        assert!(r.is_ok());
    }

    #[test]
    fn build_match_expr_escapes_quotes() {
        assert_eq!(build_match_expr("hola"), "\"hola\"");
        assert_eq!(build_match_expr("he said \"hi\""), "\"he said \"\"hi\"\"\"");
    }
}
