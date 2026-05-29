//! Labels (I-05/I-06): per-project colored tags surfaced as chips across the
//! binder / outliner / corkboard / inspector. A document carries a set of
//! label ids (`DocNode::label_ids`); the label definitions (name + color)
//! live here and are resolved in the UI for display.
//!
//! Distinct from `document_tags`: tags are free-form strings the writer types
//! ad-hoc; labels are a curated, colored, project-scoped taxonomy.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub project_id: String,
    pub name: String,
    /// Hex color string, e.g. `#ef4444`.
    pub color: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelInput {
    pub project_id: String,
    pub name: String,
    pub color: String,
}

/// Validate a hex color of the form `#rgb` or `#rrggbb` (case-insensitive).
fn is_valid_hex_color(s: &str) -> bool {
    let Some(hex) = s.strip_prefix('#') else {
        return false;
    };
    (hex.len() == 3 || hex.len() == 6) && hex.chars().all(|c| c.is_ascii_hexdigit())
}

impl LabelInput {
    pub fn validate(&self) -> AppResult<()> {
        if self.name.trim().is_empty() {
            return Err(AppError::Invariant("label name is empty".into()));
        }
        if self.name.chars().count() > 50 {
            return Err(AppError::Invariant("label name too long (>50)".into()));
        }
        if !is_valid_hex_color(self.color.trim()) {
            return Err(AppError::Invariant(format!(
                "invalid label color: {}",
                self.color
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(name: &str, color: &str) -> LabelInput {
        LabelInput {
            project_id: "p".into(),
            name: name.into(),
            color: color.into(),
        }
    }

    #[test]
    fn valid_label_passes() {
        assert!(input("Importante", "#ef4444").validate().is_ok());
        assert!(input("POV", "#abc").validate().is_ok());
    }

    #[test]
    fn empty_name_is_rejected() {
        assert!(matches!(
            input("  ", "#ef4444").validate(),
            Err(AppError::Invariant(_))
        ));
    }

    #[test]
    fn bad_color_is_rejected() {
        for bad in ["red", "ef4444", "#xyzxyz", "#12", "#1234567"] {
            assert!(
                input("x", bad).validate().is_err(),
                "expected {bad} to be rejected"
            );
        }
    }
}
