//! Project CRUD + atomic creation with template seed.

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{new_id, now_ms, Project, ProjectInput, ProjectStatus, TemplateNode};
use crate::error::{AppError, AppResult};

use super::import_seed::insert_import_nodes;
use super::row_mappers::row_to_project;
use super::template_seed::insert_template_nodes;
use crate::services::importer::ImportTree;

/// Column list for `SELECT` against `projects`. Mirrors the documents.rs
/// pattern so adding columns (e.g. `goal_words`) is a one-line change.
const COLS: &str = "id, title, template_id, status, metadata, goal_words, created_at, updated_at";

pub(super) fn create(conn: &Connection, input: ProjectInput) -> AppResult<Project> {
    input.validate()?;
    let now = now_ms();
    let project = Project {
        id: new_id(),
        title: input.title.trim().to_string(),
        template_id: input.template_id,
        status: ProjectStatus::Active,
        metadata: input.metadata,
        goal_words: None,
        created_at: now,
        updated_at: now,
    };
    let metadata_str = match &project.metadata {
        Some(v) => Some(serde_json::to_string(v)?),
        None => None,
    };
    conn.execute(
        "INSERT INTO projects(id, title, template_id, status, metadata, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            project.id,
            project.title,
            project.template_id,
            project.status.as_str(),
            metadata_str,
            project.created_at,
            project.updated_at,
        ],
    )?;
    Ok(project)
}

pub(super) fn create_atomic(
    conn: &mut Connection,
    input: ProjectInput,
    structure: &[TemplateNode],
) -> AppResult<Project> {
    input.validate()?;
    let tx = conn.transaction()?;

    let now = now_ms();
    let project = Project {
        id: new_id(),
        title: input.title.trim().to_string(),
        template_id: input.template_id,
        status: ProjectStatus::Active,
        metadata: input.metadata,
        goal_words: None,
        created_at: now,
        updated_at: now,
    };
    let metadata_str = match &project.metadata {
        Some(v) => Some(serde_json::to_string(v)?),
        None => None,
    };

    tx.execute(
        "INSERT INTO projects(id, title, template_id, status, metadata, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            project.id,
            project.title,
            project.template_id,
            project.status.as_str(),
            metadata_str,
            project.created_at,
            project.updated_at,
        ],
    )?;

    insert_template_nodes(&tx, &project.id, None, structure, now)?;

    tx.commit()?;
    Ok(project)
}

/// Create a project + its document tree from an importer's output in a
/// single transaction. The project's title comes from the import tree's
/// `project_title` and the imported nodes are seeded under the new
/// project's root. `template_id` is "generic" because an imported tree
/// owns its structure outright — there's no template to honour.
pub(super) fn create_from_import(
    conn: &mut Connection,
    tree: &ImportTree,
    template_id: &str,
) -> AppResult<Project> {
    let trimmed_title = tree.project_title.trim();
    if trimmed_title.is_empty() {
        return Err(AppError::Invariant("project title cannot be empty".into()));
    }
    let tx = conn.transaction()?;
    let now = now_ms();
    let project = Project {
        id: new_id(),
        title: trimmed_title.to_string(),
        template_id: template_id.to_string(),
        status: ProjectStatus::Active,
        metadata: None,
        goal_words: None,
        created_at: now,
        updated_at: now,
    };
    tx.execute(
        "INSERT INTO projects(id, title, template_id, status, metadata, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            project.id,
            project.title,
            project.template_id,
            project.status.as_str(),
            None::<String>,
            project.created_at,
            project.updated_at,
        ],
    )?;
    insert_import_nodes(&tx, &project.id, None, &tree.nodes, now)?;
    tx.commit()?;
    Ok(project)
}

pub(super) fn list(conn: &Connection) -> AppResult<Vec<Project>> {
    let sql = format!("SELECT {COLS} FROM projects ORDER BY status='active' DESC, updated_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map([], row_to_project)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub(super) fn get(conn: &Connection, id: &str) -> AppResult<Option<Project>> {
    let sql = format!("SELECT {COLS} FROM projects WHERE id=?1");
    let p = conn
        .query_row(&sql, params![id], row_to_project)
        .optional()?;
    Ok(p)
}

pub(super) fn get_active(conn: &Connection) -> AppResult<Option<Project>> {
    let sql = format!("SELECT {COLS} FROM projects WHERE status='active' LIMIT 1");
    let p = conn.query_row(&sql, [], row_to_project).optional()?;
    Ok(p)
}

pub(super) fn set_status(conn: &Connection, id: &str, status: ProjectStatus) -> AppResult<()> {
    let updated = conn.execute(
        "UPDATE projects SET status=?1, updated_at=?2 WHERE id=?3",
        params![status.as_str(), now_ms(), id],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("project {id}")));
    }
    Ok(())
}

