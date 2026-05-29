//! User-authored templates. Lives at `<app_data>/templates/user/*.json` and
//! is layered on top of `BuiltInTemplates` by `LayeredTemplatesService`.
//!
//! Re-scanned on every call — templates are read only when the wizard opens,
//! so caching would add state for no measurable win.

use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{
    new_id, DocNode, DocumentType, Project, Template, TemplateKind, TemplateNode, TemplateTier,
    TEMPLATE_SCHEMA_VERSION,
};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct UserTemplatesLoader {
    dir: PathBuf,
}

impl UserTemplatesLoader {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Scan the user dir, parsing every `.json` file as a `Template`. Files
    /// that fail to parse are skipped with a warning — they don't poison the
    /// catalogue. Returns the templates in deterministic name order.
    pub fn list(&self) -> Vec<Template> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            match read_template(&path) {
                Ok(t) => out.push(t),
                Err(e) => {
                    tracing::warn!(file = %path.display(), error = %e, "skipping malformed user template")
                }
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    pub fn get(&self, id: &str) -> Option<Template> {
        self.list().into_iter().find(|t| t.id == id)
    }

    /// Persist `template` to disk under `<dir>/<id>.json`. Caller picks the id.
    pub fn save(&self, template: &Template) -> AppResult<()> {
        template.validate()?;
        if !self.dir.exists() {
            fs::create_dir_all(&self.dir)?;
        }
        let path = self.dir.join(format!("{}.json", template.id));
        let json = serde_json::to_string_pretty(template)?;
        fs::write(path, json)?;
        Ok(())
    }
}

fn read_template(path: &Path) -> AppResult<Template> {
    let raw = fs::read_to_string(path)?;
    let template: Template = serde_json::from_str(&raw)
        .map_err(|e| AppError::Invariant(format!("user template '{}': {}", path.display(), e)))?;
    template.validate()?;
    Ok(template)
}

/// Build a `Template` from a live project + its documents. The structure
/// mirrors the document tree (DFS by `position`); `content` is intentionally
/// dropped — templates seed empty bodies. Synopses are preserved because
/// they encode authorial intent for each structural slot.
pub fn template_from_project(
    project: &Project,
    documents: &[DocNode],
    name: &str,
    description: Option<&str>,
    locale: &str,
) -> Template {
    let id = format!("user-{}", new_id().to_lowercase());
    let kind = guess_kind(project, documents);
    Template {
        schema_version: TEMPLATE_SCHEMA_VERSION,
        id,
        name: name.trim().to_string(),
        description: description
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        kind,
        locale: locale.to_string(),
        tier: TemplateTier::Free,
        structure: build_structure(documents, None),
        // We don't carry the project's runtime metadata onto the template —
        // users can add their own fields later by editing the file. The
        // standard "Author" field is enough to make the wizard usable.
        metadata_fields: vec![crate::domain::MetadataField {
            key: "author".into(),
            label: "Author".into(),
            field_type: crate::domain::FieldType::String,
            required: false,
            default: None,
        }],
    }
}

fn build_structure(documents: &[DocNode], parent: Option<&str>) -> Vec<TemplateNode> {
    let mut children: Vec<&DocNode> = documents
        .iter()
        .filter(|d| {
            d.parent_id.as_deref() == parent && d.status != crate::domain::DocumentStatus::Trashed
        })
        .collect();
    children.sort_by_key(|d| d.position);
    children
        .into_iter()
        .map(|d| TemplateNode {
            title: d.title.clone(),
            doc_type: d.doc_type,
            synopsis: d.synopsis.clone().filter(|s| !s.trim().is_empty()),
            children: build_structure(documents, Some(&d.id)),
        })
        .collect()
}

/// Heuristic for `TemplateKind` from a project. We don't try to be clever —
/// novel by default, manga if any node is `manga_page`, paper if more than
/// half of leaves are `note`-typed.
fn guess_kind(_project: &Project, documents: &[DocNode]) -> TemplateKind {
    if documents
        .iter()
        .any(|d| d.doc_type == DocumentType::MangaPage)
    {
        return TemplateKind::Manga;
    }
    let leaves: Vec<&DocNode> = documents
        .iter()
        .filter(|d| d.doc_type != DocumentType::Folder)
        .collect();
    if !leaves.is_empty() {
        let notes = leaves
            .iter()
            .filter(|d| d.doc_type == DocumentType::Note)
            .count();
        if notes * 2 > leaves.len() {
            return TemplateKind::Paper;
        }
    }
    TemplateKind::Novel
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DocumentStatus, ProjectStatus};

