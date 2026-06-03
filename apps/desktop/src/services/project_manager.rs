//! Orchestrates project lifecycle, enforcing the single-active invariant:
//! activating/creating a project archives the currently active one.
//!
//! The single-active rule is an unconditional design invariant — exactly one
//! project is `active` at any time, and switching projects archives the
//! previous one atomically (no partial state on failure).
//!
//! Behind `ProjectManagerService` so a future `CloudProjectManager`
//! (remote-sync) can swap in without touching `AppState`, commands or the
//! rest of the wiring. Pattern: §2 CLAUDE.md ("Trait + impl NoOp services").

use std::sync::Arc;

use crate::domain::{Project, ProjectInput, ProjectStatus};
use crate::error::AppResult;
use crate::services::importer::ImportTree;
use crate::services::storage::StorageService;
use crate::services::templates::TemplatesService;
pub trait ProjectManagerService: Send + Sync {
    /// Create a new project, seeding it from the requested template if known.
    /// If single-active is enforced (free tier), any currently-active project
    /// is archived first.
    fn create(&self, input: ProjectInput) -> AppResult<Project>;

    /// Activate an existing project. Archives the currently-active one
    /// when single-active is enforced.
    fn activate(&self, id: &str) -> AppResult<Project>;

    /// Create a project from an imported document tree, archiving the
    /// currently-active project first when single-active is enforced. Routes
    /// imports through the same invariant as `create` (AUD-06).
    fn create_from_import(&self, tree: &ImportTree, template_id: &str) -> AppResult<Project>;

    fn archive(&self, id: &str) -> AppResult<()>;
    fn delete(&self, id: &str) -> AppResult<()>;

    fn list(&self) -> AppResult<Vec<Project>>;
    fn get(&self, id: &str) -> AppResult<Option<Project>>;
    fn active(&self) -> AppResult<Option<Project>>;
}

pub struct LocalProjectManager {
    storage: Arc<dyn StorageService>,
    templates: Arc<dyn TemplatesService>,
}

impl LocalProjectManager {
    pub fn new(storage: Arc<dyn StorageService>, templates: Arc<dyn TemplatesService>) -> Self {
        Self { storage, templates }
    }
}

impl ProjectManagerService for LocalProjectManager {
    fn create(&self, input: ProjectInput) -> AppResult<Project> {
        let structure = self
            .templates
            .get(&input.template_id)
            .map(|t| t.structure)
            .unwrap_or_default();
        // Archive + create run in one transaction inside storage, so a failed
        // create can't leave the user with zero active projects (AUD-05).
        self.storage.create_project_atomic(input, &structure, true)
    }

    fn activate(&self, id: &str) -> AppResult<Project> {
        self.storage.activate_project_atomic(id, true)
    }

    fn create_from_import(&self, tree: &ImportTree, template_id: &str) -> AppResult<Project> {
        self.storage
            .create_project_from_import(tree, template_id, true)
    }

    fn archive(&self, id: &str) -> AppResult<()> {
        self.storage.set_project_status(id, ProjectStatus::Archived)
    }

    fn delete(&self, id: &str) -> AppResult<()> {
        self.storage.delete_project(id)
    }

    fn list(&self) -> AppResult<Vec<Project>> {
        self.storage.list_projects()
    }

    fn get(&self, id: &str) -> AppResult<Option<Project>> {
        self.storage.get_project(id)
    }

    fn active(&self) -> AppResult<Option<Project>> {
        self.storage.get_active_project()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        DocumentType, ProjectInput, Template, TemplateKind, TemplateNode, TemplateTier,
    };
    use crate::services::storage::LocalStorageService;

    /// Minimal in-test templates source: a single `novela` template with one
    /// folder + one chapter, plus a `generic` empty template. Lets the manager
    /// tests cover both seeded and bare creations without loading JSON.
    struct StubTemplates {
        templates: Vec<Template>,
    }

    impl StubTemplates {
        fn empty() -> Self {
            Self { templates: vec![] }
        }

        fn with_seed() -> Self {
            Self {
                templates: vec![
                    Template {
                        schema_version: 1,
                        id: "generic".into(),
                        name: "Generic".into(),
                        description: None,
                        kind: TemplateKind::Generic,
                        locale: "es".into(),
                        tier: TemplateTier::Free,
                        structure: vec![],
                        metadata_fields: vec![],
                    },
                    Template {
                        schema_version: 1,
                        id: "novela".into(),
                        name: "Novela".into(),
                        description: None,
                        kind: TemplateKind::Novel,
                        locale: "es".into(),
                        tier: TemplateTier::Free,
                        structure: vec![TemplateNode {
                            title: "Acto 1".into(),
                            doc_type: DocumentType::Folder,
                            synopsis: None,
                            children: vec![TemplateNode {
                                title: "Capítulo 1".into(),
                                doc_type: DocumentType::Chapter,
                                synopsis: None,
                                children: vec![],
                            }],
                        }],
                        metadata_fields: vec![],
                    },
                ],
            }
        }
    }

    impl TemplatesService for StubTemplates {
        fn list(&self) -> Vec<Template> {
            self.templates.clone()
        }
        fn get(&self, id: &str) -> Option<Template> {
            self.templates.iter().find(|t| t.id == id).cloned()
        }
    }

