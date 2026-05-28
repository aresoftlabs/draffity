//! Storage abstraction. The free MVP ships only `LocalStorageService`
//! (single SQLite file holding all projects). Premium can add a
//! `CloudSyncStorageService` that wraps this one without changing the trait.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::Value as JsonValue;

use crate::domain::{
    new_id, now_ms, DocNode, DocumentInput, DocumentType, Project, ProjectInput, ProjectStatus,
    Snapshot, TemplateNode, WritingStats,
};
use crate::error::{AppError, AppResult};

/// Embedded migrations applied in order. Each entry is `(version, sql)`.
/// Premium adds entries in the 100_* range without touching MVP migrations.
const MIGRATIONS: &[(u32, &str)] = &[(1, include_str!("../migrations/001_init.sql"))];

pub trait StorageService: Send + Sync {
    fn migrate(&self) -> AppResult<()>;

    // Projects
    fn create_project(&self, input: ProjectInput) -> AppResult<Project>;
    /// Atomically create a project and seed its initial document tree from a
    /// template structure. The whole operation lives in a single SQLite
    /// transaction — on any failure nothing is persisted.
    fn create_project_atomic(
        &self,
        input: ProjectInput,
        structure: &[TemplateNode],
    ) -> AppResult<Project>;
    fn list_projects(&self) -> AppResult<Vec<Project>>;
    fn get_project(&self, id: &str) -> AppResult<Option<Project>>;
    fn get_active_project(&self) -> AppResult<Option<Project>>;
    fn set_project_status(&self, id: &str, status: ProjectStatus) -> AppResult<()>;
    fn delete_project(&self, id: &str) -> AppResult<()>;

    // Documents
    fn create_document(&self, input: DocumentInput) -> AppResult<DocNode>;
    fn list_documents(&self, project_id: &str) -> AppResult<Vec<DocNode>>;
    fn get_document(&self, id: &str) -> AppResult<Option<DocNode>>;
    fn update_document(
        &self,
        id: &str,
        title: Option<&str>,
        content: Option<&str>,
    ) -> AppResult<DocNode>;
    fn move_document(&self, id: &str, parent_id: Option<&str>, position: i64) -> AppResult<()>;
    fn delete_document(&self, id: &str) -> AppResult<()>;

    // Snapshots
    fn create_snapshot(&self, document_id: &str, label: Option<&str>) -> AppResult<Snapshot>;
    fn list_snapshots(&self, document_id: &str) -> AppResult<Vec<Snapshot>>;
    /// Restore a document's content from a snapshot. Creates an automatic
    /// "auto-restore" snapshot of the current state first so the user can
    /// undo the operation.
    fn restore_snapshot(&self, snapshot_id: &str) -> AppResult<DocNode>;

    // Settings
    fn get_setting(&self, key: &str) -> AppResult<Option<String>>;
    fn set_setting(&self, key: &str, value: &str) -> AppResult<()>;

    // Writing stats
    /// Record that the user wrote today. Updates the streak counters and
    /// returns the resulting stats.
    fn record_writing_activity(&self) -> AppResult<WritingStats>;
    /// Read the current writing stats. If the last activity was more than
    /// one day ago the streak is reported as 0 (the broken streak is not
    /// persisted until the next `record_writing_activity`).
    fn get_writing_stats(&self) -> AppResult<WritingStats>;
}

/// Local SQLite-backed implementation. Single connection guarded by Mutex —
/// SQLite serializes writes anyway, and our access pattern is light.
pub struct LocalStorageService {
    conn: Mutex<Connection>,
    #[allow(dead_code)] // useful for diagnostics / future cloud sync hooks
    path: PathBuf,
}

