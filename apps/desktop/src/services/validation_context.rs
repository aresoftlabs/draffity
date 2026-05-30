//! Assembles a `ValidationInput` (document text + codex block + voice anchor)
//! for a validation run. Pure — no storage or Tauri state — so it is unit
//! tested directly; the command layer just fetches the rows and hands them
//! over (AUD-27, CLAUDE.md §3: no domain logic in commands).

use crate::domain::{CodexEntry, DocNode, DocumentType};
use crate::services::ValidationInput;

/// Strip HTML tags and collapse whitespace to plain text for the model.
pub(crate) fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn cap(mut s: String, max: usize) -> String {
    if s.chars().count() > max {
        s = s.chars().take(max).collect();
        s.push('…');
    }
    s
}

/// Builds the per-document validation context, applying char caps so prompts
/// stay bounded regardless of chapter size.
pub struct ValidationContextBuilder {
    text_cap: usize,
    codex_cap: usize,
    anchor_cap: usize,
    anchor_docs: usize,
}

impl Default for ValidationContextBuilder {
    fn default() -> Self {
        Self {
            text_cap: 12_000,
            codex_cap: 4_000,
            anchor_cap: 4_000,
            anchor_docs: 3,
        }
    }
}

impl ValidationContextBuilder {
    /// Assemble the context for `doc`. `codex` is the project's codex entries;
    /// `documents` is the project's document list (used to pull the preceding
    /// chapters/scenes as a voice anchor).
    pub fn build(
        &self,
        doc: &DocNode,
        codex: &[CodexEntry],
        documents: &[DocNode],
    ) -> ValidationInput {
        let text = cap(
            strip_html(doc.content.as_deref().unwrap_or("")),
            self.text_cap,
        );

        // Codex block (all entries) for character + plot validators.
        let codex_block = cap(
            codex
                .iter()
                .map(|e| {
                    let body = e.body.as_deref().unwrap_or("—");
                    let tags = if e.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [tags: {}]", e.tags.join(", "))
                    };
                    format!("- {} ({}): {}{}", e.name, e.kind.as_str(), body, tags)
                })
                .collect::<Vec<_>>()
                .join("\n"),
            self.codex_cap,
        );

        // Voice anchor: the last few prior chapters/scenes, in order.
        let mut prior: Vec<&DocNode> = documents
            .iter()
            .filter(|d| {
                d.position < doc.position
                    && matches!(d.doc_type, DocumentType::Chapter | DocumentType::Scene)
                    && d.content.is_some()
            })
            .collect();
        prior.sort_by_key(|d| d.position);
        let start = prior.len().saturating_sub(self.anchor_docs);
        let anchor_text = cap(
            prior[start..]
                .iter()
                .map(|d| strip_html(d.content.as_deref().unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n\n"),
            self.anchor_cap,
        );

        ValidationInput {
            text,
            codex_block,
            anchor_text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CodexKind, DocumentStatus};
    use std::collections::HashMap;

    fn doc(id: &str, pos: i64, doc_type: DocumentType, content: Option<&str>) -> DocNode {
        DocNode {
            id: id.into(),
            project_id: "p".into(),
            parent_id: None,
            title: id.into(),
            doc_type,
            content: content.map(|c| c.to_string()),
            content_json: None,
            synopsis: None,
            position: pos,
            status: DocumentStatus::Draft,
            tags: Vec::new(),
            label_ids: Vec::new(),
            metadata: HashMap::new(),
            is_research: false,
            is_front_matter: false,
            is_back_matter: false,
            goal_words: None,
            created_at: 0,
            updated_at: 0,
        }
    }

    fn codex_entry(name: &str, kind: CodexKind, body: Option<&str>, tags: &[&str]) -> CodexEntry {
        CodexEntry {
            id: name.into(),
            project_id: "p".into(),
            kind,
            name: name.into(),
            body: body.map(|b| b.to_string()),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn strips_html_and_builds_codex_block() {
        let target = doc(
            "c2",
            1,
            DocumentType::Chapter,
            Some("<p>Hola <b>mundo</b>.</p>"),
        );
        let codex = vec![codex_entry(
            "Aragorn",
            CodexKind::Character,
            Some("Montaraz"),
            &["heroe"],
        )];
        let ctx = ValidationContextBuilder::default().build(
            &target,
            &codex,
            std::slice::from_ref(&target),
        );
        assert_eq!(ctx.text, "Hola mundo.");
        assert!(ctx
            .codex_block
            .contains("- Aragorn (character): Montaraz [tags: heroe]"));
    }

    #[test]
    fn anchor_pulls_only_preceding_chapters_and_scenes_in_order() {
        let c0 = doc("c0", 0, DocumentType::Chapter, Some("<p>uno</p>"));
        let folder = doc("f", 1, DocumentType::Folder, Some("<p>ignore</p>"));
        let target = doc("c2", 2, DocumentType::Chapter, Some("<p>actual</p>"));
        let later = doc("c3", 3, DocumentType::Chapter, Some("<p>despues</p>"));
        let docs = vec![c0, folder, target.clone(), later];

        let ctx = ValidationContextBuilder::default().build(&target, &[], &docs);
        // Only the preceding chapter contributes; folder and later docs don't.
        assert_eq!(ctx.anchor_text, "uno");
    }

    #[test]
    fn anchor_keeps_only_the_last_few_documents() {
        let builder = ValidationContextBuilder {
            anchor_docs: 2,
            ..ValidationContextBuilder::default()
        };
        let docs = vec![
            doc("c0", 0, DocumentType::Chapter, Some("a")),
            doc("c1", 1, DocumentType::Chapter, Some("b")),
            doc("c2", 2, DocumentType::Chapter, Some("c")),
            doc("c3", 3, DocumentType::Chapter, Some("d")),
        ];
        let ctx = builder.build(&docs[3], &[], &docs);
        // Only the two closest preceding chapters (b, c), in order.
        assert_eq!(ctx.anchor_text, "b\n\nc");
    }
}
