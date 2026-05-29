use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Chapter,
    Scene,
    Note,
    Folder,
    MangaPage,
}

impl DocumentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocumentType::Chapter => "chapter",
            DocumentType::Scene => "scene",
            DocumentType::Note => "note",
            DocumentType::Folder => "folder",
            DocumentType::MangaPage => "manga_page",
        }
    }

    pub fn parse(s: &str) -> AppResult<Self> {
        match s {
            "chapter" => Ok(DocumentType::Chapter),
            "scene" => Ok(DocumentType::Scene),
            "note" => Ok(DocumentType::Note),
            "folder" => Ok(DocumentType::Folder),
            "manga_page" => Ok(DocumentType::MangaPage),
            other => Err(AppError::Invariant(format!(
                "unknown document type: {other}"
            ))),
        }
    }
}

/// Position in the writing pipeline. New documents land in `Draft`;
/// the user moves them through `Revised` → `Final`. `Trashed` is a soft
/// delete that still lives in storage so the user can recover it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum DocumentStatus {
    #[default]
    Draft,
    Revised,
    Final,
    Trashed,
}

impl DocumentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocumentStatus::Draft => "draft",
            DocumentStatus::Revised => "revised",
            DocumentStatus::Final => "final",
            DocumentStatus::Trashed => "trashed",
        }
    }

    pub fn parse(s: &str) -> AppResult<Self> {
        match s {
            "draft" => Ok(DocumentStatus::Draft),
            "revised" => Ok(DocumentStatus::Revised),
            "final" => Ok(DocumentStatus::Final),
            "trashed" => Ok(DocumentStatus::Trashed),
            other => Err(AppError::Invariant(format!(
                "unknown document status: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocNode {
    pub id: String,
    pub project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub title: String,
    pub doc_type: DocumentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Canonical ProseMirror state for the editor, stored as a JSON string.
    /// Preferred over `content` (HTML) when both are present.
    #[serde(default)]
    pub content_json: Option<String>,
    /// Short description surfaced in Corkboard / Outliner views.
    /// Independent of `content`: changing one doesn't touch the other.
    #[serde(default)]
    pub synopsis: Option<String>,
    pub position: i64,
    #[serde(default)]
    pub status: DocumentStatus,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Ids of the project labels attached to this document. The label
    /// definitions (name + color) are resolved separately for display.
    #[serde(default)]
    pub label_ids: Vec<String>,
    /// Custom metadata values (I-08): `custom_field id → string value`. The
    /// field definitions (name, kind, options) are resolved separately.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
    /// Research material (I-10): when true, this document (and its subtree) is
    /// excluded from word-count stats and from export by default.
    #[serde(default)]
    pub is_research: bool,
    /// Target word count for this document. `None` means no goal set.
    #[serde(default)]
    pub goal_words: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInput {
    pub project_id: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    pub title: String,
    pub doc_type: DocumentType,
    #[serde(default)]
    pub content: Option<String>,
}

impl DocumentInput {
    pub fn validate(&self) -> AppResult<()> {
        if self.project_id.trim().is_empty() {
            return Err(AppError::Invariant("project_id is empty".into()));
        }
        if self.title.trim().is_empty() {
            return Err(AppError::Invariant("document title is empty".into()));
        }
        if self.title.chars().count() > 200 {
            return Err(AppError::Invariant("document title too long (>200)".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_type_round_trip() {
        for t in [
            DocumentType::Chapter,
            DocumentType::Scene,
            DocumentType::Note,
            DocumentType::Folder,
            DocumentType::MangaPage,
        ] {
            assert_eq!(DocumentType::parse(t.as_str()).unwrap(), t);
        }
    }

    #[test]
    fn unknown_type_is_invariant_error() {
        assert!(matches!(
            DocumentType::parse("paragraph"),
            Err(AppError::Invariant(_))
        ));
    }

    #[test]
    fn doc_status_round_trip() {
        for s in [
            DocumentStatus::Draft,
            DocumentStatus::Revised,
            DocumentStatus::Final,
            DocumentStatus::Trashed,
        ] {
            assert_eq!(DocumentStatus::parse(s.as_str()).unwrap(), s);
        }
    }

    #[test]
    fn unknown_status_is_invariant_error() {
        assert!(matches!(
            DocumentStatus::parse("archived"),
            Err(AppError::Invariant(_))
        ));
    }

    #[test]
    fn doc_status_default_is_draft() {
        assert_eq!(DocumentStatus::default(), DocumentStatus::Draft);
    }

    #[test]
    fn input_validates_required_fields() {
        let ok = DocumentInput {
            project_id: "p1".into(),
            parent_id: None,
            title: "Capítulo 1".into(),
            doc_type: DocumentType::Chapter,
            content: None,
        };
        assert!(ok.validate().is_ok());

        let bad_title = DocumentInput {
            project_id: "p1".into(),
            parent_id: None,
            title: "".into(),
            doc_type: DocumentType::Chapter,
            content: None,
        };
        assert!(bad_title.validate().is_err());

        let bad_project = DocumentInput {
            project_id: "".into(),
            parent_id: None,
            title: "x".into(),
            doc_type: DocumentType::Chapter,
            content: None,
        };
        assert!(bad_project.validate().is_err());
    }
}