impl LocalStorageService {
    pub fn open(path: impl AsRef<Path>) -> AppResult<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        Self::tune(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    pub fn open_in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory()?;
        Self::tune(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: PathBuf::from(":memory:"),
        })
    }

    fn tune(conn: &Connection) -> AppResult<()> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        Ok(())
    }

    fn current_version(conn: &Connection) -> AppResult<u32> {
        // If the meta table doesn't exist yet, version is 0.
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='meta'",
                [],
                |_| Ok(true),
            )
            .optional()?
            .unwrap_or(false);
        if !exists {
            return Ok(0);
        }
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key='schema_version'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(v.and_then(|s| s.parse().ok()).unwrap_or(0))
    }
}

fn row_to_project(r: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    let metadata_json: Option<String> = r.get("metadata")?;
    let metadata = metadata_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<JsonValue>(s).ok());
    let status_str: String = r.get("status")?;
    let status = match status_str.as_str() {
        "active" => ProjectStatus::Active,
        _ => ProjectStatus::Archived,
    };
    Ok(Project {
        id: r.get("id")?,
        title: r.get("title")?,
        template_id: r.get("template_id")?,
        status,
        metadata,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
    })
}

fn row_to_document(r: &rusqlite::Row<'_>) -> rusqlite::Result<DocNode> {
    let doc_type_str: String = r.get("doc_type")?;
    let doc_type = match doc_type_str.as_str() {
        "chapter" => DocumentType::Chapter,
        "scene" => DocumentType::Scene,
        "note" => DocumentType::Note,
        "folder" => DocumentType::Folder,
        "manga_page" => DocumentType::MangaPage,
        _ => DocumentType::Note,
    };
    Ok(DocNode {
        id: r.get("id")?,
        project_id: r.get("project_id")?,
        parent_id: r.get("parent_id")?,
        title: r.get("title")?,
        doc_type,
        content: r.get("content")?,
        position: r.get("position")?,
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
    })
}

impl StorageService for LocalStorageService {
    fn migrate(&self) -> AppResult<()> {
        let mut conn = self.conn.lock().unwrap();
        let mut current = Self::current_version(&conn)?;
        for (target, sql) in MIGRATIONS {
            if *target > current {
                let tx = conn.transaction()?;
                tx.execute_batch(sql)?;
                tx.execute(
                    "INSERT OR REPLACE INTO meta(key, value) VALUES('schema_version', ?1)",
                    params![target.to_string()],
                )?;
                tx.commit()?;
                current = *target;
                tracing::info!(version = target, "applied migration");
            }
        }
        Ok(())
    }

