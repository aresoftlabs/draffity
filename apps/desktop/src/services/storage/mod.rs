//! Storage abstraction. The free MVP ships only `LocalStorageService`
//! (single SQLite file holding all projects). Premium can add a
//! `CloudSyncStorageService` that wraps this one without changing the trait.
//!
//! The trait impl is intentionally thin: it locks the connection and
//! delegates to per-topic submodules (`projects`, `documents`, `snapshots`,
//! `settings`, `stats`). New operations live in the matching submodule —
//! not here.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{
    AiHistoryEntry, AiValidation, Citation, CodexEntry, CodexInput, CodexKind, CodexUpdate,
    Collection, CollectionInput, CollectionQuery, CustomField, CustomFieldInput, DailyWriting,
    DocNode, DocumentInput, DocumentStatus, Label, LabelInput, MediaAsset, Project, ProjectInput,
    ProjectStatus, SearchHit, Snapshot, TemplateNode, WritingStats,
};
use crate::error::{AppError, AppResult};
use crate::services::importer::ImportTree;

pub use citations::UpsertEntry as CitationUpsert;

mod ai_history;
mod ai_validations;
mod citations;
mod codex;
mod collections;
mod custom_fields;
mod document_positions;
mod document_tags;
mod documents;
mod import_seed;
mod labels;
mod media;
mod projects;
mod row_mappers;
mod search;
mod settings;
mod snapshots;
mod stats;
mod template_seed;

/// Embedded migrations applied in order. Each entry is `(version, sql)`.
/// All migrations are additive and idempotent.
const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!("../../migrations/001_init.sql")),
    (2, include_str!("../../migrations/002_fts.sql")),
    (3, include_str!("../../migrations/003_status_tags.sql")),
    (4, include_str!("../../migrations/004_goals.sql")),
    (5, include_str!("../../migrations/005_synopsis.sql")),
    (6, include_str!("../../migrations/006_doc_json.sql")),
    (7, include_str!("../../migrations/007_citations.sql")),
    (9, include_str!("../../migrations/009_codex.sql")),
    (10, include_str!("../../migrations/010_media.sql")),
    (11, include_str!("../../migrations/011_daily_writing.sql")),
    (12, include_str!("../../migrations/012_ai_history.sql")),
    (13, include_str!("../../migrations/013_ai_validations.sql")),
    (14, include_str!("../../migrations/014_voice_notes.sql")),
    (15, include_str!("../../migrations/015_collections.sql")),
    (16, include_str!("../../migrations/016_labels.sql")),
    (17, include_str!("../../migrations/017_custom_metadata.sql")),
    (18, include_str!("../../migrations/018_research.sql")),
    (
        19,
        include_str!("../../migrations/019_project_deadline.sql"),
    ),
    (20, include_str!("../../migrations/020_session_targets.sql")),
    (
        21,
        include_str!("../../migrations/021_front_back_matter.sql"),
    ),
];

pub trait StorageService: Send + Sync {
    fn migrate(&self) -> AppResult<()>;