    fn tempdir(prefix: &str) -> PathBuf {
        use std::time::{SystemTime, UNIX_EPOCH};
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("draffity-{prefix}-{n:x}"));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn mk_project(title: &str) -> Project {
        Project {
            id: "p".into(),
            title: title.into(),
            template_id: "x".into(),
            status: ProjectStatus::Active,
            metadata: None,
            goal_words: None,
            deadline: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    fn mk_doc(
        id: &str,
        parent: Option<&str>,
        title: &str,
        doc_type: DocumentType,
        position: i64,
    ) -> DocNode {
        DocNode {
            id: id.into(),
            project_id: "p".into(),
            parent_id: parent.map(String::from),
            title: title.into(),
            doc_type,
            content: Some("<p>contenido</p>".into()),
            content_json: None,
            synopsis: Some("S".into()),
            position,
            status: DocumentStatus::Draft,
            tags: vec![],
            label_ids: vec![],
            metadata: std::collections::HashMap::new(),
            is_research: false,
            goal_words: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn save_then_list_round_trips() {
        let dir = tempdir("ut-roundtrip");
        let loader = UserTemplatesLoader::new(dir.clone());
        let p = mk_project("Mi novela");
        let docs = vec![
            mk_doc("a", None, "Acto 1", DocumentType::Folder, 0),
            mk_doc("b", Some("a"), "Cap 1", DocumentType::Chapter, 0),
        ];
        let t = template_from_project(&p, &docs, "Mi plantilla", Some("d"), "es");
        loader.save(&t).unwrap();
        let listed = loader.list();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Mi plantilla");
        assert_eq!(listed[0].structure.len(), 1);
        assert_eq!(listed[0].structure[0].children.len(), 1);
        assert_eq!(listed[0].structure[0].children[0].title, "Cap 1");
    }

    #[test]
    fn list_skips_malformed_files() {
        let dir = tempdir("ut-bad");
        fs::write(dir.join("garbage.json"), b"not a template").unwrap();
        let loader = UserTemplatesLoader::new(dir);
        assert!(loader.list().is_empty());
    }

    #[test]
    fn template_from_project_drops_content_but_keeps_synopsis() {
        let p = mk_project("X");
        let docs = vec![mk_doc("a", None, "Ch 1", DocumentType::Chapter, 0)];
        let t = template_from_project(&p, &docs, "Plant", None, "es");
        assert_eq!(t.structure.len(), 1);
        assert_eq!(t.structure[0].synopsis.as_deref(), Some("S"));
    }

    #[test]
    fn template_from_project_ignores_trashed_documents() {
        let p = mk_project("X");
        let mut trashed = mk_doc("a", None, "Ch 1", DocumentType::Chapter, 0);
        trashed.status = DocumentStatus::Trashed;
        let kept = mk_doc("b", None, "Ch 2", DocumentType::Chapter, 1);
        let t = template_from_project(&p, &[trashed, kept], "Plant", None, "es");
        assert_eq!(t.structure.len(), 1);
        assert_eq!(t.structure[0].title, "Ch 2");
    }

    #[test]
    fn template_from_project_detects_manga_kind() {
        let p = mk_project("X");
        let docs = vec![mk_doc("a", None, "Page 1", DocumentType::MangaPage, 0)];
        let t = template_from_project(&p, &docs, "Plant", None, "es");
        assert_eq!(t.kind, TemplateKind::Manga);
    }

    #[test]
    fn get_returns_none_for_unknown_id() {
        let dir = tempdir("ut-none");
        let loader = UserTemplatesLoader::new(dir);
        assert!(loader.get("nope").is_none());
    }
}