    fn create_project(&self, input: ProjectInput) -> AppResult<Project> {
        input.validate()?;
        let conn = self.conn.lock().unwrap();
        let now = now_ms();
        let project = Project {
            id: new_id(),
            title: input.title.trim().to_string(),
            template_id: input.template_id,
            status: ProjectStatus::Active,
            metadata: input.metadata,
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

    fn create_project_atomic(
        &self,
        input: ProjectInput,
        structure: &[TemplateNode],
    ) -> AppResult<Project> {
        input.validate()?;
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let now = now_ms();
        let project = Project {
            id: new_id(),
            title: input.title.trim().to_string(),
            template_id: input.template_id,
            status: ProjectStatus::Active,
            metadata: input.metadata,
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

    fn list_projects(&self) -> AppResult<Vec<Project>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, template_id, status, metadata, created_at, updated_at
             FROM projects ORDER BY status='active' DESC, updated_at DESC",
        )?;
        let rows = stmt
            .query_map([], row_to_project)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn get_project(&self, id: &str) -> AppResult<Option<Project>> {
        let conn = self.conn.lock().unwrap();
        let p = conn
            .query_row(
                "SELECT id, title, template_id, status, metadata, created_at, updated_at
                 FROM projects WHERE id=?1",
                params![id],
                row_to_project,
            )
            .optional()?;
        Ok(p)
    }

    fn get_active_project(&self) -> AppResult<Option<Project>> {
        let conn = self.conn.lock().unwrap();
        let p = conn
            .query_row(
                "SELECT id, title, template_id, status, metadata, created_at, updated_at
                 FROM projects WHERE status='active' LIMIT 1",
                [],
                row_to_project,
            )
            .optional()?;
        Ok(p)
    }

    fn set_project_status(&self, id: &str, status: ProjectStatus) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE projects SET status=?1, updated_at=?2 WHERE id=?3",
            params![status.as_str(), now_ms(), id],
        )?;
        if updated == 0 {
            return Err(AppError::NotFound(format!("project {id}")));
        }
        Ok(())
    }

    fn delete_project(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let removed = conn.execute("DELETE FROM projects WHERE id=?1", params![id])?;
        if removed == 0 {
            return Err(AppError::NotFound(format!("project {id}")));
        }
        Ok(())
    }

    fn create_document(&self, input: DocumentInput) -> AppResult<DocNode> {
        input.validate()?;
        let conn = self.conn.lock().unwrap();

        // Verify parent project exists.
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

        // Compute next position within (project, parent).
        let next_pos: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM documents
                 WHERE project_id=?1 AND IFNULL(parent_id, '')=IFNULL(?2, '')",
                params![input.project_id, input.parent_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let now = now_ms();
        let doc = DocNode {
            id: new_id(),
            project_id: input.project_id,
            parent_id: input.parent_id,
            title: input.title.trim().to_string(),
            doc_type: input.doc_type,
            content: input.content,
            position: next_pos,
            created_at: now,
            updated_at: now,
        };

        conn.execute(
            "INSERT INTO documents(id, project_id, parent_id, title, doc_type, content, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                doc.id,
                doc.project_id,
                doc.parent_id,
                doc.title,
                doc.doc_type.as_str(),
                doc.content,
                doc.position,
                doc.created_at,
                doc.updated_at,
            ],
        )?;
        Ok(doc)
    }

    fn list_documents(&self, project_id: &str) -> AppResult<Vec<DocNode>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, parent_id, title, doc_type, content, position, created_at, updated_at
             FROM documents
             WHERE project_id=?1
             ORDER BY IFNULL(parent_id,''), position ASC",
        )?;
        let rows = stmt
            .query_map(params![project_id], row_to_document)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn get_document(&self, id: &str) -> AppResult<Option<DocNode>> {
        let conn = self.conn.lock().unwrap();
        let d = conn
            .query_row(
                "SELECT id, project_id, parent_id, title, doc_type, content, position, created_at, updated_at
                 FROM documents WHERE id=?1",
                params![id],
                row_to_document,
            )
            .optional()?;
        Ok(d)
    }

    fn update_document(
        &self,
        id: &str,
        title: Option<&str>,
        content: Option<&str>,
    ) -> AppResult<DocNode> {
        let conn = self.conn.lock().unwrap();
        if title.is_none() && content.is_none() {
            // No-op: still bump updated_at to reflect read.
        }
        if let Some(t) = title {
            if t.trim().is_empty() {
                return Err(AppError::Invariant("title cannot be empty".into()));
            }
        }
        let now = now_ms();
        let updated = conn.execute(
            "UPDATE documents
             SET title = COALESCE(?2, title),
                 content = COALESCE(?3, content),
                 updated_at = ?4
             WHERE id=?1",
            params![id, title, content, now],
        )?;
        if updated == 0 {
            return Err(AppError::NotFound(format!("document {id}")));
        }
        // Re-read.
        let doc = conn
            .query_row(
                "SELECT id, project_id, parent_id, title, doc_type, content, position, created_at, updated_at
                 FROM documents WHERE id=?1",
                params![id],
                row_to_document,
            )?;
        Ok(doc)
    }