    // Projects
    fn create_project(&self, input: ProjectInput) -> AppResult<Project>;
    /// Atomically create a project and seed its initial document tree from a
    /// template structure. The whole operation lives in a single SQLite
    /// transaction — on any failure nothing is persisted. When `archive_active`
    /// is true the currently-active project is archived in the same transaction
    /// (free-tier single-active invariant).
    fn create_project_atomic(
        &self,
        input: ProjectInput,
        structure: &[TemplateNode],
        archive_active: bool,
    ) -> AppResult<Project>;
    /// Activate `id`, optionally archiving the other active project(s) in the
    /// same transaction. A failed activation rolls the archive back.
    fn activate_project_atomic(&self, id: &str, archive_others: bool) -> AppResult<Project>;
    /// Create a project and seed its document tree from an importer's
    /// `ImportTree` in a single transaction. Use this instead of
    /// `create_project_atomic` when the source is a file import (the tree
    /// already comes with rendered HTML content per node).
    fn create_project_from_import(
        &self,
        tree: &ImportTree,
        template_id: &str,
        archive_active: bool,
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
    /// Update a document and record writing stats (activity + daily word
    /// delta) atomically in one transaction. Use this for user-driven saves so
    /// the word delta is computed against the committed content, not a stale
    /// read (AUD-07).
    fn update_document_with_stats(
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
    /// Flag or unflag a document as research material (I-10).
    fn set_document_research(&self, id: &str, is_research: bool) -> AppResult<DocNode>;
    /// Set a document's front/back matter flags (K-03).
    fn set_document_matter(&self, id: &str, is_front: bool, is_back: bool) -> AppResult<DocNode>;
    /// Set or clear a project's target word count.
    fn set_project_goal(&self, id: &str, goal: Option<i64>) -> AppResult<Project>;
    /// Set or clear a project's deadline (epoch ms).
    fn set_project_deadline(&self, id: &str, deadline: Option<i64>) -> AppResult<Project>;
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
    /// Accumulate a positive word-count delta onto today's row and bump the
    /// session counter. Zero deltas are accepted (and still count as a
    /// session) so saves that only delete text are still tracked.
    fn record_daily_writing(&self, words_delta: u32) -> AppResult<()>;
    /// Last `days` days of activity, oldest first, with missing days padded
    /// by zero rows. Powers the Settings sparkline.
    fn list_recent_daily_writing(&self, days: u32) -> AppResult<Vec<DailyWriting>>;
    /// The persisted daily word goal (J-04), or `None` when unset.
    fn get_daily_goal(&self) -> AppResult<Option<i64>>;
    /// Set or clear the daily word goal; recomputes today's `goal_met`.
    fn set_daily_goal(&self, goal: Option<i64>) -> AppResult<()>;

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

    // Media registry. The bytes themselves live on disk; this trait only
    // tracks the catalogue row. Callers (the MediaService) own the file
    // write + sha256 computation before calling `insert_media_row`.
    fn find_media_by_hash(&self, project_id: &str, sha256: &str) -> AppResult<Option<MediaAsset>>;
    fn insert_media_row(
        &self,
        project_id: &str,
        path_relative: &str,
        mime: &str,
        sha256: &str,
        bytes: i64,
    ) -> AppResult<MediaAsset>;
    fn get_media(&self, id: &str) -> AppResult<Option<MediaAsset>>;
    fn list_media(&self, project_id: &str) -> AppResult<Vec<MediaAsset>>;
    /// Returns the deleted row so the `MediaService` knows which file to
    /// unlink on disk; `None` when the id was already gone.
    fn delete_media_row(&self, id: &str) -> AppResult<Option<MediaAsset>>;
    /// Flag a media asset as a voice note with duration + optional transcript.
    fn set_media_voice_note(
        &self,
        id: &str,
        duration_ms: Option<i64>,
        transcribed_text: Option<&str>,
    ) -> AppResult<MediaAsset>;
    /// A project's voice notes, newest first.
    fn list_voice_notes(&self, project_id: &str) -> AppResult<Vec<MediaAsset>>;

    // Search
    /// Full-text search across documents of a single project. Returns up to
    /// 50 hits ordered by FTS5 rank, each with a `<mark>`-wrapped excerpt.
    /// Empty or whitespace-only queries return `[]` without hitting the DB.
    fn search_documents(&self, project_id: &str, query: &str) -> AppResult<Vec<SearchHit>>;

    // AI history (F-12). Append-only log of accepted generations.
    /// Persist an accepted AI generation; returns the stored row.
    fn record_ai_history(
        &self,
        project_id: &str,
        doc_id: Option<&str>,
        action: &str,
        model: Option<&str>,
        response: &str,
    ) -> AppResult<AiHistoryEntry>;
    /// List a project's accepted generations, newest first, capped at `limit`.
    fn list_ai_history(&self, project_id: &str, limit: u32) -> AppResult<Vec<AiHistoryEntry>>;

    // AI validations (G-02). Append-only reports per (document, validator).
    fn record_ai_validation(
        &self,
        document_id: &str,
        validator_name: &str,
        results_json: &str,
        severity_summary: &str,
    ) -> AppResult<AiValidation>;
    /// All reports for a document, newest first.
    fn list_ai_validations(&self, document_id: &str) -> AppResult<Vec<AiValidation>>;

    // Collections (I-01..I-03)
    fn create_collection(&self, input: CollectionInput) -> AppResult<Collection>;
    fn list_collections(&self, project_id: &str) -> AppResult<Vec<Collection>>;
    fn get_collection(&self, id: &str) -> AppResult<Option<Collection>>;
    fn rename_collection(&self, id: &str, name: &str) -> AppResult<Collection>;
    /// Update a smart collection's query.
    fn set_collection_query(&self, id: &str, query: &CollectionQuery) -> AppResult<Collection>;
    fn delete_collection(&self, id: &str) -> AppResult<()>;
    /// Replace a manual collection's ordered membership.
    fn set_collection_members(&self, collection_id: &str, ordered_ids: &[String]) -> AppResult<()>;
    /// Resolve a collection to its documents (manual order or smart filter).
    fn resolve_collection(&self, id: &str) -> AppResult<Vec<DocNode>>;

    // Labels (I-05/I-06): per-project colored labels assigned to documents.
    fn create_label(&self, input: LabelInput) -> AppResult<Label>;
    fn list_labels(&self, project_id: &str) -> AppResult<Vec<Label>>;
    fn update_label(&self, id: &str, name: &str, color: &str) -> AppResult<Label>;
    fn delete_label(&self, id: &str) -> AppResult<()>;
    /// Replace the entire label set of a document atomically.
    fn set_document_labels(&self, document_id: &str, label_ids: &[String]) -> AppResult<DocNode>;

    // Custom metadata fields (I-08/I-09): per-project field definitions +
    // per-document values surfaced on `DocNode::metadata`.
    fn create_custom_field(&self, input: CustomFieldInput) -> AppResult<CustomField>;
    fn list_custom_fields(&self, project_id: &str) -> AppResult<Vec<CustomField>>;
    /// Rename and/or change a field's options (kind is immutable).
    fn update_custom_field(
        &self,
        id: &str,
        name: &str,
        options: &[String],
    ) -> AppResult<CustomField>;
    fn delete_custom_field(&self, id: &str) -> AppResult<()>;
    /// Set or clear (`value=None`) a document's value for one field.
    fn set_document_metadata(
        &self,
        document_id: &str,
        field_id: &str,
        value: Option<&str>,
    ) -> AppResult<DocNode>;
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

    /// Lock the connection, mapping a poisoned mutex to a typed error instead
    /// of panicking. A panic while holding the lock would otherwise turn every
    /// subsequent operation into a panic and take down session persistence.
    fn db(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| AppError::Unexpected("storage connection poisoned".into()))
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
        let mut conn = self.db()?;
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
        projects::create(&*self.db()?, input)
    }

    fn create_project_atomic(
        &self,
        input: ProjectInput,
        structure: &[TemplateNode],
        archive_active: bool,
    ) -> AppResult<Project> {
        projects::create_atomic(&mut *self.db()?, input, structure, archive_active)
    }

    fn activate_project_atomic(&self, id: &str, archive_others: bool) -> AppResult<Project> {
        projects::activate_atomic(&mut *self.db()?, id, archive_others)
    }

    fn create_project_from_import(
        &self,
        tree: &ImportTree,
        template_id: &str,
        archive_active: bool,
    ) -> AppResult<Project> {
        projects::create_from_import(&mut *self.db()?, tree, template_id, archive_active)
    }

    fn list_projects(&self) -> AppResult<Vec<Project>> {
        projects::list(&*self.db()?)
    }

    fn get_project(&self, id: &str) -> AppResult<Option<Project>> {
        projects::get(&*self.db()?, id)
    }

    fn get_active_project(&self) -> AppResult<Option<Project>> {
        projects::get_active(&*self.db()?)
    }

    fn set_project_status(&self, id: &str, status: ProjectStatus) -> AppResult<()> {
        projects::set_status(&*self.db()?, id, status)
    }

    fn delete_project(&self, id: &str) -> AppResult<()> {
        projects::delete(&*self.db()?, id)
    }

    fn create_document(&self, input: DocumentInput) -> AppResult<DocNode> {
        documents::create(&*self.db()?, input)
    }

    fn list_documents(&self, project_id: &str) -> AppResult<Vec<DocNode>> {
        documents::list(&*self.db()?, project_id)
    }

    fn get_document(&self, id: &str) -> AppResult<Option<DocNode>> {
        documents::get(&*self.db()?, id)
    }

    fn update_document(
        &self,
        id: &str,
        title: Option<&str>,
        content: Option<&str>,
        content_json: Option<&str>,
    ) -> AppResult<DocNode> {
        documents::update(&*self.db()?, id, title, content, content_json)
    }

    fn update_document_with_stats(
        &self,
        id: &str,
        title: Option<&str>,
        content: Option<&str>,
        content_json: Option<&str>,
    ) -> AppResult<DocNode> {
        documents::update_with_stats(&mut *self.db()?, id, title, content, content_json)
    }

    fn move_document(&self, id: &str, parent_id: Option<&str>, position: i64) -> AppResult<()> {
        documents::move_to(&*self.db()?, id, parent_id, position)
    }

    fn delete_document(&self, id: &str) -> AppResult<()> {
        documents::delete(&*self.db()?, id)
    }

    fn reorder_documents(
        &self,
        project_id: &str,
        parent_id: Option<&str>,
        ordered_ids: &[String],
    ) -> AppResult<()> {
        document_positions::reorder(&mut *self.db()?, project_id, parent_id, ordered_ids)
    }

    fn set_document_status(&self, id: &str, status: DocumentStatus) -> AppResult<DocNode> {
        documents::set_status(&*self.db()?, id, status)
    }

    fn set_document_tags(&self, id: &str, tags: &[String]) -> AppResult<DocNode> {
        document_tags::set(&mut *self.db()?, id, tags)
    }

    fn list_project_tags(&self, project_id: &str) -> AppResult<Vec<String>> {
        document_tags::list_project_tags(&*self.db()?, project_id)
    }

    fn set_document_goal(&self, id: &str, goal: Option<i64>) -> AppResult<DocNode> {
        documents::set_goal(&*self.db()?, id, goal)
    }

    fn set_document_research(&self, id: &str, is_research: bool) -> AppResult<DocNode> {
        documents::set_research(&*self.db()?, id, is_research)
    }

    fn set_document_matter(&self, id: &str, is_front: bool, is_back: bool) -> AppResult<DocNode> {
        documents::set_matter(&*self.db()?, id, is_front, is_back)
    }

    fn set_project_goal(&self, id: &str, goal: Option<i64>) -> AppResult<Project> {
        projects::set_goal(&*self.db()?, id, goal)
    }

    fn set_project_deadline(&self, id: &str, deadline: Option<i64>) -> AppResult<Project> {
        projects::set_deadline(&*self.db()?, id, deadline)
    }

    fn set_document_synopsis(&self, id: &str, synopsis: Option<&str>) -> AppResult<DocNode> {
        documents::set_synopsis(&*self.db()?, id, synopsis)
    }

    fn create_snapshot(&self, document_id: &str, label: Option<&str>) -> AppResult<Snapshot> {
        snapshots::create(&*self.db()?, document_id, label)
    }

    fn list_snapshots(&self, document_id: &str) -> AppResult<Vec<Snapshot>> {
        snapshots::list(&*self.db()?, document_id)
    }

    fn restore_snapshot(&self, snapshot_id: &str) -> AppResult<DocNode> {
        snapshots::restore(&mut *self.db()?, snapshot_id)
    }

    fn get_setting(&self, key: &str) -> AppResult<Option<String>> {
        settings::get(&*self.db()?, key)
    }

    fn set_setting(&self, key: &str, value: &str) -> AppResult<()> {
        settings::set(&*self.db()?, key, value)
    }

    fn record_writing_activity(&self) -> AppResult<WritingStats> {
        stats::record_activity(&*self.db()?)
    }

    fn get_writing_stats(&self) -> AppResult<WritingStats> {
        stats::get(&*self.db()?)
    }

    fn record_daily_writing(&self, words_delta: u32) -> AppResult<()> {
        stats::record_daily(&*self.db()?, words_delta)
    }

    fn list_recent_daily_writing(&self, days: u32) -> AppResult<Vec<DailyWriting>> {
        stats::list_recent_daily(&*self.db()?, days)
    }

    fn get_daily_goal(&self) -> AppResult<Option<i64>> {
        stats::get_daily_goal(&*self.db()?)
    }

    fn set_daily_goal(&self, goal: Option<i64>) -> AppResult<()> {
        stats::set_daily_goal(&*self.db()?, goal)
    }

    fn search_documents(&self, project_id: &str, query: &str) -> AppResult<Vec<SearchHit>> {
        search::search(&*self.db()?, project_id, query)
    }

    fn record_ai_history(
        &self,
        project_id: &str,
        doc_id: Option<&str>,
        action: &str,
        model: Option<&str>,
        response: &str,
    ) -> AppResult<AiHistoryEntry> {
        ai_history::record(&*self.db()?, project_id, doc_id, action, model, response)
    }

    fn list_ai_history(&self, project_id: &str, limit: u32) -> AppResult<Vec<AiHistoryEntry>> {
        ai_history::list(&*self.db()?, project_id, limit)
    }

    fn record_ai_validation(
        &self,
        document_id: &str,
        validator_name: &str,
        results_json: &str,
        severity_summary: &str,
    ) -> AppResult<AiValidation> {
        ai_validations::record(
            &*self.db()?,
            document_id,
            validator_name,
            results_json,
            severity_summary,
        )
    }

    fn list_ai_validations(&self, document_id: &str) -> AppResult<Vec<AiValidation>> {
        ai_validations::list_for_document(&*self.db()?, document_id)
    }

    fn create_collection(&self, input: CollectionInput) -> AppResult<Collection> {
        collections::create(&*self.db()?, input)
    }

    fn list_collections(&self, project_id: &str) -> AppResult<Vec<Collection>> {
        collections::list(&*self.db()?, project_id)
    }

    fn get_collection(&self, id: &str) -> AppResult<Option<Collection>> {
        collections::get(&*self.db()?, id)
    }

    fn rename_collection(&self, id: &str, name: &str) -> AppResult<Collection> {
        collections::rename(&*self.db()?, id, name)
    }

    fn set_collection_query(&self, id: &str, query: &CollectionQuery) -> AppResult<Collection> {
        collections::set_query(&*self.db()?, id, query)
    }

    fn delete_collection(&self, id: &str) -> AppResult<()> {
        collections::delete(&*self.db()?, id)
    }

    fn set_collection_members(&self, collection_id: &str, ordered_ids: &[String]) -> AppResult<()> {
        collections::set_members(&mut *self.db()?, collection_id, ordered_ids)
    }

    fn resolve_collection(&self, id: &str) -> AppResult<Vec<DocNode>> {
        collections::resolve(&*self.db()?, id)
    }

    fn create_label(&self, input: LabelInput) -> AppResult<Label> {
        labels::create(&*self.db()?, input)
    }

    fn list_labels(&self, project_id: &str) -> AppResult<Vec<Label>> {
        labels::list(&*self.db()?, project_id)
    }

    fn update_label(&self, id: &str, name: &str, color: &str) -> AppResult<Label> {
        labels::update(&*self.db()?, id, name, color)
    }

    fn delete_label(&self, id: &str) -> AppResult<()> {
        labels::delete(&*self.db()?, id)
    }

    fn set_document_labels(&self, document_id: &str, label_ids: &[String]) -> AppResult<DocNode> {
        labels::set_document(&mut *self.db()?, document_id, label_ids)
    }

    fn create_custom_field(&self, input: CustomFieldInput) -> AppResult<CustomField> {
        custom_fields::create(&*self.db()?, input)
    }

    fn list_custom_fields(&self, project_id: &str) -> AppResult<Vec<CustomField>> {
        custom_fields::list(&*self.db()?, project_id)
    }

    fn update_custom_field(
        &self,
        id: &str,
        name: &str,
        options: &[String],
    ) -> AppResult<CustomField> {
        custom_fields::update(&*self.db()?, id, name, options)
    }

    fn delete_custom_field(&self, id: &str) -> AppResult<()> {
        custom_fields::delete(&*self.db()?, id)
    }

    fn set_document_metadata(
        &self,
        document_id: &str,
        field_id: &str,
        value: Option<&str>,
    ) -> AppResult<DocNode> {
        custom_fields::set_value(&*self.db()?, document_id, field_id, value)
    }

    fn list_citations(&self, project_id: &str) -> AppResult<Vec<Citation>> {
        citations::list(&*self.db()?, project_id)
    }

    fn list_citation_keys(&self, project_id: &str) -> AppResult<Vec<String>> {
        citations::list_keys(&*self.db()?, project_id)
    }

    fn upsert_citations(
        &self,
        project_id: &str,
        entries: &[CitationUpsert],
    ) -> AppResult<Vec<Citation>> {
        citations::upsert_batch(&mut *self.db()?, project_id, entries)
    }

    fn delete_citation(&self, id: &str) -> AppResult<()> {
        citations::delete_one(&*self.db()?, id)
    }

    fn create_codex_entry(&self, input: CodexInput) -> AppResult<CodexEntry> {
        codex::create(&*self.db()?, input)
    }

    fn list_codex_entries(&self, project_id: &str) -> AppResult<Vec<CodexEntry>> {
        codex::list(&*self.db()?, project_id)
    }

    fn get_codex_entry(&self, id: &str) -> AppResult<Option<CodexEntry>> {
        codex::get(&*self.db()?, id)
    }

    fn update_codex_entry(&self, id: &str, patch: CodexUpdate) -> AppResult<CodexEntry> {
        codex::update(&*self.db()?, id, patch)
    }

    fn delete_codex_entry(&self, id: &str) -> AppResult<()> {
        codex::delete(&*self.db()?, id)
    }

    fn search_codex_entries(
        &self,
        project_id: &str,
        query: &str,
        kind: Option<CodexKind>,
    ) -> AppResult<Vec<CodexEntry>> {
        codex::search(&*self.db()?, project_id, query, kind)
    }

    fn find_media_by_hash(&self, project_id: &str, sha256: &str) -> AppResult<Option<MediaAsset>> {
        media::find_by_hash(&*self.db()?, project_id, sha256)
    }

    fn insert_media_row(
        &self,
        project_id: &str,
        path_relative: &str,
        mime: &str,
        sha256: &str,
        bytes: i64,
    ) -> AppResult<MediaAsset> {
        media::insert(&*self.db()?, project_id, path_relative, mime, sha256, bytes)
    }

    fn get_media(&self, id: &str) -> AppResult<Option<MediaAsset>> {
        media::get(&*self.db()?, id)
    }

    fn list_media(&self, project_id: &str) -> AppResult<Vec<MediaAsset>> {
        media::list(&*self.db()?, project_id)
    }

    fn delete_media_row(&self, id: &str) -> AppResult<Option<MediaAsset>> {
        media::delete(&*self.db()?, id)
    }

    fn set_media_voice_note(
        &self,
        id: &str,
        duration_ms: Option<i64>,
        transcribed_text: Option<&str>,
    ) -> AppResult<MediaAsset> {
        media::set_voice_note(&*self.db()?, id, duration_ms, transcribed_text)
    }

    fn list_voice_notes(&self, project_id: &str) -> AppResult<Vec<MediaAsset>> {
        media::list_voice_notes(&*self.db()?, project_id)
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
    use crate::error::AppError;

    #[test]
    fn migrate_is_idempotent() {
        let s = fresh();
        // Re-running migrations must not error nor duplicate data.
        s.migrate().unwrap();
        s.migrate().unwrap();
    }

    #[test]
    fn poisoned_lock_returns_typed_error_not_panic() {
        let s = fresh();
        // Poison the connection mutex by panicking while holding the guard.
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = s.conn.lock().unwrap();
            panic!("poison the lock");
        }));
        assert!(panicked.is_err(), "the closure should have panicked");

        // A subsequent operation must surface a typed error, not panic the app.
        let outcome = s.list_projects();
        assert!(
            matches!(outcome, Err(AppError::Unexpected(_))),
            "expected a typed poison error, got {outcome:?}"
        );
    }
}