/// Set or clear the project's target word count. `None` removes the goal.
pub(super) fn set_goal(conn: &Connection, id: &str, goal: Option<i64>) -> AppResult<Project> {
    let updated = conn.execute(
        "UPDATE projects SET goal_words=?2, updated_at=?3 WHERE id=?1",
        params![id, goal, now_ms()],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("project {id}")));
    }
    let sql = format!("SELECT {COLS} FROM projects WHERE id=?1");
    Ok(conn.query_row(&sql, params![id], row_to_project)?)
}

pub(super) fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let removed = conn.execute("DELETE FROM projects WHERE id=?1", params![id])?;
    if removed == 0 {
        return Err(AppError::NotFound(format!("project {id}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::test_helpers::fresh;
    use super::super::StorageService;
    use crate::domain::{DocumentInput, DocumentType, ProjectInput, ProjectStatus, TemplateNode};
    use crate::error::AppError;

    #[test]
    fn create_and_list_project() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "Mi novela".into(),
                template_id: "novela-tres-actos".into(),
                metadata: None,
            })
            .unwrap();
        assert_eq!(p.status, ProjectStatus::Active);
        let all = s.list_projects().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, p.id);
    }

    #[test]
    fn unique_active_project_constraint_enforced_by_db() {
        let s = fresh();
        s.create_project(ProjectInput {
            title: "A".into(),
            template_id: "x".into(),
            metadata: None,
        })
        .unwrap();
        // Attempting to create a second active project must fail at SQL level
        // (unique partial index). The ProjectManager is the layer that turns
        // this into the "archive previous, then activate" workflow.
        let err = s.create_project(ProjectInput {
            title: "B".into(),
            template_id: "x".into(),
            metadata: None,
        });
        assert!(err.is_err());
    }

    #[test]
    fn invalid_project_input_rejected_before_db() {
        let s = fresh();
        let err = s
            .create_project(ProjectInput {
                title: "  ".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap_err();
        assert!(matches!(err, AppError::Invariant(_)));
    }

    #[test]
    fn deleting_project_cascades_documents() {
        let s = fresh();
        let p = s
            .create_project(ProjectInput {
                title: "P".into(),
                template_id: "x".into(),
                metadata: None,
            })
            .unwrap();
        s.create_document(DocumentInput {
            project_id: p.id.clone(),
            parent_id: None,
            title: "C".into(),
            doc_type: DocumentType::Chapter,
            content: None,
        })
        .unwrap();
        s.delete_project(&p.id).unwrap();
        let docs = s.list_documents(&p.id).unwrap();
        assert!(docs.is_empty());
    }

    #[test]
    fn create_project_atomic_seeds_full_tree_in_one_tx() {
        let s = fresh();
        let structure = vec![
            TemplateNode {
                title: "Acto 1".into(),
                doc_type: DocumentType::Folder,
                synopsis: Some("Planteamiento".into()),
                children: vec![
                    TemplateNode {
                        title: "Capítulo 1".into(),
                        doc_type: DocumentType::Chapter,
                        synopsis: None,
                        children: vec![],
                    },
                    TemplateNode {
                        title: "Capítulo 2".into(),
                        doc_type: DocumentType::Chapter,
                        synopsis: Some("Inciting incident <con HTML>".into()),
                        children: vec![],
                    },
                ],
            },
            TemplateNode {
                title: "Acto 2".into(),
                doc_type: DocumentType::Folder,
                synopsis: None,
                children: vec![],
            },
        ];

        let p = s
            .create_project_atomic(
                ProjectInput {
                    title: "Mi novela".into(),
                    template_id: "novela-tres-actos".into(),
                    metadata: None,
                },
                &structure,
            )
            .unwrap();

        let docs = s.list_documents(&p.id).unwrap();
        assert_eq!(docs.len(), 4);

        // Folders at the root, in order
        let roots: Vec<_> = docs.iter().filter(|d| d.parent_id.is_none()).collect();
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].title, "Acto 1");
        assert_eq!(roots[1].title, "Acto 2");
        assert_eq!(roots[0].position, 0);
        assert_eq!(roots[1].position, 1);

        // Children of Acto 1, ordered
        let acto1_id = roots[0].id.clone();
        let children: Vec<_> = docs
            .iter()
            .filter(|d| d.parent_id.as_deref() == Some(&acto1_id))
            .collect();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].position, 0);
        assert_eq!(children[1].position, 1);

        // Synopsis lives on its own column now (no HTML wrapping). Content
        // stays NULL for seeded docs; the user fills it in afterwards.
        let cap2 = children.iter().find(|d| d.title == "Capítulo 2").unwrap();
        assert_eq!(
            cap2.synopsis.as_deref(),
            Some("Inciting incident <con HTML>")
        );
        assert!(cap2.content.is_none());
    }

    #[test]
    fn create_project_atomic_rolls_back_on_invalid_input() {
        let s = fresh();
        // Empty title triggers validation; nothing should be inserted.
        let result = s.create_project_atomic(
            ProjectInput {
                title: " ".into(),
                template_id: "x".into(),
                metadata: None,
            },
            &[],
        );
        assert!(result.is_err());
        assert_eq!(s.list_projects().unwrap().len(), 0);
    }
}
