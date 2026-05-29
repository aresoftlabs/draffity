//! Custom metadata fields (I-08/I-09): user-defined, per-project fields the
//! writer attaches to documents — e.g. "POV character" (select), "Word target
//! reviewer" (text), "Due" (date). Field definitions live here; per-document
//! values are a flat `field id → string` map carried on `DocNode::metadata`.
//!
//! Distinct from template [`MetadataField`](crate::domain::MetadataField),
//! which is project-level and template-defined. These are document-level and
//! edited at runtime.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CustomFieldKind {
    Text,
    Number,
    Date,
    Select,
}

impl CustomFieldKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CustomFieldKind::Text => "text",
            CustomFieldKind::Number => "number",
            CustomFieldKind::Date => "date",
            CustomFieldKind::Select => "select",
        }
    }
    pub fn parse(s: &str) -> AppResult<Self> {
        match s {
            "text" => Ok(CustomFieldKind::Text),
            "number" => Ok(CustomFieldKind::Number),
            "date" => Ok(CustomFieldKind::Date),
            "select" => Ok(CustomFieldKind::Select),
            other => Err(AppError::Invariant(format!(
                "unknown custom field kind: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomField {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub kind: CustomFieldKind,
    /// Allowed values for `Select`; empty for other kinds.
    #[serde(default)]
    pub options: Vec<String>,
    pub position: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldInput {
    pub project_id: String,
    pub name: String,
    pub kind: CustomFieldKind,
    #[serde(default)]
    pub options: Vec<String>,
}

/// Normalise option strings: trim, drop blanks, dedupe (order preserved).
pub fn clean_options(options: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for raw in options {
        let t = raw.trim();
        if t.is_empty() || !seen.insert(t.to_string()) {
            continue;
        }
        out.push(t.to_string());
    }
    out
}

impl CustomFieldInput {
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Invariant("custom field name is empty".into()));
        }
        if self.name.chars().count() > 60 {
            return Err(AppError::Invariant(
                "custom field name too long (>60)".into(),
            ));
        }
        if self.kind == CustomFieldKind::Select && clean_options(&self.options).is_empty() {
            return Err(AppError::Invariant(
                "a select field needs at least one option".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_round_trip() {
        for k in [
            CustomFieldKind::Text,
            CustomFieldKind::Number,
            CustomFieldKind::Date,
            CustomFieldKind::Select,
        ] {
            assert_eq!(CustomFieldKind::parse(k.as_str()).unwrap(), k);
        }
    }

    #[test]
    fn clean_options_trims_dedupes_drops_blanks() {
        let got = clean_options(&[
            "Alice".into(),
            "  ".into(),
            "Bob".into(),
            "Alice".into(),
            " Bob ".into(),
        ]);
        assert_eq!(got, vec!["Alice".to_string(), "Bob".to_string()]);
    }

    #[test]
    fn select_requires_options() {
        let bad = CustomFieldInput {
            project_id: "p".into(),
            name: "POV".into(),
            kind: CustomFieldKind::Select,
            options: vec!["  ".into()],
        };
        assert!(bad.validate().is_err());

        let ok = CustomFieldInput {
            project_id: "p".into(),
            name: "POV".into(),
            kind: CustomFieldKind::Select,
            options: vec!["Alice".into()],
        };
        assert!(ok.validate().is_ok());
    }

    #[test]
    fn non_select_does_not_need_options() {
        let f = CustomFieldInput {
            project_id: "p".into(),
            name: "Notes".into(),
            kind: CustomFieldKind::Text,
            options: vec![],
        };
        assert!(f.validate().is_ok());
    }

    #[test]
    fn empty_name_rejected() {
        let f = CustomFieldInput {
            project_id: "p".into(),
            name: "  ".into(),
            kind: CustomFieldKind::Text,
            options: vec![],
        };
        assert!(matches!(f.validate(), Err(AppError::Invariant(_))));
    }
}
