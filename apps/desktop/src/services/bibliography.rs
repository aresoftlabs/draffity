//! Bibliographic import. Parses a BibTeX/BibLaTeX string with the `biblatex`
//! crate, normalises every entry to `(key, entry_type, field map)` and
//! delegates persistence to the storage layer. Premium can later add
//! `RemoteBibliographyService` (Zotero, BibSonomy…) implementing the same
//! trait.

use std::collections::BTreeMap;

use biblatex::{Bibliography, ChunksExt};

use crate::error::{AppError, AppResult};
use crate::services::storage::CitationUpsert;

#[derive(Debug, Default, Clone)]
pub struct ParseSummary {
    pub entries: Vec<CitationUpsert>,
    /// Soft parse errors: BibLaTeX entries that the parser skipped because
    /// they were malformed. The caller surfaces this count in the UI.
    pub skipped_entries: usize,
}

pub trait BibliographyService: Send + Sync {
    /// Parse a BibTeX string into normalised upsert entries. Errors only on
    /// global parser failure (file isn't BibTeX at all). Per-entry issues
    /// are reported via `ParseSummary::skipped_entries`.
    fn parse(&self, bib_text: &str) -> AppResult<ParseSummary>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalBibliographyService;

impl BibliographyService for LocalBibliographyService {
    fn parse(&self, bib_text: &str) -> AppResult<ParseSummary> {
        let bib = Bibliography::parse(bib_text)
            .map_err(|e| AppError::Invariant(format!("invalid BibTeX input: {e}")))?;

        let mut entries = Vec::new();
        let mut skipped = 0usize;
        for entry in bib.iter() {
            let key = entry.key.trim();
            if key.is_empty() {
                skipped += 1;
                continue;
            }
            let entry_type = entry.entry_type.to_string().to_ascii_lowercase();
            let mut fields = BTreeMap::new();
            for (name, chunks) in entry.fields.iter() {
                let raw = chunks.format_verbatim();
                let cleaned = clean_value(&raw);
                fields.insert(name.to_ascii_lowercase(), cleaned);
            }
            entries.push(CitationUpsert {
                key: key.to_string(),
                entry_type,
                fields,
            });
        }
        Ok(ParseSummary {
            entries,
            skipped_entries: skipped,
        })
    }
}

/// Strip BibTeX-isms from a value: outer braces/quotes, escaped braces,
/// collapsed whitespace. Diacritic escapes (`\'a` → `á`) are out of scope
/// for the MVP — the `biblatex` parser already does best-effort decoding.
fn clean_value(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    while s.starts_with('{') && s.ends_with('}') && s.len() >= 2 {
        s = s[1..s.len() - 1].to_string();
    }
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s = s[1..s.len() - 1].to_string();
    }
    s = s.replace("\\{", "{").replace("\\}", "}");
    // Collapse internal whitespace runs to a single space.
    let mut out = String::with_capacity(s.len());
    let mut prev_ws = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                out.push(' ');
            }
            prev_ws = true;
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_article() {
        let bib = r#"
            @article{borges1944,
              author = {Borges, Jorge Luis},
              title  = {Ficciones},
              year   = {1944},
            }
        "#;
        let s = LocalBibliographyService;
        let summary = s.parse(bib).unwrap();
        assert_eq!(summary.entries.len(), 1);
        assert_eq!(summary.skipped_entries, 0);
        let e = &summary.entries[0];
        assert_eq!(e.key, "borges1944");
        assert_eq!(e.entry_type, "article");
        assert_eq!(e.fields.get("author").unwrap(), "Borges, Jorge Luis");
        assert_eq!(e.fields.get("title").unwrap(), "Ficciones");
        assert_eq!(e.fields.get("year").unwrap(), "1944");
    }

    #[test]
    fn parses_multiple_entries_with_different_types() {
        let bib = r#"
            @book{a, author={A}, title={X}, year={2000}}
            @inproceedings{b, author={B}, title={Y}, year={2010}, booktitle={Conf}}
        "#;
        let s = LocalBibliographyService;
        let summary = s.parse(bib).unwrap();
        assert_eq!(summary.entries.len(), 2);
        let types: Vec<&str> = summary
            .entries
            .iter()
            .map(|e| e.entry_type.as_str())
            .collect();
        assert!(types.contains(&"book"));
        assert!(types.contains(&"inproceedings"));
    }

    #[test]
    fn rejects_non_bibtex_input() {
        let s = LocalBibliographyService;
        // biblatex 0.10 is tolerant — only completely empty / no `@`-prefixed
        // entries should not produce results. An entirely non-BibTeX string
        // either errors out or parses to zero entries; both are acceptable.
        let summary = s.parse("this is not BibTeX at all").unwrap_or_default();
        assert!(summary.entries.is_empty());
    }

    #[test]
    fn clean_value_strips_outer_braces_and_quotes() {
        assert_eq!(clean_value("{Hello}"), "Hello");
        assert_eq!(clean_value("\"Hello\""), "Hello");
        assert_eq!(clean_value("{{Hello}}"), "Hello");
    }

    #[test]
    fn clean_value_collapses_whitespace() {
        assert_eq!(clean_value("Hola   mundo\n  cruel"), "Hola mundo cruel");
    }
}
