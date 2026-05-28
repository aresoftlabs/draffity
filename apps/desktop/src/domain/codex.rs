//! Codex entries: worldbuilding pieces attached to a project. The editor
//! cites them via `[[name]]` cross-refs which resolve to the entry's `id`,
//! so renames don't break the manuscript.

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CodexKind {
    Character,
    Place,
    Object,
    Note,
}

impl CodexKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CodexKind::Character => "character",
            CodexKind::Place => "place",
            CodexKind::Object => "object",
            CodexKind::Note => "note",
        }
    }

    /// Parse from the snake-case wire form. Named `parse` (not `from_str`)
    /// to avoid shadowing `std::str::FromStr`.
    pub fn parse(s: &str) -> AppResult<Self> {
        match s {
            "character" => Ok(CodexKind::Character),
            "place" => Ok(CodexKind::Place),
            "object" => Ok(CodexKind::Object),
            "note" => Ok(CodexKind::Note),
            other => Err(AppError::Invariant(format!("unknown codex kind '{other}'"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexEntry {
    pub id: String,
    pub project_id: String,
    pub kind: CodexKind,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexInput {
    pub project_id: String,
    pub kind: CodexKind,
    pub name: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexUpdate {
    pub name: Option<String>,
    pub kind: Option<CodexKind>,
    pub body: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl CodexInput {
    /// Validate at the boundary before the SQL hit. Empty names produce a
    /// useless catalogue; trimming both name and tags keeps lookups stable.
    pub fn validate(&self) -> AppResult<()> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(AppError::Invariant("codex entry name is empty".into()));
        }
        for tag in &self.tags {
            if tag.trim().is_empty() {
                return Err(AppError::Invariant("codex entry has an empty tag".into()));
            }
        }
        Ok(())
    }

    /// Normalise: trim name, trim tags, dedupe tags preserving first-seen
    /// order. Returns a copy so callers don't surprise themselves with
    /// in-place mutation.
    pub fn normalised(&self) -> Self {
        let mut seen = std::collections::HashSet::new();
        let mut tags = Vec::with_capacity(self.tags.len());
        for raw in &self.tags {
            let t = raw.trim().to_string();
            if !t.is_empty() && seen.insert(t.clone()) {
                tags.push(t);
            }
        }
        Self {
            project_id: self.project_id.clone(),
            kind: self.kind,
            name: self.name.trim().to_string(),
            body: self.body.clone(),
            tags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_round_trips_via_str() {
        for k in [
            CodexKind::Character,
            CodexKind::Place,
            CodexKind::Object,
            CodexKind::Note,
        ] {
            assert_eq!(CodexKind::parse(k.as_str()).unwrap(), k);
        }
    }

    #[test]
    fn unknown_kind_str_is_rejected() {
        assert!(CodexKind::parse("alien").is_err());
    }

    #[test]
    fn empty_name_fails_validation() {
        let inp = CodexInput {
            project_id: "p".into(),
            kind: CodexKind::Character,
            name: "  ".into(),
            body: None,
            tags: vec![],
        };
        assert!(inp.validate().is_err());
    }

    #[test]
    fn empty_tag_fails_validation() {
        let inp = CodexInput {
            project_id: "p".into(),
            kind: CodexKind::Character,
            name: "Aragorn".into(),
            body: None,
            tags: vec!["hero".into(), " ".into()],
        };
        assert!(inp.validate().is_err());
    }

    #[test]
    fn normalise_trims_and_dedupes_tags() {
        let inp = CodexInput {
            project_id: "p".into(),
            kind: CodexKind::Character,
            name: "  Aragorn  ".into(),
            body: None,
            tags: vec![
                "hero".into(),
                "Hero".into(),
                "  hero  ".into(),
                "ranger".into(),
            ],
        };
        let n = inp.normalised();
        assert_eq!(n.name, "Aragorn");
        // Case-sensitive dedupe (matches DocNode tags behaviour); "Hero"
        // and "hero" are distinct, but the second whitespace "hero" is the
        // same string after trim so it collapses.
        assert_eq!(n.tags, vec!["hero", "Hero", "ranger"]);
    }

    #[test]
    fn normalise_keeps_first_seen_order() {
        let inp = CodexInput {
            project_id: "p".into(),
            kind: CodexKind::Place,
            name: "Mordor".into(),
            body: None,
            tags: vec!["dark".into(), "volcano".into(), "dark".into()],
        };
        assert_eq!(inp.normalised().tags, vec!["dark", "volcano"]);
    }
}
