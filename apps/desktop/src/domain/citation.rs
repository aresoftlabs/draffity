//! Imported bibliographic entries. The wire shape mirrors what BibTeX gives
//! us: an opaque `entry_type` + a flat string map of fields. We don't try to
//! normalise across BibTeX entry types — that lives at the renderer layer
//! (citations export, S5-06).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    pub id: String,
    pub project_id: String,
    /// BibTeX citation key (`@article{key, ...}`). Unique within a project.
    pub key: String,
    /// `article`, `book`, `inproceedings`, etc. Lowercased.
    pub entry_type: String,
    /// Field values from the BibTeX entry, with BibTeX brace/quote stripping
    /// already applied. Keys are lowercased BibTeX field names (`author`,
    /// `title`, `year`, …).
    pub fields: BTreeMap<String, String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Citation {
    /// First author's surname when available, otherwise an empty string.
    /// Used by the renderer for `(Surname, year)` short references.
    pub fn first_author_surname(&self) -> String {
        let raw = match self.fields.get("author") {
            Some(s) => s.as_str(),
            None => return String::new(),
        };
        // BibTeX joins multiple authors with " and ". Take the first.
        let first = raw.split(" and ").next().unwrap_or(raw).trim();
        // Two BibTeX conventions: "Lastname, Firstname" or "Firstname Lastname".
        if let Some((last, _)) = first.split_once(',') {
            last.trim().to_string()
        } else {
            // Take the last space-separated token as the surname.
            first
                .split_whitespace()
                .next_back()
                .unwrap_or(first)
                .to_string()
        }
    }

    /// 4-digit year as a string when present.
    pub fn year(&self) -> Option<&str> {
        self.fields.get("year").map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make(fields: &[(&str, &str)]) -> Citation {
        let mut m = BTreeMap::new();
        for (k, v) in fields {
            m.insert((*k).into(), (*v).into());
        }
        Citation {
            id: "id".into(),
            project_id: "p".into(),
            key: "k".into(),
            entry_type: "article".into(),
            fields: m,
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn extracts_lastname_from_comma_form() {
        let c = make(&[("author", "García Márquez, Gabriel")]);
        assert_eq!(c.first_author_surname(), "García Márquez");
    }

    #[test]
    fn extracts_lastname_from_space_form() {
        let c = make(&[("author", "Jorge Luis Borges")]);
        assert_eq!(c.first_author_surname(), "Borges");
    }

    #[test]
    fn picks_first_when_multiple_authors() {
        let c = make(&[("author", "Borges, J. L. and Casares, A. B.")]);
        assert_eq!(c.first_author_surname(), "Borges");
    }

    #[test]
    fn returns_empty_when_no_author() {
        let c = make(&[]);
        assert_eq!(c.first_author_surname(), "");
    }

    #[test]
    fn reads_year_field() {
        let c = make(&[("year", "1944")]);
        assert_eq!(c.year(), Some("1944"));
    }
}
