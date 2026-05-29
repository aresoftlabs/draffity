//! Collections: saved groupings of documents (I-01..I-03). A `Manual`
//! collection is an explicit ordered list; a `Smart` collection carries a
//! `CollectionQuery` resolved live against the project's documents.
//!
//! `CollectionQuery::matches` is pure domain logic (no SQL), so smart
//! resolution is just `documents.filter(query.matches)` — unit-testable and
//! trivially extensible (add a field + a check).

use serde::{Deserialize, Serialize};

use crate::domain::{DocNode, DocumentStatus};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollectionKind {
    Manual,
    Smart,
}

impl CollectionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CollectionKind::Manual => "manual",
            CollectionKind::Smart => "smart",
        }
    }
    pub fn parse(s: &str) -> AppResult<Self> {
        match s {
            "manual" => Ok(CollectionKind::Manual),
            "smart" => Ok(CollectionKind::Smart),
            other => Err(AppError::Invariant(format!(
                "unknown collection kind: {other}"
            ))),
        }
    }
}

/// A smart-collection query. All present criteria are ANDed; `tags_any` is an
/// OR within tags. Empty/None fields are ignored.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionQuery {
    #[serde(default)]
    pub tags_any: Vec<String>,
    #[serde(default)]
    pub statuses: Vec<DocumentStatus>,
    #[serde(default)]
    pub title_contains: Option<String>,
}

impl CollectionQuery {
    /// Whether a document satisfies the query.
    pub fn matches(&self, doc: &DocNode) -> bool {
        if !self.tags_any.is_empty() && !self.tags_any.iter().any(|t| doc.tags.contains(t)) {
            return false;
        }
        if !self.statuses.is_empty() && !self.statuses.contains(&doc.status) {
            return false;
        }
        if let Some(q) = &self.title_contains {
            let q = q.trim();
            if !q.is_empty() && !doc.title.to_lowercase().contains(&q.to_lowercase()) {
                return false;
            }
        }
        true
    }

    /// A query with no criteria matches everything — usually a mistake for a
    /// smart collection, so callers can reject it.
    pub fn is_empty(&self) -> bool {
        self.tags_any.is_empty()
            && self.statuses.is_empty()
            && self
                .title_contains
                .as_deref()
                .map(str::trim)
                .unwrap_or("")
                .is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub kind: CollectionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<CollectionQuery>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionInput {
    pub project_id: String,
    pub name: String,
    pub kind: CollectionKind,
    #[serde(default)]
    pub query: Option<CollectionQuery>,
}

impl CollectionInput {
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Invariant("collection name is empty".into()));
        }
        if self.kind == CollectionKind::Smart {
            match &self.query {
                Some(q) if !q.is_empty() => {}
                _ => {
                    return Err(AppError::Invariant(
                        "smart collection needs at least one filter".into(),
                    ))
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DocNode, DocumentType};

    fn doc(title: &str, status: DocumentStatus, tags: &[&str]) -> DocNode {
        DocNode {
            id: "d".into(),
            project_id: "p".into(),
            parent_id: None,
            title: title.into(),
            doc_type: DocumentType::Scene,
            content: None,
            content_json: None,
            synopsis: None,
            position: 0,
            status,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            label_ids: Vec::new(),
            metadata: std::collections::HashMap::new(),
            is_research: false,
            is_front_matter: false,
            is_back_matter: false,
            goal_words: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn empty_query_matches_everything() {
        let q = CollectionQuery::default();
        assert!(q.is_empty());
        assert!(q.matches(&doc("anything", DocumentStatus::Draft, &[])));
    }

    #[test]
    fn tags_any_is_or_within_tags() {
        let q = CollectionQuery {
            tags_any: vec!["flashback".into(), "pov:alice".into()],
            ..Default::default()
        };
        assert!(q.matches(&doc("x", DocumentStatus::Draft, &["flashback"])));
        assert!(q.matches(&doc("x", DocumentStatus::Draft, &["pov:alice", "other"])));
        assert!(!q.matches(&doc("x", DocumentStatus::Draft, &["other"])));
    }

    #[test]
    fn status_and_title_are_anded_with_tags() {
        let q = CollectionQuery {
            tags_any: vec!["flashback".into()],
            statuses: vec![DocumentStatus::Final],
            title_contains: Some("cap".into()),
        };
        // All three must hold.
        assert!(q.matches(&doc("Capítulo 3", DocumentStatus::Final, &["flashback"])));
        // Wrong status.
        assert!(!q.matches(&doc("Capítulo 3", DocumentStatus::Draft, &["flashback"])));
        // Title doesn't contain.
        assert!(!q.matches(&doc("Escena", DocumentStatus::Final, &["flashback"])));
    }

    #[test]
    fn smart_input_requires_a_filter() {
        let empty_smart = CollectionInput {
            project_id: "p".into(),
            name: "X".into(),
            kind: CollectionKind::Smart,
            query: Some(CollectionQuery::default()),
        };
        assert!(empty_smart.validate().is_err());

        let ok = CollectionInput {
            project_id: "p".into(),
            name: "X".into(),
            kind: CollectionKind::Smart,
            query: Some(CollectionQuery {
                tags_any: vec!["a".into()],
                ..Default::default()
            }),
        };
        assert!(ok.validate().is_ok());
    }
}
