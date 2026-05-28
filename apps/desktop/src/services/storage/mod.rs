//! Storage abstraction. The free MVP ships only `LocalStorageService`
//! (single SQLite file holding all projects). Premium can add a
//! `CloudSyncStorageService` that wraps this one without changing the trait.
//!
//! The trait impl is intentionally thin: it locks the connection and
//! delegates to per-topic submodules (`projects`, `documents`, `snapshots`,
//! `settings`, `stats`). New operations live in the matching submodule —
//! not here.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{
    Citation, CodexEntry, CodexInput, CodexKind, CodexUpdate, DocNode, DocumentInput,
    DocumentStatus, Project, ProjectInput, ProjectStatus, SearchHit, Snapshot, TemplateNode,
    WritingStats,
};
use crate::error::AppResult;

pub use citations::UpsertEntry as CitationUpsert;

mod citations;
mod codex;
mod documents;
mod projects;
mod row_mappers;
mod search;
mod settings;
mod snapshots;
mod stats;
mod template_seed;

/// Embedded migrations applied in order. Each entry is `(version, sql)`.
/// Premium adds entries in the 100_* range without touching MVP migrations.
const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!("../../migrations/001_init.sql")),
    (2, include_str!("../../migrations/002_fts.sql")),
    (3, include_str!("../../migrations/003_status_tags.sql")),
    (4, include_str!("../../migrations/004_goals.sql")),
    (5, include_str!("../../migrations/005_synopsis.sql")),
    (6, include_str!("../../migrations/006_doc_json.sql")),
    (7, include_str!("../../migrations/007_citations.sql")),
    (9, include_str!("../../migrations/009_codex.sql")),
];

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
        content_json: Option<&str>,
    ) -> AppResult<DocNode>;
    fn move_document(&self, id: &str, parent_id: Option<&str>, position: i64) -> AppResult<()>;
    fn delete_document(&self, id: &str) -> AppResult<()>;
    /// Atomically set `position` (and optionally `parent_id`) for every id
    /// in `ordered_ids` to its index in the slice. Used by binder drag&drop.
    fn reorder_documents(
        &self,
        project_id: &str,
        parent_id: Option<&str>,
        ordered_ids: &[String],
    ) -> AppResult<()>;
    /// Update the writing-pipeline `status` of a document and return the
    /// refreshed `DocNode`.
    fn set_document_status(&self, id: &str, status: DocumentStatus) -> AppResult<DocNode>;
    /// Replace the entire tag set of a document atomically.
    fn set_document_tags(&self, id: &str, tags: &[String]) -> AppResult<DocNode>;
    /// Distinct tags in use across all documents of a project (sorted).
    fn list_project_tags(&self, project_id: &str) -> AppResult<Vec<String>>;
    /// Set or clear a document's target word count.
    fn set_document_goal(&self, id: &str, goal: Option<i64>) -> AppResult<DocNode>;
    /// Set or clear a project's target word count.
    fn set_project_goal(&self, id: &str, goal: Option<i64>) -> AppResult<Project>;
    /// Set or clear a document's synopsis (short description surfaced in
    /// Corkboard / Outliner views, independent of content).
    fn set_document_synopsis(&self, id: &str, synopsis: Option<&str>) -> AppResult<DocNode>;

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

    // Citations (bibliography)
    /// List all citations attached to a project, sorted by key.
    fn list_citations(&self, project_id: &str) -> AppResult<Vec<Citation>>;
    /// List only the keys (cheaper than `list_citations`). Used by the
    /// editor autocomplete and the export rendering passes.
    fn list_citation_keys(&self, project_id: &str) -> AppResult<Vec<String>>;
    /// Upsert a batch atomically. Existing `(project_id, key)` pairs are
    /// overwritten in place (id and created_at preserved).
    fn upsert_citations(
        &self,
        project_id: &str,
        entries: &[CitationUpsert],
    ) -> AppResult<Vec<Citation>>;
    fn delete_citation(&self, id: &str) -> AppResult<()>;

    // Codex (worldbuilding)
    /// Create a new codex entry. `input` is validated and normalised first
    /// (trimmed name, deduped tags).
    fn create_codex_entry(&self, input: CodexInput) -> AppResult<CodexEntry>;
    /// List all entries of a project, alphabetical case-insensitive.
    fn list_codex_entries(&self, project_id: &str) -> AppResult<Vec<CodexEntry>>;
    fn get_codex_entry(&self, id: &str) -> AppResult<Option<CodexEntry>>;
    /// Patch-style update — only the `Some` fields move. An empty `body`
    /// (whitespace) clears the field, by design (matches the editor UX).
    fn update_codex_entry(&self, id: &str, patch: CodexUpdate) -> AppResult<CodexEntry>;
    fn delete_codex_entry(&self, id: &str) -> AppResult<()>;
    /// `LIKE`-based scan across name + body + tag JSON. Empty `query` lists
    /// everything (optionally narrowed by `kind`).
    fn search_codex_entries(
        &self,
        project_id: &str,
        query: &str,
        kind: Option<CodexKind>,
    ) -> AppResult<Vec<CodexEntry>>;

    // Search
    /// Full-text search across documents of a single project. Returns up to
    /// 50 hits ordered by FTS5 rank, each with a `<mark>`-wrapped excerpt.
    /// Empty or whitespace-only queries return `[]` without hitting the DB.
    fn search_documents(&self, project_id: &str, query: &str) -> AppResult<Vec<SearchHit>>;
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
        projects::create(&self.conn.lock().unwrap(), input)
    }

    fn create_project_atomic(
        &self,
        input: ProjectInput,
        structure: &[TemplateNode],
    ) -> AppResult<Project> {
        projects::create_atomic(&mut self.conn.lock().unwrap(), input, structure)
    }

    fn list_projects(&self) -> AppResult<Vec<Project>> {
        projects::list(&self.conn.lock().unwrap())
    }

    fn get_project(&self, id: &str) -> AppResult<Option<Project>> {
        projects::get(&self.conn.lock().unwrap(), id)
    }

    fn get_active_project(&self) -> AppResult<Option<Project>> {
        projects::get_active(&self.conn.lock().unwrap())
    }

    fn set_project_status(&self, id: &str, status: ProjectStatus) -> AppResult<()> {
        projects::set_status(&self.conn.lock().unwrap(), id, status)
    }

    fn delete_project(&self, id: &str) -> AppResult<()> {
        projects::delete(&self.conn.lock().unwrap(), id)
    }

    fn create_document(&self, input: DocumentInput) -> AppResult<DocNode> {
        documents::create(&self.conn.lock().unwrap(), input)
    }

    fn list_documents(&self, project_id: &str) -> AppResult<Vec<DocNode>> {
        documents::list(&self.conn.lock().unwrap(), project_id)
    }

    fn get_document(&self, id: &str) -> AppResult<Option<DocNode>> {
        documents::get(&self.conn.lock().unwrap(), id)
    }

    fn update_document(
        &self,
        id: &str,
        title: Option<&str>,
        content: Option<&str>,
        content_json: Option<&str>,
    ) -> AppResult<DocNode> {
        documents::update(&self.conn.lock().unwrap(), id, title, content, content_json)
    }

    fn move_document(&self, id: &str, parent_id: Option<&str>, position: i64) -> AppResult<()> {
        documents::move_to(&self.conn.lock().unwrap(), id, parent_id, position)
    }

    fn delete_document(&self, id: &str) -> AppResult<()> {
        documents::delete(&self.conn.lock().unwrap(), id)
    }

    fn reorder_documents(
        &self,
        project_id: &str,
        parent_id: Option<&str>,
        ordered_ids: &[String],
    ) -> AppResult<()> {
        documents::reorder(
            &mut self.conn.lock().unwrap(),
            project_id,
            parent_id,
            ordered_ids,
        )
    }

    fn set_document_status(&self, id: &str, status: DocumentStatus) -> AppResult<DocNode> {
        documents::set_status(&self.conn.lock().unwrap(), id, status)
    }

    fn set_document_tags(&self, id: &str, tags: &[String]) -> AppResult<DocNode> {
        documents::set_tags(&mut self.conn.lock().unwrap(), id, tags)
    }

    fn list_project_tags(&self, project_id: &str) -> AppResult<Vec<String>> {
        documents::list_project_tags(&self.conn.lock().unwrap(), project_id)
    }

    fn set_document_goal(&self, id: &str, goal: Option<i64>) -> AppResult<DocNode> {
        documents::set_goal(&self.conn.lock().unwrap(), id, goal)
    }

    fn set_project_goal(&self, id: &str, goal: Option<i64>) -> AppResult<Project> {
        projects::set_goal(&self.conn.lock().unwrap(), id, goal)
    }

    fn set_document_synopsis(&self, id: &str, synopsis: Option<&str>) -> AppResult<DocNode> {
        documents::set_synopsis(&self.conn.lock().unwrap(), id, synopsis)
    }

    fn create_snapshot(&self, document_id: &str, label: Option<&str>) -> AppResult<Snapshot> {
        snapshots::create(&self.conn.lock().unwrap(), document_id, label)
    }

    fn list_snapshots(&self, document_id: &str) -> AppResult<Vec<Snapshot>> {
        snapshots::list(&self.conn.lock().unwrap(), document_id)
    }

    fn restore_snapshot(&self, snapshot_id: &str) -> AppResult<DocNode> {
        snapshots::restore(&mut self.conn.lock().unwrap(), snapshot_id)
    }

    fn get_setting(&self, key: &str) -> AppResult<Option<String>> {
        settings::get(&self.conn.lock().unwrap(), key)
    }

    fn set_setting(&self, key: &str, value: &str) -> AppResult<()> {
        settings::set(&self.conn.lock().unwrap(), key, value)
    }

    fn record_writing_activity(&self) -> AppResult<WritingStats> {
        stats::record_activity(&self.conn.lock().unwrap())
    }

    fn get_writing_stats(&self) -> AppResult<WritingStats> {
        stats::get(&self.conn.lock().unwrap())
    }

    fn search_documents(&self, project_id: &str, query: &str) -> AppResult<Vec<SearchHit>> {
        search::search(&self.conn.lock().unwrap(), project_id, query)
    }

    fn list_citations(&self, project_id: &str) -> AppResult<Vec<Citation>> {
        citations::list(&self.conn.lock().unwrap(), project_id)
    }

    fn list_citation_keys(&self, project_id: &str) -> AppResult<Vec<String>> {
        citations::list_keys(&self.conn.lock().unwrap(), project_id)
    }

    fn upsert_citations(
        &self,
        project_id: &str,
        entries: &[CitationUpsert],
    ) -> AppResult<Vec<Citation>> {
        citations::upsert_batch(&mut self.conn.lock().unwrap(), project_id, entries)
    }

    fn delete_citation(&self, id: &str) -> AppResult<()> {
        citations::delete_one(&self.conn.lock().unwrap(), id)
    }

    fn create_codex_entry(&self, input: CodexInput) -> AppResult<CodexEntry> {
        codex::create(&self.conn.lock().unwrap(), input)
    }

    fn list_codex_entries(&self, project_id: &str) -> AppResult<Vec<CodexEntry>> {
        codex::list(&self.conn.lock().unwrap(), project_id)
    }

    fn get_codex_entry(&self, id: &str) -> AppResult<Option<CodexEntry>> {
        codex::get(&self.conn.lock().unwrap(), id)
    }

    fn update_codex_entry(&self, id: &str, patch: CodexUpdate) -> AppResult<CodexEntry> {
        codex::update(&self.conn.lock().unwrap(), id, patch)
    }

    fn delete_codex_entry(&self, id: &str) -> AppResult<()> {
        codex::delete(&self.conn.lock().unwrap(), id)
    }

    fn search_codex_entries(
        &self,
        project_id: &str,
        query: &str,
        kind: Option<CodexKind>,
    ) -> AppResult<Vec<CodexEntry>> {
        codex::search(&self.conn.lock().unwrap(), project_id, query, kind)
    }
}

#[cfg(test)]
pub(super) mod test_helpers {
    use super::{LocalStorageService, StorageService};
    use crate::domain::{Project, ProjectInput};

    /// Fresh in-memory storage with migrations applied. Shared by all
    /// submodule tests so each one stays focused on its operation.
    pub fn fresh() -> LocalStorageService {
        let s = LocalStorageService::open_in_memory().expect("in-memory SQLite should always open");
        s.migrate().expect("fresh DB migrate should succeed");
        s
    }

    /// Create a project on a fresh DB without seeding documents — useful for
    /// tests that only need a `project_id` to scope other tables.
    pub fn seed_project(s: &LocalStorageService, title: &str) -> Project {
        s.create_project(ProjectInput {
            title: title.into(),
            template_id: "x".into(),
            metadata: None,
        })
        .expect("create project")
    }
}

#[cfg(test)]
mod tests {
    use super::test_helpers::fresh;
    use super::StorageService;

    #[test]
    fn migrate_is_idempotent() {
        let s = fresh();
        // Re-running migrations must not error nor duplicate data.
        s.migrate().unwrap();
        s.migrate().unwrap();
    }
}
