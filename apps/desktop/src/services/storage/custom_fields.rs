//! Custom metadata fields (I-08/I-09): per-project field definitions
//! (`custom_fields`) + per-document values (`document_custom_values`). Field
//! values surface on `DocNode::metadata` (a `field id → value` map loaded in
//! the documents SELECT). Setting a value validates that the field belongs to
//! the document's project and, for `Select` fields, that the value is allowed.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{
    clean_options, new_id, now_ms, CustomField, CustomFieldInput, CustomFieldKind, DocNode,
};
use crate::error::{AppError, AppResult};

use super::documents::select_one;

const COLS: &str = "id, project_id, name, kind, options_json, position, created_at";

fn row_to_field(r: &rusqlite::Row<'_>) -> rusqlite::Result<CustomField> {
    let kind_str: String = r.get("kind")?;
    let kind = CustomFieldKind::parse(&kind_str).unwrap_or(CustomFieldKind::Text);
    let options = r
        .get::<_, Option<String>>("options_json")?
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default();
    Ok(CustomField {
        id: r.get("id")?,
        project_id: r.get("project_id")?,
        name: r.get("name")?,
        kind,
        options,
        position: r.get("position")?,
        created_at: r.get("created_at")?,
    })
}

/// Serialize the option list for storage — `None` for non-select kinds so the
/// column stays NULL when it carries no meaning.
fn options_json(kind: CustomFieldKind, options: &[String]) -> Option<String> {
    if kind == CustomFieldKind::Select {
        serde_json::to_string(&clean_options(options)).ok()
    } else {
        None
    }
}

pub(super) fn create(conn: &Connection, input: CustomFieldInput) -> AppResult<CustomField> {
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

    let next_pos: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM custom_fields WHERE project_id=?1",
            params![input.project_id],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);

    let field = CustomField {
        id: new_id(),
        project_id: input.project_id,
        name: input.name.trim().to_string(),
        kind: input.kind,
        options: clean_options(&input.options),
        position: next_pos,
        created_at: now_ms(),
    };
    conn.execute(
        "INSERT INTO custom_fields(id, project_id, name, kind, options_json, position, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            field.id,
            field.project_id,
            field.name,
            field.kind.as_str(),
            options_json(field.kind, &field.options),
            field.position,
            field.created_at,
        ],
    )
    .map_err(map_unique)?;
    Ok(field)
}

/// A project's custom fields, ordered by their configured position.
pub(super) fn list(conn: &Connection, project_id: &str) -> AppResult<Vec<CustomField>> {
    let sql = format!("SELECT {COLS} FROM custom_fields WHERE project_id=?1 ORDER BY position ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params![project_id], row_to_field)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn get_one(conn: &Connection, id: &str) -> AppResult<CustomField> {
    let sql = format!("SELECT {COLS} FROM custom_fields WHERE id=?1");
    conn.query_row(&sql, params![id], row_to_field)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("custom field {id}")))
}

/// Rename and/or change the option list of a field. The kind is immutable —
/// changing it would silently invalidate stored values.
pub(super) fn update(
    conn: &Connection,
    id: &str,
    name: &str,
    options: &[String],
) -> AppResult<CustomField> {
    let existing = get_one(conn, id)?;
    // Re-validate via the input rules (kind carried from the stored field).
    CustomFieldInput {
        project_id: existing.project_id.clone(),
        name: name.into(),
        kind: existing.kind,
        options: options.to_vec(),
    }
    .validate()?;

    conn.execute(
        "UPDATE custom_fields SET name=?2, options_json=?3 WHERE id=?1",
        params![id, name.trim(), options_json(existing.kind, options),],
    )
    .map_err(map_unique)?;
    get_one(conn, id)
}

pub(super) fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    // `document_custom_values` rows cascade via the FK.
    let removed = conn.execute("DELETE FROM custom_fields WHERE id=?1", params![id])?;
    if removed == 0 {
        return Err(AppError::NotFound(format!("custom field {id}")));
    }
    Ok(())
}

/// Set (or clear, when `value` is `None`/blank) a document's value for one
/// field. Validates the field belongs to the document's project and, for
/// `Select` fields, that the value is one of the allowed options. Returns the
/// refreshed `DocNode`.
pub(super) fn set_value(
    conn: &Connection,
    doc_id: &str,
    field_id: &str,
    value: Option<&str>,
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

    let field = get_one(conn, field_id)?;
    if field.project_id != project_id {
        return Err(AppError::Invariant(format!(
            "field {field_id} not in project {project_id}"
        )));
    }

    let trimmed = value.map(str::trim).filter(|s| !s.is_empty());
    match trimmed {
        None => {
            conn.execute(
                "DELETE FROM document_custom_values WHERE document_id=?1 AND field_id=?2",
                params![doc_id, field_id],
            )?;
        }
        Some(v) => {
            if field.kind == CustomFieldKind::Select && !field.options.iter().any(|o| o == v) {
                return Err(AppError::Invariant(format!(
                    "'{v}' is not an allowed option for field {field_id}"
                )));
            }
            conn.execute(
                "INSERT INTO document_custom_values(document_id, field_id, value)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(document_id, field_id) DO UPDATE SET value=excluded.value",
                params![doc_id, field_id, v],
            )?;
        }
    }
    conn.execute(
        "UPDATE documents SET updated_at=?2 WHERE id=?1",
        params![doc_id, now_ms()],
    )?;
    select_one(conn, doc_id)
}

