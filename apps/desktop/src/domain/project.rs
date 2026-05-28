use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Archived,
}

impl ProjectStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Archived => "archived",
        }
    }

    pub fn parse(s: &str) -> AppResult<Self> {
        match s {
            "active" => Ok(ProjectStatus::Active),
            "archived" => Ok(ProjectStatus::Archived),
            other => Err(AppError::Invariant(format!(
                "unknown project status: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub title: String,
    pub template_id: String,
    pub status: ProjectStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    pub title: String,
    pub template_id: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl ProjectInput {
    pub fn validate(&self) -> AppResult<()> {
        if self.title.trim().is_empty() {
            return Err(AppError::Invariant("project title is empty".into()));
        }
        if self.title.chars().count() > 200 {
            return Err(AppError::Invariant("project title too long (>200)".into()));
        }
        if self.template_id.trim().is_empty() {
            return Err(AppError::Invariant("template_id is empty".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_round_trip() {
        for s in [ProjectStatus::Active, ProjectStatus::Archived] {
            assert_eq!(ProjectStatus::parse(s.as_str()).unwrap(), s);
        }
    }

    #[test]
    fn unknown_status_is_invariant_error() {
        let err = ProjectStatus::parse("on-hold").unwrap_err();
        assert!(matches!(err, AppError::Invariant(_)));
    }

    #[test]
    fn input_rejects_empty_title() {
        let bad = ProjectInput {
            title: "  ".into(),
            template_id: "novela".into(),
            metadata: None,
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn input_rejects_empty_template() {
        let bad = ProjectInput {
            title: "Mi novela".into(),
            template_id: "".into(),
            metadata: None,
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn input_accepts_valid() {
        let ok = ProjectInput {
            title: "Mi novela".into(),
            template_id: "novela-tres-actos".into(),
            metadata: None,
        };
        assert!(ok.validate().is_ok());
    }
}