    fn move_document(&self, id: &str, parent_id: Option<&str>, position: i64) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE documents SET parent_id=?2, position=?3, updated_at=?4 WHERE id=?1",
            params![id, parent_id, position, now_ms()],
        )?;
        if updated == 0 {
            return Err(AppError::NotFound(format!("document {id}")));
        }
        Ok(())
    }

    fn delete_document(&self, id: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        let removed = conn.execute("DELETE FROM documents WHERE id=?1", params![id])?;
        if removed == 0 {
            return Err(AppError::NotFound(format!("document {id}")));
        }
        Ok(())
    }

    fn create_snapshot(&self, document_id: &str, label: Option<&str>) -> AppResult<Snapshot> {
        let conn = self.conn.lock().unwrap();
        // Read current document content first.
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

    fn list_snapshots(&self, document_id: &str) -> AppResult<Vec<Snapshot>> {
        let conn = self.conn.lock().unwrap();
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

    fn restore_snapshot(&self, snapshot_id: &str) -> AppResult<DocNode> {
        let mut conn = self.conn.lock().unwrap();
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
            "SELECT id, project_id, parent_id, title, doc_type, content, position, created_at, updated_at
             FROM documents WHERE id=?1",
            params![document_id],
            row_to_document,
        )?;

        tx.commit()?;
        Ok(doc)
    }

    fn get_setting(&self, key: &str) -> AppResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let v: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key=?1",
                params![key],
                |r| r.get(0),
            )
            .optional()?;
        Ok(v)
    }

    fn set_setting(&self, key: &str, value: &str) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    fn record_writing_activity(&self) -> AppResult<WritingStats> {
        use chrono::{Datelike, Local, NaiveDate};

        let conn = self.conn.lock().unwrap();
        let today: NaiveDate = Local::now().date_naive();
        let today_str = format!(
            "{:04}-{:02}-{:02}",
            today.year(),
            today.month(),
            today.day()
        );

        let last: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key='writing.last_date'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let stored_streak: u32 = conn
            .query_row(
                "SELECT value FROM settings WHERE key='writing.current_streak'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let mut longest: u32 = conn
            .query_row(
                "SELECT value FROM settings WHERE key='writing.longest_streak'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let new_streak = match last
            .as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        {
            Some(d) if d == today => stored_streak.max(1),
            Some(d) if d.succ_opt() == Some(today) => stored_streak + 1,
            _ => 1,
        };
        if new_streak > longest {
            longest = new_streak;
        }

        let upsert = |k: &str, v: &str| -> AppResult<()> {
            conn.execute(
                "INSERT INTO settings(key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                params![k, v],
            )?;
            Ok(())
        };
        upsert("writing.last_date", &today_str)?;
        upsert("writing.current_streak", &new_streak.to_string())?;
        upsert("writing.longest_streak", &longest.to_string())?;

        Ok(WritingStats {
            current_streak: new_streak,
            longest_streak: longest,
            last_writing_date: Some(today_str),
        })
    }

    fn get_writing_stats(&self) -> AppResult<WritingStats> {
        use chrono::{Local, NaiveDate};

        let conn = self.conn.lock().unwrap();
        let last: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key='writing.last_date'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let stored_streak: u32 = conn
            .query_row(
                "SELECT value FROM settings WHERE key='writing.current_streak'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let longest: u32 = conn
            .query_row(
                "SELECT value FROM settings WHERE key='writing.longest_streak'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // The streak persists if the most recent activity was today or yesterday.
        let current = match last
            .as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        {
            Some(d) => {
                let today = Local::now().date_naive();
                let diff = today.signed_duration_since(d).num_days();
                if diff <= 1 {
                    stored_streak
                } else {
                    0
                }
            }
            None => 0,
        };

        Ok(WritingStats {
            current_streak: current,
            longest_streak: longest,
            last_writing_date: last,
        })
    }
}

/// Recursively insert a template's structure as documents. Position is
/// per (project, parent) starting at 0. Synopsis becomes seed content
/// wrapped in a `<p>` so it round-trips through TipTap.
fn insert_template_nodes(
    tx: &Transaction<'_>,
    project_id: &str,
    parent_id: Option<&str>,
    nodes: &[TemplateNode],
    now: i64,
) -> AppResult<()> {
    for (idx, node) in nodes.iter().enumerate() {
        let id = new_id();
        let content = node
            .synopsis
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| format!("<p>{}</p>", escape_html(s)));
        tx.execute(
            "INSERT INTO documents(id, project_id, parent_id, title, doc_type, content, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                project_id,
                parent_id,
                node.title.trim(),
                node.doc_type.as_str(),
                content,
                idx as i64,
                now,
                now,
            ],
        )?;
        if !node.children.is_empty() {
            insert_template_nodes(tx, project_id, Some(&id), &node.children, now)?;
        }
    }
    Ok(())
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TemplateNode;

    fn fresh() -> LocalStorageService {
        let s = LocalStorageService::open_in_memory().expect("in-memory SQLite should always open");
        s.migrate().expect("fresh DB migrate should succeed");
        s
    }

    #[test]
    fn migrate_is_idempotent() {
        let s = fresh();
        // Re-running migrations must not error nor duplicate data.
        s.migrate().unwrap();
        s.migrate().unwrap();
    }

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
    fn document_crud_round_trip() {
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
                project_id: p.id.clone(),
                parent_id: None,
                title: "Cap 1".into(),
                doc_type: DocumentType::Chapter,
                content: Some("hola".into()),
            })
            .unwrap();
        assert_eq!(d.position, 0);

        let d2 = s
            .create_document(DocumentInput {
                project_id: p.id.clone(),
                parent_id: None,
                title: "Cap 2".into(),
                doc_type: DocumentType::Chapter,
                content: None,
            })
            .unwrap();
        assert_eq!(d2.position, 1);

        let updated = s
            .update_document(&d.id, Some("Cap 1 — bis"), Some("nuevo"))
            .unwrap();
        assert_eq!(updated.title, "Cap 1 — bis");
        assert_eq!(updated.content.as_deref(), Some("nuevo"));

        s.delete_document(&d2.id).unwrap();
        let docs = s.list_documents(&p.id).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].id, d.id);
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

        // Synopsis is escaped and wrapped as a paragraph
        let cap2 = children.iter().find(|d| d.title == "Capítulo 2").unwrap();
        let content = cap2.content.as_deref().unwrap();
        assert!(content.contains("<p>"));
        assert!(content.contains("&lt;con HTML&gt;"));
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

    #[test]
    fn settings_round_trip_and_upsert() {
        let s = fresh();
        assert!(s.get_setting("k").unwrap().is_none());
        s.set_setting("k", "v1").unwrap();
        assert_eq!(s.get_setting("k").unwrap().as_deref(), Some("v1"));
        s.set_setting("k", "v2").unwrap();
        assert_eq!(s.get_setting("k").unwrap().as_deref(), Some("v2"));
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
        s.update_document(&d.id, None, Some("v2")).unwrap();

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

    #[test]
    fn record_writing_activity_initializes_streak_and_sets_today() {
        let s = fresh();
        let stats = s.record_writing_activity().unwrap();
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.longest_streak, 1);
        assert!(stats.last_writing_date.is_some());
    }

    #[test]
    fn record_writing_activity_is_idempotent_within_same_day() {
        let s = fresh();
        let a = s.record_writing_activity().unwrap();
        let b = s.record_writing_activity().unwrap();
        assert_eq!(b.current_streak, a.current_streak);
    }

    #[test]
    fn get_writing_stats_reports_zero_streak_when_last_activity_is_old() {
        let s = fresh();
        s.set_setting("writing.last_date", "2020-01-01").unwrap();
        s.set_setting("writing.current_streak", "42").unwrap();
        s.set_setting("writing.longest_streak", "42").unwrap();
        let stats = s.get_writing_stats().unwrap();
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.longest_streak, 42);
    }

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
}
