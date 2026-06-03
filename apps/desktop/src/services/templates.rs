//! Template loader. Built-in templates are embedded at compile time.
//! The loader trait allows swapping in user/cloud templates without changing this trait.

use std::collections::HashMap;

use crate::domain::Template;
use crate::error::{AppError, AppResult};
use crate::services::user_templates::UserTemplatesLoader;

pub trait TemplatesService: Send + Sync {
    fn list(&self) -> Vec<Template>;
    fn get(&self, id: &str) -> Option<Template>;
}

/// Embedded JSON for every built-in template. Keep this list in sync with
/// `packages/templates/`.
const RAW: &[(&str, &str)] = &[
    (
        "generic",
        include_str!("../../../../packages/templates/generic.json"),
    ),
    (
        "novela-tres-actos",
        include_str!("../../../../packages/templates/novela-tres-actos.json"),
    ),
    (
        "paper-imrad",
        include_str!("../../../../packages/templates/paper-imrad.json"),
    ),
    (
        "manga-shonen",
        include_str!("../../../../packages/templates/manga-shonen.json"),
    ),
];

pub struct BuiltInTemplates {
    by_id: HashMap<String, Template>,
}

impl BuiltInTemplates {
    /// Parse and validate every built-in template at startup. A malformed
    /// built-in is a programmer bug — we surface it loudly rather than silently
    /// shipping a broken catalogue.
    pub fn load() -> AppResult<Self> {
        let mut by_id = HashMap::with_capacity(RAW.len());
        for (expected_id, raw) in RAW {
            let template: Template = serde_json::from_str(raw).map_err(|e| {
                AppError::Invariant(format!("template '{expected_id}': parse error: {e}"))
            })?;
            template.validate()?;
            if template.id != *expected_id {
                return Err(AppError::Invariant(format!(
                    "template file expected id '{expected_id}', got '{}'",
                    template.id
                )));
            }
            by_id.insert(template.id.clone(), template);
        }
        Ok(Self { by_id })
    }
}

impl TemplatesService for BuiltInTemplates {
    fn list(&self) -> Vec<Template> {
        let mut all: Vec<Template> = self.by_id.values().cloned().collect();
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all
    }

    fn get(&self, id: &str) -> Option<Template> {
        self.by_id.get(id).cloned()
    }
}

/// Composes built-in + user templates behind the same `TemplatesService`
/// surface. The wizard sees a single sorted list; lookup tries user first
/// so a user template whose id collides with a built-in (shouldn't happen
/// thanks to the `user-` prefix) would shadow the built-in — that's the
/// least surprising behaviour for "the user owns their templates".
pub struct LayeredTemplatesService {
    built_in: BuiltInTemplates,
    user: UserTemplatesLoader,
}

impl LayeredTemplatesService {
    pub fn new(built_in: BuiltInTemplates, user: UserTemplatesLoader) -> Self {
        Self { built_in, user }
    }

    /// Expose the user loader so commands can save new templates without
    /// reaching back through `AppState` for the path.
    pub fn user_loader(&self) -> &UserTemplatesLoader {
        &self.user
    }
}

impl TemplatesService for LayeredTemplatesService {
    fn list(&self) -> Vec<Template> {
        let mut all = self.built_in.list();
        all.extend(self.user.list());
        all.sort_by(|a, b| a.name.cmp(&b.name));
        all
    }

    fn get(&self, id: &str) -> Option<Template> {
        self.user.get(id).or_else(|| self.built_in.get(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TemplateKind;

    #[test]
    fn loads_all_builtin_templates() {
        let s = BuiltInTemplates::load().unwrap();
        let ids: Vec<String> = s.list().into_iter().map(|t| t.id).collect();
        assert!(ids.contains(&"generic".into()));
        assert!(ids.contains(&"novela-tres-actos".into()));
        assert!(ids.contains(&"paper-imrad".into()));
        assert!(ids.contains(&"manga-shonen".into()));
    }

    #[test]
    fn list_is_sorted_by_name() {
        let s = BuiltInTemplates::load().unwrap();
        let names: Vec<String> = s.list().into_iter().map(|t| t.name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn get_returns_some_for_known_id() {
        let s = BuiltInTemplates::load().unwrap();
        let t = s.get("novela-tres-actos").unwrap();
        assert_eq!(t.kind, TemplateKind::Novel);
        assert!(!t.structure.is_empty());
    }

    #[test]
    fn get_returns_none_for_unknown_id() {
        let s = BuiltInTemplates::load().unwrap();
        assert!(s.get("save-the-cat").is_none());
    }

    #[test]
    fn every_builtin_template_has_at_least_one_metadata_field() {
        // Sanity check: built-ins should at least ask for an author by default.
        let s = BuiltInTemplates::load().unwrap();
        for t in s.list() {
            assert!(
                !t.metadata_fields.is_empty(),
                "template '{}' has no metadataFields",
                t.id
            );
        }
    }
}