/// Map a UNIQUE-constraint violation to a friendly invariant error.
fn map_unique(e: rusqlite::Error) -> AppError {
    if e.to_string().contains("UNIQUE") {
        AppError::Invariant("a field with that name already exists".into())
    } else {
        AppError::from(e)
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::{fresh, seed_project};
    use super::super::StorageService;
    use crate::domain::{CustomFieldInput, CustomFieldKind, DocumentInput, DocumentType};
    use crate::error::AppError;

    fn make_chapter(s: &impl StorageService, project_id: &str) -> String {
        s.create_document(DocumentInput {
            project_id: project_id.into(),
            parent_id: None,
            title: "A".into(),
            doc_type: DocumentType::Chapter,
            content: None,
        })
        .unwrap()
        .id
    }

    fn mk_field(
        s: &impl StorageService,
        project_id: &str,
        name: &str,
        kind: CustomFieldKind,
        options: &[&str],
    ) -> String {
        s.create_custom_field(CustomFieldInput {
            project_id: project_id.into(),
            name: name.into(),
            kind,
            options: options.iter().map(|o| o.to_string()).collect(),
        })
        .unwrap()
        .id
    }

    #[test]
    fn create_lists_in_position_order() {
        let s = fresh();
        let p = seed_project(&s, "P");
        mk_field(&s, &p.id, "First", CustomFieldKind::Text, &[]);
        mk_field(&s, &p.id, "Second", CustomFieldKind::Number, &[]);
        let fields = s.list_custom_fields(&p.id).unwrap();
        assert_eq!(
            fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
            vec!["First", "Second"]
        );
        assert_eq!(fields[0].position, 0);
        assert_eq!(fields[1].position, 1);
    }

    #[test]
    fn duplicate_name_rejected() {
        let s = fresh();
        let p = seed_project(&s, "P");
        mk_field(&s, &p.id, "POV", CustomFieldKind::Text, &[]);
        let err = s
            .create_custom_field(CustomFieldInput {
                project_id: p.id.clone(),
                name: "POV".into(),
                kind: CustomFieldKind::Text,
                options: vec![],
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Invariant(_)));
    }

    #[test]
    fn set_value_round_trips_on_docnode_metadata() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d = make_chapter(&s, &p.id);
        let f = mk_field(&s, &p.id, "Reviewer", CustomFieldKind::Text, &[]);

        let after = s.set_document_metadata(&d, &f, Some("Alice")).unwrap();
        assert_eq!(after.metadata.get(&f).map(String::as_str), Some("Alice"));

        // Clearing removes the entry.
        let cleared = s.set_document_metadata(&d, &f, None).unwrap();
        assert!(!cleared.metadata.contains_key(&f));
    }

    #[test]
    fn set_value_upserts_existing() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d = make_chapter(&s, &p.id);
        let f = mk_field(&s, &p.id, "Reviewer", CustomFieldKind::Text, &[]);
        s.set_document_metadata(&d, &f, Some("Alice")).unwrap();
        let after = s.set_document_metadata(&d, &f, Some("Bob")).unwrap();
        assert_eq!(after.metadata.get(&f).map(String::as_str), Some("Bob"));
    }

    #[test]
    fn select_value_must_be_allowed() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d = make_chapter(&s, &p.id);
        let f = mk_field(&s, &p.id, "POV", CustomFieldKind::Select, &["Alice", "Bob"]);

        assert!(s.set_document_metadata(&d, &f, Some("Alice")).is_ok());
        let err = s.set_document_metadata(&d, &f, Some("Zoe")).unwrap_err();
        assert!(matches!(err, AppError::Invariant(_)));
    }

    #[test]
    fn deleting_field_cascades_values() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let d = make_chapter(&s, &p.id);
        let f = mk_field(&s, &p.id, "Tmp", CustomFieldKind::Text, &[]);
        s.set_document_metadata(&d, &f, Some("x")).unwrap();
        s.delete_custom_field(&f).unwrap();
        assert!(s.get_document(&d).unwrap().unwrap().metadata.is_empty());
    }

    #[test]
    fn update_renames_and_changes_options() {
        let s = fresh();
        let p = seed_project(&s, "P");
        let f = mk_field(&s, &p.id, "POV", CustomFieldKind::Select, &["Alice"]);
        let updated = s
            .update_custom_field(&f, "Point of View", &["Alice".into(), "Bob".into()])
            .unwrap();
        assert_eq!(updated.name, "Point of View");
        assert_eq!(
            updated.options,
            vec!["Alice".to_string(), "Bob".to_string()]
        );
    }
}