    /// Returns a tuple so tests can reach `storage` (for `list_documents`
    /// assertions) without having to expose the field on `LocalProjectManager`.
    fn pm(tpl: StubTemplates) -> (LocalProjectManager, Arc<dyn StorageService>) {
        let storage: Arc<dyn StorageService> = Arc::new(
            LocalStorageService::open_in_memory().expect("in-memory SQLite should always open"),
        );
        storage.migrate().expect("fresh DB migrate should succeed");
        let m = LocalProjectManager::new(storage.clone(), Arc::new(tpl));
        (m, storage)
    }

    fn input(title: &str, template_id: &str) -> ProjectInput {
        ProjectInput {
            title: title.into(),
            template_id: template_id.into(),
            metadata: None,
        }
    }

    #[test]
    fn first_project_becomes_active() {
        let (m, _) = pm(StubTemplates::empty());
        let p = m.create(input("A", "anything")).unwrap();
        assert_eq!(p.status, ProjectStatus::Active);
        assert_eq!(m.active().unwrap().unwrap().id, p.id);
    }

    #[test]
    fn second_project_archives_first() {
        let (m, _) = pm(StubTemplates::empty());
        let a = m.create(input("A", "x")).unwrap();
        let b = m.create(input("B", "x")).unwrap();

        let stored_a = m.get(&a.id).unwrap().unwrap();
        let stored_b = m.get(&b.id).unwrap().unwrap();
        assert_eq!(stored_a.status, ProjectStatus::Archived);
        assert_eq!(stored_b.status, ProjectStatus::Active);

        let all = m.list().unwrap();
        assert_eq!(all.len(), 2);
        let active_count = all
            .iter()
            .filter(|p| p.status == ProjectStatus::Active)
            .count();
        assert_eq!(active_count, 1);
    }

    #[test]
    fn activate_archives_previous_active() {
        let (m, _) = pm(StubTemplates::empty());
        let a = m.create(input("A", "x")).unwrap();
        let b = m.create(input("B", "x")).unwrap();

        let reactivated = m.activate(&a.id).unwrap();
        assert_eq!(reactivated.status, ProjectStatus::Active);
        let stored_b = m.get(&b.id).unwrap().unwrap();
        assert_eq!(stored_b.status, ProjectStatus::Archived);
    }

    #[test]
    fn activate_same_project_is_noop_no_double_archive() {
        let (m, _) = pm(StubTemplates::empty());
        let a = m.create(input("A", "x")).unwrap();
        let still_a = m.activate(&a.id).unwrap();
        assert_eq!(still_a.status, ProjectStatus::Active);
        assert_eq!(m.list().unwrap().len(), 1);
    }

    #[test]
    fn activate_missing_project_rolls_back_and_keeps_current_active() {
        let (m, _) = pm(StubTemplates::empty());
        let a = m.create(input("A", "x")).unwrap();
        // Activating a non-existent project must fail WITHOUT archiving A:
        // the archive + activate run in one transaction, so the failed
        // activation rolls the archive back (AUD-05).
        assert!(m.activate("does-not-exist").is_err());
        let active = m.active().unwrap();
        assert_eq!(
            active.map(|p| p.id),
            Some(a.id),
            "A must remain the single active project after the rollback"
        );
    }

    #[test]
    fn import_archives_previous_active_project() {
        use crate::services::importer::{ImportNode, ImportTree};
        let (m, _) = pm(StubTemplates::empty());
        let a = m.create(input("A", "x")).unwrap();
        let tree = ImportTree {
            project_title: "Imported".into(),
            nodes: vec![ImportNode {
                title: "Chapter".into(),
                content_html: "<p>hi</p>".into(),
                children: vec![],
            }],
        };
        // Importing with an active project must archive it (single-active
        // invariant), not hit a raw SQL error on idx_projects_one_active.
        let imported = m.create_from_import(&tree, "generic").unwrap();
        assert_eq!(imported.status, ProjectStatus::Active);
        assert_eq!(m.active().unwrap().map(|p| p.id), Some(imported.id));
        assert_eq!(
            m.get(&a.id).unwrap().unwrap().status,
            ProjectStatus::Archived
        );
    }

    #[test]
    fn delete_removes_project() {
        let (m, _) = pm(StubTemplates::empty());
        let a = m.create(input("A", "x")).unwrap();
        m.delete(&a.id).unwrap();
        assert!(m.get(&a.id).unwrap().is_none());
    }

    #[test]
    fn create_with_known_template_seeds_documents() {
        let (m, storage) = pm(StubTemplates::with_seed());
        let p = m.create(input("Mi novela", "novela")).unwrap();
        // Storage list_documents should include the seeded folder + chapter.
        let docs = storage.list_documents(&p.id).unwrap();
        assert_eq!(docs.len(), 2);
        assert!(docs.iter().any(|d| d.title == "Acto 1"));
        assert!(docs.iter().any(|d| d.title == "Capítulo 1"));
    }

    #[test]
    fn create_with_unknown_template_creates_empty_project() {
        let (m, storage) = pm(StubTemplates::with_seed());
        let p = m.create(input("Sin plantilla", "does-not-exist")).unwrap();
        let docs = storage.list_documents(&p.id).unwrap();
        assert!(docs.is_empty());
    }
}
